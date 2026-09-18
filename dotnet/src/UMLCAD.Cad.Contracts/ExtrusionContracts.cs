namespace UMLCAD.Cad.Contracts;

public sealed record PlanarProfilePoint(double U, double V)
{
    public PlanarProfilePoint
    {
        if (!double.IsFinite(U) || !double.IsFinite(V))
            throw new ArgumentOutOfRangeException(nameof(U), "Profile coordinates must be finite.");
    }
}

public sealed record ExtrusionRequest(
    string OperationIdentity,
    KernelVector3 Origin,
    KernelVector3 UDirection,
    KernelVector3 VDirection,
    IReadOnlyList<PlanarProfilePoint> Profile,
    double Depth,
    KernelTolerance Tolerance,
    ContractVersion ContractVersion)
{
    public const string ContractId = "UMLCAD.Geometry.ExtrudeConvexPlanarProfile";
    public const string ContractSchema = "uml-cad-extrude-convex-planar-profile/1.0.0";

    public ExtrusionRequest
    {
        if (string.IsNullOrWhiteSpace(OperationIdentity))
            throw new ArgumentException("OperationIdentity is required.", nameof(OperationIdentity));

        if (ContractVersion.Value != "1.0")
            throw new ArgumentException("Unsupported extrusion contract version.", nameof(ContractVersion));

        Profile = Profile?.ToArray() ??
            throw new ArgumentNullException(nameof(Profile));

        if (Profile.Count < 3)
            throw new ArgumentException("At least three profile points are required.", nameof(Profile));

        if (!double.IsFinite(Depth) || Depth <= 0d)
            throw new ArgumentOutOfRangeException(nameof(Depth));
    }
}

public sealed record ExtrusionTopology(
    string Kind,
    string Key)
{
    public ExtrusionTopology
    {
        if (string.IsNullOrWhiteSpace(Kind))
            throw new ArgumentException("Topology kind is required.", nameof(Kind));
        if (string.IsNullOrWhiteSpace(Key))
            throw new ArgumentException("Topology key is required.", nameof(Key));
    }
}

public sealed record ExtrusionKernelResult(
    GeometryKernelStatus Status,
    ContractResultId? ResultId,
    string? EvidenceHash,
    IReadOnlyList<ExtrusionTopology> Topology,
    double? Volume,
    double? SurfaceArea,
    KernelVector3? Centroid,
    IReadOnlyList<string> Diagnostics)
{
    public ExtrusionKernelResult
    {
        Topology = Topology?.ToArray() ??
            throw new ArgumentNullException(nameof(Topology));

        Diagnostics = Diagnostics?.ToArray() ??
            throw new ArgumentNullException(nameof(Diagnostics));

        if (Status == GeometryKernelStatus.Succeeded)
        {
            if (ResultId is null || string.IsNullOrWhiteSpace(EvidenceHash))
                throw new ArgumentException("Successful extrusion requires result and evidence identities.");
            if (Volume is null || SurfaceArea is null || Centroid is null)
                throw new ArgumentException("Successful extrusion requires solid metrics.");
            if (Topology.Count < 5)
                throw new ArgumentException("Successful extrusion requires a complete topological boundary.");
        }
    }
}

public interface IExtrusionGeometryService
{
    Task<ExtrusionKernelResult> ExtrudeConvexPlanarProfileAsync(
        ExtrusionRequest request,
        CancellationToken cancellationToken = default);
}
