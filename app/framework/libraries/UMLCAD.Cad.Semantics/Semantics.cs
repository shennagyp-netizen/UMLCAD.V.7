using System.Text;
using UMLCAD.Cad.Contracts;
using UMLCAD.Cad.Expressions;

namespace UMLCAD.Cad.Semantics;

public enum SketchGeometryKind { Line, Circle, Arc }
public enum SketchConstraintKind { Horizontal, Vertical, Coincident, Fixed, Distance, Radius, Diameter, Tangent }
public enum ReferenceTargetKind { Result, Topology, Publication, Semantic }

public abstract record SketchGeometry(
    CadId Id,
    SketchGeometryKind Kind)
{
    protected SketchGeometry
    {
        if (!Id.IsValid)
            throw new ArgumentException("Geometry ID is required.", nameof(Id));
    }

    public abstract string CanonicalForm { get; }
    public abstract IReadOnlySet<string> ParameterNames { get; }
}

public sealed record LineGeometry(
    CadId Id,
    UMLCAD.Cad.Expressions.CadNumericValue X1,
    UMLCAD.Cad.Expressions.CadNumericValue Y1,
    UMLCAD.Cad.Expressions.CadNumericValue X2,
    UMLCAD.Cad.Expressions.CadNumericValue Y2)
    : SketchGeometry(Id, SketchGeometryKind.Line)
{
    public override string CanonicalForm =>
        $"line:{Id}:{X1.CanonicalForm}:{Y1.CanonicalForm}:{X2.CanonicalForm}:{Y2.CanonicalForm}";

    public override IReadOnlySet<string> ParameterNames =>
        X1.ParameterNames
            .Concat(Y1.ParameterNames)
            .Concat(X2.ParameterNames)
            .Concat(Y2.ParameterNames)
            .ToHashSet(StringComparer.Ordinal);

    public LineGeometry(
        CadId id,
        double x1,
        double y1,
        double x2,
        double y2)
        : this(
            id,
            UMLCAD.Cad.Expressions.CadNumericValue.Constant(x1),
            UMLCAD.Cad.Expressions.CadNumericValue.Constant(y1),
            UMLCAD.Cad.Expressions.CadNumericValue.Constant(x2),
            UMLCAD.Cad.Expressions.CadNumericValue.Constant(y2))
    {
    }
}

public sealed record CircleGeometry(
    CadId Id,
    UMLCAD.Cad.Expressions.CadNumericValue X,
    UMLCAD.Cad.Expressions.CadNumericValue Y,
    UMLCAD.Cad.Expressions.CadNumericValue Radius)
    : SketchGeometry(Id, SketchGeometryKind.Circle)
{
    public CircleGeometry
    {
        ArgumentNullException.ThrowIfNull(X);
        ArgumentNullException.ThrowIfNull(Y);
        ArgumentNullException.ThrowIfNull(Radius);
    }

    public override string CanonicalForm =>
        $"circle:{Id}:{X.CanonicalForm}:{Y.CanonicalForm}:{Radius.CanonicalForm}";

    public override IReadOnlySet<string> ParameterNames =>
        X.ParameterNames
            .Concat(Y.ParameterNames)
            .Concat(Radius.ParameterNames)
            .ToHashSet(StringComparer.Ordinal);

    public CircleGeometry(
        CadId id,
        double x,
        double y,
        double radius)
        : this(
            id,
            UMLCAD.Cad.Expressions.CadNumericValue.Constant(x),
            UMLCAD.Cad.Expressions.CadNumericValue.Constant(y),
            UMLCAD.Cad.Expressions.CadNumericValue.Constant(radius))
    {
    }
}

