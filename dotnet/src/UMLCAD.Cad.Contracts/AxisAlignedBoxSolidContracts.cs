namespace UMLCAD.Cad.Contracts;

public readonly record struct KernelVector3
{
    public double X { get; }
    public double Y { get; }
    public double Z { get; }

    public KernelVector3(double x, double y, double z)
    {
        if (!double.IsFinite(x) || !double.IsFinite(y) || !double.IsFinite(z))
            throw new ArgumentOutOfRangeException(nameof(x), "Kernel vector components must be finite.");

        X = x;
        Y = y;
        Z = z;
    }

    public bool IsFinite =>
        double.IsFinite(X) && double.IsFinite(Y) && double.IsFinite(Z);

    public double Length =>
        Math.Sqrt((X * X) + (Y * Y) + (Z * Z));

    public KernelVector3 Normalize(string name)
    {
        if (!IsFinite || !double.IsFinite(Length) || Length <= 0d)
            throw new ArgumentException($"{name} must be finite and non-zero.");

        var length = Length;
        return new KernelVector3(X / length, Y / length, Z / length);
    }
}

public readonly record struct KernelTolerance
{
    public double Absolute { get; }
    public double Relative { get; }

    public KernelTolerance(double absolute, double relative)
    {
        if (!double.IsFinite(absolute) || !double.IsFinite(relative) ||
            absolute < 0d || relative < 0d)
            throw new ArgumentOutOfRangeException(
                nameof(absolute),
                "Kernel tolerance must be finite and non-negative.");

        Absolute = absolute;
        Relative = relative;
    }
}

public sealed record AxisAlignedBoxSolidRequest
{
    public string OperationIdentity { get; }
    public KernelVector3 Min { get; }
    public KernelVector3 Max { get; }
    public KernelTolerance Tolerance { get; }
    public ContractVersion ContractVersion { get; }

    public const string ContractId = "UMLCAD.Geometry.AxisAlignedBoxSolid";
    public const string ContractSchema = "uml-cad-axis-aligned-box-solid/1.0.0";

    public AxisAlignedBoxSolidRequest(
        string operationIdentity,
        KernelVector3 min,
        KernelVector3 max,
        KernelTolerance tolerance,
        ContractVersion contractVersion)
    {
        if (string.IsNullOrWhiteSpace(operationIdentity))
            throw new ArgumentException(
                "OperationIdentity is required.",
                nameof(operationIdentity));

        if (contractVersion.Value != "1.0")
            throw new ArgumentException(
                "Unsupported axis-aligned box-solid contract version.",
                nameof(contractVersion));

        if (max.X <= min.X || max.Y <= min.Y || max.Z <= min.Z)
            throw new ArgumentException(
                "Axis-aligned box maximum must be strictly greater than minimum.",
                nameof(max));

        OperationIdentity = operationIdentity;
        Min = min;
        Max = max;
        Tolerance = tolerance;
        ContractVersion = contractVersion;
    }
}

public sealed record AxisAlignedBoxSolidKernelTopology
{
    public string Kind { get; }
    public string Key { get; }

    public AxisAlignedBoxSolidKernelTopology(string kind, string key)
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

public sealed record AxisAlignedBoxSolidKernelResult
{
    public GeometryKernelStatus Status { get; }
    public ContractResultId? ResultId { get; }
    public string? EvidenceHash { get; }
    public IReadOnlyList<AxisAlignedBoxSolidKernelTopology> Topology { get; }
    public double? Volume { get; }
    public double? SurfaceArea { get; }
    public KernelVector3? Centroid { get; }
    public IReadOnlyList<string> Diagnostics { get; }

    public AxisAlignedBoxSolidKernelResult(
        GeometryKernelStatus status,
        ContractResultId? resultId,
        string? evidenceHash,
        IReadOnlyList<AxisAlignedBoxSolidKernelTopology> topology,
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
            if (resultId is null || string.IsNullOrWhiteSpace(evidenceHash))
                throw new ArgumentException(
                    "Successful box-solid results require result and evidence identities.");

            if (volume is null || surfaceArea is null || centroid is null)
                throw new ArgumentException(
                    "Successful box-solid results require solid metrics.");

            if (Topology.Count != 6)
                throw new ArgumentException(
                    "The bounded box-solid contract requires six face topology bindings.");
        }

        Status = status;
        ResultId = resultId;
        EvidenceHash = evidenceHash;
        Volume = volume;
        SurfaceArea = surfaceArea;
        Centroid = centroid;
    }
}

public interface IAuthoritativeGeometryService
{
    Task<AxisAlignedBoxSolidKernelResult> BuildAxisAlignedBoxSolidAsync(
        AxisAlignedBoxSolidRequest request,
        CancellationToken cancellationToken = default);
}
