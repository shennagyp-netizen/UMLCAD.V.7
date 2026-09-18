using UMLCAD.Cad.Contracts;
using UMLCAD.Cad.Semantics;

namespace UMLCAD.Cad.Engine;

public static class ResultIntegrator
{
    public static AuthoritativeCadResult Integrate(
        SemanticId producingSemanticId,
        AuthoritativeResultKind resultKind,
        KernelRequest request,
        KernelResult kernelResult)
    {
        if (producingSemanticId.Value == Guid.Empty)
            throw new ArgumentException("ProducingSemanticId is required.", nameof(producingSemanticId));

        ArgumentNullException.ThrowIfNull(request);
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
                new AuthoritativeResultIdentity($"failed:{producingSemanticId.Value:D}"),
                resultKind,
                status,
                producingSemanticId,
                Array.Empty<TopologyBinding>(),
                new ResultEvidence(
                    request.ContractId,
                    request.ContractVersion.Value,
                    kernelResult.EvidenceHash,
                    Array.Empty<TopologyEvolution>(),
                    kernelResult.Diagnostics));
        }

        var topologyBindings = kernelResult.Topology
            .OrderBy(x => x.ResultId.Value, StringComparer.Ordinal)
            .ThenBy(x => x.TopologyKind, StringComparer.Ordinal)
            .ThenBy(x => x.TopologyKey, StringComparer.Ordinal)
            .Select(x => new TopologyBinding(
                x.TopologyKind,
                x.TopologyKey,
                producingSemanticId))
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
                request.ContractId,
                request.ContractVersion.Value,
                kernelResult.EvidenceHash,
                Array.Empty<TopologyEvolution>(),
                kernelResult.Diagnostics));
    }
    public static AuthoritativeCadResult Integrate(
        SemanticId producingSemanticId,
        AxisAlignedBoxSolidKernelResult kernelResult)
    {
        if (producingSemanticId.Value == Guid.Empty)
            throw new ArgumentException("ProducingSemanticId is required.", nameof(producingSemanticId));

        ArgumentNullException.ThrowIfNull(kernelResult);

        var status = kernelResult.Status switch
        {
            AxisAlignedBoxSolidKernelStatus.Succeeded => AuthoritativeResultStatus.Succeeded,
            AxisAlignedBoxSolidKernelStatus.Failed => AuthoritativeResultStatus.Failed,
            AxisAlignedBoxSolidKernelStatus.Unsupported => AuthoritativeResultStatus.Unsupported,
            AxisAlignedBoxSolidKernelStatus.Ambiguous => AuthoritativeResultStatus.Ambiguous,
            AxisAlignedBoxSolidKernelStatus.Indeterminate => AuthoritativeResultStatus.Indeterminate,
            _ => throw new InvalidOperationException(
                $"Unknown box-solid kernel status '{kernelResult.Status}'."),
        };

        if (kernelResult.ResultId is null || kernelResult.EvidenceHash is null)
        {
            return new AuthoritativeCadResult(
                new AuthoritativeResultIdentity($"failed:{producingSemanticId.Value:D}"),
                AuthoritativeResultKind.Solid,
                status,
                producingSemanticId,
                Array.Empty<TopologyBinding>(),
                new ResultEvidence(
                    AxisAlignedBoxSolidRequest.ContractId,
                    "unknown",
                    kernelResult.EvidenceHash ?? "missing",
                    Array.Empty<TopologyEvolution>(),
                    kernelResult.Diagnostics));
        }

        var topology = kernelResult.Topology
            .OrderBy(x => x.Kind, StringComparer.Ordinal)
            .ThenBy(x => x.Key, StringComparer.Ordinal)
            .Select(x => new TopologyBinding(x.Kind, x.Key, producingSemanticId))
            .ToArray();

        if (status == AuthoritativeResultStatus.Succeeded && topology.Length != 6)
            throw new InvalidOperationException(
                "A successful box-solid result must contain exactly six face bindings.");

        return new AuthoritativeCadResult(
            new AuthoritativeResultIdentity(kernelResult.ResultId.Value.Value),
            AuthoritativeResultKind.Solid,
            status,
            producingSemanticId,
            topology,
            new ResultEvidence(
                AxisAlignedBoxSolidRequest.ContractId,
                "1.0",
                kernelResult.EvidenceHash,
                Array.Empty<TopologyEvolution>(),
                kernelResult.Diagnostics));
    }

}
