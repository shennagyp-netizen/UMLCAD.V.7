using System.Collections.ObjectModel;
using System.Globalization;
using System.Text;
using UMLCAD.Cad.Contracts;
using UMLCAD.Cad.Expressions;

namespace UMLCAD.Cad.Semantics;

public enum SketchGeometryKind
{
    Line,
    Circle,
    Arc
}

public enum SketchConstraintKind
{
    Horizontal,
    Vertical,
    Coincident,
    Fixed,
    Distance,
    Radius,
    Diameter,
    Tangent
}

public enum ReferenceTargetKind
{
    OperationResult,
    Topology,
    Publication,
    Semantic
}

public abstract record SketchGeometry
{
    protected SketchGeometry(CadId id, SketchGeometryKind kind)
    {
        if (!id.IsValid)
            throw new ArgumentException("Geometry identity is required.", nameof(id));

        Id = id;
        Kind = kind;
    }

    public CadId Id { get; init; }
    public SketchGeometryKind Kind { get; init; }
}

public sealed record LineGeometry : SketchGeometry
{
    public LineGeometry(CadId id, double x1, double y1, double x2, double y2)
        : base(id, SketchGeometryKind.Line)
    {
        ValidateFinite(x1, nameof(x1));
        ValidateFinite(y1, nameof(y1));
        ValidateFinite(x2, nameof(x2));
        ValidateFinite(y2, nameof(y2));

        if (x1 == x2 && y1 == y2)
            throw new ArgumentException("Line endpoints must be distinct.");

        X1 = x1;
        Y1 = y1;
        X2 = x2;
        Y2 = y2;
    }

    public double X1 { get; init; }
    public double Y1 { get; init; }
    public double X2 { get; init; }
    public double Y2 { get; init; }

    private static void ValidateFinite(double value, string name)
    {
        if (!double.IsFinite(value))
            throw new ArgumentOutOfRangeException(name);
    }
}

public sealed record CircleGeometry : SketchGeometry
{
    public CircleGeometry(CadId id, double x, double y, double radius)
        : base(id, SketchGeometryKind.Circle)
    {
        if (!double.IsFinite(x) || !double.IsFinite(y))
            throw new ArgumentException("Circle center must be finite.");

        if (!double.IsFinite(radius) || radius <= 0)
            throw new ArgumentOutOfRangeException(nameof(radius));

        X = x;
        Y = y;
        Radius = radius;
    }

    public double X { get; init; }
    public double Y { get; init; }
    public double Radius { get; init; }
}

public sealed record ArcGeometry : SketchGeometry
{
    public ArcGeometry(
        CadId id,
        double centerX,
        double centerY,
        double radius,
        double startAngle,
        double endAngle)
        : base(id, SketchGeometryKind.Arc)
    {
        foreach (var value in new[] { centerX, centerY, radius, startAngle, endAngle })
        {
            if (!double.IsFinite(value))
                throw new ArgumentException("Arc geometry values must be finite.");
        }

        if (radius <= 0)
            throw new ArgumentOutOfRangeException(nameof(radius));

        CenterX = centerX;
        CenterY = centerY;
        Radius = radius;
        StartAngle = startAngle;
        EndAngle = endAngle;
    }

    public double CenterX { get; init; }
    public double CenterY { get; init; }
    public double Radius { get; init; }
    public double StartAngle { get; init; }
    public double EndAngle { get; init; }
}

public sealed record SketchConstraint
{
    public SketchConstraint(
        CadId id,
        SketchConstraintKind kind,
        IReadOnlyList<CadId> geometryIds,
        double? value = null)
    {
        if (!id.IsValid)
            throw new ArgumentException("Constraint identity is required.", nameof(id));

        ArgumentNullException.ThrowIfNull(geometryIds);

        if (geometryIds.Any(x => !x.IsValid))
            throw new ArgumentException(
                "Constraint geometry references must be valid.",
                nameof(geometryIds));

        if (value is not null && !double.IsFinite(value.Value))
            throw new ArgumentOutOfRangeException(nameof(value));

        Id = id;
        Kind = kind;
        GeometryIds = geometryIds.ToArray();
        Value = value;
    }

