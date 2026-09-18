using System.Net;
using System.Text;
using UMLCAD.Cad.Contracts;
using UMLCAD.Kernel.Client;

namespace UMLCAD.Engineering.Tests;

public sealed class SketchConstraintAdapterTests
{
    [Fact]
    public async Task RustSketchConstraintAdapterMapsSuccessfulTypedSolve()
    {
        const string json = """
        {
          "schema": "uml-cad-sketch-solve/1.0.0",
          "status": "succeeded",
          "succeeded": true,
          "reason": "converged",
          "iterations": 0,
          "initialResidualNorm": 0.0,
          "finalResidualNorm": 0.0,
          "initialScaledResidualNorm": 0.0,
          "finalScaledResidualNorm": 0.0,
          "finalStepNorm": 0.0,
          "variableCount": 6,
          "equationCount": 0,
          "rank": 0,
          "degreesOfFreedom": 6,
          "conditionEstimate": 1.0,
          "geometry": [
            { "id": "circle-a", "kind": "circle", "x": 20.0, "y": 20.0, "radius": 5.0 },
            { "id": "circle-b", "kind": "circle", "x": 70.0, "y": 30.0, "radius": 4.0 }
          ],
          "diagnostics": []
        }
        """;

        using var client = new HttpClient(new FakeHandler(json))
        {
            BaseAddress = new Uri("http://kernel.test/")
        };

        var options = Microsoft.Extensions.Options.Options.Create(
            new RustKernelOptions
            {
                BaseAddress = client.BaseAddress,
                RequestTimeout = TimeSpan.FromSeconds(10),
            });

        var service = new RustSketchConstraintService(client, options);

        var result = await service.SolveAsync(
            new SketchSolveRequest(
                "sketch-evaluation-001",
                new[]
                {
                    new SketchKernelCircle("circle-a", 20d, 20d, 5d),
                    new SketchKernelCircle("circle-b", 70d, 30d, 4d),
                },
                new[]
                {
                    new SketchKernelFixedConstraint("constraint-a", "circle-a"),
                    new SketchKernelFixedConstraint("constraint-b", "circle-b"),
                },
                new KernelTolerance(1e-9, 1e-9),
                new KernelSolveOptions(
                    100,
                    1e-8,
                    1e-10,
                    1e-3)));

        Assert.Equal(GeometryKernelStatus.Succeeded, result.Status);
        Assert.Equal("converged", result.Reason);
        Assert.True(result.Succeeded);
        Assert.Equal(2, result.Geometry.Count);
        Assert.Equal("circle-a", result.Geometry[0].Id);
        Assert.Equal(5d, result.Geometry[0].Radius);
        Assert.Equal(0d, result.FinalResidualNorm);
    }

    [Fact]
    public void SketchSolveRequestCanonicalizesNonSemanticCollectionOrder()
    {
        var first = new SketchSolveRequest(
            "same-operation",
            new[]
            {
                new SketchKernelCircle("circle-b", 2d, 3d, 1d),
                new SketchKernelCircle("circle-a", 1d, 2d, 2d),
            },
            new[]
            {
                new SketchKernelFixedConstraint("fixed-b", "circle-b"),
                new SketchKernelFixedConstraint("fixed-a", "circle-a"),
            },
            new KernelTolerance(1e-9, 1e-9),
            KernelSolveOptions.Default);

        var second = new SketchSolveRequest(
            "same-operation",
            new[]
            {
                new SketchKernelCircle("circle-a", 1d, 2d, 2d),
                new SketchKernelCircle("circle-b", 2d, 3d, 1d),
            },
            new[]
            {
                new SketchKernelFixedConstraint("fixed-a", "circle-a"),
                new SketchKernelFixedConstraint("fixed-b", "circle-b"),
            },
            new KernelTolerance(1e-9, 1e-9),
            KernelSolveOptions.Default);

        Assert.Equal(first.Circles, second.Circles);
        Assert.Equal(first.FixedConstraints, second.FixedConstraints);
    }

    [Fact]
    public void SketchSolveRequestRejectsDuplicateAndStaleConstraintReferences()
    {
        Assert.Throws<ArgumentException>(() =>
            new SketchSolveRequest(
                "duplicate-circle",
                new[]
                {
                    new SketchKernelCircle("circle-a", 0d, 0d, 1d),
                    new SketchKernelCircle("circle-a", 1d, 0d, 1d),
                },
                Array.Empty<SketchKernelFixedConstraint>(),
                new KernelTolerance(1e-9, 1e-9),
                KernelSolveOptions.Default));

        Assert.Throws<ArgumentException>(() =>
            new SketchSolveRequest(
                "stale-reference",
                new[] { new SketchKernelCircle("circle-a", 0d, 0d, 1d) },
                new[]
                {
                    new SketchKernelFixedConstraint("fixed-a", "missing-circle")
                },
                new KernelTolerance(1e-9, 1e-9),
                KernelSolveOptions.Default));
    }

    [Fact]
    public async Task RustSketchConstraintAdapterFailsClosedOnInconsistentStatus()
    {
        const string json = """
        {
          "schema": "uml-cad-sketch-solve/1.0.0",
          "status": "succeeded",
          "succeeded": false,
          "reason": "max-iterations",
          "iterations": 100,
          "initialResidualNorm": 1.0,
          "finalResidualNorm": 1.0,
          "initialScaledResidualNorm": 1.0,
          "finalScaledResidualNorm": 1.0,
          "finalStepNorm": 0.0,
          "variableCount": 6,
          "equationCount": 0,
          "rank": 0,
          "degreesOfFreedom": 6,
          "conditionEstimate": 1.0,
          "geometry": [],
          "diagnostics": ["solver status is inconsistent"]
        }
        """;

        using var client = new HttpClient(new FakeHandler(json))
        {
            BaseAddress = new Uri("http://kernel.test/")
        };

        var service = new RustSketchConstraintService(
            client,
            Microsoft.Extensions.Options.Options.Create(
                new RustKernelOptions
                {
                    BaseAddress = client.BaseAddress,
                    RequestTimeout = TimeSpan.FromSeconds(10)
                }));

        var result = await service.SolveAsync(
            new SketchSolveRequest(
                "sketch-evaluation-002",
                new[] { new SketchKernelCircle("circle-a", 0d, 0d, 1d) },
                new[] { new SketchKernelFixedConstraint("fixed-a", "circle-a") },
                new KernelTolerance(1e-9, 1e-9),
                new KernelSolveOptions(100, 1e-8, 1e-10, 1e-3)));

        Assert.Equal(GeometryKernelStatus.Failed, result.Status);
        Assert.False(result.Succeeded);
        Assert.Contains(result.Diagnostics, x =>
            x.Contains("inconsistent", StringComparison.Ordinal));
    }

    private sealed class FakeHandler(string response) : HttpMessageHandler
    {
        protected override Task<HttpResponseMessage> SendAsync(
            HttpRequestMessage request,
            CancellationToken cancellationToken)
        {
            Assert.Equal(HttpMethod.Post, request.Method);
            Assert.Equal(
                "http://kernel.test/v1/geometry/solve-sketch",
                request.RequestUri!.ToString());

            return Task.FromResult(
                new HttpResponseMessage(HttpStatusCode.OK)
                {
                    Content = new StringContent(response, Encoding.UTF8, "application/json"),
                    RequestMessage = request,
                });
        }
    }
}
