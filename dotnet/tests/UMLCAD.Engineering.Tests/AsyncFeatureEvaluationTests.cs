using UMLCAD.Cad.Contracts;
using UMLCAD.Cad.Engine;
using UMLCAD.Cad.Semantics;

namespace UMLCAD.Engineering.Tests;

public sealed class AsyncFeatureEvaluationTests
{
    [Fact]
    public async Task CommonEvaluationEngineExecutesDependentTypedFeatures()
    {
        var part = new SemanticId(Guid.Parse("11111111-1111-1111-1111-111111111111"));
        var boxFeature = new SemanticId(Guid.Parse("22222222-2222-2222-2222-222222222222"));
        var extrusionFeature = new SemanticId(Guid.Parse("33333333-3333-3333-3333-333333333333"));
        var profileId = new SemanticId(Guid.Parse("44444444-4444-4444-4444-444444444444"));

        var box = new AxisAlignedBoxSolidSpecification(
            boxFeature,
            part,
            "Base",
            Array.Empty<SemanticId>(),
            0d, 0d, 0d,
            10d, 10d, 10d);

        var profile = new ConvexSketchProfileDefinition(
            profileId,
            part,
            "Profile",
            new SemanticVector3(0d, 0d, 10d),
            new SemanticVector3(1d, 0d, 0d),
            new SemanticVector3(0d, 1d, 0d),
            new[]
            {
                new SketchProfilePoint(1d, 1d),
                new SketchProfilePoint(3d, 1d),
                new SketchProfilePoint(3d, 3d),
                new SketchProfilePoint(1d, 3d),
            });

        var extrusion = new ExtrusionFeatureSpecification(
            extrusionFeature,
            part,
            "Pad",
            new[] { boxFeature },
            profile,
            5d);

        var boxEvaluator = new AxisAlignedBoxFeatureEvaluator(
            new FakeBoxGeometryService());

        var extrusionEvaluator = new ConvexProfileExtrusionFeatureEvaluator(
            new FakeExtrusionGeometryService());

        var options = new FeatureEvaluationOptions(
            "configuration:default",
            new KernelTolerance(1e-9, 1e-9),
            "model-standard-v1",
            "representation:none");

        var executors = FeatureEvaluationExecutorFactory.Create(
            new FeatureSpecification[] { box, extrusion },
            new IAuthoritativeFeatureEvaluator[] { boxEvaluator, extrusionEvaluator },
            options);

        var plan = EvaluationPlanner.Plan(
            new FeatureSpecification[] { box, extrusion }
                .Select(specification =>
                    EvaluationStepFactory.Create(
                        specification,
                        options.ConfigurationContext,
                        options.TolerancePolicy,
                        options.RepresentationPolicy)));

        var results = await new AsyncEvaluationEngine(executors).EvaluateAsync(plan);

        Assert.Equal(2, results.Count);
        Assert.All(results, result =>
            Assert.Equal(EvaluationOutcomeStatus.Succeeded, result.Status));
        Assert.Equal("solid:box", results[0].ResultIdentity!.Value);
        Assert.Equal("solid:extrusion", results[1].ResultIdentity!.Value);
        Assert.NotNull(results[0].Result);
        Assert.NotNull(results[1].Result);
    }

    [Fact]
    public async Task UnsuccessfulDependencyBlocksDownstreamFeature()
    {
        var first = new SemanticId(Guid.Parse("55555555-5555-5555-5555-555555555555"));
        var second = new SemanticId(Guid.Parse("66666666-6666-6666-6666-666666666666"));
        var part = new SemanticId(Guid.Parse("77777777-7777-7777-7777-777777777777"));

        var firstSpec = new AxisAlignedBoxSolidSpecification(
            first, part, "Bad Base", Array.Empty<SemanticId>(),
            0d, 0d, 0d, 1d, 1d, 1d);

        var secondSpec = new AxisAlignedBoxSolidSpecification(
            second, part, "Downstream", new[] { first },
            0d, 0d, 0d, 2d, 2d, 2d);

        var options = new FeatureEvaluationOptions(
            "configuration:default",
            new KernelTolerance(1e-9, 1e-9),
            "model-standard-v1",
            "representation:none");

        var failing = new AxisAlignedBoxFeatureEvaluator(
            new FakeBoxGeometryService
            {
                Result = new AxisAlignedBoxSolidKernelResult(
                    GeometryKernelStatus.Failed,
                    null,
                    null,
                    Array.Empty<AxisAlignedBoxSolidKernelTopology>(),
                    null,
                    null,
                    null,
                    ["intentional test failure"]),
            });

        var plan = EvaluationPlanner.Plan(
            new[] { firstSpec, secondSpec }.Select(spec =>
                EvaluationStepFactory.Create(
                    spec,
                    options.ConfigurationContext,
                    options.TolerancePolicy,
                    options.RepresentationPolicy)));

        var executors = FeatureEvaluationExecutorFactory.Create(
            new FeatureSpecification[] { firstSpec, secondSpec },
            new IAuthoritativeFeatureEvaluator[] { failing },
            options);

        var results = await new AsyncEvaluationEngine(executors).EvaluateAsync(plan);

        Assert.Single(results);
        Assert.Equal(EvaluationOutcomeStatus.Failed, results[0].Status);
    }

    private sealed class FakeBoxGeometryService
        : IAuthoritativeGeometryService
    {
        public AxisAlignedBoxSolidKernelResult Result { get; init; } =
            new(
                GeometryKernelStatus.Succeeded,
                new ContractResultId("solid:box"),
                "evidence:box",
                new[]
                {
                    new AxisAlignedBoxSolidKernelTopology("Face", "f_bottom"),
                    new AxisAlignedBoxSolidKernelTopology("Face", "f_top"),
                    new AxisAlignedBoxSolidKernelTopology("Face", "f_back"),
                    new AxisAlignedBoxSolidKernelTopology("Face", "f_front"),
                    new AxisAlignedBoxSolidKernelTopology("Face", "f_left"),
                    new AxisAlignedBoxSolidKernelTopology("Face", "f_right"),
                },
                1000d,
                600d,
                new KernelVector3(5d, 5d, 5d),
                Array.Empty<string>());

        public Task<AxisAlignedBoxSolidKernelResult> BuildAxisAlignedBoxSolidAsync(
            AxisAlignedBoxSolidRequest request,
            CancellationToken cancellationToken = default) =>
            Task.FromResult(Result);
    }

    private sealed class FakeExtrusionGeometryService
        : IExtrusionGeometryService
    {
        public Task<ExtrusionKernelResult> ExtrudeConvexPlanarProfileAsync(
            ExtrusionRequest request,
            CancellationToken cancellationToken = default) =>
            Task.FromResult(
                new ExtrusionKernelResult(
                    GeometryKernelStatus.Succeeded,
                    new ContractResultId("solid:extrusion"),
                    "evidence:extrusion",
                    new[]
                    {
                        new ExtrusionTopology("Face", "f_bottom"),
                        new ExtrusionTopology("Face", "f_top"),
                        new ExtrusionTopology("Face", "f_side_0"),
                        new ExtrusionTopology("Face", "f_side_1"),
                        new ExtrusionTopology("Face", "f_side_2"),
                        new ExtrusionTopology("Face", "f_side_3"),
                    },
                    20d,
                    52d,
                    new KernelVector3(2d, 2d, 12.5d),
                    Array.Empty<string>()));
    }
}
