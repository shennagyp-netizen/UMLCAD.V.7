using UMLCAD.Cad.Contracts;
using UMLCAD.Cad.Engine;
using UMLCAD.Kernel;

namespace UMLCAD.Framework;

public sealed class UmlcadApplication(
    UmlcadKernel kernel)
{
    private readonly CadEvaluationEngine _evaluation = new(kernel ?? throw new ArgumentNullException(nameof(kernel)));

    public Task<KernelEvaluationOutcome> BuildAsync(
        CadBuildDefinition definition,
        CancellationToken cancellationToken = default) =>
        _evaluation.EvaluateAsync(definition, cancellationToken);
}
