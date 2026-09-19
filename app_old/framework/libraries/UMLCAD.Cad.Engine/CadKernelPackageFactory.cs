using System.Globalization;
using System.Text.Json;
using UMLCAD.Cad.Contracts;
using UMLCAD.Cad.Semantics;

namespace UMLCAD.Cad.Engine;

public sealed class CadKernelBuildPackageFactory
{
    private static readonly JsonSerializerOptions JsonOptions = new()
    {
        WriteIndented = false
    };

    public CadBuildDefinition Create(
        CadPartDefinition part,
        CadBuildIdentity identity)
    {
        ArgumentNullException.ThrowIfNull(part);

        var geometry = part.Geometry
            .OrderBy(item => item.Id.Value, StringComparer.Ordinal)
            .Select(ToGeometry)
            .ToArray();

        var constraints = part.Constraints
            .OrderBy(item => item.Id.Value, StringComparer.Ordinal)
            .Select(ToConstraint)
            .ToArray();

        var payload = new
        {
            schema = "uml-cad-build-package/1.0.0",
            applicationId = identity.ApplicationId,
            applicationVersion = identity.ApplicationVersion,
            buildIdentity = identity.SemanticIdentity,
            semantic = new
            {
                id = part.Id.Value,
                version = "1.0.0",
                buildIdentity = identity.SemanticIdentity,
                parts = new[]
                {
                    new
                    {
                        id = part.Id.Value,
                        name = part.Name,
                        geometry,
                        constraints
                    }
                }
            }
        };

        var json = JsonSerializer.Serialize(payload, JsonOptions);
        return new CadBuildDefinition(identity, json);
    }

    private static object ToGeometry(CadPlanarGeometry geometry) =>
        geometry switch
        {
            CadLineSegment line => new
            {
                id = line.Id.Value,
                kind = "line",
                properties = new
                {
                    start = Point(line.Start),
                    end = Point(line.End)
                }
            },
            CadCircle circle => new
            {
                id = circle.Id.Value,
                kind = "circle",
                properties = new
                {
                    center = Point(circle.Center),
                    radius = Number(circle.Radius)
                }
            },
            CadArc arc => new
            {
                id = arc.Id.Value,
                kind = "arc",
                properties = new
                {
                    center = Point(arc.Center),
                    radius = Number(arc.Radius),
                    startAngle = Number(arc.StartAngle),
                    endAngle = Number(arc.EndAngle)
                }
            },
            _ => throw new InvalidOperationException(
                $"Unsupported CAD planar geometry type '{geometry.GetType().FullName}'.")
        };

    private static object ToConstraint(CadPlanarConstraint constraint) =>
        new
        {
            id = constraint.Id.Value,
            kind = constraint.Kind.ToString().ToLowerInvariant(),
            references = new[] { constraint.EntityId.Value }
        };

    private static string Point(CadPoint2D point) =>
        Number(point.X) + "," + Number(point.Y);

    private static string Number(double value) =>
        value.ToString("R", CultureInfo.InvariantCulture);
}
