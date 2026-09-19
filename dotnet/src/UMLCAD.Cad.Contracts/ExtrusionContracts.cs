namespace UMLCAD.Cad.Contracts;

public sealed record PlanarProfilePoint
{
    public double U { get; }
    public double V { get; }

    public PlanarProfilePoint(double u, double v)
    {
        if (!double.IsFinite(u) || !double.IsFinite(v))
            throw new ArgumentOutOfRangeException(
                nameof(u),
                "Profile coordinates must be finite.");

        U = u;
        V = v;
    }
}

public sealed record ExtrusionRequest
{
    public string OperationIdentity { get; }
    public KernelVector3 Origin { get; }
    public KernelVector3 UDirection { get; }
    public KernelVector3 VDirection { get; }
    public IReadOnlyList<PlanarProfilePoint> Profile { get; }
    public double Depth { get; }
    public KernelTolerance Tolerance { get; }
    public ContractVersion ContractVersion { get; }

    public const string ContractId = "UMLCAD.Geometry.ExtrudeConvexPlanarProfile";
    public const string ContractSchema = "uml-cad-extrude-convex-planar-profile/1.0.0";

    public ExtrusionRequest(
        string operationIdentity,
        KernelVector3 origin,
        KernelVector3 uDirection,
        KernelVector3 vDirection,
        IReadOnlyList<PlanarProfilePoint> profile,
        double depth,
        KernelTolerance tolerance,
        ContractVersion contractVersion)
    {
        if (string.IsNullOrWhiteSpace(operationIdentity))
            throw new ArgumentException(
                "OperationIdentity is required.",
                nameof(operationIdentity));

        if (contractVersion.Value != "1.0")
            throw new ArgumentException(
                "Unsupported extrusion contract version.",
                nameof(contractVersion));

        Profile = profile?.ToArray()
            ?? throw new ArgumentNullException(nameof(profile));

        if (Profile.Count < 3)
            throw new ArgumentException(
                "At least three profile points are required.",
                nameof(profile));

        if (!double.IsFinite(depth) || depth <= 0d)
            throw new ArgumentOutOfRangeException(nameof(depth));

        OperationIdentity = operationIdentity;
        Origin = origin;
        UDirection = uDirection;
        VDirection = vDirection;
        Depth = depth;
        Tolerance = tolerance;
        ContractVersion = contractVersion;
    }
}

public sealed record ExtrusionTopology
{
    public string Kind { get; }
    public string Key { get; }

    public ExtrusionTopology(string kind, string key)
    {
        if (string.IsNullOrWhiteSpace(kind))
            throw new ArgumentException(
                "Topology kind is required.",
                nameof(kind));
        if (string.IsNullOrWhiteSpace(key))
            throw new ArgumentException(
                "Topology key is required.",
                nameof(key));

        Kind = kind;
        Key = key;
    }
}

public sealed record ExtrusionKernelResult
{
    public GeometryKernelStatus Status { get; }
    public ContractResultId? ResultId { get; }
    public string? EvidenceHash { get; }
    public IReadOnlyList<ExtrusionTopology> Topology { get; }
    public double? Volume { get; }
    public double? SurfaceArea { get; }
    public KernelVector3? Centroid { get; }
    public IReadOnlyList<string> Diagnostics { get; }

    public ExtrusionKernelResult(
        GeometryKernelStatus status,
        ContractResultId? resultId,
        string? evidenceHash,
        IReadOnlyList<ExtrusionTopology> topology,
        double? volume,
        double? surfaceArea,
        KernelVector3? centroid,
        IReadOnlyList<string> diagnostics)
    {
        Topology = topology?.ToArray()
            ?? throw new ArgumentNullException(nameof(topology));
        Diagnostics = diagnostics?.ToArray()
            ?? throw new ArgumentNullException(nameof(diagnostics));

        if (status == GeometryKernelStatus.Succeeded)
        {
            if (resultId is null ||
                string.IsNullOrWhiteSpace(evidenceHash))
                throw new ArgumentException(
                    "Successful extrusion requires result and evidence identities.");

            if (volume is null ||
                surfaceArea is null ||
                centroid is null)
                throw new ArgumentException(
                    "Successful extrusion requires solid metrics.");

            if (Topology.Count < 5)
                throw new ArgumentException(
                    "Successful extrusion requires a complete topological boundary.");
        }

        Status = status;
        ResultId = resultId;
        EvidenceHash = evidenceHash;
        Volume = volume;
        SurfaceArea = surfaceArea;
        Centroid = centroid;
    }
}

public interface IExtrusionGeometryService
{
    Task<ExtrusionKernelResult> ExtrudeConvexPlanarProfileAsync(
        ExtrusionRequest request,
        CancellationToken cancellationToken = default);
}