    public CadId Id { get; init; }
    public SketchConstraintKind Kind { get; init; }
    public IReadOnlyList<CadId> GeometryIds { get; init; }
    public double? Value { get; init; }
}

public sealed record CadReference
{
    public CadReference(
        CadId id,
        ReferenceTargetKind targetKind,
        CadId? operationId = null,
        string? topologyKind = null,
        string? topologyKey = null,
        CadId? publicationId = null)
    {
        if (!id.IsValid)
            throw new ArgumentException("Reference identity is required.", nameof(id));

        Id = id;
        TargetKind = targetKind;
        OperationId = operationId;
        TopologyKind = topologyKind;
        TopologyKey = topologyKey;
        PublicationId = publicationId;
    }

    public CadId Id { get; init; }
    public ReferenceTargetKind TargetKind { get; init; }
    public CadId? OperationId { get; init; }
    public string? TopologyKind { get; init; }
    public string? TopologyKey { get; init; }
    public CadId? PublicationId { get; init; }
}

public sealed record Publication
{
    public Publication(
        CadId id,
        string name,
        CadId producingOperationId,
        string topologyKind,
        string topologyKey)
    {
        if (!id.IsValid ||
            !producingOperationId.IsValid ||
            string.IsNullOrWhiteSpace(name) ||
            string.IsNullOrWhiteSpace(topologyKind) ||
            string.IsNullOrWhiteSpace(topologyKey))
        {
            throw new ArgumentException("Publication is incomplete.");
        }

        Id = id;
        Name = name;
        ProducingOperationId = producingOperationId;
        TopologyKind = topologyKind;
        TopologyKey = topologyKey;
    }

    public CadId Id { get; init; }
    public string Name { get; init; }
    public CadId ProducingOperationId { get; init; }
    public string TopologyKind { get; init; }
    public string TopologyKey { get; init; }
}

public sealed record Sketch
{
    public Sketch(
        CadId id,
        string name,
        IReadOnlyList<SketchGeometry> geometry,
        IReadOnlyList<SketchConstraint> constraints,
        IReadOnlyList<CadReference> supports)
    {
        if (!id.IsValid || string.IsNullOrWhiteSpace(name))
            throw new ArgumentException("Sketch identity/name is required.");

        ArgumentNullException.ThrowIfNull(geometry);
        ArgumentNullException.ThrowIfNull(constraints);
        ArgumentNullException.ThrowIfNull(supports);

        if (geometry.Any(x => x is null))
            throw new ArgumentException("Sketch geometry cannot contain null values.", nameof(geometry));

        if (constraints.Any(x => x is null))
            throw new ArgumentException("Sketch constraints cannot contain null values.", nameof(constraints));

        if (supports.Any(x => x is null))
            throw new ArgumentException("Sketch references cannot contain null values.", nameof(supports));

        if (geometry.Select(x => x.Id).Distinct().Count() != geometry.Count)
            throw new ArgumentException("Duplicate sketch geometry IDs.");

        if (constraints.Select(x => x.Id).Distinct().Count() != constraints.Count)
            throw new ArgumentException("Duplicate sketch constraint IDs.");

        Id = id;
        Name = name;
        Geometry = geometry.ToArray();
        Constraints = constraints.ToArray();
        Supports = supports.ToArray();
    }

    public CadId Id { get; init; }
    public string Name { get; init; }
    public IReadOnlyList<SketchGeometry> Geometry { get; init; }
    public IReadOnlyList<SketchConstraint> Constraints { get; init; }
    public IReadOnlyList<CadReference> Supports { get; init; }
}

