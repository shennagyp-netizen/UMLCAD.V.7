using System.Collections.ObjectModel;
using UMLCAD.Cad.Contracts;

namespace UMLCAD.Cad.Semantics;

public readonly record struct CadPoint2D(double X, double Y)
{
    public CadPoint2D
    {
        if (!double.IsFinite(X) || !double.IsFinite(Y))
            throw new ArgumentOutOfRangeException(
                nameof(X),
                "Planar point coordinates must be finite.");
    }
}

public abstract record CadPlanarGeometry
{
    protected CadPlanarGeometry(CadId id)
    {
        Id = id;
    }

    public CadId Id { get; }
}

public sealed record CadLineSegment(
    CadId GeometryId,
    CadPoint2D Start,
    CadPoint2D End) : CadPlanarGeometry(GeometryId)
{
    public CadLineSegment
    {
        if (Start == End)
            throw new ArgumentException(
                "A line segment requires distinct endpoints.",
                nameof(End));
    }
}

public sealed record CadCircle(
    CadId GeometryId,
    CadPoint2D Center,
    double Radius) : CadPlanarGeometry(GeometryId)
{
    public CadCircle
    {
        if (!double.IsFinite(Radius) || Radius <= 0)
            throw new ArgumentOutOfRangeException(
                nameof(Radius),
                "Circle radius must be finite and positive.");
    }
}

public sealed record CadArc(
    CadId GeometryId,
    CadPoint2D Center,
    double Radius,
    double StartAngle,
    double EndAngle) : CadPlanarGeometry(GeometryId)
{
    public CadArc
    {
        if (!double.IsFinite(Radius) || Radius <= 0)
            throw new ArgumentOutOfRangeException(
                nameof(Radius),
                "Arc radius must be finite and positive.");

        if (!double.IsFinite(StartAngle) || !double.IsFinite(EndAngle))
            throw new ArgumentOutOfRangeException(
                nameof(StartAngle),
                "Arc angles must be finite.");
    }
}

public enum CadPlanarConstraintKind
{
    Horizontal,
    Vertical,
    Fixed
}

public sealed record CadPlanarConstraint(
    CadId Id,
    CadPlanarConstraintKind Kind,
    CadId EntityId);

public sealed record CadPartDefinition
{
    public CadPartDefinition(
        CadId id,
        string name,
        IReadOnlyList<CadPlanarGeometry> geometry,
        IReadOnlyList<CadPlanarConstraint>? constraints = null)
    {
        if (string.IsNullOrWhiteSpace(name))
            throw new ArgumentException("Part name cannot be empty.", nameof(name));

        ArgumentNullException.ThrowIfNull(geometry);
        constraints ??= [];

        var copiedGeometry =
            new ReadOnlyCollection<CadPlanarGeometry>(geometry.ToArray());
        var copiedConstraints =
            new ReadOnlyCollection<CadPlanarConstraint>(constraints.ToArray());

        var geometryIds = copiedGeometry
            .Select(item => item.Id)
            .ToArray();
        if (geometryIds.Distinct().Count() != geometryIds.Length)
            throw new ArgumentException(
                "Part geometry IDs must be unique.",
                nameof(geometry));

        var constraintIds = copiedConstraints
            .Select(item => item.Id)
            .ToArray();
        if (constraintIds.Distinct().Count() != constraintIds.Length)
            throw new ArgumentException(
                "Part constraint IDs must be unique.",
                nameof(constraints));

        var knownGeometry =
            geometryIds.ToHashSet();
        var unknownConstraintEntities = copiedConstraints
            .Where(constraint => !knownGeometry.Contains(constraint.EntityId))
            .Select(constraint => constraint.EntityId.Value)
            .Distinct(StringComparer.Ordinal)
            .ToArray();

        if (unknownConstraintEntities.Length > 0)
            throw new ArgumentException(
                "Constraints reference unknown geometry: " +
                string.Join(", ", unknownConstraintEntities),
                nameof(constraints));

        Id = id;
        Name = name;
        Geometry = copiedGeometry;
        Constraints = copiedConstraints;
    }

    public CadId Id { get; }

    public string Name { get; }

    public IReadOnlyList<CadPlanarGeometry> Geometry { get; }

    public IReadOnlyList<CadPlanarConstraint> Constraints { get; }
}