public sealed record ArcGeometry(
    CadId Id,
    UMLCAD.Cad.Expressions.CadNumericValue CenterX,
    UMLCAD.Cad.Expressions.CadNumericValue CenterY,
    UMLCAD.Cad.Expressions.CadNumericValue Radius,
    UMLCAD.Cad.Expressions.CadNumericValue StartAngle,
    UMLCAD.Cad.Expressions.CadNumericValue EndAngle)
    : SketchGeometry(Id, SketchGeometryKind.Arc)
{
    public ArcGeometry
    {
        ArgumentNullException.ThrowIfNull(CenterX);
        ArgumentNullException.ThrowIfNull(CenterY);
        ArgumentNullException.ThrowIfNull(Radius);
        ArgumentNullException.ThrowIfNull(StartAngle);
        ArgumentNullException.ThrowIfNull(EndAngle);
    }

    public override string CanonicalForm =>
        $"arc:{Id}:{CenterX.CanonicalForm}:{CenterY.CanonicalForm}:{Radius.CanonicalForm}:{StartAngle.CanonicalForm}:{EndAngle.CanonicalForm}";

    public override IReadOnlySet<string> ParameterNames =>
        CenterX.ParameterNames
            .Concat(CenterY.ParameterNames)
            .Concat(Radius.ParameterNames)
            .Concat(StartAngle.ParameterNames)
            .Concat(EndAngle.ParameterNames)
            .ToHashSet(StringComparer.Ordinal);
}

public sealed record SketchConstraint(
    CadId Id,
    SketchConstraintKind Kind,
    IReadOnlyList<CadId> GeometryIds,
    CadExpression? Value = null)
{
    public SketchConstraint
    {
        if (!Id.IsValid)
            throw new ArgumentException("Constraint ID is required.", nameof(Id));
        ArgumentNullException.ThrowIfNull(GeometryIds);

        GeometryIds = GeometryIds.ToArray();

        if (GeometryIds.Count == 0)
            throw new ArgumentException(
                "A sketch constraint must reference geometry.",
                nameof(GeometryIds));

        if (GeometryIds.Distinct().Count() != GeometryIds.Count)
            throw new ArgumentException(
                "Sketch constraint geometry references must be unique.",
                nameof(GeometryIds));
    }

    public string CanonicalForm =>
        $"{Id}:{Kind}:{string.Join(",", GeometryIds.OrderBy(x => x.Value, StringComparer.Ordinal))}:{Value?.CanonicalForm ?? "-"}";
}

public sealed record CadReference(
    CadId Id,
    ReferenceTargetKind TargetKind,
    CadResultId? ResultId = null,
    string? TopologyKind = null,
    string? TopologyKey = null,
    CadId? PublicationId = null)
{
    public CadReference
    {
        if (TargetKind == ReferenceTargetKind.Result && ResultId is null)
            throw new ArgumentException("Result reference requires a result ID.");
        if (TargetKind == ReferenceTargetKind.Topology &&
            (ResultId is null || string.IsNullOrWhiteSpace(TopologyKind) ||
             string.IsNullOrWhiteSpace(TopologyKey)))
            throw new ArgumentException("Topology reference is incomplete.");
        if (TargetKind == ReferenceTargetKind.Publication && PublicationId is null)
            throw new ArgumentException("Publication reference requires publication ID.");
    }
}

public sealed record Publication(
    CadId Id,
    string Name,
    CadResultId SourceResult,
    string TopologyKind,
    string TopologyKey);

