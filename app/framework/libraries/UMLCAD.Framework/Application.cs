using UMLCAD.Cad.Contracts;
using UMLCAD.Cad.Engine;
using UMLCAD.Cad.Semantics;
using UMLCAD.Kernel.Client;

namespace UMLCAD.Framework;

public sealed class UmlcadApplication : IDisposable
{
    private readonly CadEvaluationEngine _engine;
    private readonly IKernelGateway _kernel;
    private CadEvaluationSnapshot? _currentEvaluation;
    private bool _disposed;

    public UmlcadApplication()
        : this(new UmlcadKernelClient())
    {
    }

    public UmlcadApplication(IKernelGateway kernel)
    {
        _kernel = kernel ?? throw new ArgumentNullException(nameof(kernel));
        _engine = new CadEvaluationEngine(_kernel);
    }

    /// <summary>
    /// The latest immutable authoritative .NET evaluation snapshot published by this
    /// application instance. The snapshot, including its history, is replaced rather
    /// than mutated after each completed build/rebuild.
    /// </summary>
    public CadEvaluationSnapshot? CurrentEvaluation => _currentEvaluation;

    /// <summary>
    /// The latest immutable .NET-owned operation/result history.
    /// The mathematical kernel does not own, persist, or advance this value.
    /// </summary>
    public CadEvaluationHistory? CurrentHistory => _currentEvaluation?.History;

    public async Task<CadEvaluationSnapshot> BuildAsync(
        CadPart part,
        CadEvaluationOptions? options = null,
        CancellationToken cancellationToken = default)
    {
        Ensure();

        var snapshot = await _engine.EvaluateAsync(
            part,
            options,
            cancellationToken);

        _currentEvaluation = snapshot;
        return snapshot;
    }

    public async Task<CadEvaluationSnapshot> RebuildAsync(
        CadPart part,
        IReadOnlySet<CadId> changed,
        CadEvaluationOptions? options = null,
        CancellationToken cancellationToken = default)
    {
        Ensure();

        var snapshot = await _engine.RebuildAsync(
            part,
            changed,
            options,
            cancellationToken);

        _currentEvaluation = snapshot;
        return snapshot;
    }

    public void Dispose()
    {
        if (_disposed)
            return;

        _disposed = true;

        if (_kernel is IDisposable disposable)
            disposable.Dispose();
    }

    private void Ensure()
    {
        if (_disposed)
            throw new ObjectDisposedException(nameof(UmlcadApplication));
    }
}
