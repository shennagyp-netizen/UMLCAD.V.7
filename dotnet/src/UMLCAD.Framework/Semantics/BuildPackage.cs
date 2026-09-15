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
        return new BuildPackage(
            "uml-cad-build-package/1.0.0",
            application.Id,
            application.Version,
            application.BuildIdentity,
            application);
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
