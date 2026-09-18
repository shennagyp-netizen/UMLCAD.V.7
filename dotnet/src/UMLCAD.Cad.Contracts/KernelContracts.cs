namespace UMLCAD.Cad.Contracts;

public readonly record struct ContractVersion
{
    public string Value { get; }

    public ContractVersion(string value)
    {
        if (string.IsNullOrWhiteSpace(value))
            throw new ArgumentException(
                "Contract version is required.",
                nameof(value));

        Value = value;
    }

    public override string ToString() => Value;
}

public readonly record struct ContractResultId
{
    public string Value { get; }

    public ContractResultId(string value)
    {
        if (string.IsNullOrWhiteSpace(value))
            throw new ArgumentException(
                "Result identity is required.",
                nameof(value));

        Value = value;
    }

    public override string ToString() => Value;
}

public readonly record struct ContractTopologyId
{
    public ContractResultId ResultId { get; }
    public string TopologyKind { get; }
    public string TopologyKey { get; }

    public ContractTopologyId(
        ContractResultId resultId,
        string topologyKind,
        string topologyKey)
    {
        if (string.IsNullOrWhiteSpace(topologyKind))
            throw new ArgumentException(
                "TopologyKind is required.",
                nameof(topologyKind));

        if (string.IsNullOrWhiteSpace(topologyKey))
            throw new ArgumentException(
                "TopologyKey is required.",
                nameof(topologyKey));

        ResultId = resultId;
        TopologyKind = topologyKind;
        TopologyKey = topologyKey;
    }
}

public enum KernelOperationKind
{
    EvaluateCurve,
    EvaluateSurface,
    Intersect,
    Project,
    BuildWire,
    BuildSurface,
    BuildSolid,
    ExtrudeConvexPlanarProfile,
    Boolean,
    Tessellate,
    SolveConstraints,
}

public sealed record KernelInputBinding
{
    public string Role { get; }
    public ContractResultId? ResultId { get; }
    public ContractTopologyId? TopologyId { get; }
    public string GeometryContractKey { get; }

    public KernelInputBinding(
        string role,
        ContractResultId? resultId,
        ContractTopologyId? topologyId,
        string geometryContractKey)
    {
        if (string.IsNullOrWhiteSpace(role))
            throw new ArgumentException(
                "Role is required.",
                nameof(role));

        if (string.IsNullOrWhiteSpace(geometryContractKey))
            throw new ArgumentException(
                "GeometryContractKey is required.",
                nameof(geometryContractKey));

        if (resultId is null && topologyId is null)
            throw new ArgumentException(
                "A kernel input binding must reference a result or topology identity.",
                nameof(resultId));

        if (resultId is not null &&
            topologyId is not null &&
            topologyId.Value.ResultId != resultId.Value)
            throw new ArgumentException(
                "TopologyId must belong to the supplied ResultId.",
                nameof(topologyId));

        Role = role;
        ResultId = resultId;
        TopologyId = topologyId;
        GeometryContractKey = geometryContractKey;
    }
}

public sealed record KernelRequest
{
    public string ContractId { get; }
    public ContractVersion ContractVersion { get; }
    public KernelOperationKind Operation { get; }
    public string OperationIdentity { get; }
    public IReadOnlyList<KernelInputBinding> Inputs { get; }
    public string CanonicalParameters { get; }

    public KernelRequest(
        string contractId,
        ContractVersion contractVersion,
        KernelOperationKind operation,
        string operationIdentity,
        IReadOnlyList<KernelInputBinding> inputs,
        string canonicalParameters)
    {
        if (string.IsNullOrWhiteSpace(contractId))
            throw new ArgumentException(
                "ContractId is required.",
                nameof(contractId));

        if (string.IsNullOrWhiteSpace(operationIdentity))
            throw new ArgumentException(
                "OperationIdentity is required.",
                nameof(operationIdentity));

        if (string.IsNullOrWhiteSpace(canonicalParameters))
            throw new ArgumentException(
                "CanonicalParameters is required.",
                nameof(canonicalParameters));

        ContractId = contractId;
        ContractVersion = contractVersion;
        Operation = operation;
        OperationIdentity = operationIdentity;
        Inputs = inputs?.ToArray()
            ?? throw new ArgumentNullException(nameof(inputs));
        CanonicalParameters = canonicalParameters;
    }
}

public enum KernelResultStatus
{
    Succeeded,
    Failed,
    Unsupported,
    Ambiguous,
    Indeterminate,
}

public sealed record KernelResult
{
    public ContractResultId? ResultId { get; }
    public KernelResultStatus Status { get; }
    public IReadOnlyList<ContractTopologyId> Topology { get; }
    public string EvidenceHash { get; }
    public IReadOnlyList<string> Diagnostics { get; }

    public KernelResult(
        ContractResultId? resultId,
        KernelResultStatus status,
        IReadOnlyList<ContractTopologyId> topology,
        string evidenceHash,
        IReadOnlyList<string> diagnostics)
    {
        if (status == KernelResultStatus.Succeeded && resultId is null)
            throw new ArgumentException(
                "A successful kernel result requires a result identity.",
                nameof(resultId));

        if (string.IsNullOrWhiteSpace(evidenceHash))
            throw new ArgumentException(
                "EvidenceHash is required.",
                nameof(evidenceHash));

        ResultId = resultId;
        Status = status;
        Topology = topology?.ToArray()
            ?? throw new ArgumentNullException(nameof(topology));
        EvidenceHash = evidenceHash;
        Diagnostics = diagnostics?.ToArray()
            ?? throw new ArgumentNullException(nameof(diagnostics));
    }
}
