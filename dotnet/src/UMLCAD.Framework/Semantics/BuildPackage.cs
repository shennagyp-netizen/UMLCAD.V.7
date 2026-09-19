using System.Text.Json;
using System.Text.Json.Serialization;

namespace UMLCAD.Framework.Semantics;

public sealed record BuildPackage(
    string Schema,
    string ApplicationId,
    string ApplicationVersion,
    string BuildIdentity,
    SemanticApplication Semantic);

public interface IBuildPackageService
{
    BuildPackage CreatePackage(SemanticApplication application);
    byte[] Serialize(BuildPackage package);
    string SerializeToString(BuildPackage package);
}

internal sealed class BuildPackageService : IBuildPackageService
{
    public BuildPackage CreatePackage(SemanticApplication application)
    {
        ArgumentNullException.ThrowIfNull(application);
        // Authoritative results are derived evaluation state. They do not form
        // the semantic kernel input package and therefore must not alter the
        // payload sent to the mathematical authority.
        var packageSemantic = application with
        {
            AuthoritativeResults = Array.Empty<AuthoritativeResultSemantic>()
        };

        return new BuildPackage(
            "uml-cad-build-package/1.0.0",
            application.Id,
            application.Version,
            application.BuildIdentity,
            packageSemantic);
    }

    public byte[] Serialize(BuildPackage package) =>
        JsonSerializer.SerializeToUtf8Bytes(package, JsonDefaults.Options);

    public string SerializeToString(BuildPackage package) =>
        JsonSerializer.Serialize(package, JsonDefaults.Options);

    private static class JsonDefaults
    {
        public static readonly JsonSerializerOptions Options = new(JsonSerializerDefaults.General)
        {
            PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
            WriteIndented = false,
            DefaultIgnoreCondition = JsonIgnoreCondition.Never
        };
    }
}
