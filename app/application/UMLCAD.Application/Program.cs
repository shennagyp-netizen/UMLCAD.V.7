using UMLCAD.Cad.Contracts;
using UMLCAD.Cad.Engine;
using UMLCAD.Cad.Expressions;
using UMLCAD.Cad.Semantics;
using UMLCAD.Framework;
using UMLCAD.Kernel;

var part = CadPartProgram.Create(
        "bench-vise",
        "Bench Vise")
    .AddSketch(
        new SketchOperation(
            new CadId("op-sketch"),
            new CadId("body"),
            new Sketch(
                new CadId("sketch"),
                "Main Profile",
                new SketchGeometry[]
                {
                    new CircleGeometry(
                        new CadId("profile"),
                        0,
                        0,
                        25)
                },
                Array.Empty<SketchConstraint>(),
                Array.Empty<CadReference>())))
    .Extrude(
        new ExtrusionOperation(
            new CadId("op-extrude"),
            new CadId("body"),
            new CadId("op-sketch"),
            CadExpression.Constant(80),
            "+Z"))
    .Hole(
        new HoleOperation(
            new CadId("op-hole"),
            new CadId("body"),
            new CadId("op-extrude"),
            CadExpression.Constant(10),
            CadExpression.Constant(25)));

using var application = new UmlcadApplication(
    new DemoKernelGateway());

var snapshot = await application.BuildAsync(part.Definition);

Console.WriteLine($"Succeeded: {snapshot.Succeeded}");
Console.WriteLine($"Pipeline: {string.Join(" -> ", snapshot.Plan.OperationIds)}");
Console.WriteLine(
    $"Current Body: {snapshot.CurrentBody(new CadId("body"))}");

sealed class DemoKernelGateway : IKernelGateway
{
    public Task<KernelOperationResponse> EvaluateAsync(
        KernelOperationRequest request,
        CancellationToken cancellationToken = default)
    {
        cancellationToken.ThrowIfCancellationRequested();

        return Task.FromResult(
            new KernelOperationResponse(
                CadEvaluationStatus.Succeeded,
                new CadResultId("demo:" + request.OperationId.Value),
                "demo:" + request.EvaluationIdentity.Value,
                new[]
                {
                    new KernelTopologyBinding(
                        "operation",
                        request.OperationId.Value)
                },
                Array.Empty<CadDiagnostic>()));
    }
}
