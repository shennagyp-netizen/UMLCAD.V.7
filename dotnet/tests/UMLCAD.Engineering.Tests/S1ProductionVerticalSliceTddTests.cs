using UMLCAD.Cad.Contracts;
using UMLCAD.Cad.Engine;
using UMLCAD.Kernel.Client;

namespace UMLCAD.Engineering.Tests;

public sealed class S1ProductionVerticalSliceTddTests
{
    [Fact]
    public async Task Production_evaluator_must_execute_the_mandatory_sketch_to_add_remove_slice()
    {
        var baseFeature = new BoxFeatureSpecification(
            new CadId("base"),
            new CadFrame(new CadId("part"), CadFrameKind.Part, 0d, 0d, 0d),
            100d, 80d, 10d);

        var support = new CadReference(
            new CadId("support-face"),
            ReferenceKind.Support,
            baseFeature.Id,
            TopologySelector.PlanarFaceByNormalAndPoint(
                new CadVector3(0d, 0d, 1d),
                new CadVector3(50d, 40d, 10d)),
            new ReferenceContext(CadFrameKind.Part, "default"));

        var sketch = new SketchFeatureSpecification(
            new CadId("sketch"),
            support,
            new CadFrame(new CadId("sketch-frame"), CadFrameKind.Sketch, 50d, 40d, 10d),
            new[]
            {
                new SketchCircle(new CadId("circle-a"), 20d, 20d, 5d),
                new SketchCircle(new CadId("circle-b"), 70d, 30d, 4d)
            },
            new[]
            {
                new SketchConstraintSpecification(
                    new CadId("circle-a-fixed"),
                    SketchConstraintKind.Fixed,
                    new CadId("circle-a")),
                new SketchConstraintSpecification(
                    new CadId("circle-b-fixed"),
                    SketchConstraintKind.Fixed,
                    new CadId("circle-b"))
            });

        var featureA = new ExtrusionFeatureSpecification(
            new CadId("feature-a"),
            sketch.Id,
            support,
            new CadVector3(0d, 0d, 1d),
            10d,
            FeatureBooleanOperation.Add);

        var featureB = new ExtrusionFeatureSpecification(
            new CadId("feature-b"),
            sketch.Id,
            support,
            new CadVector3(0d, 0d, -1d),
            8d,
            FeatureBooleanOperation.Remove);

        var document = new CadDocumentSpecification(
            new CadId("s1-production-red"),
            "A",
            "mm",
            new CadFeatureSpecification[] { baseFeature, sketch, featureA, featureB })
        {
            EvaluationTolerance = new KernelTolerance(1e-9, 1e-9)
        };

        var evaluator = new RustCadKernelEvaluator(new DeterministicBoxService());
        var result = await new CadEvaluationEngine(evaluator).RecomputeAsync(document);

        Assert.True(
            result.Succeeded,
            $"Mandatory S1 production slice is not executable: {result.Failure?.Code} {result.Failure?.Message}");

        Assert.Equal(
            new[] { "base", "sketch", "feature-a", "feature-b" },
            result.EvaluationPlan.Select(x => x.Value));

        Assert.NotNull(result.FinalAuthoritativeResult);
        Assert.NotNull(result.Representation);
        Assert.Equal(
            result.FinalAuthoritativeResult!.ResultId,
            result.Representation!.SourceResultId);
    }

    private sealed class DeterministicBoxService : IAuthoritativeGeometryService
    {
        public Task<AxisAlignedBoxSolidKernelResult> BuildAxisAlignedBoxSolidAsync(
            AxisAlignedBoxSolidRequest request,
            CancellationToken cancellationToken = default)
        {
            cancellationToken.ThrowIfCancellationRequested();

            return Task.FromResult(
                new AxisAlignedBoxSolidKernelResult(
                    GeometryKernelStatus.Succeeded,
                    new ContractResultId("solid:s1-base"),
                    "evidence:s1-base",
                    new[]
                    {
                        new AxisAlignedBoxSolidKernelTopology("Face", "f_bottom"),
                        new AxisAlignedBoxSolidKernelTopology("Face", "f_top"),
                        new AxisAlignedBoxSolidKernelTopology("Face", "f_back"),
                        new AxisAlignedBoxSolidKernelTopology("Face", "f_front"),
                        new AxisAlignedBoxSolidKernelTopology("Face", "f_left"),
                        new AxisAlignedBoxSolidKernelTopology("Face", "f_right")
                    },
                    80000d,
                    35600d,
                    new KernelVector3(50d, 40d, 5d),
                    Array.Empty<string>()));
        }
    }
}
