using UMLCAD.Cad.Contracts;
using UMLCAD.Cad.Engine;
using UMLCAD.Cad.Expressions;
using UMLCAD.Cad.Semantics;

namespace UMLCAD.ApplicationLayer.Tests;

public sealed class ReferenceResolutionTests
{
    [Fact]
    public async Task TopologyReferenceMustResolveAgainstAuthoritativeResult()
    {
        var part = SemanticPartFactory.CreateWithValidTopologyReference();

        var gateway = new TestGateway();
        var snapshot = await new CadEvaluationEngine(gateway)
            .EvaluateAsync(part.Definition);

        Assert.True(snapshot.Succeeded);
        Assert.Empty(snapshot.BodyDiagnostics);
    }

    [Fact]
    public async Task MissingTopologyReferenceFailsClosed()
    {
        var basePart = SemanticPartFactory.Create().Definition;
        var extrusion = (ExtrusionOperation)basePart.Operations[1];

        var invalid = basePart with
        {
            Operations = new CadOperation[]
            {
                basePart.Operations[0],
                extrusion with
                {
                    References = new[]
                    {
                        new CadReference(
                            new CadId("support"),
                            ReferenceTargetKind.Topology,
                            new CadResultId("r:sketch"),
                            "face",
                            "missing")
                    }
                },
                basePart.Operations[2]
            }
        };

        var snapshot = await new CadEvaluationEngine(new TestGateway())
            .EvaluateAsync(invalid);

        Assert.False(snapshot.Succeeded);
        Assert.Contains(
            snapshot.Outcomes[new CadId("extrude")].Diagnostics,
            diagnostic => diagnostic.Code == "CAD_REFERENCE_RESOLUTION_FAILED");
    }

    [Fact]
    public void ParameterizedGeometryChangesOperationDependencyIdentity()
    {
        var part = SemanticPartFactory.Create();
        var first = part.Definition.Operations[0];

        var original = CadEvaluationIdentityBuilder.Build(
            part.Definition,
            first,
            Array.Empty<CadResultEnvelope>(),
            new CadEvaluationOptions());

        var changed = part.AddParameter(
            new CadParameter(
                "unused",
                CadExpression.Constant(12),
                "mm"));

        var second = CadEvaluationIdentityBuilder.Build(
            changed.Definition,
            changed.Definition.Operations[0],
            Array.Empty<CadResultEnvelope>(),
            new CadEvaluationOptions());

        Assert.Equal(original, second);

        var parameterized = part.Definition with
        {
            Parameters = new[]
            {
                new CadParameter(
                    "base_radius",
                    CadExpression.Constant(25),
                    "mm")
            }
        };

        var third = CadEvaluationIdentityBuilder.Build(
            parameterized,
            parameterized.Operations[0],
            Array.Empty<CadResultEnvelope>(),
            new CadEvaluationOptions());

        Assert.NotEqual(original, third);
    }

    private sealed class TestGateway : IKernelGateway
    {
        public Task<KernelOperationResponse> EvaluateAsync(
            KernelOperationRequest request,
            CancellationToken cancellationToken = default)
        {
            cancellationToken.ThrowIfCancellationRequested();

            var result = request.OperationKind switch
            {
                "Cad.Sketch" => "r:sketch",
                "Cad.Extrusion" => "r:extrude",
                "Cad.Hole" => "r:hole",
                _ => "r:unknown"
            };

            var topology = request.OperationKind == "Cad.Sketch"
                ? new[]
                {
                    new KernelTopologyBinding("face", "profile"),
                    new KernelTopologyBinding("edge", "edge-1")
                }
                : new[]
                {
                    new KernelTopologyBinding("face", request.OperationId.Value)
                };

            return Task.FromResult(
                new KernelOperationResponse(
                    CadEvaluationStatus.Succeeded,
                    new CadResultId(result),
                    "evidence:" + request.EvaluationIdentity.Value,
                    topology,
                    Array.Empty<CadDiagnostic>()));
        }
    }

    private static class SemanticPartFactory
    {
        public static CadPartProgram Create()
        {
            var sketch = new Sketch(
                new CadId("sketch"),
                "Sketch",
                new SketchGeometry[]
                {
                    new CircleGeometry(
                        new CadId("profile"),
                        0,
                        0,
                        CadNumericValue.Expression(
                            CadExpression.Parameter("base_radius")))
                },
                Array.Empty<SketchConstraint>(),
                Array.Empty<CadReference>());

            return CadPartProgram.Create("p", "P")
                .AddParameter(
                    new CadParameter(
                        "base_radius",
                        CadExpression.Constant(20),
                        "mm"))
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
                        CadExpression.Constant(30),
                        "+Z"))
                .Hole(
                    new HoleOperation(
                        new CadId("hole"),
                        new CadId("body"),
                        new CadId("extrude"),
                        CadExpression.Constant(5),
                        CadExpression.Constant(10)));
        }
        
        public static CadPartProgram CreateWithValidTopologyReference()
        {
            var part = Create();
            var extrusion = (ExtrusionOperation)part.Definition.Operations[1];

            var updated = part.Definition with
            {
                Operations = new CadOperation[]
                {
                    part.Definition.Operations[0],
                    extrusion with
                    {
                        References = new[]
                        {
                            new CadReference(
                                new CadId("support"),
                                ReferenceTargetKind.Topology,
                                new CadResultId("r:sketch"),
                                "face",
                                "profile")
                        }
                    },
                    part.Definition.Operations[2]
                }
            };

            return new CadPartProgram(updated);
        }
    }
}