public sealed record BodyDefinition
{
    public BodyDefinition(CadId id, string name)
    {
        if (!id.IsValid || string.IsNullOrWhiteSpace(name))
            throw new ArgumentException("Body identity/name is required.", nameof(id));

        Id = id;
        Name = name;
    }

    public CadId Id { get; init; }
    public string Name { get; init; }
}

public abstract record CadOperation
{
    protected CadOperation(
        CadId id,
        CadId bodyId,
        string operationKind,
        IReadOnlyList<CadId> inputOperationIds)
    {
        if (!id.IsValid || !bodyId.IsValid)
            throw new ArgumentException("Operation and body identities are required.");

        if (string.IsNullOrWhiteSpace(operationKind))
            throw new ArgumentException("Operation kind is required.", nameof(operationKind));

        ArgumentNullException.ThrowIfNull(inputOperationIds);

        if (inputOperationIds.Any(x => !x.IsValid))
            throw new ArgumentException("Operation input identities must be valid.", nameof(inputOperationIds));

        Id = id;
        BodyId = bodyId;
        OperationKind = operationKind;
        InputOperationIds = inputOperationIds.ToArray();
    }

    public CadId Id { get; init; }
    public CadId BodyId { get; init; }
    public string OperationKind { get; init; }
    public IReadOnlyList<CadId> InputOperationIds { get; init; }
    public IReadOnlyList<CadReference> References { get; init; } = Array.Empty<CadReference>();
    public IReadOnlyDictionary<string, string> SemanticInputs { get; init; } =
        new ReadOnlyDictionary<string, string>(
            new Dictionary<string, string>(StringComparer.Ordinal));

    public virtual string CanonicalDefinition()
    {
        var builder = new StringBuilder(OperationKind)
            .Append('|')
            .Append(Id.Value)
            .Append('|')
            .Append(BodyId.Value);

        foreach (var input in InputOperationIds.OrderBy(x => x.Value, StringComparer.Ordinal))
            builder.Append("|input-op=").Append(input.Value);

        foreach (var reference in References.OrderBy(x => x.Id.Value, StringComparer.Ordinal))
        {
            builder.Append("|ref=")
                .Append(reference.Id.Value)
                .Append(':')
                .Append(reference.TargetKind)
                .Append(':')
                .Append(reference.OperationId?.Value ?? "-")
                .Append(':')
                .Append(reference.TopologyKind ?? "-")
                .Append(':')
                .Append(reference.TopologyKey ?? "-")
                .Append(':')
                .Append(reference.PublicationId?.Value ?? "-");
        }

        foreach (var input in SemanticInputs.OrderBy(x => x.Key, StringComparer.Ordinal))
            builder.Append("|input=").Append(input.Key).Append('=').Append(input.Value);

        return builder.ToString();
    }
}

public sealed record SketchOperation : CadOperation
{
    public SketchOperation(CadId id, CadId bodyId, Sketch sketch)
        : base(id, bodyId, "Cad.Sketch", Array.Empty<CadId>())
    {
        ArgumentNullException.ThrowIfNull(sketch);
        Sketch = sketch;
    }

    public Sketch Sketch { get; init; }