public sealed record Sketch(
    CadId Id,
    string Name,
    IReadOnlyList<SketchGeometry> Geometry,
    IReadOnlyList<SketchConstraint> Constraints,
    IReadOnlyList<CadReference> Supports)
{
    public Sketch
    {
        if (!Id.IsValid || string.IsNullOrWhiteSpace(Name))
            throw new ArgumentException("Sketch identity/name is required.");
        ArgumentNullException.ThrowIfNull(Geometry);
        ArgumentNullException.ThrowIfNull(Constraints);
        ArgumentNullException.ThrowIfNull(Supports);

        Geometry = Geometry.ToArray();
        Constraints = Constraints.ToArray();
        Supports = Supports.ToArray();

        var geometryIds = Geometry.Select(x => x.Id).ToArray();
        if (geometryIds.Distinct().Count() != geometryIds.Length)
            throw new ArgumentException("Sketch geometry IDs must be unique.");

        var constraintIds = Constraints.Select(x => x.Id).ToArray();
        if (constraintIds.Distinct().Count() != constraintIds.Length)
            throw new ArgumentException("Sketch constraint IDs must be unique.");

        var geometrySet = geometryIds.ToHashSet();
        if (Constraints.Any(
                constraint => constraint.GeometryIds.Any(
                    geometryId => !geometrySet.Contains(geometryId))))
            throw new ArgumentException(
                "A sketch constraint references geometry that is not in the sketch.");
    }

    public IReadOnlySet<string> ParameterNames =>
        Geometry
            .SelectMany(x => x.ParameterNames)
            .Concat(
                Constraints
                    .Where(x => x.Value is not null)
                    .SelectMany(x => x.Value!.ParameterNames))
            .ToHashSet(StringComparer.Ordinal);

    public string CanonicalForm =>
        "sketch:" + Id + ":" + Name +
        "|" + string.Join("|",
            Geometry.OrderBy(x => x.Id.Value, StringComparer.Ordinal)
                .Select(x => x.CanonicalForm)) +
        "|" + string.Join("|",
            Constraints.OrderBy(x => x.Id.Value, StringComparer.Ordinal)
                .Select(x => x.CanonicalForm)) +
        "|" + string.Join("|",
            Supports.OrderBy(x => x.Id.Value, StringComparer.Ordinal)
                .Select(x =>
                    $"{x.Id}:{x.TargetKind}:{x.ResultId?.Value ?? "-"}:{x.TopologyKind ?? "-"}:{x.TopologyKey ?? "-"}:{x.PublicationId?.Value ?? "-"}"));
}

public sealed record BodyDefinition(CadId Id, string Name);

public abstract record CadOperation(
    CadId Id,
    CadId BodyId,
    string OperationKind,
    IReadOnlyList<CadId> InputOperationIds)
{
    protected CadOperation
    {
        if (!Id.IsValid || !BodyId.IsValid ||
            string.IsNullOrWhiteSpace(OperationKind))
            throw new ArgumentException(
                "Operation identity/body/kind is required.");

        ArgumentNullException.ThrowIfNull(InputOperationIds);
        InputOperationIds = InputOperationIds.ToArray();

        if (InputOperationIds.Distinct().Count() != InputOperationIds.Count)
            throw new ArgumentException(
                "Operation dependencies must be unique.",
                nameof(InputOperationIds));

        if (InputOperationIds.Contains(Id))
            throw new ArgumentException(
                "An operation cannot depend on itself.",
                nameof(InputOperationIds));
    }

    public abstract CadResultKind OutputKind { get; }

    public virtual IReadOnlySet<string> ParameterNames =>
        new HashSet<string>(StringComparer.Ordinal);

    public IReadOnlyList<CadReference> References { get; init; } =
        Array.Empty<CadReference>();

    public IReadOnlyDictionary<string, string> Inputs { get; init; } =
        new Dictionary<string, string>(StringComparer.Ordinal);

    public virtual string CanonicalDefinition()
    {
        var builder = new StringBuilder()
            .Append(OperationKind).Append('|')
            .Append(Id.Value).Append('|')
            .Append(BodyId.Value).Append('|');

        foreach (var dependency in InputOperationIds.OrderBy(x => x.Value, StringComparer.Ordinal))
            builder.Append("dependency=").Append(dependency.Value).Append('|');

        foreach (var reference in References.OrderBy(x => x.Id.Value, StringComparer.Ordinal))
            builder.Append("reference=").Append(reference.Id.Value)
                .Append(':').Append(reference.TargetKind)
                .Append(':').Append(reference.TargetOperationId?.Value ?? "-")
                .Append(':').Append(reference.TopologyKind ?? "-")
                .Append(':').Append(reference.TopologyKey ?? "-")
                .Append(':').Append(reference.PublicationId?.Value ?? "-")
                .Append('|');

        foreach (var input in Inputs.OrderBy(x => x.Key, StringComparer.Ordinal))
            builder.Append("input=").Append(input.Key).Append(':')
                .Append(input.Value).Append('|');

        return builder.ToString();
    }
}

