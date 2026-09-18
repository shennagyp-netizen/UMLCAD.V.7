using System.Net;
using System.Text;
using UMLCAD.Cad.Contracts;

namespace UMLCAD.Engineering.Tests;

public sealed class ExtrusionAdapterTests
{
    [Fact]
    public async Task RustExtrusionAdapterMapsSuccessfulWireResult()
    {
        const string json = """
        {
          "schema": "uml-cad-extrude-convex-planar-profile/1.0.0",
          "status": "succeeded",
          "succeeded": true,
          "resultId": "solid:extrude-http-001",
          "evidenceHash": "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
          "topology": [
            { "kind": "Face", "key": "f_bottom" },
            { "kind": "Face", "key": "f_top" },
            { "kind": "Face", "key": "f_side_0" },
            { "kind": "Face", "key": "f_side_1" },
            { "kind": "Face", "key": "f_side_2" }
          ],
          "volume": 120.0,
          "surfaceArea": 148.0,
          "centroid": { "x": 3.0, "y": 4.5, "z": 6.0 },
          "diagnostics": []
        }
        """;

        using var client = new HttpClient(new FakeHandler(json))
        {
            BaseAddress = new Uri("http://kernel.test/")
        };

        var options = Microsoft.Extensions.Options.Options.Create(
            new UMLCAD.Kernel.Client.RustKernelOptions
            {
                BaseAddress = client.BaseAddress,
                RequestTimeout = TimeSpan.FromSeconds(10),
            });

        var service = new UMLCAD.Kernel.Client.RustExtrusionGeometryService(client, options);

        var result = await service.ExtrudeConvexPlanarProfileAsync(
            new ExtrusionRequest(
                "extrude-http-001",
                new KernelVector3(1d, 2d, 3d),
                new KernelVector3(1d, 0d, 0d),
                new KernelVector3(0d, 1d, 0d),
                new[]
                {
                    new PlanarProfilePoint(0d, 0d),
                    new PlanarProfilePoint(4d, 0d),
                    new PlanarProfilePoint(4d, 5d),
                    new PlanarProfilePoint(0d, 5d),
                },
                6d,
                new KernelTolerance(1e-9, 1e-9),
                new ContractVersion("1.0")));

        Assert.Equal(GeometryKernelStatus.Succeeded, result.Status);
        Assert.Equal(new ContractResultId("solid:extrude-http-001"), result.ResultId);
        Assert.Equal(5, result.Topology.Count);
        Assert.Equal(120d, result.Volume);
        Assert.Equal(148d, result.SurfaceArea);
        Assert.Equal(new KernelVector3(3d, 4.5d, 6d), result.Centroid);
    }

    [Fact]
    public void ExtrusionRequestRejectsUnsupportedContractVersion()
    {
        Assert.Throws<ArgumentException>(() =>
            new ExtrusionRequest(
                "bad-version",
                new KernelVector3(0d, 0d, 0d),
                new KernelVector3(1d, 0d, 0d),
                new KernelVector3(0d, 1d, 0d),
                new[]
                {
                    new PlanarProfilePoint(0d, 0d),
                    new PlanarProfilePoint(1d, 0d),
                    new PlanarProfilePoint(0d, 1d),
                },
                1d,
                new KernelTolerance(1e-9, 1e-9),
                new ContractVersion("99.0")));
    }

    private sealed class FakeHandler(string response) : HttpMessageHandler
    {
        protected override Task<HttpResponseMessage> SendAsync(
            HttpRequestMessage request,
            CancellationToken cancellationToken)
        {
            return Task.FromResult(
                new HttpResponseMessage(HttpStatusCode.OK)
                {
                    Content = new StringContent(response, Encoding.UTF8, "application/json"),
                    RequestMessage = request,
                });
        }
    }
}
