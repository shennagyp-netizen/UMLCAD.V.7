using UMLCAD.Cad.Contracts;
using UMLCAD.Cad.Engine;
using UMLCAD.Engineering.Runtime;
using UMLCAD.Kernel;
using UMLCAD.Science;

namespace UMLCAD.Framework;

public sealed class UmlcadApplication : IDisposable
{
    private readonly UmlcadKernel _kernel;
    private readonly CadEvaluationEngine _evaluation;
    private bool _disposed;

    private UmlcadApplication(UmlcadKernel kernel)
    {
        _kernel = kernel ?? throw new ArgumentNullException(nameof(kernel));
        _evaluation = new CadEvaluationEngine(_kernel);
    }

    public static UmlcadApplication ConnectDefault(UmlcadKernelOptions? options = null) =>
        new(UmlcadKernel.Connect(options));

    internal static UmlcadApplication ForTest(UmlcadKernel kernel) =>
        new(kernel);

    public Task<EngineeringBuildValidationResult> ValidateEngineeringAsync(
        CadDocumentDefinition document,
        IEnumerable<IEngineeringRule> rules,
        IPhenomenaSimulationService simulation,
        CancellationToken cancellationToken = default)
    {
        ThrowIfDisposed();
        ArgumentNullException.ThrowIfNull(document);
        ArgumentNullException.ThrowIfNull(rules);
        ArgumentNullException.ThrowIfNull(simulation);

        var store = new CadDocumentStore(document);
        return new EngineeringBuildValidator().ValidateAsync(
            store,
            rules,
            simulation,
            cancellationToken);
    }

    public Task<KernelEvaluationOutcome> BuildAsync(
        CadBuildDefinition definition,
        CancellationToken cancellationToken = default)
    {
        ThrowIfDisposed();
        return _evaluation.EvaluateAsync(definition, cancellationToken);
    }

    public void Dispose()
    {
        if (_disposed)
            return;

        _disposed = true;
        _kernel.Dispose();
    }

    private void ThrowIfDisposed()
    {
        if (_disposed)
            throw new ObjectDisposedException(nameof(UmlcadApplication));
    }
}
