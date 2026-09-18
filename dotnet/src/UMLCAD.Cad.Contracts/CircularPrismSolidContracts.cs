namespace UMLCAD.Cad.Contracts;

public sealed record CircularPrismSolidRequest
{
    public string OperationIdentity { get; }
    public KernelVector3 Origin { get; }
    public KernelVector3 Axis { get; }
    public double Radius { get; }
    public double Depth { get; }
    public KernelTolerance Tolerance { get; }
    public ContractVersion ContractVersion { get; }

    public const string ContractId = "UMLCAD.Geometry.CircularPrismSolid";
    public const string ContractSchema = "uml-cad-circular-prism-solid/1.0.0";

    public CircularPrismSolidRequest(
        string operationIdentity,
        KernelVector3 origin,
        KernelVector3 axis,
        double radius,
        double depth,
        KernelTolerance tolerance,
        ContractVersion contractVersion)
    {
        if (string.IsNullOrWhiteSpace(operationIdentity))
            throw new ArgumentException(
                "OperationIdentity is required.",
                nameof(operationIdentity));

        if (!origin.IsFinite)
            throw new ArgumentException(
                "Origin must be finite.",
                nameof(origin));

        axis = axis.Normalize(nameof(axis));

        if (!double.IsFinite(radius) || radius <= 0d)
            throw new ArgumentException(
                "Radius must be finite and positive.",
                nameof(radius));

        if (!double.IsFinite(depth) || depth <= 0d)
            throw new ArgumentException(
                "Depth must be finite and positive.",
                nameof(depth));

        if (contractVersion.Value != "1.0")
            throw new ArgumentException(
                "Unsupported circular-prism contract version.",
                nameof(contractVersion));

        OperationIdentity = operationIdentity;
        Origin = origin;
        Axis = axis;
        Radius = radius;
        Depth = depth;
        Tolerance = tolerance;
        ContractVersion = contractVersion;
    }
}

public sealed record CircularPrismKernelTopology(
    string Kind,
    string Key);

public sealed record CircularPrismSolidKernelResult
{
    public GeometryKernelStatus Status { get; }
    public bool Succeeded { get; }
    public ContractResultId? ResultId { get; }
    public string? EvidenceHash { get; }
    public KernelVector3 Origin { get; }
    public KernelVector3 Axis { get; }
    public double Radius { get; }
    public double Depth { get; }
    public double? Volume { get; }
    public double? SurfaceArea { get; }
    public KernelVector3? Centroid { get; }
    public CadBoundingBox3? Bounds { get; }
    public IReadOnlyList<CircularPrismKernelTopology> Topology { get; }
    public IReadOnlyList<string> Diagnostics { get; }

    public CircularPrismSolidKernelResult(
        GeometryKernelStatus status,
        bool succeeded,
        ContractResultId? resultId,
        string? evidenceHash,
        KernelVector3 origin,
        KernelVector3 axis,
        double radius,
        double depth,
        double? volume,
        double? surfaceArea,
        KernelVector3? centroid,
        CadBoundingBox3? bounds,
        IReadOnlyList<CircularPrismKernelTopology> topology,
        IReadOnlyList<string> diagnostics)
    {
        Topology = topology?.ToArray()
            ?? throw new ArgumentNullException(nameof(topology));
        Diagnostics = diagnostics?.ToArray()
            ?? throw new ArgumentNullException(nameof(diagnostics));

        if (!Enum.IsDefined(status))
            throw new ArgumentException("Circular-prism status is invalid.");

        if (succeeded != (status == GeometryKernelStatus.Succeeded))
            throw new ArgumentException(
                "Circular-prism status is inconsistent with succeeded.");

        if (succeeded &&
            (resultId is null ||
             string.IsNullOrWhiteSpace(evidenceHash) ||
             volume is null ||
             surfaceArea is null ||
             centroid is null ||
             bounds is null))
            throw new ArgumentException(
                "Successful circular-prism result is incomplete.");

        if (!double.IsFinite(radius) ||
            radius <= 0d ||
            !double.IsFinite(depth) ||
            depth <= 0d)
            throw new ArgumentException(
                "Circular-prism dimensions are invalid.");

        if (succeeded)
        {
            if (volume < 0d ||
                !double.IsFinite(volume.Value) ||
                surfaceArea < 0d ||
                !double.IsFinite(surfaceArea.Value))
                throw new ArgumentException(
                    "Circular-prism measures are invalid.");

            bounds!.Validate();

            if (!centroid!.Value.IsFinite)
                throw new ArgumentException(
                    "Circular-prism centroid must be finite.");
        }

        Status = status;
        Succeeded = succeeded;
        ResultId = resultId;
        EvidenceHash = evidenceHash;
        Origin = origin;
        Axis = axis;
        Radius = radius;
        Depth = depth;
        Volume = volume;
        SurfaceArea = surfaceArea;
        Centroid = centroid;
        Bounds = bounds;
    }
}

public interface ICircularPrismGeometryService
{
    Task<CircularPrismSolidKernelResult> BuildCircularPrismSolidAsync(
        CircularPrismSolidRequest request,
        CancellationToken cancellationToken = default);
}
