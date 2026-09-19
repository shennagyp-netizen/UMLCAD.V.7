using System.Net;
using System.Text;
using UMLCAD.Kernel;

namespace UMLCAD.ApplicationLayer.Tests;

public sealed class KernelGatewayTests
{
    [Fact]
    public void KernelApi_Is_Concrete_And_Sealed()
    {
        Assert.True(typeof(UmlcadKernel).IsSealed);
        Assert.False(typeof(UmlcadKernel).IsAbstract);
        Assert.Null(typeof(UMLCAD.Kernel).Assembly.GetType("UMLCAD.Kernel.IRustKernelService"));
    }

    [Fact]
    public async Task KernelGateway_Returns_Explicit_Failure_For_Invalid_Response()
    {
        using var httpClient = new HttpClient(new StubHandler(
            HttpStatusCode.OK,
            "{ invalid"))
        {
            BaseAddress = new Uri("http://127.0.0.1/")
        };

        var kernel = new UmlcadKernel(httpClient);

        using var semantic = System.Text.Json.JsonDocument.Parse("{\"parts\":[]}");
        var request = new KernelBuildRequest("test-app", "1.0.0", "identity-1", semantic.RootElement.Clone());

        var result = await kernel.EvaluateBuildAsync(request);

        Assert.False(result.Succeeded);
        Assert.Contains(result.Diagnostics, diagnostic => diagnostic.Code == "KERNEL_INVALID_RESPONSE");
    }

    [Fact]
    public async Task KernelGateway_Preserves_Explicit_Kernel_Diagnostic()
    {
        const string response = """
        {
          "succeeded": false,
          "compiledModel": null,
          "diagnostics": [
            {
              "code": "KERNEL_BUILD_SCHEMA",
              "severity": "error",
              "message": "Unsupported build package schema."
            }
          ]
        }
        """;

        using var httpClient = new HttpClient(new StubHandler(HttpStatusCode.OK, response))
        {
            BaseAddress = new Uri("http://127.0.0.1/")
        };

        var kernel = new UmlcadKernel(httpClient);
        using var semantic = System.Text.Json.JsonDocument.Parse("{\"parts\":[]}");

        var result = await kernel.EvaluateBuildAsync(
            new KernelBuildRequest("test-app", "1.0.0", "identity-1", semantic.RootElement.Clone()));

        Assert.False(result.Succeeded);
        var diagnostic = Assert.Single(result.Diagnostics);
        Assert.Equal("KERNEL_BUILD_SCHEMA", diagnostic.Code);
        Assert.Equal(KernelDiagnosticSeverity.Error, diagnostic.Severity);
    }

    private sealed class StubHandler(HttpStatusCode statusCode, string body) : HttpMessageHandler
    {
        protected override Task<HttpResponseMessage> SendAsync(
            HttpRequestMessage request,
            CancellationToken cancellationToken)
        {
            var response = new HttpResponseMessage(statusCode)
            {
                Content = new StringContent(body, Encoding.UTF8, "application/json"),
                RequestMessage = request
            };
            return Task.FromResult(response);
        }
    }
}
