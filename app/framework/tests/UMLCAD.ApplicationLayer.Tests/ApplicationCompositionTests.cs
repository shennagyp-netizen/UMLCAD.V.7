using System.Net;
using System.Text;
using UMLCAD.Cad.Contracts;
using UMLCAD.Framework;
using UMLCAD.Kernel;
using UMLCAD.Engineering.Runtime;
using UMLCAD.Science;

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

    [Fact]
    public async Task Framework_Engineering_Validation_Executes_Rules_And_Returns_Final_Semantic_State()
    {
        using var application = UmlcadApplication.ForTest(
            UmlcadKernel.ForTest(
                new StubHandler(
                    HttpStatusCode.OK,
                    """
                    {
                      "succeeded": true,
                      "compiledModel": null,
                      "diagnostics": []
                    }
                    """)));

        var document = new CadDocumentDefinition(
            new CadId("document-1"),
            "Engineering Test",
            []);

        var result = await application.ValidateEngineeringAsync(
            document,
            [new AddEngineeringFeatureRule()],
            new EmptySimulationService());

        Assert.True(result.Accepted);
        Assert.Contains(
            result.FinalState.Features,
            feature => feature.Id.Value == "engineering-feature");
    }

    private sealed class AddEngineeringFeatureRule : IEngineeringRule
    {
        public EngineeringRuleIdentity Identity { get; } =
            new("framework.test.add", "1.0.0");

        public ValueTask<EngineeringRuleResult> ExecuteAsync(
            EngineeringContext context,
            IEngineeringServices services,
            CancellationToken cancellationToken = default)
        {
            services.Cad.AddFeature(
                new CadFeatureDefinition(
                    new CadId("engineering-feature"),
                    CadFeatureKind.Feature,
                    "Engineering Feature"));

            return ValueTask.FromResult(
                new EngineeringRuleResult(
                    EngineeringRuleOutcomeKind.ApplyChange,
                    []));
        }
    }

    private sealed class EmptySimulationService : IPhenomenaSimulationService
    {
        public Task<SimulationResult> GetOrRunAsync(
            SimulationRequest request,
            CancellationToken cancellationToken = default) =>
            Task.FromResult(
                new SimulationResult(
                    request,
                    SimulationResultStatus.Completed,
                    [],
                    []));
    }

}
