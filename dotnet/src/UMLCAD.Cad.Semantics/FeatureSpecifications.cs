namespace UMLCAD.Cad.Semantics;

public abstract record FeatureSpecification
{
    public SemanticId FeatureId { get; }
    public SemanticId PartId { get; }
    public string Name { get; }
    public IReadOnlyList<SemanticId> Dependencies { get; }

    protected FeatureSpecification(
        SemanticId featureId,
        SemanticId partId,
        string name,
        IReadOnlyList<SemanticId> dependencies)
    {
        if (featureId.Value == Guid.Empty)
            throw new ArgumentException("FeatureId is required.", nameof(featureId));
        if (partId.Value == Guid.Empty)
            throw new ArgumentException("PartId is required.", nameof(partId));
        if (string.IsNullOrWhiteSpace(name))
            throw new ArgumentException("Feature name is required.", nameof(name));

        Dependencies = dependencies?.Distinct().ToArray()
            ?? throw new ArgumentNullException(nameof(dependencies));

        if (Dependencies.Contains(featureId))
            throw new ArgumentException("A feature cannot depend on itself.", nameof(dependencies));

        FeatureId = featureId;
        PartId = partId;
        Name = name;
    }

    public abstract string OperationKind { get; }

    public abstract string CanonicalDefinition { get; }
}

public sealed record AxisAlignedBoxSolidSpecification : FeatureSpecification
{
    public double MinXmm { get; }
    public double MinYmm { get; }
    public double MinZmm { get; }
    public double MaxXmm { get; }
    public double MaxYmm { get; }
    public double MaxZmm { get; }

    public AxisAlignedBoxSolidSpecification(
        SemanticId featureId,
        SemanticId partId,
        string name,
        IReadOnlyList<SemanticId> dependencies,
        double minXmm,
        double minYmm,
        double minZmm,
        double maxXmm,
        double maxYmm,
        double maxZmm)
        : base(featureId, partId, name, dependencies)
    {
        var values = new[] { minXmm, minYmm, minZmm, maxXmm, maxYmm, maxZmm };
        if (values.Any(value => !double.IsFinite(value)))
            throw new ArgumentOutOfRangeException(nameof(minXmm), "Box coordinates must be finite.");

        if (maxXmm <= minXmm || maxYmm <= minYmm || maxZmm <= minZmm)
            throw new ArgumentException(
                "Box maxima must be strictly greater than minima.",
                nameof(maxXmm));

        MinXmm = minXmm;
        MinYmm = minYmm;
        MinZmm = minZmm;
        MaxXmm = maxXmm;
        MaxYmm = maxYmm;
        MaxZmm = maxZmm;
    }

    public override string OperationKind => "PartDesign.AxisAlignedBoxSolid";

    public override string CanonicalDefinition =>
        string.Join(
            ";",
            "uml-cad-axis-box/1",
            FormattableString.Invariant($"min=({MinXmm:R},{MinYmm:R},{MinZmm:R})"),
            FormattableString.Invariant($"max=({MaxXmm:R},{MaxYmm:R},{MaxZmm:R})"));
}

public readonly record struct SemanticVector3
{
    public double X { get; }
    public double Y { get; }
    public double Z { get; }

    public SemanticVector3(double x, double y, double z)
    {
        if (!double.IsFinite(x) || !double.IsFinite(y) || !double.IsFinite(z))
            throw new ArgumentOutOfRangeException(
                nameof(x),
                "Coordinate components must be finite.");

        X = x;
        Y = y;
        Z = z;
    }
}

public readonly record struct SketchProfilePoint
{
    public double U { get; }
    public double V { get; }

    public SketchProfilePoint(double u, double v)
    {
        if (!double.IsFinite(u) || !double.IsFinite(v))
            throw new ArgumentOutOfRangeException(
                nameof(u),
                "Profile coordinates must be finite.");

        U = u;
        V = v;
    }
}

public sealed record ConvexSketchProfileDefinition
{
    public SemanticId ProfileId { get; }
    public SemanticId PartId { get; }
    public string Name { get; }
    public SemanticVector3 OriginMm { get; }
    public SemanticVector3 UDirection { get; }
    public SemanticVector3 VDirection { get; }
    public IReadOnlyList<SketchProfilePoint> Points { get; }

    public ConvexSketchProfileDefinition(
        SemanticId profileId,
        SemanticId partId,
        string name,
        SemanticVector3 originMm,
        SemanticVector3 uDirection,
        SemanticVector3 vDirection,
        IReadOnlyList<SketchProfilePoint> points)
    {
        if (profileId.Value == Guid.Empty)
            throw new ArgumentException("ProfileId is required.", nameof(profileId));
        if (partId.Value == Guid.Empty)
            throw new ArgumentException("PartId is required.", nameof(partId));
        if (string.IsNullOrWhiteSpace(name))
            throw new ArgumentException("Profile name is required.", nameof(name));

        Points = points?.ToArray()
            ?? throw new ArgumentNullException(nameof(points));

        if (Points.Count < 3)
            throw new ArgumentException(
                "A convex profile requires at least three points.",
                nameof(points));

        var uLength =
            (uDirection.X * uDirection.X) +
            (uDirection.Y * uDirection.Y) +
            (uDirection.Z * uDirection.Z);
        var vLength =
            (vDirection.X * vDirection.X) +
            (vDirection.Y * vDirection.Y) +
            (vDirection.Z * vDirection.Z);
        var dot =
            (uDirection.X * vDirection.X) +
            (uDirection.Y * vDirection.Y) +
            (uDirection.Z * vDirection.Z);

        if (!double.IsFinite(uLength) ||
            !double.IsFinite(vLength) ||
            !double.IsFinite(dot) ||
            Math.Abs(uLength - 1d) > 1e-12 ||
            Math.Abs(vLength - 1d) > 1e-12 ||
            Math.Abs(dot) > 1e-12)
        {
            throw new ArgumentException(
                "Sketch U/V directions must form an orthonormal frame.",
                nameof(uDirection));
        }

        ProfileId = profileId;
        PartId = partId;
        Name = name;
        OriginMm = originMm;
        UDirection = uDirection;
        VDirection = vDirection;
    }
}

public sealed record ExtrusionFeatureSpecification : FeatureSpecification
{
    public ConvexSketchProfileDefinition Profile { get; }
    public double DepthMm { get; }

    public ExtrusionFeatureSpecification(
        SemanticId featureId,
        SemanticId partId,
        string name,
        IReadOnlyList<SemanticId> dependencies,
        ConvexSketchProfileDefinition profile,
        double depthMm)
        : base(featureId, partId, name, dependencies)
    {
        ArgumentNullException.ThrowIfNull(profile);

        if (profile.PartId != partId)
            throw new ArgumentException(
                "Extrusion profile must belong to the same part as the extrusion feature.",
                nameof(profile));

        if (!double.IsFinite(depthMm) || depthMm <= 0d)
            throw new ArgumentOutOfRangeException(nameof(depthMm));

        Profile = profile;
        DepthMm = depthMm;
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
            $"v=({Profile.VDirection.X:R},{Profile.VDirection.Y:R},{Profile.UDirection.Z:R})",
            $"points={string.Join(",", Profile.Points.Select(p => $"({p.U:R},{p.V:R})"))}");
}
