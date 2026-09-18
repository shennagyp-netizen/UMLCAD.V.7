using UMLCAD.Cad.Contracts;
using UMLCAD.Cad.Semantics;

namespace UMLCAD.Cad.Engine;

public sealed class ConvexProfileExtrusionEvaluator
{
    private readonly IExtrusionGeometryService _geometry;

    public ConvexProfileExtrusionEvaluator(IExtrusionGeometryService geometry)
    {
        _geometry = geometry ?? throw new ArgumentNullException(nameof(geometry));
    }

    public async Task<AuthoritativeCadResult> EvaluateAsync(
        ExtrusionFeatureSpecification specification,
        CancellationToken cancellationToken = default)
    {
        ArgumentNullException.ThrowIfNull(specification);

        var request = new ExtrusionRequest(
            OperationIdentity: specification.FeatureId.Value.ToString("D"),
            Origin: new KernelVector3(
                specification.Profile.OriginMm.X,
                specification.Profile.OriginMm.Y,
                specification.Profile.OriginMm.Z),
            UDirection: new KernelVector3(
                specification.Profile.UDirection.X,
                specification.Profile.UDirection.Y,
                specification.Profile.UDirection.Z),
            VDirection: new KernelVector3(
                specification.Profile.VDirection.X,
                specification.Profile.VDirection.Y,
                specification.Profile.VDirection.Z),
            Profile: specification.Profile.Points
                .Select(point => new PlanarProfilePoint(point.U, point.V))
                .ToArray(),
            Depth: specification.DepthMm,
            Tolerance: new KernelTolerance(1e-9, 1e-9),
            ContractVersion: new ContractVersion("1.0"));

        var kernelResult = await _geometry.ExtrudeConvexPlanarProfileAsync(
            request,
            cancellationToken);

        var status = kernelResult.Status switch
        {
            AxisAlignedBoxSolidKernelStatus.Succeeded => AuthoritativeResultStatus.Succeeded,
            AxisAlignedBoxSolidKernelStatus.Failed => AuthoritativeResultStatus.Failed,
            AxisAlignedBoxSolidKernelStatus.Unsupported => AuthoritativeResultStatus.Unsupported,
            AxisAlignedBoxSolidKernelStatus.Ambiguous => AuthoritativeResultStatus.Ambiguous,
            AxisAlignedBoxSolidKernelStatus.Indeterminate => AuthoritativeResultStatus.Indeterminate,
            _ => throw new InvalidOperationException(
                $"Unknown extrusion kernel status '{kernelResult.Status}'."),
        };

        if (kernelResult.ResultId is null || kernelResult.EvidenceHash is null)
        {
            return new AuthoritativeCadResult(
                new AuthoritativeResultIdentity(
                    $"failed:{specification.FeatureId.Value:D}"),
                AuthoritativeResultKind.Solid,
                status,
                specification.FeatureId,
                Array.Empty<TopologyBinding>(),
                new ResultEvidence(
                    ExtrusionRequest.ContractId,
                    "1.0",
                    kernelResult.EvidenceHash ?? "missing",
                    Array.Empty<TopologyEvolution>(),
                    kernelResult.Diagnostics));
        }

        var topology = kernelResult.Topology
            .OrderBy(x => x.Kind, StringComparer.Ordinal)
            .ThenBy(x => x.Key, StringComparer.Ordinal)
            .Select(x => new TopologyBinding(
                x.Kind,
                x.Key,
                specification.FeatureId))
            .ToArray();

        if (status == AuthoritativeResultStatus.Succeeded && topology.Count < 6)
            throw new InvalidOperationException(
                "A successful convex profile extrusion must expose a complete bounded B-Rep topology.");

        return new AuthoritativeCadResult(
            new AuthoritativeResultIdentity(kernelResult.ResultId.Value.Value),
            AuthoritativeResultKind.Solid,
            status,
            specification.FeatureId,
            topology,
            new ResultEvidence(
                ExtrusionRequest.ContractId,
                "1.0",
                kernelResult.EvidenceHash,
                Array.Empty<TopologyEvolution>(),
                kernelResult.Diagnostics));
    }
}
