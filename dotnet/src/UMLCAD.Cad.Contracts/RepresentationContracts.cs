namespace UMLCAD.Cad.Contracts;

public readonly record struct RepresentationIdentity
{
    public string Value { get; }

    public RepresentationIdentity(string value)
    {
        if (string.IsNullOrWhiteSpace(value))
            throw new ArgumentException(
                "Representation identity is required.",
                nameof(value));

        Value = value;
    }

    public override string ToString() => Value;
}

public enum RepresentationKind
{
    RenderMesh,
    DrawingProjection,
    SimulationMesh,
    ReviewSnapshot,
    LightweightShape,
}

public enum RepresentationBuildStatus
{
    Succeeded,
    Failed,
    Unsupported,
    Stale,
    Ambiguous,
}

public sealed record RepresentationRequest
{
    public RepresentationKind Kind { get; }
    public ContractResultId SourceResultId { get; }
    public string DisplayPolicy { get; }
    public string RepresentationPolicyVersion { get; }

    public RepresentationRequest(
        RepresentationKind kind,
        ContractResultId sourceResultId,
        string displayPolicy,
        string representationPolicyVersion)
    {
        if (string.IsNullOrWhiteSpace(displayPolicy))
            throw new ArgumentException(
                "DisplayPolicy is required.",
                nameof(displayPolicy));
        if (string.IsNullOrWhiteSpace(representationPolicyVersion))
            throw new ArgumentException(
                "RepresentationPolicyVersion is required.",
                nameof(representationPolicyVersion));

        Kind = kind;
        SourceResultId = sourceResultId;
        DisplayPolicy = displayPolicy;
        RepresentationPolicyVersion = representationPolicyVersion;
    }
}

public sealed record RepresentationResult
{
    public RepresentationIdentity Identity { get; }
    public RepresentationBuildStatus Status { get; }
    public ContractResultId SourceResultId { get; }
    public string ContentHash { get; }
    public IReadOnlyList<string> Diagnostics { get; }

    public RepresentationResult(
        RepresentationIdentity identity,
        RepresentationBuildStatus status,
        ContractResultId sourceResultId,
        string contentHash,
        IReadOnlyList<string> diagnostics)
    {
        if (string.IsNullOrWhiteSpace(contentHash))
            throw new ArgumentException(
                "ContentHash is required.",
                nameof(contentHash));

        Diagnostics = diagnostics?.ToArray()
            ?? throw new ArgumentNullException(nameof(diagnostics));

        Identity = identity;
        Status = status;
        SourceResultId = sourceResultId;
        ContentHash = contentHash;
    }
}
