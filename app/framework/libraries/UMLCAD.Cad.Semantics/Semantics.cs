using System.Text;
using UMLCAD.Cad.Contracts;
using UMLCAD.Cad.Expressions;

namespace UMLCAD.Cad.Semantics;

public enum SketchGeometryKind { Line, Circle, Arc }
public enum SketchConstraintKind { Horizontal, Vertical, Coincident, Fixed, Distance, Radius, Diameter, Tangent }
public enum ReferenceTargetKind { Result, Topology, Publication, Semantic }

public abstract record SketchGeometry(CadId Id, SketchGeometryKind Kind)
{
    public abstract string CanonicalForm { get; }
}

public sealed record LineGeometry(
    CadId Id, double X1, double Y1, double X2, double Y2)
    : SketchGeometry(Id, SketchGeometryKind.Line)
{
    public override string CanonicalForm =>
        $"line:{Id}:{X1:R}:{Y1:R}:{X2:R}:{Y2:R}";
}

public sealed record CircleGeometry(
    CadId Id, double X, double Y, double Radius)
    : SketchGeometry(Id, SketchGeometryKind.Circle)
{
    public CircleGeometry
    {
        if (!double.IsFinite(Radius) || Radius <= 0)
            throw new ArgumentOutOfRangeException(nameof(Radius));
    }

    public override string CanonicalForm =>
        $"circle:{Id}:{X:R}:{Y:R}:{Radius:R}";
}

public sealed record SketchConstraint(
    CadId Id,
    SketchConstraintKind Kind,
    IReadOnlyList<CadId> GeometryIds,
    CadExpression? Value = null)
{
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
        if (string.IsNullOrWhiteSpace(Name))
            throw new ArgumentException("Sketch name is required.", nameof(Name));
        ArgumentNullException.ThrowIfNull(Geometry);
        ArgumentNullException.ThrowIfNull(Constraints);
        ArgumentNullException.ThrowIfNull(Supports);

        var geometryIds = Geometry.Select(x => x.Id).ToArray();
        if (geometryIds.Distinct().Count() != geometryIds.Length)
            throw new ArgumentException("Sketch geometry IDs must be unique.");
    }

    public string CanonicalForm =>
        "sketch:" + Id + ":" + Name +
        "|" + string.Join("|",
            Geometry.OrderBy(x => x.Id.Value, StringComparer.Ordinal)
                .Select(x => x.CanonicalForm)) +
        "|" + string.Join("|",
            Constraints.OrderBy(x => x.Id.Value, StringComparer.Ordinal)
                .Select(x => x.CanonicalForm));
}

public sealed record BodyDefinition(CadId Id, string Name);

public abstract record CadOperation(
    CadId Id,
    CadId BodyId,
    string OperationKind,
    IReadOnlyList<CadId> InputOperationIds)
{
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
                .Append(':').Append(reference.ResultId?.Value ?? "-")
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
        if (string.IsNullOrWhiteSpace(Name))
            throw new ArgumentException("Part name is required.", nameof(Name));
        ArgumentNullException.ThrowIfNull(Bodies);
        ArgumentNullException.ThrowIfNull(Parameters);
        ArgumentNullException.ThrowIfNull(Publications);
        ArgumentNullException.ThrowIfNull(Operations);

        EnsureUnique(Bodies.Select(x => x.Id), "body");
        EnsureUnique(Parameters.Select(x => x.Name), "parameter");
        EnsureUnique(Publications.Select(x => x.Id), "publication");
        EnsureUnique(Operations.Select(x => x.Id), "operation");

        var bodyIds = Bodies.Select(x => x.Id).ToHashSet();
        if (Operations.Any(x => !bodyIds.Contains(x.BodyId)))
            throw new ArgumentException("Operation references an undeclared body.");
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

    public CadPartProgram AddParameter(CadParameter parameter) =>
        this with
        {
            Definition = Definition with
            {
                Parameters = Definition.Parameters.Append(parameter).ToArray()
            }
        };

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
