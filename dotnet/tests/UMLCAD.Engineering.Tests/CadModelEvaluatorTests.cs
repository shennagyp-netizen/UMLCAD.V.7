using UMLCAD.Cad.Contracts;
using UMLCAD.Cad.Engine;
using UMLCAD.Cad.Semantics;

namespace UMLCAD.Engineering.Tests;

public sealed class CadModelEvaluatorTests
{
    [Fact]
    public async Task CadModelEvaluatorExecutesOrderedTypedSpecifications()
    {
        var partId = new SemanticId(Guid.Parse("01010101-0101-0101-0101-010101010101"));
        var firstId = new SemanticId(Guid.Parse("02020202-0202-0202-0202-020202020202"));
        var secondId = new SemanticId(Guid.Parse("03030303-0303-0303-0303-030303030303"));
        var profileId = new SemanticId(Guid.Parse("04040404-0404-0404-0404-040404040404"));

        var baseFeature = new AxisAlignedBoxSolidSpecification(
            firstId,
            partId,
            "Base",
            Array.Empty<SemanticId>(),
            0d, 0d, 0d,
            10d, 10d, 10d);

        var profile = new ConvexSketchProfileDefinition(
            profileId,
            partId,
            "PadProfile",
            new SemanticVector3(2d, 2d, 10d),
            new SemanticVector3(1d, 0d, 0d),
            new SemanticVector3(0d, 1d, 0d),
            new[]
            {
                new SketchProfilePoint(0d, 0d),
                new SketchProfilePoint(2d, 0d),
                new SketchProfilePoint(2d, 2d),
                new SketchProfilePoint(0d, 2d),
            });

        var pad = new ExtrusionFeatureSpecification(
            secondId,
            partId,
            "Pad",
            new[] { firstId },
            profile,
            5d);

        var evaluator = new CadModelEvaluator(
            new IAuthoritativeFeatureEvaluator[]
            {
                new AxisAlignedBoxFeatureEvaluator(
                    new FakeBoxService()),
                new ConvexProfileExtrusionFeatureEvaluator(
                    new FakeExtrusionService()),
            });

        var snapshot = await evaluator.EvaluateAsync(
            new FeatureSpecification[] { pad, baseFeature },
            new FeatureEvaluationOptions(
                "configuration:default",
                new KernelTolerance(1e-9, 1e-9),
                "model-standard-v1",
                "representation:none"));

        Assert.Equal(new[] { firstId, secondId }, snapshot.Plan.OrderedStepIds);
        Assert.Equal(2, snapshot.Outcomes.Count);
        Assert.Equal(2, snapshot.SuccessfulResults.Count);
        Assert.All(snapshot.SuccessfulResults, result =>
            Assert.Equal(AuthoritativeResultStatus.Succeeded, result.Status));
    }

    private sealed class FakeBoxService : IAuthoritativeGeometryService
    {
        public Task<AxisAlignedBoxSolidKernelResult> BuildAxisAlignedBoxSolidAsync(
            AxisAlignedBoxSolidRequest request,
            CancellationToken cancellationToken = default) =>
            Task.FromResult(
                new AxisAlignedBoxSolidKernelResult(
                    GeometryKernelStatus.Succeeded,
                    new ContractResultId("solid:base"),
                    "evidence:base",
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
                    Array.Empty<string>()));
    }

    private sealed class FakeExtrusionService : IExtrusionGeometryService
    {
        public Task<ExtrusionKernelResult> ExtrudeConvexPlanarProfileAsync(
            ExtrusionRequest request,
            CancellationToken cancellationToken = default) =>
            Task.FromResult(
                new ExtrusionKernelResult(
                    GeometryKernelStatus.Succeeded,
                    new ContractResultId("solid:pad"),
                    "evidence:pad",
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
                    new KernelVector3(3d, 3d, 12.5d),
                    Array.Empty<string>()));
    }
}
