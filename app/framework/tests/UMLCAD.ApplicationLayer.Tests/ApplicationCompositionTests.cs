using System.Net;
using System.Text;
using UMLCAD.Cad.Contracts;
using UMLCAD.Framework;
using UMLCAD.Kernel;

namespace UMLCAD.ApplicationLayer.Tests;

public sealed class ApplicationCompositionTests
{
    [Fact]
    public async Task Framework_Composition_Uses_Only_Concrete_Kernel_Gateway()
    {
        const string response = """
        {
          "succeeded": true,
          "compiledModel": {
            "schema": "uml-cad-compiled-model/1.1.0"
          },
          "diagnostics": []
        }
        """;

        using var client = new HttpClient(
            new StubHandler(HttpStatusCode.OK, response))
        {
            BaseAddress = new Uri("http://127.0.0.1/")
        };

        using var application = UmlcadApplication.ForTest(UmlcadKernel.ForTest(new StubHandler(HttpStatusCode.OK, response)));

        const string semantic = """{"parts":[]}""";
        var definition = new CadBuildDefinition(
            new CadBuildIdentity("app-test", "1.0.0", "build-001"),
            semantic);

        var result = await application.BuildAsync(definition);

        Assert.True(result.Succeeded);
        Assert.NotNull(result.CompiledModel);
        Assert.Empty(result.Diagnostics);
    }

    private sealed class StubHandler(HttpStatusCode statusCode, string body) : HttpMessageHandler
    {
        protected override Task<HttpResponseMessage> SendAsync(
            HttpRequestMessage request,
            CancellationToken cancellationToken)
        {
            var response = new HttpResponseMessage(statusCode)
            {
                RequestMessage = request,
                Content = new StringContent(body, Encoding.UTF8, "application/json")
            };

            return Task.FromResult(response);
        }
    }
}
