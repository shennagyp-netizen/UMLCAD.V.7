namespace UMLCAD.Cad.Contracts;

public readonly record struct CadId(string Value)
{
    public CadId
    {
        if (string.IsNullOrWhiteSpace(Value))
            throw new ArgumentException("CAD ID is required.", nameof(Value));
    }

    public override string ToString() => Value;
}

public readonly record struct CadResultId(string Value)
{
    public CadResultId
    {
        if (string.IsNullOrWhiteSpace(Value))
            throw new ArgumentException("Result ID is required.", nameof(Value));
    }

    public override string ToString() => Value;
}

public readonly record struct CadEvaluationIdentity(string Value)
{
    public CadEvaluationIdentity
    {
        if (string.IsNullOrWhiteSpace(Value))
            throw new ArgumentException("Evaluation identity is required.", nameof(Value));
    }

    public override string ToString() => Value;
}

public enum CadResultKind
{
    SketchProfile,
    Body,
    Wire,
    Surface,
    Compound,
    AssemblyState,
    DrawingState,
    ManufacturingState
}

public enum CadEvaluationStatus
{
    Succeeded,
    Failed,
    Unsupported,
    Ambiguous,
    Indeterminate,
    Cancelled
}

public enum KernelEvaluationMode
{
    Full,
    Incremental
}

public static class CadContractVersions
{
    public const string Semantic = "uml-cad-semantic/2.0.0";
    public const string Kernel = "uml-cad-kernel-evaluation/2.0.0";
}

public sealed record CadDiagnostic(
    string Code,
    string Message,
    CadEvaluationStatus Status = CadEvaluationStatus.Failed,
    CadId? Target = null);

public sealed record KernelTopologyBinding(
    string Kind,
    string Key);

public sealed record KernelOperationRequest(
    string ContractVersion,
    CadEvaluationIdentity EvaluationIdentity,
    CadId PartId,
    CadId OperationId,
    string OperationKind,
    KernelEvaluationMode Mode,
    CadResultId? IncrementalBaseResultId,
    IReadOnlyList<CadResultId> InputResults,
    string SemanticPayloadJson,
    IReadOnlyDictionary<string, string> Inputs);

public sealed record KernelOperationResponse(
    CadEvaluationStatus Status,
    CadResultId? AuthoritativeResultId,
    string? EvidenceHash,
    IReadOnlyList<KernelTopologyBinding> Topology,
    IReadOnlyList<CadDiagnostic> Diagnostics);

public interface IKernelGateway
{
    Task<KernelOperationResponse> EvaluateAsync(
        KernelOperationRequest request,
        CancellationToken cancellationToken = default);
}
