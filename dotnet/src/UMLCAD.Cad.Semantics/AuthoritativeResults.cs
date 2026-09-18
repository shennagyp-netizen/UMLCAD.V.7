namespace UMLCAD.Cad.Semantics;

public readonly record struct AuthoritativeResultIdentity(string Value)
{
    public AuthoritativeResultIdentity
    {
        if (string.IsNullOrWhiteSpace(Value))
            throw new ArgumentException("Result identity is required.", nameof(Value));
    }

    public override string ToString() => Value;
}

public enum AuthoritativeResultKind
{
    Wire,
    Surface,
    Solid,
    Compound,
    SketchSolution,
    AssemblyState,
    KinematicState,
    ManufacturingState,
}

public enum AuthoritativeResultStatus
{
    Succeeded,
    Failed,
    Unsupported,
    Ambiguous,
    Indeterminate,
}

public sealed record TopologyBinding(
    string TopologyKind,
    string TopologyKey,
    SemanticId SourceSemanticId)
{
    public TopologyBinding
    {
        if (string.IsNullOrWhiteSpace(TopologyKind))
            throw new ArgumentException("TopologyKind is required.", nameof(TopologyKind));
        if (string.IsNullOrWhiteSpace(TopologyKey))
            throw new ArgumentException("TopologyKey is required.", nameof(TopologyKey));
        if (SourceSemanticId.Value == Guid.Empty)
            throw new ArgumentException("SourceSemanticId is required.", nameof(SourceSemanticId));
    }
}

public sealed record ResultEvidence(
    string ContractId,
    string ContractVersion,
    string EvidenceHash,
    IReadOnlyList<TopologyEvolution> TopologyEvolution,
    IReadOnlyList<string> Diagnostics)
{
    public ResultEvidence
    {
        if (string.IsNullOrWhiteSpace(ContractId))
            throw new ArgumentException("ContractId is required.", nameof(ContractId));
        if (string.IsNullOrWhiteSpace(ContractVersion))
            throw new ArgumentException("ContractVersion is required.", nameof(ContractVersion));
        if (string.IsNullOrWhiteSpace(EvidenceHash))
            throw new ArgumentException("EvidenceHash is required.", nameof(EvidenceHash));

        TopologyEvolution = TopologyEvolution?.ToArray() ??
            throw new ArgumentNullException(nameof(TopologyEvolution));
        Diagnostics = Diagnostics?.ToArray() ??
            throw new ArgumentNullException(nameof(Diagnostics));
    }
}

public sealed record AuthoritativeCadResult(
    AuthoritativeResultIdentity Identity,
    AuthoritativeResultKind Kind,
    AuthoritativeResultStatus Status,
    SemanticId ProducingSemanticId,
    IReadOnlyList<TopologyBinding> TopologyBindings,
    ResultEvidence Evidence)
{
    public AuthoritativeCadResult
    {
        if (ProducingSemanticId.Value == Guid.Empty)
            throw new ArgumentException("ProducingSemanticId is required.", nameof(ProducingSemanticId));

        TopologyBindings = TopologyBindings?.ToArray() ??
            throw new ArgumentNullException(nameof(TopologyBindings));
        ArgumentNullException.ThrowIfNull(Evidence);

        if (Status == AuthoritativeResultStatus.Succeeded && TopologyBindings.Count == 0 &&
            Kind is AuthoritativeResultKind.Wire or AuthoritativeResultKind.Surface or AuthoritativeResultKind.Solid)
        {
            throw new ArgumentException(
                "A successful geometric authoritative result requires topology bindings.",
                nameof(TopologyBindings));
        }
    }
}
