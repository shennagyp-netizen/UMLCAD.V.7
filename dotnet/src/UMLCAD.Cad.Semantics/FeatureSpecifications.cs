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


public readonly record struct SemanticVector3(double X, double Y, double Z)
{
    public SemanticVector3
    {
        if (!double.IsFinite(X) || !double.IsFinite(Y) || !double.IsFinite(Z))
            throw new ArgumentOutOfRangeException(nameof(X), "Coordinate components must be finite.");
    }
}

public readonly record struct SketchProfilePoint(double U, double V)
{
    public SketchProfilePoint
    {
        if (!double.IsFinite(U) || !double.IsFinite(V))
            throw new ArgumentOutOfRangeException(nameof(U), "Profile coordinates must be finite.");
    }
}

public sealed record ConvexSketchProfileDefinition(
    SemanticId ProfileId,
    SemanticId PartId,
    string Name,
    SemanticVector3 OriginMm,
    SemanticVector3 UDirection,
    SemanticVector3 VDirection,
    IReadOnlyList<SketchProfilePoint> Points)
{
    public ConvexSketchProfileDefinition
    {
        if (ProfileId.Value == Guid.Empty)
            throw new ArgumentException("ProfileId is required.", nameof(ProfileId));
        if (PartId.Value == Guid.Empty)
            throw new ArgumentException("PartId is required.", nameof(PartId));
        if (string.IsNullOrWhiteSpace(Name))
            throw new ArgumentException("Profile name is required.", nameof(Name));

        Points = Points?.ToArray() ??
            throw new ArgumentNullException(nameof(Points));

        if (Points.Count < 3)
            throw new ArgumentException("A convex profile requires at least three points.", nameof(Points));

        var uLength = UDirection.X * UDirection.X + UDirection.Y * UDirection.Y + UDirection.Z * UDirection.Z;
        var vLength = VDirection.X * VDirection.X + VDirection.Y * VDirection.Y + VDirection.Z * VDirection.Z;
        var dot = UDirection.X * VDirection.X + UDirection.Y * VDirection.Y + UDirection.Z * VDirection.Z;

        if (!double.IsFinite(uLength) || !double.IsFinite(vLength) ||
            !double.IsFinite(dot) ||
            Math.Abs(uLength - 1d) > 1e-12 ||
            Math.Abs(vLength - 1d) > 1e-12 ||
            Math.Abs(dot) > 1e-12)
        {
            throw new ArgumentException(
                "Sketch U/V directions must form an orthonormal frame.",
                nameof(UDirection));
        }
    }
}

public sealed record ExtrusionFeatureSpecification(
    SemanticId FeatureId,
    SemanticId PartId,
    string Name,
    IReadOnlyList<SemanticId> Dependencies,
    ConvexSketchProfileDefinition Profile,
    double DepthMm)
    : FeatureSpecification(FeatureId, PartId, Name, Dependencies)
{
    public ExtrusionFeatureSpecification
    {
        ArgumentNullException.ThrowIfNull(Profile);

        if (Profile.PartId != PartId)
            throw new ArgumentException(
                "Extrusion profile must belong to the same part as the extrusion feature.",
                nameof(Profile));

        if (!double.IsFinite(DepthMm) || DepthMm <= 0d)
            throw new ArgumentOutOfRangeException(nameof(DepthMm));
    }

    public override string OperationKind => "PartDesign.ExtrudeConvexPlanarProfile";

    public override string CanonicalDefinition =>
        string.Join(
            ";",
            "uml-cad-convex-extrusion/1",
            $"profile={Profile.ProfileId.Value:D}",
            $"depth={DepthMm:R}",
            $"origin=({Profile.OriginMm.X:R},{Profile.OriginMm.Y:R},{Profile.OriginMm.Z:R})",
            $"u=({Profile.UDirection.X:R},{Profile.UDirection.Y:R},{Profile.UDirection.Z:R})",
            $"v=({Profile.VDirection.X:R},{Profile.VDirection.Y:R},{Profile.VDirection.Z:R})",
            $"points={string.Join(",", Profile.Points.Select(p => $"({p.U:R},{p.V:R})"))}");
}
