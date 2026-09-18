using UMLCAD.Engineering.Drawing;
using UMLCAD.Integration.Simulation;
using UMLCAD.Science;

namespace UMLCAD.Engineering.Tests;

public sealed class ProviderAndDrawingTests
{
    [Fact]
    public void ExternalSimulationAdapterImplementsScienceProviderContract()
    {
        var adapter = new TestAdapter("External.Solver.A");
        var provider = new SimulationApplicationProvider(adapter);

        var service = new PhenomenaSimulationService(new[] { provider });
        var result = service.Simulate(
            new PhenomenaSimulationRequest(
                "SIM-EXT-01",
                PhenomenonKind.CuttingProcessResponse,
                new Dictionary<string, double> { ["force"] = 100d }));

        Assert.Equal("External.Solver.A", result.ProviderId);
        Assert.Equal(123d, result.Outputs["response"]);
    }

    [Fact]
    public void SimulationAdapterCannotReturnAResultForAnotherPhenomenon()
    {
        var adapter = new TestAdapter("External.Solver.B");
        var provider = new SimulationApplicationProvider(adapter);

        var wrongPhenomenonResult = new PhenomenaSimulationResult(
            "SIM-EXT-02",
            PhenomenonKind.ThermalResponse,
            true,
            new Dictionary<string, double> { ["temperature"] = 50d },
            "External.Solver.B",
            "wrong");

        var throwingAdapter = new MismatchedAdapter(wrongPhenomenonResult);
        var throwingProvider = new SimulationApplicationProvider(throwingAdapter);
        var request = new PhenomenaSimulationRequest(
            "SIM-EXT-03",
            PhenomenonKind.CuttingProcessResponse,
            new Dictionary<string, double> { ["force"] = 10d });

        Assert.Throws<InvalidOperationException>(() => throwingProvider.Simulate(request));
    }

    [Fact]
    public void DrawingViewSpecificationPreservesAssociativityStateAndOccurrenceFilter()
    {
        var source = SemanticId.New();
        var occurrence = SemanticId.New();

        var view = new DrawingViewSpecification(
            SemanticId.New(),
            DrawingViewKind.Section,
            source,
            new ViewAxis(0d, 0d, 1d),
            ViewDisplayMode.HiddenLine,
            DrawingAssociativityState.Associative,
            new HashSet<SemanticId> { occurrence });

        Assert.Equal(DrawingViewKind.Section, view.Kind);
        Assert.Equal(DrawingAssociativityState.Associative, view.Associativity);
        Assert.Contains(occurrence, view.IncludedOccurrences);
    }

    private sealed class TestAdapter(string id) : ISimulationApplicationAdapter
    {
        public string ApplicationId => id;

        public bool Supports(PhenomenonKind phenomenon) =>
            phenomenon == PhenomenonKind.CuttingProcessResponse;

        public PhenomenaSimulationResult Simulate(PhenomenaSimulationRequest request) =>
            new(
                request.RequestId,
                request.Phenomenon,
                true,
                new Dictionary<string, double> { ["response"] = 123d },
                ApplicationId,
                "external-adapter-test");
    }

    private sealed class MismatchedAdapter(
        PhenomenaSimulationResult mismatchedResult) : ISimulationApplicationAdapter
    {
        public string ApplicationId => mismatchedResult.ProviderId;

        public bool Supports(PhenomenonKind phenomenon) =>
            phenomenon == PhenomenonKind.CuttingProcessResponse;

        public PhenomenaSimulationResult Simulate(PhenomenaSimulationRequest request) =>
            mismatchedResult;
    }
}