    public override string CanonicalDefinition()
    {
        var builder = new StringBuilder(base.CanonicalDefinition())
            .Append("|sketch.id=").Append(Sketch.Id.Value)
            .Append("|sketch.name=").Append(Sketch.Name);

        foreach (var geometry in Sketch.Geometry.OrderBy(x => x.Id.Value, StringComparer.Ordinal))
        {
            builder.Append("|geometry=")
                .Append(geometry.Id.Value)
                .Append(':')
                .Append(geometry.Kind);

            switch (geometry)
            {
                case LineGeometry line:
                    builder.Append(':').Append(Number(line.X1))
                        .Append(':').Append(Number(line.Y1))
                        .Append(':').Append(Number(line.X2))
                        .Append(':').Append(Number(line.Y2));
                    break;

                case CircleGeometry circle:
                    builder.Append(':').Append(Number(circle.X))
                        .Append(':').Append(Number(circle.Y))
                        .Append(':').Append(Number(circle.Radius));
                    break;

                case ArcGeometry arc:
                    builder.Append(':').Append(Number(arc.CenterX))
                        .Append(':').Append(Number(arc.CenterY))
                        .Append(':').Append(Number(arc.Radius))
                        .Append(':').Append(Number(arc.StartAngle))
                        .Append(':').Append(Number(arc.EndAngle));
                    break;
            }
        }

        foreach (var constraint in Sketch.Constraints.OrderBy(x => x.Id.Value, StringComparer.Ordinal))
        {
            builder.Append("|constraint=")
                .Append(constraint.Id.Value)
                .Append(':')
                .Append(constraint.Kind)
                .Append(':')
                .Append(Number(constraint.Value));

            foreach (var geometryId in constraint.GeometryIds.OrderBy(x => x.Value, StringComparer.Ordinal))
                builder.Append(':').Append(geometryId.Value);
        }

        foreach (var support in Sketch.Supports.OrderBy(x => x.Id.Value, StringComparer.Ordinal))
        {
            builder.Append("|support=")
                .Append(support.Id.Value)
                .Append(':')
                .Append(support.TargetKind)
                .Append(':')
                .Append(support.OperationId?.Value ?? "-")
                .Append(':')
                .Append(support.TopologyKind ?? "-")
                .Append(':')
                .Append(support.TopologyKey ?? "-")
                .Append(':')
                .Append(support.PublicationId?.Value ?? "-");
        }

        return builder.ToString();
    }

    private static string Number(double? value) =>
        value?.ToString("R", CultureInfo.InvariantCulture) ?? "-";

    private static string Number(double value) =>
        value.ToString("R", CultureInfo.InvariantCulture);
}

public sealed record ExtrusionOperation : CadOperation
{
    public ExtrusionOperation(
        CadId id,
        CadId bodyId,
        CadId sketchOperationId,
        CadExpression distance,
        string direction)
        : base(id, bodyId, "Cad.Extrusion", new[] { sketchOperationId })
    {
        ArgumentNullException.ThrowIfNull(distance);

        if (!sketchOperationId.IsValid || string.IsNullOrWhiteSpace(direction))
            throw new ArgumentException("Extrusion input is incomplete.");

        SketchOperationId = sketchOperationId;
        Distance = distance;
        Direction = direction;

        SemanticInputs = new ReadOnlyDictionary<string, string>(
            new Dictionary<string, string>(StringComparer.Ordinal)
            {
                ["distance"] = distance.CanonicalForm,
                ["direction"] = direction
            });
    }

    public CadId SketchOperationId { get; init; }
    public CadExpression Distance { get; init; }
    public string Direction { get; init; }
}

public sealed record HoleOperation : CadOperation
{
    public HoleOperation(
        CadId id,
        CadId bodyId,
        CadId baseOperationId,
        CadExpression diameter,
        CadExpression depth)
        : base(id, bodyId, "Cad.Hole", new[] { baseOperationId })
    {
        ArgumentNullException.ThrowIfNull(diameter);
        ArgumentNullException.ThrowIfNull(depth);

        if (!baseOperationId.IsValid)
            throw new ArgumentException("Hole input is incomplete.", nameof(baseOperationId));

        BaseOperationId = baseOperationId;
        Diameter = diameter;
        Depth = depth;

        SemanticInputs = new ReadOnlyDictionary<string, string>(
            new Dictionary<string, string>(StringComparer.Ordinal)
            {
                ["diameter"] = diameter.CanonicalForm,
                ["depth"] = depth.CanonicalForm
            });
    }

    public CadId BaseOperationId { get; init; }
    public CadExpression Diameter { get; init; }
    public CadExpression Depth { get; init; }
}

