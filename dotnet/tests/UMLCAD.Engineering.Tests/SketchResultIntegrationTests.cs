using UMLCAD.Cad.Contracts;
using UMLCAD.Kernel.Client;

namespace UMLCAD.Engineering.Tests;

public sealed class SketchResultIntegrationTests
{
    [Fact]
    public async Task Production_sketch_evaluation_preserves_the_authoritative_sketch_frame()
    {
        var frame = new CadFrame(
            new CadId("sketch-frame"),
            CadFrameKind.Sketch,
            50d,
            40d,
            10d)
        {
            XAxis = new CadVector3(0d, 1d, 0d),
            YAxis = new CadVector3(-1d, 0d, 0d),
            ZAxis = new CadVector3(0d, 0d, 1d)
        };

        var sketch = new SketchFeatureSpecification(
            new CadId("sketch"),
            new CadReference(
                new CadId("support"),
                ReferenceKind.Support,
                new CadId("base"),
                TopologySelector.PlanarFaceByNormalAndPoint(
                    new CadVector3(0d, 0d, 1d),
                    new CadVector3(50d, 40d, 10d)),
                new ReferenceContext(CadFrameKind.Part, "default")),
            frame,
            new[]
            {
                new SketchCircle(new CadId("circle-a"), 2d, 3d, 5d)
            },
            new[]
            {
                new SketchConstraintSpecification(
                    new CadId("fixed-a"),
                    SketchConstraintKind.Fixed,
                    new CadId("circle-a"))
            });

        var solver = new DeterministicSketchService();
        var evaluator = new RustCadKernelEvaluator(
            new UnusedBoxService(),
            solver);

        var response = await evaluator.EvaluateAsync(
            new KernelEvaluationRequest(
                new CadId("sketch-evaluation"),
                sketch,
                null,
                Array.Empty<ReferenceResolution>(),
                Array.Empty<CadFeatureEvaluationResult>())
            {
                Tolerance = new KernelTolerance(1e-9, 1e-9)
            });

        Assert.Equal(CadEvaluationStatus.Succeeded, response.Status);
        Assert.NotNull(response.SketchResult);
        Assert.Equal(frame, response.SketchResult!.Frame);
        Assert.Equal(sketch.Id, response.SketchResult.SketchId);
        Assert.Single(response.SketchResult.Circles);
    }

    private sealed class DeterministicSketchService : ISketchConstraintService
    {
        public Task<SketchSolveKernelResult> SolveAsync(
            SketchSolveRequest request,
            CancellationToken cancellationToken = default)
        {
            cancellationToken.ThrowIfCancellationRequested();
            return Task.FromResult(
                new SketchSolveKernelResult(
                    GeometryKernelStatus.Succeeded,
                    true,
                    "converged",
                    0,
                    0d,
                    0d,
                    0d,
                    0d,
                    0d,
                    request.Circles.Count * 3,
                    0,
                    0,
                    request.Circles.Count * 3,
                    1d,
                    request.Circles.Select(circle =>
                        new SketchSolvedCircle(circle.Id, circle.X, circle.Y, circle.Radius)).ToArray(),
                    Array.Empty<string>()));
        }
    }

    private sealed class UnusedBoxService : IAuthoritativeGeometryService
    {
        public Task<AxisAlignedBoxSolidKernelResult> BuildAxisAlignedBoxSolidAsync(
            AxisAlignedBoxSolidRequest request,
            CancellationToken cancellationToken = default) =>
            throw new InvalidOperationException("The box service must not be called by sketch evaluation.");
    }
}
