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

public sealed record ReferencePathSegment
{
    public SemanticId OwnerId { get; }
    public string PublicationName { get; }

    public ReferencePathSegment(
        SemanticId ownerId,
        string publicationName)
    {
        if (ownerId.Value == Guid.Empty)
            throw new ArgumentException("OwnerId is required.", nameof(ownerId));
        if (string.IsNullOrWhiteSpace(publicationName))
            throw new ArgumentException("PublicationName is required.", nameof(publicationName));

        OwnerId = ownerId;
        PublicationName = publicationName;
    }
}

public sealed record ReferencePath
{
    public IReadOnlyList<ReferencePathSegment> Segments { get; }

    public ReferencePath(IReadOnlyList<ReferencePathSegment> segments)
    {
        Segments = segments?.ToArray()
            ?? throw new ArgumentNullException(nameof(segments));

        if (Segments.Count == 0)
            throw new ArgumentException(
                "A reference path requires at least one segment.",
                nameof(segments));
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

    private string _geometryKey = string.Empty;

    public required string GeometryKey
    {
        get => _geometryKey;
        init
        {
            if (string.IsNullOrWhiteSpace(value))
                throw new ArgumentException(
                    "GeometryKey is required.",
                    nameof(value));

            _geometryKey = value;
        }
    }

    public GeometricReference()
    {
        TargetKind = ReferenceTargetKind.Geometric;
    }
}

public sealed record TopologyReference : CadReference
{
    private string _authoritativeResultId = string.Empty;
    private string _topologyKey = string.Empty;
    private string _topologyKind = string.Empty;

    public required string AuthoritativeResultId
    {
        get => _authoritativeResultId;
        init
        {
            if (string.IsNullOrWhiteSpace(value))
                throw new ArgumentException(
                    "AuthoritativeResultId is required.",
                    nameof(value));

            _authoritativeResultId = value;
        }
    }

    public required string TopologyKey
    {
        get => _topologyKey;
        init
        {
            if (string.IsNullOrWhiteSpace(value))
                throw new ArgumentException(
                    "TopologyKey is required.",
                    nameof(value));

            _topologyKey = value;
        }
    }

    public required string TopologyKind
    {
        get => _topologyKind;
        init
        {
            if (string.IsNullOrWhiteSpace(value))
                throw new ArgumentException(
                    "TopologyKind is required.",
                    nameof(value));

            _topologyKind = value;
        }
    }

    public TopologyReference()
    {
        TargetKind = ReferenceTargetKind.Topology;
    }
}

public sealed record ReferenceResolution
{
    public CadReference Reference { get; }
    public ReferenceResolutionStatus Status { get; }
    public string? ResolvedResultId { get; }
    public string? Diagnostic { get; }

    public ReferenceResolution(
        CadReference reference,
        ReferenceResolutionStatus status,
        string? resolvedResultId,
        string? diagnostic)
    {
        ArgumentNullException.ThrowIfNull(reference);

        if (status == ReferenceResolutionStatus.Resolved &&
            string.IsNullOrWhiteSpace(resolvedResultId))
        {
            throw new ArgumentException(
                "A resolved reference requires a resolved result identity.",
                nameof(resolvedResultId));
        }

        Reference = reference;
        Status = status;
        ResolvedResultId = resolvedResultId;
        Diagnostic = string.IsNullOrWhiteSpace(diagnostic) ? null : diagnostic;
    }
}

public sealed record TopologyEvolution
{
    public string PreviousTopologyKey { get; }
    public string CurrentTopologyKey { get; }
    public TopologyEvolutionKind Kind { get; }

    public TopologyEvolution(
        string previousTopologyKey,
        string currentTopologyKey,
        TopologyEvolutionKind kind)
    {
        if (string.IsNullOrWhiteSpace(previousTopologyKey))
            throw new ArgumentException(
                "PreviousTopologyKey is required.",
                nameof(previousTopologyKey));
        if (string.IsNullOrWhiteSpace(currentTopologyKey))
            throw new ArgumentException(
                "CurrentTopologyKey is required.",
                nameof(currentTopologyKey));

        PreviousTopologyKey = previousTopologyKey;
        CurrentTopologyKey = currentTopologyKey;
        Kind = kind;
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