public sealed record SketchOperation(
    CadId Id,
    CadId BodyId,
    Sketch Definition)
    : CadOperation(Id, BodyId, "Cad.Sketch", Array.Empty<CadId>)
{
    public override CadResultKind OutputKind =>
        CadResultKind.SketchProfile;

    public override IReadOnlySet<string> ParameterNames =>
        Definition.ParameterNames;

    public override string CanonicalDefinition() =>
        base.CanonicalDefinition() + "|definition=" + Definition.CanonicalForm;
}

public sealed record ExtrusionOperation(
    CadId Id,
    CadId BodyId,
    CadId SketchOperationId,
    CadExpression Distance,
    string Direction)
    : CadOperation(Id, BodyId, "Cad.Extrusion", new[] { SketchOperationId })
{
    public override CadResultKind OutputKind =>
        CadResultKind.Body;

    public override IReadOnlySet<string> ParameterNames =>
        Distance.ParameterNames;

    public ExtrusionOperation
    {
        ArgumentNullException.ThrowIfNull(Distance);
        if (string.IsNullOrWhiteSpace(Direction))
            throw new ArgumentException("Extrusion direction is required.");
        Inputs = new Dictionary<string, string>(StringComparer.Ordinal)
        {
            ["distance"] = Distance.CanonicalForm,
            ["direction"] = Direction
        };
    }
}

public sealed record HoleOperation(
    CadId Id,
    CadId BodyId,
    CadId BaseOperationId,
    CadExpression Diameter,
    CadExpression Depth)
    : CadOperation(Id, BodyId, "Cad.Hole", new[] { BaseOperationId })
{
    public override CadResultKind OutputKind =>
        CadResultKind.Body;

    public override IReadOnlySet<string> ParameterNames =>
        Diameter.ParameterNames
            .Concat(Depth.ParameterNames)
            .ToHashSet(StringComparer.Ordinal);

    public HoleOperation
    {
        ArgumentNullException.ThrowIfNull(Diameter);
        ArgumentNullException.ThrowIfNull(Depth);
        Inputs = new Dictionary<string, string>(StringComparer.Ordinal)
        {
            ["diameter"] = Diameter.CanonicalForm,
            ["depth"] = Depth.CanonicalForm
        };
    }
}

public sealed record CadResultEnvelope(
    CadResultId Id,
    CadResultKind Kind,
    CadId ProducingOperation,
    IReadOnlyList<CadResultId> InputResults,
    string EvidenceHash,
    IReadOnlyList<KernelTopologyBinding> Topology,
    CadEvaluationStatus Status);

