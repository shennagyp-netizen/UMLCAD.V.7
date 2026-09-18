namespace UMLCAD.Cad.Semantics;

public enum ReferenceTargetKind
{
    Semantic,
    Geometric,
    Topology,
    Publication,
    Support,
}

public enum ReferenceResolutionStatus
{
    Resolved,
    Missing,
    Ambiguous,
    Indeterminate,
    Unsupported,
}

public readonly record struct ReferenceContextId(Guid Value)
{
    public static ReferenceContextId New() => new(Guid.NewGuid());

    public override string ToString() => Value.ToString("D");
}

public sealed record ReferencePathSegment(
    SemanticId OwnerId,
    string PublicationName)
{
    public ReferencePathSegment
    {
        if (OwnerId.Value == Guid.Empty)
            throw new ArgumentException("OwnerId is required.", nameof(OwnerId));

        if (string.IsNullOrWhiteSpace(PublicationName))
            throw new ArgumentException("PublicationName is required.", nameof(PublicationName));
    }
}

public sealed record ReferencePath(
    IReadOnlyList<ReferencePathSegment> Segments)
{
    public ReferencePath
    {
        Segments = Segments?.ToArray() ??
            throw new ArgumentNullException(nameof(Segments));

        if (Segments.Count == 0)
            throw new ArgumentException("A reference path requires at least one segment.", nameof(Segments));
    }
}

public abstract record CadReference
{
    public required SemanticId ReferenceId { get; init; }

    public required ReferenceTargetKind TargetKind { get; init; }

    public required ReferenceContextId ContextId { get; init; }

    public ReferencePath? Path { get; init; }
}

public sealed record SemanticReference : CadReference
{
    public required SemanticId TargetId { get; init; }

    public SemanticReference()
    {
        TargetKind = ReferenceTargetKind.Semantic;
    }
}

public sealed record GeometricReference : CadReference
{
    public required SemanticId GeometryOwnerId { get; init; }

    public required string GeometryKey { get; init; }

    public GeometricReference()
    {
        TargetKind = ReferenceTargetKind.Geometric;
    }

    public GeometricReference
    {
        if (string.IsNullOrWhiteSpace(GeometryKey))
            throw new ArgumentException("GeometryKey is required.", nameof(GeometryKey));
    }
}

public sealed record TopologyReference : CadReference
{
    public required string AuthoritativeResultId { get; init; }

    public required string TopologyKey { get; init; }

    public required string TopologyKind { get; init; }

    public TopologyReference()
    {
        TargetKind = ReferenceTargetKind.Topology;
    }

    public TopologyReference
    {
        if (string.IsNullOrWhiteSpace(AuthoritativeResultId))
            throw new ArgumentException("AuthoritativeResultId is required.", nameof(AuthoritativeResultId));
        if (string.IsNullOrWhiteSpace(TopologyKey))
            throw new ArgumentException("TopologyKey is required.", nameof(TopologyKey));
        if (string.IsNullOrWhiteSpace(TopologyKind))
            throw new ArgumentException("TopologyKind is required.", nameof(TopologyKind));
    }
}

public sealed record ReferenceResolution(
    CadReference Reference,
    ReferenceResolutionStatus Status,
    string? ResolvedResultId,
    string? Diagnostic)
{
    public ReferenceResolution
    {
        ArgumentNullException.ThrowIfNull(Reference);

        if (Status == ReferenceResolutionStatus.Resolved &&
            string.IsNullOrWhiteSpace(ResolvedResultId))
        {
            throw new ArgumentException(
                "A resolved reference requires a resolved result identity.",
                nameof(ResolvedResultId));
        }

        Diagnostic = string.IsNullOrWhiteSpace(Diagnostic) ? null : Diagnostic;
    }
}

public sealed record TopologyEvolution(
    string PreviousTopologyKey,
    string CurrentTopologyKey,
    TopologyEvolutionKind Kind)
{
    public TopologyEvolution
    {
        if (string.IsNullOrWhiteSpace(PreviousTopologyKey))
            throw new ArgumentException("PreviousTopologyKey is required.", nameof(PreviousTopologyKey));
        if (string.IsNullOrWhiteSpace(CurrentTopologyKey))
            throw new ArgumentException("CurrentTopologyKey is required.", nameof(CurrentTopologyKey));
    }
}

public enum TopologyEvolutionKind
{
    Preserved,
    Replaced,
    Split,
    Merged,
    Removed,
    Introduced,
    Ambiguous,
}
