namespace UMLCAD.Cad.Contracts;

public readonly record struct ContractVersion(string Value)
{
    public ContractVersion
    {
        if (string.IsNullOrWhiteSpace(Value))
            throw new ArgumentException("Contract version is required.", nameof(Value));
    }

    public override string ToString() => Value;
}

public readonly record struct ContractResultId(string Value)
{
    public ContractResultId
    {
        if (string.IsNullOrWhiteSpace(Value))
            throw new ArgumentException("Result identity is required.", nameof(Value));
    }

    public override string ToString() => Value;
}

public readonly record struct ContractTopologyId(
    ContractResultId ResultId,
    string TopologyKind,
    string TopologyKey)
{
    public ContractTopologyId
    {
        if (string.IsNullOrWhiteSpace(TopologyKind))
            throw new ArgumentException("TopologyKind is required.", nameof(TopologyKind));

        if (string.IsNullOrWhiteSpace(TopologyKey))
            throw new ArgumentException("TopologyKey is required.", nameof(TopologyKey));
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
    Boolean,
    Tessellate,
    SolveConstraints,
}

public sealed record KernelInputBinding(
    string Role,
    ContractResultId? ResultId,
    ContractTopologyId? TopologyId,
    string GeometryContractKey)
{
    public KernelInputBinding
    {
        if (string.IsNullOrWhiteSpace(Role))
            throw new ArgumentException("Role is required.", nameof(Role));

        if (string.IsNullOrWhiteSpace(GeometryContractKey))
            throw new ArgumentException("GeometryContractKey is required.", nameof(GeometryContractKey));

        if (ResultId is null && TopologyId is null)
            throw new ArgumentException(
                "A kernel input binding must reference a result or topology identity.",
                nameof(ResultId));

        if (ResultId is not null && TopologyId is not null &&
            TopologyId.Value.ResultId != ResultId.Value)
        {
            throw new ArgumentException(
                "TopologyId must belong to the supplied ResultId.",
                nameof(TopologyId));
        }
    }
}

public sealed record KernelRequest(
    string ContractId,
    ContractVersion ContractVersion,
    KernelOperationKind Operation,
    string OperationIdentity,
    IReadOnlyList<KernelInputBinding> Inputs,
    string CanonicalParameters)
{
    public KernelRequest
    {
        if (string.IsNullOrWhiteSpace(ContractId))
            throw new ArgumentException("ContractId is required.", nameof(ContractId));

        if (string.IsNullOrWhiteSpace(OperationIdentity))
            throw new ArgumentException("OperationIdentity is required.", nameof(OperationIdentity));

        if (string.IsNullOrWhiteSpace(CanonicalParameters))
            throw new ArgumentException("CanonicalParameters is required.", nameof(CanonicalParameters));

        Inputs = Inputs?.ToArray() ??
            throw new ArgumentNullException(nameof(Inputs));
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

public sealed record KernelResult(
    ContractResultId? ResultId,
    KernelResultStatus Status,
    IReadOnlyList<ContractTopologyId> Topology,
    string EvidenceHash,
    IReadOnlyList<string> Diagnostics)
{
    public KernelResult
    {
        if (Status == KernelResultStatus.Succeeded && ResultId is null)
            throw new ArgumentException(
                "A successful kernel result requires a result identity.",
                nameof(ResultId));

        if (string.IsNullOrWhiteSpace(EvidenceHash))
            throw new ArgumentException("EvidenceHash is required.", nameof(EvidenceHash));

        Topology = Topology?.ToArray() ??
            throw new ArgumentNullException(nameof(Topology));
        Diagnostics = Diagnostics?.ToArray() ??
            throw new ArgumentNullException(nameof(Diagnostics));
    }
}
