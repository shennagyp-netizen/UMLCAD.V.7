using UMLCAD.Cad.Contracts;
using UMLCAD.Cad.Engine;
using UMLCAD.Cad.Expressions;
using UMLCAD.Cad.Semantics;

namespace UMLCAD.ApplicationLayer.Tests;

public sealed class SemanticPipelineTests
{
    [Fact]
    public async Task SketchExtrusionHoleProducesCurrentFinalBody()
    {
        var part = CreatePart();
        var gateway = new RecordingGateway();
        var engine = new CadEvaluationEngine(gateway);

        var snapshot = await engine.EvaluateAsync(part.Definition);

        Assert.True(snapshot.Succeeded);
        Assert.Equal(
            new[]
            {
                new CadId("sketch"),
                new CadId("extrude"),
                new CadId("hole")
            },
            snapshot.Plan.OperationIds);

        Assert.Equal(
            new CadResultId("r:hole"),
            snapshot.CurrentBody(new CadId("body")));

        Assert.Equal(
            new[] { "Cad.Sketch", "Cad.Extrusion", "Cad.Hole" },
            gateway.Requests.Select(x => x.OperationKind));

        Assert.Equal(
            new[] { 0, 1, 1 },
            gateway.Requests.Select(x => x.InputResults.Count));
    }

    [Fact]
    public void SketchChangeInvalidatesEveryDownstreamOperation()
    {
        var graph = new CadDependencyGraph(CreatePart().Definition);

        var affected = graph.InvalidationClosure(
            new[] { new CadId("sketch") });

        Assert.Equal(
            new[] { "extrude", "hole", "sketch" },
            affected.OrderBy(
                x => x.Value,
                StringComparer.Ordinal)
                .Select(x => x.Value));
    }

    [Fact]
    public void CycleFailsClosed()
    {
        var part = CadPartProgram.Create("p", "P")
            .Extrude(
                new ExtrusionOperation(
                    new CadId("a"),
                    new CadId("body"),
                    new CadId("b"),
                    CadExpression.Constant(1),
                    "+Z"))
            .Extrude(
                new ExtrusionOperation(
                    new CadId("b"),
                    new CadId("body"),
                    new CadId("a"),
                    CadExpression.Constant(1),
                    "+Z"));

        Assert.Throws<InvalidOperationException>(
            () => new CadDependencyGraph(part.Definition).Plan());
    }

    [Fact]
    public void SketchContentsContributeToSemanticIdentity()
    {
        var part = CreatePart().Definition;
        var original = part.Operations[0];
        var alteredSketch =
            ((SketchOperation)original).Definition with
            {
                Geometry = new SketchGeometry[]
                {
                    new CircleGeometry(
                        new CadId("circle"),
                        0,
                        0,
                        11)
                }
            };

        var altered = part with
        {
            Operations = new CadOperation[]
            {
                new SketchOperation(
                    new CadId("sketch"),
                    new CadId("body"),
                    alteredSketch),
                part.Operations[1],
                part.Operations[2]
            }
        };

        var firstIdentity =
            CadEvaluationIdentityBuilder.Build(
                part,
                part.Operations[0],
                Array.Empty<CadResultEnvelope>(),
                new CadEvaluationOptions());

        var secondIdentity =
            CadEvaluationIdentityBuilder.Build(
                altered,
                altered.Operations[0],
                Array.Empty<CadResultEnvelope>(),
                new CadEvaluationOptions());

        Assert.NotEqual(firstIdentity, secondIdentity);
    }

    private static CadPartProgram CreatePart()
    {
        var sketch = new Sketch(
            new CadId("sketch"),
            "Sketch",
            new SketchGeometry[]
            {
                new CircleGeometry(
                    new CadId("circle"),
                    0,
                    0,
                    10)
            },
            Array.Empty<SketchConstraint>(),
            Array.Empty<CadReference>());

        return CadPartProgram.Create("p", "P")
            .AddSketch(
                new SketchOperation(
                    new CadId("sketch"),
                    new CadId("body"),
                    sketch))
            .Extrude(
                new ExtrusionOperation(
                    new CadId("extrude"),
                    new CadId("body"),
                    new CadId("sketch"),
                    CadExpression.Constant(20),
                    "+Z"))
            .Hole(
                new HoleOperation(
                    new CadId("hole"),
                    new CadId("body"),
                    new CadId("extrude"),
                    CadExpression.Constant(5),
                    CadExpression.Constant(10)));
    }

    private sealed class RecordingGateway : IKernelGateway
    {
        public List<KernelOperationRequest> Requests { get; } = [];

        public Task<KernelOperationResponse> EvaluateAsync(
            KernelOperationRequest request,
            CancellationToken cancellationToken = default)
        {
            Requests.Add(request);

            var result = request.OperationKind switch
            {
                "Cad.Sketch" => "r:sketch",
                "Cad.Extrusion" => "r:extrude",
                "Cad.Hole" => "r:hole",
                _ => "r:unknown"
            };

            return Task.FromResult(
                new KernelOperationResponse(
                    CadEvaluationStatus.Succeeded,
                    new CadResultId(result),
                    "evidence:" + request.OperationId.Value,
                    new[]
                    {
                        new KernelTopologyBinding(
                            "operation",
                            request.OperationId.Value)
                    },
                    Array.Empty<CadDiagnostic>()));
        }
    }
}
