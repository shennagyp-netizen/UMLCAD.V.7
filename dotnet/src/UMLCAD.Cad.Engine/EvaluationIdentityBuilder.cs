using System.Security.Cryptography;
using System.Text;
using UMLCAD.Cad.Semantics;

namespace UMLCAD.Cad.Engine;

public static class EvaluationIdentityBuilder
{
    public static EvaluationIdentity Build(EvaluationStep step)
    {
        ArgumentNullException.ThrowIfNull(step);

        var dependencies = step.Dependencies
            .Select(x => x.Value.ToString("D"))
            .OrderBy(x => x, StringComparer.Ordinal)
            .ToArray();

        var canonical = string.Join(
            "|",
            "uml-cad-evaluation/1",
            step.OperationKind,
            step.NormalizedDefinition,
            string.Join(",", dependencies),
            step.KernelContractVersion);

        var hash = SHA256.HashData(Encoding.UTF8.GetBytes(canonical));
        return new EvaluationIdentity(Convert.ToHexString(hash).ToLowerInvariant());
    }
}
