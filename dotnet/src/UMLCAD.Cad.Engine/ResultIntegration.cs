using UMLCAD.Cad.Contracts;
using UMLCAD.Cad.Semantics;

namespace UMLCAD.Cad.Engine;

public static class ResultIntegrator
{
    public static AuthoritativeCadResult Integrate(
        SemanticId producingSemanticId,
        AuthoritativeResultKind resultKind,
        KernelResult kernelResult)
    {
        if (producingSemanticId.Value == Guid.Empty)
            throw new ArgumentException("ProducingSemanticId is required.", nameof(producingSemanticId));

        ArgumentNullException.ThrowIfNull(kernelResult);

        var status = kernelResult.Status switch
        {
            KernelResultStatus.Succeeded => AuthoritativeResultStatus.Succeeded,
            KernelResultStatus.Failed => AuthoritativeResultStatus.Failed,
            KernelResultStatus.Unsupported => AuthoritativeResultStatus.Unsupported,
            KernelResultStatus.Ambiguous => AuthoritativeResultStatus.Ambiguous,
            KernelResultStatus.Indeterminate => AuthoritativeResultStatus.Indeterminate,
            _ => throw new InvalidOperationException(
                $"Unknown kernel result status '{kernelResult.Status}'."),
        };

        if (kernelResult.ResultId is null)
        {
            return new AuthoritativeCadResult(
                new AuthoritativeResultIdentity(
                    $"failed:{producingSemanticId.Value:D}"),
                resultKind,
                status,
                producingSemanticId,
                Array.Empty<TopologyBinding>(),
                new ResultEvidence(
                    ContractId: "unknown",
                    ContractVersion: "unknown",
                    EvidenceHash: kernelResult.EvidenceHash,
                    TopologyEvolution: Array.Empty<TopologyEvolution>(),
                    Diagnostics: kernelResult.Diagnostics));
        }

        var topologyBindings = kernelResult.Topology
            .OrderBy(x => x.ResultId.Value, StringComparer.Ordinal)
            .ThenBy(x => x.TopologyKey, StringComparer.Ordinal)
            .Select(x => new TopologyBinding(
                TopologyKind: ExtractTopologyKind(x.TopologyKey),
                TopologyKey: x.TopologyKey,
                SourceSemanticId: producingSemanticId))
            .ToArray();

        if (status == AuthoritativeResultStatus.Succeeded &&
            resultKind is AuthoritativeResultKind.Wire or AuthoritativeResultKind.Surface or
                AuthoritativeResultKind.Solid &&
            topologyBindings.Length == 0)
        {
            throw new InvalidOperationException(
                "A successful geometric kernel result must contain topology bindings.");
        }

        return new AuthoritativeCadResult(
            new AuthoritativeResultIdentity(kernelResult.ResultId.Value.Value),
            resultKind,
            status,
            producingSemanticId,
            topologyBindings,
            new ResultEvidence(
                ContractId: kernelResultContractIdFallback(kernelResult),
                ContractVersion: "transport",
                EvidenceHash: kernelResult.EvidenceHash,
                TopologyEvolution: Array.Empty<TopologyEvolution>(),
                Diagnostics: kernelResult.Diagnostics));
    }

    private static string ExtractTopologyKind(string topologyKey)
    {
        var separator = topologyKey.IndexOf(':');
        return separator > 0
            ? topologyKey[..separator]
            : "Unknown";
    }

    private static string kernelResultContractIdFallback(KernelResult result) =>
        result.ResultId is null ? "unknown" : "UMLCAD.KernelResult";
}
