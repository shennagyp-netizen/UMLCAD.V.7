namespace UMLCAD.Cad.Semantics;

public readonly record struct AuthoritativeResultIdentity
{
    public string Value { get; }

    public AuthoritativeResultIdentity(string value)
    {
        if (string.IsNullOrWhiteSpace(value))
            throw new ArgumentException("Result identity is required.", nameof(value));

        Value = value;
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

public sealed record TopologyBinding
{
    public string TopologyKind { get; }
    public string TopologyKey { get; }
    public SemanticId SourceSemanticId { get; }

    public TopologyBinding(
        string topologyKind,
        string topologyKey,
        SemanticId sourceSemanticId)
    {
        if (string.IsNullOrWhiteSpace(topologyKind))
            throw new ArgumentException("TopologyKind is required.", nameof(topologyKind));
        if (string.IsNullOrWhiteSpace(topologyKey))
            throw new ArgumentException("TopologyKey is required.", nameof(topologyKey));
        if (sourceSemanticId.Value == Guid.Empty)
            throw new ArgumentException("SourceSemanticId is required.", nameof(sourceSemanticId));

        TopologyKind = topologyKind;
        TopologyKey = topologyKey;
        SourceSemanticId = sourceSemanticId;
    }
}

public sealed record ResultEvidence
{
    public string ContractId { get; }
    public string ContractVersion { get; }
    public string EvidenceHash { get; }
    public IReadOnlyList<TopologyEvolution> TopologyEvolution { get; }
    public IReadOnlyList<string> Diagnostics { get; }

    public ResultEvidence(
        string contractId,
        string contractVersion,
        string evidenceHash,
        IReadOnlyList<TopologyEvolution> topologyEvolution,
        IReadOnlyList<string> diagnostics)
    {
        if (string.IsNullOrWhiteSpace(contractId))
            throw new ArgumentException("ContractId is required.", nameof(contractId));
        if (string.IsNullOrWhiteSpace(contractVersion))
            throw new ArgumentException("ContractVersion is required.", nameof(contractVersion));
        if (string.IsNullOrWhiteSpace(evidenceHash))
            throw new ArgumentException("EvidenceHash is required.", nameof(evidenceHash));

        ContractId = contractId;
        ContractVersion = contractVersion;
        EvidenceHash = evidenceHash;
        TopologyEvolution = topologyEvolution?.ToArray()
            ?? throw new ArgumentNullException(nameof(topologyEvolution));
        Diagnostics = diagnostics?.ToArray()
            ?? throw new ArgumentNullException(nameof(diagnostics));
    }
}

public sealed record AuthoritativeCadResult
{
    public AuthoritativeResultIdentity Identity { get; }
    public AuthoritativeResultKind Kind { get; }
    public AuthoritativeResultStatus Status { get; }
    public SemanticId ProducingSemanticId { get; }
    public IReadOnlyList<TopologyBinding> TopologyBindings { get; }
    public ResultEvidence Evidence { get; }

    public AuthoritativeCadResult(
        AuthoritativeResultIdentity identity,
        AuthoritativeResultKind kind,
        AuthoritativeResultStatus status,
        SemanticId producingSemanticId,
        IReadOnlyList<TopologyBinding> topologyBindings,
        ResultEvidence evidence)
    {
        if (producingSemanticId.Value == Guid.Empty)
            throw new ArgumentException("ProducingSemanticId is required.", nameof(producingSemanticId));

        TopologyBindings = topologyBindings?.ToArray()
            ?? throw new ArgumentNullException(nameof(topologyBindings));
        ArgumentNullException.ThrowIfNull(evidence);

        if (status == AuthoritativeResultStatus.Succeeded &&
            TopologyBindings.Count == 0 &&
            kind is AuthoritativeResultKind.Wire or AuthoritativeResultKind.Surface or AuthoritativeResultKind.Solid)
        {
            throw new ArgumentException(
                "A successful geometric authoritative result requires topology bindings.",
                nameof(topologyBindings));
        }

        Identity = identity;
        Kind = kind;
        Status = status;
        ProducingSemanticId = producingSemanticId;
        Evidence = evidence;
    }
}
