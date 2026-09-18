namespace UMLCAD.Cad.Contracts;

public readonly record struct RepresentationIdentity(string Value)
{
    public RepresentationIdentity
    {
        if (string.IsNullOrWhiteSpace(Value))
            throw new ArgumentException("Representation identity is required.", nameof(Value));
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

public sealed record RepresentationRequest(
    RepresentationKind Kind,
    ContractResultId SourceResultId,
    string DisplayPolicy,
    string RepresentationPolicyVersion)
{
    public RepresentationRequest
    {
        if (string.IsNullOrWhiteSpace(DisplayPolicy))
            throw new ArgumentException("DisplayPolicy is required.", nameof(DisplayPolicy));
        if (string.IsNullOrWhiteSpace(RepresentationPolicyVersion))
            throw new ArgumentException("RepresentationPolicyVersion is required.", nameof(RepresentationPolicyVersion));
    }
}

public sealed record RepresentationResult(
    RepresentationIdentity Identity,
    RepresentationBuildStatus Status,
    ContractResultId SourceResultId,
    string ContentHash,
    IReadOnlyList<string> Diagnostics)
{
    public RepresentationResult
    {
        if (string.IsNullOrWhiteSpace(ContentHash))
            throw new ArgumentException("ContentHash is required.", nameof(ContentHash));

        Diagnostics = Diagnostics?.ToArray() ??
            throw new ArgumentNullException(nameof(Diagnostics));
    }
}
