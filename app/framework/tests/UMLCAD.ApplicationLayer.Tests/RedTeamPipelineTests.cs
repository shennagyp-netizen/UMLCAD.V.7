using Xunit;
using UMLCAD.Cad.Contracts;
using UMLCAD.Cad.Engine;
using UMLCAD.Cad.Expressions;
using UMLCAD.Cad.Semantics;

namespace UMLCAD.ApplicationLayer.Tests;

public sealed class RedTeamPipelineTests
{
    [Fact]
    public async Task KernelResponseIdentityMismatchFailsClosedAndDoesNotCreateResult()
    {
        var part = Factory.Create();
        var engine = new CadEvaluationEngine(new MismatchingGateway());

        var snapshot = await engine.EvaluateAsync(part, cancellationToken: TestContext.Current.CancellationToken);

        var outcome = snapshot.Outcomes[new CadId("sketch")];
        Assert.False(snapshot.Succeeded);
        Assert.Equal(CadEvaluationStatus.Failed, outcome.Status);
        Assert.Null(outcome.Result);
        Assert.Contains(
            outcome.Diagnostics,
            diagnostic => diagnostic.Code == "KERNEL_RESPONSE_IDENTITY_MISMATCH");
    }

    [Fact]
    public async Task FailedUpstreamBlocksOnlyItsDependents()
    {
        var part = Factory.CreateWithIndependentSketch();
        var engine = new CadEvaluationEngine(new FailingFirstGateway());

        var snapshot = await engine.EvaluateAsync(
            part,
            cancellationToken: TestContext.Current.CancellationToken);

        Assert.False(snapshot.Succeeded);
        Assert.Equal(CadEvaluationStatus.Failed, snapshot.Outcomes[new CadId("a")].Status);
        Assert.Equal(CadEvaluationStatus.Failed, snapshot.Outcomes[new CadId("b")].Status);
        Assert.Equal(CadEvaluationStatus.Succeeded, snapshot.Outcomes[new CadId("independent")].Status);
        Assert.Contains(
            snapshot.Outcomes[new CadId("b")].Diagnostics,
            diagnostic => diagnostic.Code == "DEPENDENCY_FAILED");
    }

    private static class Factory
    {
        public static CadPart Create() =>
            CadPartProgram.Create("p", "P")
                .Sketch(new SketchOperation(
                    new CadId("sketch"),
                    new CadId("body"),
                    new Sketch(
                        new CadId("sketch"),
                        "Sketch",
                        new SketchGeometry[]
                        {
                            new CircleGeometry(new CadId("c"), 0, 0, 10)
                        },
                        Array.Empty<SketchConstraint>(),
                        Array.Empty<CadReference>())))
                .Extrude(new ExtrusionOperation(
                    new CadId("extrude"),
                    new CadId("body"),
                    new CadId("sketch"),
                    CadExpression.Constant(20),
                    "+Z"))
                .Hole(new HoleOperation(
                    new CadId("hole"),
                    new CadId("body"),
                    new CadId("extrude"),
                    CadExpression.Constant(5),
                    CadExpression.Constant(10)))
                .Part;

        public static CadPart CreateWithIndependentSketch() =>
            CadPartProgram.Create("p", "P")
                .Sketch(new SketchOperation(
                    new CadId("a"),
                    new CadId("body"),
                    new Sketch(
                        new CadId("a"),
                        "A",
                        new SketchGeometry[]
                        {
                            new CircleGeometry(new CadId("ca"), 0, 0, 10)
                        },
                        Array.Empty<SketchConstraint>(),
                        Array.Empty<CadReference>())))
                .Extrude(new ExtrusionOperation(
                    new CadId("b"),
                    new CadId("body"),
                    new CadId("a"),
                    CadExpression.Constant(20),
                    "+Z"))
                .Sketch(new SketchOperation(
                    new CadId("independent"),
                    new CadId("body"),
                    new Sketch(
                        new CadId("independent"),
                        "Independent",
                        new SketchGeometry[]
                        {
                            new CircleGeometry(new CadId("ci"), 50, 0, 4)
                        },
                        Array.Empty<SketchConstraint>(),
                        Array.Empty<CadReference>())))
                .Part;
    }

    private sealed class MismatchingGateway : IKernelGateway
    {
        public Task<KernelOperationResponse> EvaluateAsync(
            KernelOperationRequest request,
            CancellationToken cancellationToken = default) =>
            Task.FromResult(
                KernelOperationResponse.Success(
                        request,
                        new CadResultId("wrong-result"),
                        "evidence")
                    with
                    {
                        OperationId = new CadId("wrong-operation")
                    });
    }

    private sealed class FailingFirstGateway : IKernelGateway
    {
        public Task<KernelOperationResponse> EvaluateAsync(
            KernelOperationRequest request,
            CancellationToken cancellationToken = default) =>
            request.OperationId.Value == "a"
                ? Task.FromResult(
                    KernelOperationResponse.Failure(
                        request,
                        "EXPECTED_TEST_FAILURE",
                        "Injected red-team failure."))
                : Task.FromResult(
                    KernelOperationResponse.Success(
                        request,
                        new CadResultId("result:" + request.OperationId.Value),
                        "evidence:" + request.OperationId.Value,
                        new[]
                        {
                            new KernelTopologyBinding("operation", request.OperationId.Value)
                        }));
    }
}
