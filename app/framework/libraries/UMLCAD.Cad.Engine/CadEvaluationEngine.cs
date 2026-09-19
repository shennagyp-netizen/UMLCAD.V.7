using System.Text.Json;
using UMLCAD.Cad.Contracts;
using UMLCAD.Cad.Semantics;
using UMLCAD.Kernel;

namespace UMLCAD.Cad.Engine;

public sealed class CadEvaluationEngine(UmlcadKernel kernel)
{
    private readonly UmlcadKernel _kernel =
        kernel ?? throw new ArgumentNullException(nameof(kernel));

    public CadEvaluationPlan Plan(CadDocumentDefinition document)
    {
        ArgumentNullException.ThrowIfNull(document);
        return new CadDependencyGraph(document).CreatePlan();
    }

    public async Task<KernelEvaluationOutcome> EvaluateAsync(
        CadBuildDefinition definition,
        CancellationToken cancellationToken = default)
    {
        ArgumentNullException.ThrowIfNull(definition);

        using var document = JsonDocument.Parse(definition.SemanticJson);
        var request = new KernelBuildRequest(
            definition.Identity.ApplicationId,
            definition.Identity.ApplicationVersion,
            definition.Identity.SemanticIdentity,
            document.RootElement.Clone());

        return await _kernel.EvaluateBuildAsync(request, cancellationToken);
    }
}
