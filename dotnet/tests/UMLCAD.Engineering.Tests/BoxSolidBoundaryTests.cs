using System.Net;
using System.Text;
using System.Text.Json;
using UMLCAD.Cad.Contracts;
using UMLCAD.Cad.Engine;
using UMLCAD.Cad.Semantics;

namespace UMLCAD.Engineering.Tests;

public sealed class BoxSolidBoundaryTests
{
    [Fact]
    public void BoxSolidResultIntegrationPreservesSixFaceTopology()
    {
        var result = new AxisAlignedBoxSolidKernelResult(
            AxisAlignedBoxSolidKernelStatus.Succeeded,
            new ContractResultId("solid:001"),
            "evidence-001",
            new[]
            {
                new AxisAlignedBoxSolidKernelTopology("Face", "f_bottom"),
                new AxisAlignedBoxSolidKernelTopology("Face", "f_top"),
                new AxisAlignedBoxSolidKernelTopology("Face", "f_back"),
                new AxisAlignedBoxSolidKernelTopology("Face", "f_front"),
                new AxisAlignedBoxSolidKernelTopology("Face", "f_left"),
                new AxisAlignedBoxSolidKernelTopology("Face", "f_right"),
            },
            6000d,
            2200d,
            new KernelVector3(5d, 10d, 15d),
            Array.Empty<string>());

        var cadResult = ResultIntegrator.Integrate(SemanticId.New(), result);

        Assert.Equal(AuthoritativeResultStatus.Succeeded, cadResult.Status);
        Assert.Equal(AuthoritativeResultKind.Solid, cadResult.Kind);
        Assert.Equal(6, cadResult.TopologyBindings.Count);
        Assert.All(cadResult.TopologyBindings, binding =>
            Assert.Equal("Face", binding.TopologyKind));
    }

    [Fact]
    public void BoxSolidContractRejectsSuccessfulResultWithoutSixFaces()
    {
        Assert.Throws<ArgumentException>(() =>
            new AxisAlignedBoxSolidKernelResult(
                AxisAlignedBoxSolidKernelStatus.Succeeded,
                new ContractResultId("solid:002"),
                "evidence-002",
                new[]
                {
                    new AxisAlignedBoxSolidKernelTopology("Face", "f_top"),
                },
                1d,
                6d,
                new KernelVector3(0.5d, 0.5d, 0.5d),
                Array.Empty<string>()));
    }

    [Fact]
    public async Task RustGeometryAdapterMapsTheTypedHttpResponse()
    {
        const string json = """
        {
          "schema": "uml-cad-axis-aligned-box-solid/1.0.0",
          "succeeded": true,
          "resultId": "solid:http-001",
          "evidenceHash": "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee",
          "topology": [
            { "kind": "Face", "key": "f_bottom" },
            { "kind": "Face", "key": "f_top" },
            { "kind": "Face", "key": "f_back" },
            { "kind": "Face", "key": "f_front" },
            { "kind": "Face", "key": "f_left" },
            { "kind": "Face", "key": "f_right" }
          ],
          "volume": 24.0,
          "surfaceArea": 52.0,
          "centroid": { "x": 1.0, "y": 1.5, "z": 2.0 },
          "diagnostics": []
        }
        """;

        using var client = new HttpClient(new FakeHandler(json))
        {
            BaseAddress = new Uri("http://kernel.test/")
        };

        var options = Microsoft.Extensions.Options.Options.Create(new UMLCAD.Kernel.Client.RustKernelOptions
        {
            BaseAddress = client.BaseAddress,
            RequestTimeout = TimeSpan.FromSeconds(10)
        });

        var service = new UMLCAD.Kernel.Client.RustGeometryKernelService(client, options);

        var result = await service.BuildAxisAlignedBoxSolidAsync(
            new AxisAlignedBoxSolidRequest(
                "http-seed-001",
                new KernelVector3(0d, 0d, 0d),
                new KernelVector3(2d, 3d, 4d),
                new KernelTolerance(1e-9, 1e-9),
                new ContractVersion("1.0")));

        Assert.Equal(AxisAlignedBoxSolidKernelStatus.Succeeded, result.Status);
        Assert.Equal(new ContractResultId("solid:http-001"), result.ResultId);
        Assert.Equal(6, result.Topology.Count);
        Assert.Equal(24d, result.Volume);
        Assert.Equal(52d, result.SurfaceArea);
        Assert.Equal(new KernelVector3(1d, 1.5d, 2d), result.Centroid);
    }

    private sealed class FakeHandler(string response) : HttpMessageHandler
    {
        protected override Task<HttpResponseMessage> SendAsync(
            HttpRequestMessage request,
            CancellationToken cancellationToken)
        {
            var body = new StringContent(response, Encoding.UTF8, "application/json");
            return Task.FromResult(new HttpResponseMessage(HttpStatusCode.OK)
            {
                Content = body,
                RequestMessage = request,
            });
        }
    }
}
