namespace UMLCAD.Cad.Contracts;

public readonly record struct KernelVector3(double X, double Y, double Z)
{
    public KernelVector3
    {
        if (!double.IsFinite(X) || !double.IsFinite(Y) || !double.IsFinite(Z))
            throw new ArgumentOutOfRangeException(nameof(X), "Kernel vector components must be finite.");
    }
}

public readonly record struct KernelTolerance(double Absolute, double Relative)
{
    public KernelTolerance
    {
        if (!double.IsFinite(Absolute) || !double.IsFinite(Relative) ||
            Absolute < 0d || Relative < 0d)
        {
            throw new ArgumentOutOfRangeException(nameof(Absolute), "Kernel tolerance must be finite and non-negative.");
        }
    }
}

public sealed record AxisAlignedBoxSolidRequest(
    string OperationIdentity,
    KernelVector3 Min,
    KernelVector3 Max,
    KernelTolerance Tolerance,
    ContractVersion ContractVersion)
{
    public const string ContractId = "UMLCAD.Geometry.AxisAlignedBoxSolid";
    public const string ContractSchema = "uml-cad-axis-aligned-box-solid/1.0.0";

    public AxisAlignedBoxSolidRequest
    {
        if (string.IsNullOrWhiteSpace(OperationIdentity))
            throw new ArgumentException("OperationIdentity is required.", nameof(OperationIdentity));

        if (ContractVersion.Value != "1.0")
            throw new ArgumentException("Unsupported axis-aligned box-solid contract version.", nameof(ContractVersion));

        if (Max.X <= Min.X || Max.Y <= Min.Y || Max.Z <= Min.Z)
            throw new ArgumentException("Axis-aligned box maximum must be strictly greater than minimum.", nameof(Max));
    }
}

public sealed record AxisAlignedBoxSolidKernelTopology(
    string Kind,
    string Key)
{
    public AxisAlignedBoxSolidKernelTopology
    {
        if (string.IsNullOrWhiteSpace(Kind))
            throw new ArgumentException("Topology kind is required.", nameof(Kind));
        if (string.IsNullOrWhiteSpace(Key))
            throw new ArgumentException("Topology key is required.", nameof(Key));
    }
}

public sealed record AxisAlignedBoxSolidKernelResult(
    GeometryKernelStatus Status,
    ContractResultId? ResultId,
    string? EvidenceHash,
    IReadOnlyList<AxisAlignedBoxSolidKernelTopology> Topology,
    double? Volume,
    double? SurfaceArea,
    KernelVector3? Centroid,
    IReadOnlyList<string> Diagnostics)
{
    public AxisAlignedBoxSolidKernelResult
    {
        Topology = Topology?.ToArray() ??
            throw new ArgumentNullException(nameof(Topology));
        Diagnostics = Diagnostics?.ToArray() ??
            throw new ArgumentNullException(nameof(Diagnostics));

        if (Status == GeometryKernelStatus.Succeeded)
        {
            if (ResultId is null || string.IsNullOrWhiteSpace(EvidenceHash))
                throw new ArgumentException(
                    "Successful box-solid results require result and evidence identities.");

            if (Volume is null || SurfaceArea is null || Centroid is null)
                throw new ArgumentException(
                    "Successful box-solid results require solid metrics.");

            if (Topology.Count != 6)
                throw new ArgumentException(
                    "The bounded box-solid contract requires six face topology bindings.");
        }
    }
}

public interface IAuthoritativeGeometryService
{
    Task<AxisAlignedBoxSolidKernelResult> BuildAxisAlignedBoxSolidAsync(
        AxisAlignedBoxSolidRequest request,
        CancellationToken cancellationToken = default);
}