public sealed record CadPartDefinition(
    CadId Id,
    string Name,
    IReadOnlyList<BodyDefinition> Bodies,
    IReadOnlyList<CadParameter> Parameters,
    IReadOnlyList<Publication> Publications,
    IReadOnlyList<CadOperation> Operations)
{
    public CadPartDefinition
    {
        if (!Id.IsValid || string.IsNullOrWhiteSpace(Name))
            throw new ArgumentException("Part identity/name is required.");
        ArgumentNullException.ThrowIfNull(Bodies);
        ArgumentNullException.ThrowIfNull(Parameters);
        ArgumentNullException.ThrowIfNull(Publications);
        ArgumentNullException.ThrowIfNull(Operations);

        Bodies = Bodies.ToArray();
        Parameters = Parameters.ToArray();
        Publications = Publications.ToArray();
        Operations = Operations.ToArray();

        EnsureUnique(Bodies.Select(x => x.Id), "body");
        EnsureUnique(Parameters.Select(x => x.Name), "parameter");
        EnsureUnique(Publications.Select(x => x.Id), "publication");
        EnsureUnique(Operations.Select(x => x.Id), "operation");

        var bodyIds = Bodies.Select(x => x.Id).ToHashSet();
        if (Operations.Any(x => !bodyIds.Contains(x.BodyId)))
            throw new ArgumentException("Operation references an undeclared body.");
        var operationIds = Operations.Select(x => x.Id).ToHashSet();
        var parameterNames = Parameters.Select(x => x.Name)
            .ToHashSet(StringComparer.Ordinal);

        var missingParameters = Operations
            .SelectMany(x => x.ParameterNames)
            .Distinct(StringComparer.Ordinal)
            .Where(name => !parameterNames.Contains(name))
            .OrderBy(name => name, StringComparer.Ordinal)
            .ToArray();

        if (missingParameters.Length > 0)
            throw new ArgumentException(
                "Operations reference undeclared parameters: " +
                string.Join(", ", missingParameters));

        var invalidPublications = Publications
            .Where(x => !operationIds.Contains(x.SourceOperationId))
            .Select(x => x.Id.Value)
            .OrderBy(x => x, StringComparer.Ordinal)
            .ToArray();

        if (invalidPublications.Length > 0)
            throw new ArgumentException(
                "Publications reference undeclared source operations: " +
                string.Join(", ", invalidPublications));

    }

    private static void EnsureUnique<T>(IEnumerable<T> values, string kind)
    {
        var list = values.ToArray();
        if (list.Distinct().Count() != list.Length)
            throw new ArgumentException($"{kind} identities must be unique.");
    }
}

public sealed record CadPartProgram(CadPartDefinition Definition)
{
    public static CadPartProgram Create(
        string id,
        string name,
        string bodyId = "body") =>
        new(new CadPartDefinition(
            new CadId(id),
            name,
            new[] { new BodyDefinition(new CadId(bodyId), "Main Body") },
            Array.Empty<CadParameter>(),
            Array.Empty<Publication>(),
            Array.Empty<CadOperation>()));

    public CadPartProgram AddParameter(CadParameter parameter)
    {
        ArgumentNullException.ThrowIfNull(parameter);

        if (Definition.Parameters.Any(
                x => string.Equals(
                    x.Name,
                    parameter.Name,
                    StringComparison.Ordinal)))
            throw new InvalidOperationException(
                $"Parameter '{parameter.Name}' already exists.");

        return this with
        {
            Definition = Definition with
            {
                Parameters = Definition.Parameters.Append(parameter).ToArray()
            }
        };
    }

    public CadPartProgram AddPublication(Publication publication)
    {
        ArgumentNullException.ThrowIfNull(publication);

        if (Definition.Publications.Any(
                x => x.Id == publication.Id))
            throw new InvalidOperationException(
                $"Publication '{publication.Id}' already exists.");

        return this with
        {
            Definition = Definition with
            {
                Publications = Definition.Publications.Append(publication).ToArray()
            }
        };
    }

    public CadPartProgram AddSketch(SketchOperation operation) => AddOperation(operation);
    public CadPartProgram Extrude(ExtrusionOperation operation) => AddOperation(operation);
    public CadPartProgram Hole(HoleOperation operation) => AddOperation(operation);

    public CadPartProgram AddOperation(CadOperation operation)
    {
        ArgumentNullException.ThrowIfNull(operation);

        if (Definition.Operations.Any(x => x.Id == operation.Id))
            throw new InvalidOperationException(
                $"Operation '{operation.Id}' already exists.");

        return this with
        {
            Definition = Definition with
            {
                Operations = Definition.Operations.Append(operation).ToArray()
            }
        };
    }
}
