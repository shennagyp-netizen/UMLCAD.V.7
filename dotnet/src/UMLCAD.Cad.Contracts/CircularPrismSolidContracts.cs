namespace UMLCAD.Cad.Contracts;

public sealed record CircularPrismSolidRequest(
    string OperationIdentity,
    KernelVector3 Origin,
    KernelVector3 Axis,
    double Radius,
    double Depth,
    KernelTolerance Tolerance,
    ContractVersion ContractVersion)
{
    public const string ContractId = "UMLCAD.Geometry.CircularPrismSolid";
    public const string ContractSchema = "uml-cad-circular-prism-solid/1.0.0";

    public CircularPrismSolidRequest
    {
        if (string.IsNullOrWhiteSpace(OperationIdentity))
            throw new ArgumentException("OperationIdentity is required.", nameof(OperationIdentity));

        if (!Origin.IsFinite)
            throw new ArgumentException("Origin must be finite.", nameof(Origin));

        Axis = Axis.Normalize(nameof(Axis));

        if (!double.IsFinite(Radius) || Radius <= 0d)
            throw new ArgumentException("Radius must be finite and positive.", nameof(Radius));

        if (!double.IsFinite(Depth) || Depth <= 0d)
            throw new ArgumentException("Depth must be finite and positive.", nameof(Depth));

        if (ContractVersion.Value != "1.0")
            throw new ArgumentException("Unsupported circular-prism contract version.", nameof(ContractVersion));
    }
}

public sealed record CircularPrismKernelTopology(
    string Kind,
    string Key);

public sealed record CircularPrismSolidKernelResult(
    GeometryKernelStatus Status,
    bool Succeeded,
    ContractResultId? ResultId,
    string? EvidenceHash,
    KernelVector3 Origin,
    KernelVector3 Axis,
    double Radius,
    double Depth,
    double? Volume,
    double? SurfaceArea,
    KernelVector3? Centroid,
    KernelBoundingBox3? Bounds,
    IReadOnlyList<CircularPrismKernelTopology> Topology,
    IReadOnlyList<string> Diagnostics)
{
    public CircularPrismSolidKernelResult
    {
        Topology = Topology?.ToArray()
            ?? throw new ArgumentNullException(nameof(Topology));
        Diagnostics = Diagnostics?.ToArray()
            ?? throw new ArgumentNullException(nameof(Diagnostics));

        if (!Enum.IsDefined(Status))
            throw new ArgumentException("Circular-prism status is invalid.");

        if (Succeeded != (Status == GeometryKernelStatus.Succeeded))
            throw new ArgumentException(
                "Circular-prism status is inconsistent with succeeded.");

        if (Succeeded &&
            (ResultId is null || string.IsNullOrWhiteSpace(EvidenceHash) ||
             Volume is null || SurfaceArea is null || Centroid is null || Bounds is null))
            throw new ArgumentException(
                "Successful circular-prism result is incomplete.");

        if (!double.IsFinite(Radius) || Radius <= 0d ||
            !double.IsFinite(Depth) || Depth <= 0d)
            throw new ArgumentException("Circular-prism dimensions are invalid.");

        if (Succeeded)
        {
            if (Volume < 0d || !double.IsFinite(Volume.Value) ||
                SurfaceArea < 0d || !double.IsFinite(SurfaceArea.Value))
                throw new ArgumentException("Circular-prism measures are invalid.");

            Bounds!.Validate();
            if (!Centroid!.Value.IsFinite)
                throw new ArgumentException("Circular-prism centroid must be finite.");
        }
    }
}

public interface ICircularPrismGeometryService
{
    Task<CircularPrismSolidKernelResult> BuildCircularPrismSolidAsync(
        CircularPrismSolidRequest request,
        CancellationToken cancellationToken = default);
}
