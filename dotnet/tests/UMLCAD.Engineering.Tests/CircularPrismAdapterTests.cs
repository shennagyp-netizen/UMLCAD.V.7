using System.Net;
using System.Text;
using UMLCAD.Cad.Contracts;
using UMLCAD.Kernel.Client;

namespace UMLCAD.Engineering.Tests;

public sealed class CircularPrismAdapterTests
{
    [Fact]
    public async Task RustCircularPrismAdapterMapsExactPrimitiveContract()
    {
        const string json = """
        {
          "schema": "uml-cad-circular-prism-solid/1.0.0",
          "status": "succeeded",
          "succeeded": true,
          "resultId": "solid:cylinder-001",
          "evidenceHash": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
          "origin": { "x": 20.0, "y": 20.0, "z": 10.0 },
          "axis": { "x": 0.0, "y": 0.0, "z": 1.0 },
          "radius": 5.0,
          "depth": 10.0,
          "volume": 785.3981633974483,
          "surfaceArea": 471.238898038469,
          "centroid": { "x": 20.0, "y": 20.0, "z": 15.0 },
          "bounds": {
            "min": { "x": 15.0, "y": 15.0, "z": 10.0 },
            "max": { "x": 25.0, "y": 25.0, "z": 20.0 }
          },
          "topology": [
            { "kind": "Face", "key": "bottom" },
            { "kind": "Face", "key": "top" },
            { "kind": "Face", "key": "lateral" }
          ],
          "diagnostics": []
        }
        """;

        using var client = new HttpClient(new FakeHandler(json))
        {
            BaseAddress = new Uri("http://kernel.test/")
        };

        var service = new RustCircularPrismGeometryService(
            client,
            Microsoft.Extensions.Options.Options.Create(
                new RustKernelOptions
                {
                    BaseAddress = client.BaseAddress,
                    RequestTimeout = TimeSpan.FromSeconds(10)
                }));

        var result = await service.BuildCircularPrismSolidAsync(
            new CircularPrismSolidRequest(
                "cylinder-001",
                new KernelVector3(20d, 20d, 10d),
                new KernelVector3(0d, 0d, 1d),
                5d,
                10d,
                new KernelTolerance(1e-9, 1e-9),
                new ContractVersion("1.0")));

        Assert.Equal(GeometryKernelStatus.Succeeded, result.Status);
        Assert.True(result.Succeeded);
        Assert.Equal(new ContractResultId("solid:cylinder-001"), result.ResultId);
        Assert.Equal(785.3981633974483, result.Volume);
        Assert.Equal(471.238898038469, result.SurfaceArea);
        Assert.Equal(new KernelVector3(20d, 20d, 15d), result.Centroid);
        Assert.Equal(3, result.Topology.Count);
    }

    [Fact]
    public void CircularPrismRequestRejectsDegenerateParameters()
    {
        Assert.Throws<ArgumentException>(() =>
            new CircularPrismSolidRequest(
                "bad-radius",
                new KernelVector3(0d, 0d, 0d),
                new KernelVector3(0d, 0d, 1d),
                0d,
                1d,
                new KernelTolerance(1e-9, 1e-9),
                new ContractVersion("1.0")));

        Assert.Throws<ArgumentException>(() =>
            new CircularPrismSolidRequest(
                "bad-axis",
                new KernelVector3(0d, 0d, 0d),
                new KernelVector3(0d, 0d, 0d),
                1d,
                1d,
                new KernelTolerance(1e-9, 1e-9),
                new ContractVersion("1.0")));

        Assert.Throws<ArgumentException>(() =>
            new CircularPrismSolidRequest(
                "bad-depth",
                new KernelVector3(0d, 0d, 0d),
                new KernelVector3(0d, 0d, 1d),
                1d,
                0d,
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
            Assert.Equal(
                "http://kernel.test/v1/geometry/circular-prism-solid",
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
