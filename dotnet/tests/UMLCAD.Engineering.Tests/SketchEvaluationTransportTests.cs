using System.Net;
using System.Text;
using System.Text.Json;
using System.Net.Http.Json;
using UMLCAD.Cad.Contracts;
using UMLCAD.Kernel.Client;

namespace UMLCAD.Engineering.Tests;

public sealed class SketchEvaluationTransportTests
{
    [Fact]
    public async Task RustSketchAdapterMapsAuthoritativeSolvedCircles()
    {
        const string json = """
        {
          "schema": "uml-cad-sketch-solve/1.0.0",
          "status": "succeeded",
          "succeeded": true,
          "resultId": "sketch:result-001",
          "evidenceHash": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
          "circles": [
            { "id": "circle-a", "x": 20.0, "y": 20.0, "radius": 5.0 },
            { "id": "circle-b", "x": 70.0, "y": 30.0, "radius": 4.0 }
          ],
          "converged": true,
          "reason": "Converged",
          "iterations": 1,
          "finalResidualNorm": 0.0,
          "finalScaledResidualNorm": 0.0,
          "finalStepNorm": 0.0,
          "degreesOfFreedom": 0,
          "variableCount": 6,
          "equationCount": 6,
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
                RequestTimeout = TimeSpan.FromSeconds(10)
            });

        var service = new RustSketchGeometryService(client, options);

        var request = new SketchSolveRequest(
            "sketch-eval-001",
            new CadId("sketch-001"),
            new CadFrame(new CadId("sketch-frame"), CadFrameKind.Sketch, 0d, 0d, 10d),
            new[]
            {
                new SketchCircle(new CadId("circle-a"), 20d, 20d, 5d),
                new SketchCircle(new CadId("circle-b"), 70d, 30d, 4d)
            },
            new[]
            {
                new SketchConstraintSpecification(
                    new CadId("fixed-a"),
                    SketchConstraintKind.Fixed,
                    new CadId("circle-a")),
                new SketchConstraintSpecification(
                    new CadId("fixed-b"),
                    SketchConstraintKind.Fixed,
                    new CadId("circle-b"))
            },
            new KernelTolerance(1e-9, 1e-9),
            new ContractVersion("1.0"));

        var result = await service.SolveAsync(request);

        Assert.Equal(GeometryKernelStatus.Succeeded, result.Status);
        Assert.Equal(new ContractResultId("sketch:result-001"), result.ResultId);
        Assert.Equal(2, result.Circles.Count);
        Assert.True(result.Converged);
        Assert.Equal(0, result.DegreesOfFreedom);
        Assert.Equal(6, result.VariableCount);
        Assert.Equal(6, result.EquationCount);
        Assert.Equal(new SketchCircleResult(
            "circle-a", 20d, 20d, 5d), result.Circles[0]);
    }

    [Fact]
    public void SketchSolveRequestRejectsNonFixedConstraintTransportUntilExplicitlyCertified()
    {
        Assert.Throws<ArgumentException>(() =>
            new SketchSolveRequest(
                "sketch-invalid-constraint",
                new CadId("sketch-001"),
                new CadFrame(new CadId("sketch-frame"), CadFrameKind.Sketch, 0d, 0d, 0d),
                new[]
                {
                    new SketchCircle(new CadId("circle-a"), 20d, 20d, 5d)
                },
                new[]
                {
                    new SketchConstraintSpecification(
                        new CadId("fully-constrained"),
                        SketchConstraintKind.FullyConstrained,
                        new CadId("circle-a"))
                },
                new KernelTolerance(1e-9, 1e-9),
                new ContractVersion("1.0")));
    }

    private sealed class FakeHandler(string response) : HttpMessageHandler
    {
        protected override Task<HttpResponseMessage> SendAsync(
            HttpRequestMessage request,
            CancellationToken cancellationToken)
        {
            Assert.Equal(HttpMethod.Post, request.Method);
            Assert.Equal("http://kernel.test/v1/sketch/solve", request.RequestUri!.ToString());

            var payload = await request.Content!.ReadFromJsonAsync<JsonElement>(cancellationToken);
            Assert.Equal(SketchSolveRequest.ContractSchema, payload.GetProperty("schema").GetString());
            Assert.Equal("sketch-eval-001", payload.GetProperty("operationIdentity").GetString());
            Assert.Equal("sketch-001", payload.GetProperty("sketchId").GetString());
            Assert.Equal("circle-a", payload.GetProperty("circles")[0].GetProperty("id").GetString());
            Assert.Equal("circle-b", payload.GetProperty("circles")[1].GetProperty("id").GetString());
            Assert.Equal("fixed", payload.GetProperty("constraints")[0].GetProperty("kind").GetString());
            Assert.Equal(1e-9, payload.GetProperty("tolerance").GetProperty("absolute").GetDouble());

            return Task.FromResult(
                new HttpResponseMessage(HttpStatusCode.OK)
                {
                    Content = new StringContent(response, Encoding.UTF8, "application/json"),
                    RequestMessage = request
                });
        }
    }
}
