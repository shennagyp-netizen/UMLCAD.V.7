namespace UMLCAD.Cad.Semantics;

public abstract record FeatureSpecification(
    SemanticId FeatureId,
    SemanticId PartId,
    string Name,
    IReadOnlyList<SemanticId> Dependencies)
{
    public FeatureSpecification
    {
        if (FeatureId.Value == Guid.Empty)
            throw new ArgumentException("FeatureId is required.", nameof(FeatureId));

        if (PartId.Value == Guid.Empty)
            throw new ArgumentException("PartId is required.", nameof(PartId));

        if (string.IsNullOrWhiteSpace(Name))
            throw new ArgumentException("Feature name is required.", nameof(Name));

        Dependencies = Dependencies?.Distinct().ToArray() ??
            throw new ArgumentNullException(nameof(Dependencies));

        if (Dependencies.Contains(FeatureId))
            throw new ArgumentException("A feature cannot depend on itself.", nameof(Dependencies));
    }

    public abstract string OperationKind { get; }

    public abstract string CanonicalDefinition { get; }
}

public sealed record AxisAlignedBoxSolidSpecification(
    SemanticId FeatureId,
    SemanticId PartId,
    string Name,
    IReadOnlyList<SemanticId> Dependencies,
    double MinXmm,
    double MinYmm,
    double MinZmm,
    double MaxXmm,
    double MaxYmm,
    double MaxZmm)
    : FeatureSpecification(FeatureId, PartId, Name, Dependencies)
{
    public AxisAlignedBoxSolidSpecification
    {
        var values = new[] { MinXmm, MinYmm, MinZmm, MaxXmm, MaxYmm, MaxZmm };
        if (values.Any(value => !double.IsFinite(value)))
            throw new ArgumentOutOfRangeException(nameof(MinXmm), "Box coordinates must be finite.");

        if (MaxXmm <= MinXmm || MaxYmm <= MinYmm || MaxZmm <= MinZmm)
            throw new ArgumentException("Box maxima must be strictly greater than minima.", nameof(MaxXmm));
    }

    public override string OperationKind => "PartDesign.AxisAlignedBoxSolid";

    public override string CanonicalDefinition =>
        string.Join(
            ";",
            "uml-cad-axis-box/1",
            FormattableString.Invariant($"min=({MinXmm:R},{MinYmm:R},{MinZmm:R})"),
            FormattableString.Invariant($"max=({MaxXmm:R},{MaxYmm:R},{MaxZmm:R})"));
}
