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

        var inputs = step.Inputs
            .OrderBy(x => x.Role, StringComparer.Ordinal)
            .ThenBy(x => x.Identity, StringComparer.Ordinal)
            .Select(x => $"{x.Role}={x.Identity}")
            .ToArray();

        var canonical = string.Join(
            "|",
            "uml-cad-evaluation/2",
            step.OperationKind,
            step.NormalizedDefinition,
            string.Join(",", dependencies),
            string.Join(",", inputs),
            step.ConfigurationContext,
            step.TolerancePolicy,
            step.KernelContractVersion,
            step.RepresentationPolicy);

        var hash = SHA256.HashData(Encoding.UTF8.GetBytes(canonical));
        return new EvaluationIdentity(Convert.ToHexString(hash).ToLowerInvariant());
    }
}
