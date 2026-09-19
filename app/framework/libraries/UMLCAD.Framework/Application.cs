using UMLCAD.Cad.Contracts;
using UMLCAD.Cad.Engine;
using UMLCAD.Cad.Semantics;
using UMLCAD.Engineering.Runtime;
using UMLCAD.Science;

namespace UMLCAD.Framework;

public sealed class UmlcadApplication : IDisposable
{
    private readonly IKernelGateway _kernel;
    private readonly CadEvaluationEngine _cad;
    private bool _disposed;

    public UmlcadApplication(IKernelGateway kernel)
    {
        _kernel = kernel ?? throw new ArgumentNullException(nameof(kernel));
        _cad = new CadEvaluationEngine(_kernel);
    }

    public static UmlcadApplication CreateDeterministicDemo() =>
        new DeterministicDemoKernelGateway();

    public Task<CadEvaluationSnapshot> BuildAsync(
        CadPartDefinition part,
        CadEvaluationOptions? options = null,
        CancellationToken cancellationToken = default)
    {
        ThrowIfDisposed();
        return _cad.EvaluateAsync(part, options, cancellationToken);
    }

    public Task<CadEvaluationSnapshot> RebuildAsync(
        CadPartDefinition part,
        IReadOnlySet<CadId> changedOperationIds,
        CadEvaluationOptions? options = null,
        CancellationToken cancellationToken = default)
    {
        ThrowIfDisposed();

        return _cad.EvaluateAsync(
            part,
            changedOperationIds,
            options,
            cancellationToken);
    }

    public ValueTask<IReadOnlyList<EngineeringRuleResult>> ValidateEngineeringAsync(
        CadEvaluationSnapshot snapshot,
        IPhenomenaSimulationService simulation,
        IEnumerable<IEngineeringRule> rules,
        CancellationToken cancellationToken = default)
    {
        ThrowIfDisposed();

        return new EngineeringRuleRuntime().EvaluateAsync(
            new EngineeringContext(snapshot, simulation),
            rules,
            cancellationToken);
    }

    public void Dispose()
    {
        if (_disposed)
            return;

        _disposed = true;

        if (_kernel is IDisposable disposable)
            disposable.Dispose();
    }

    private void ThrowIfDisposed()
    {
        if (_disposed)
            throw new ObjectDisposedException(nameof(UmlcadApplication));
    }
}


internal sealed class DeterministicDemoKernelGateway : UMLCAD.Cad.Contracts.IKernelGateway
{
    public Task<UMLCAD.Cad.Contracts.KernelOperationResponse> EvaluateAsync(
        UMLCAD.Cad.Contracts.KernelOperationRequest request,
        CancellationToken cancellationToken = default)
    {
        cancellationToken.ThrowIfCancellationRequested();

        return Task.FromResult(
            new UMLCAD.Cad.Contracts.KernelOperationResponse(
                UMLCAD.Cad.Contracts.CadEvaluationStatus.Succeeded,
                new UMLCAD.Cad.Contracts.CadResultId("demo:" + request.OperationId.Value),
                "demo:" + request.EvaluationIdentity.Value,
                new[]
                {
                    new UMLCAD.Cad.Contracts.KernelTopologyBinding(
                        "operation",
                        request.OperationId.Value)
                },
                Array.Empty<UMLCAD.Cad.Contracts.CadDiagnostic>()));
    }
}
