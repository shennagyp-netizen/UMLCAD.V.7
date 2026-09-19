namespace UMLCAD.Cad.Contracts;

public readonly record struct CadId(string Value)
{
    public bool IsValid => !string.IsNullOrWhiteSpace(Value);
}

public readonly record struct CadResultId(string Value);
public readonly record struct CadEvaluationIdentity(string Value);
public readonly record struct KernelHistoryIdentity(string Value);

public enum CadResultKind { SketchProfile, Body }
public enum CadEvaluationStatus { Succeeded, Failed, Unsupported, Ambiguous, Indeterminate, Cancelled }
public enum KernelEvaluationMode { Full, Incremental }

public static class CadContractVersions
{
    public const string Semantic = "uml-cad-semantic/2.0.0";
    public const string KernelOperation = "uml-cad-kernel-operation/1.0.0";
}

public sealed record CadDiagnostic(
    string Code,
    string Message,
    CadEvaluationStatus Status = CadEvaluationStatus.Failed,
    CadId? TargetId = null);

public sealed record KernelTopologyBinding(string Kind, string Key);

public sealed record KernelOperationRequest(
    string ContractVersion,
    CadEvaluationIdentity EvaluationIdentity,
    string PartId,
    CadId OperationId,
    string OperationKind,
    KernelEvaluationMode Mode,
    CadResultId? IncrementalBaseResultId,
    IReadOnlyList<CadResultId> InputResultIds,
    IReadOnlyDictionary<string, string> SemanticInputs,
    KernelHistoryIdentity? BaseHistoryIdentity = null);

public sealed record KernelOperationResponse(
    string ContractVersion,
    CadEvaluationIdentity EvaluationIdentity,
    CadId OperationId,
    CadEvaluationStatus Status,
    CadResultId? AuthoritativeResultId,
    string? EvidenceHash,
    IReadOnlyList<KernelTopologyBinding> Topology,
    IReadOnlyList<CadDiagnostic> Diagnostics,
    KernelHistoryIdentity? HistoryIdentity = null)
{
    public static KernelOperationResponse Success(
        KernelOperationRequest request,
        CadResultId resultId,
        string evidenceHash,
        IReadOnlyList<KernelTopologyBinding>? topology = null,
        IReadOnlyList<CadDiagnostic>? diagnostics = null) =>
        new(
            request.ContractVersion,
            request.EvaluationIdentity,
            request.OperationId,
            CadEvaluationStatus.Succeeded,
            resultId,
            evidenceHash,
            topology ?? Array.Empty<KernelTopologyBinding>(),
            diagnostics ?? Array.Empty<CadDiagnostic>(),
            new KernelHistoryIdentity("demo:" + request.EvaluationIdentity.Value));

    public static KernelOperationResponse Failure(
        KernelOperationRequest request,
        string code,
        string message,
        CadEvaluationStatus status = CadEvaluationStatus.Failed) =>
        new(
            request.ContractVersion,
            request.EvaluationIdentity,
            request.OperationId,
            status,
            null,
            null,
            Array.Empty<KernelTopologyBinding>(),
            new[]
            {
                new CadDiagnostic(code, message, status, request.OperationId)
            });
}

public interface IKernelGateway
{
    Task<KernelOperationResponse> EvaluateAsync(
        KernelOperationRequest request,
        CancellationToken cancellationToken = default);
}