public sealed record CadResult(
    CadResultId Id,
    CadResultKind Kind,
    CadId ProducingOperationId,
    IReadOnlyList<CadResultId> InputResults,
    string EvidenceHash,
    IReadOnlyList<KernelTopologyBinding> Topology);

public sealed record CadPart
{
    public CadPart(
        CadId id,
        string name,
        IReadOnlyList<BodyDefinition> bodies,
        IReadOnlyList<CadParameterBinding> parameters,
        IReadOnlyList<Publication> publications,
        IReadOnlyList<CadOperation> operations)
    {
        if (!id.IsValid || string.IsNullOrWhiteSpace(name))
            throw new ArgumentException("Part identity/name is required.");

        ArgumentNullException.ThrowIfNull(bodies);
        ArgumentNullException.ThrowIfNull(parameters);
        ArgumentNullException.ThrowIfNull(publications);
        ArgumentNullException.ThrowIfNull(operations);

        Unique(bodies.Select(x => x.Id), "body");
        Unique(parameters.Select(x => x.Name), "parameter");
        Unique(publications.Select(x => x.Id), "publication");
        Unique(operations.Select(x => x.Id), "operation");

        var bodyIds = bodies.Select(x => x.Id).ToHashSet();
        if (operations.Any(x => !bodyIds.Contains(x.BodyId)))
            throw new ArgumentException("Operation references unknown body.");

        Id = id;
        Name = name;
        Bodies = bodies.ToArray();
        Parameters = parameters.ToArray();
        Publications = publications.ToArray();
        Operations = operations.ToArray();
    }

    public CadId Id { get; init; }
    public string Name { get; init; }
    public IReadOnlyList<BodyDefinition> Bodies { get; init; }
    public IReadOnlyList<CadParameterBinding> Parameters { get; init; }
    public IReadOnlyList<Publication> Publications { get; init; }
    public IReadOnlyList<CadOperation> Operations { get; init; }

    private static void Unique<T>(IEnumerable<T> values, string kind)
    {
        var items = values.ToArray();
        if (items.Distinct().Count() != items.Length)
            throw new ArgumentException($"Duplicate {kind} identity.");
    }
}

public sealed record CadPartProgram(CadPart Part)
{
    public static CadPartProgram Create(
        string id,
        string name,
        string bodyId = "body") =>
        new(
            new CadPart(
                new CadId(id),
                name,
                new[] { new BodyDefinition(new CadId(bodyId), "Main Body") },
                Array.Empty<CadParameterBinding>(),
                Array.Empty<Publication>(),
                Array.Empty<CadOperation>()));

    public CadPartProgram Parameter(CadParameterBinding parameter)
    {
        ArgumentNullException.ThrowIfNull(parameter);

        return this with
        {
            Part = Part with
            {
                Parameters = Part.Parameters.Append(parameter).ToArray()
            }
        };
    }

    public CadPartProgram Publish(Publication publication)
    {
        ArgumentNullException.ThrowIfNull(publication);

        return this with
        {
            Part = Part with
            {
                Publications = Part.Publications.Append(publication).ToArray()
            }
        };
    }

    public CadPartProgram Sketch(SketchOperation operation) =>
        AddOperation(operation);

    public CadPartProgram Extrude(ExtrusionOperation operation) =>
        AddOperation(operation);

    public CadPartProgram Hole(HoleOperation operation) =>
        AddOperation(operation);

    public CadPartProgram AddOperation(CadOperation operation)
    {
        ArgumentNullException.ThrowIfNull(operation);

        if (Part.Operations.Any(existing => existing.Id == operation.Id))
            throw new InvalidOperationException(
                $"Duplicate operation '{operation.Id.Value}'.");

        return this with
        {
            Part = Part with
            {
                Operations = Part.Operations.Append(operation).ToArray()
            }
        };
    }
}
