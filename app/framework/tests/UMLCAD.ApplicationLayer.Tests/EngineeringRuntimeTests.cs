using UMLCAD.Cad.Contracts;
using UMLCAD.Cad.Engine;
using UMLCAD.Cad.Semantics;
using System.Threading;
using UMLCAD.Engineering.Runtime;
using UMLCAD.Science;

namespace UMLCAD.ApplicationLayer.Tests;

public sealed class EngineeringRuntimeTests
{
    [Fact]
    public async Task Passing_Rule_Commits_Cad_Changes()
    {
        var store = NewStore();
        var control = new InMemoryCadControlService(store);
        var context = Context(store);
        var rule = new ChangeRule(
            new EngineeringRuleIdentity("rule.change", "1.0.0"),
            EngineeringRuleOutcomeKind.ApplyChange);

        var result = await new EngineeringRuleRuntime().ExecuteAsync(
            rule,
            context,
            new EngineeringServices(control, new EmptySimulationService()));

        Assert.Equal(EngineeringRuleOutcomeKind.ApplyChange, result.Outcome);
        Assert.Contains(store.Snapshot().Features, feature => feature.Id.Value == "created");
    }

    [Fact]
    public async Task Rejecting_Rule_Rolls_Back_Cad_Changes()
    {
        var store = NewStore();
        var control = new InMemoryCadControlService(store);
        var context = Context(store);
        var rule = new ChangeRule(
            new EngineeringRuleIdentity("rule.reject", "1.0.0"),
            EngineeringRuleOutcomeKind.Reject);

        var result = await new EngineeringRuleRuntime().ExecuteAsync(
            rule,
            context,
            new EngineeringServices(control, new EmptySimulationService()));

        Assert.Equal(EngineeringRuleOutcomeKind.Reject, result.Outcome);
        Assert.DoesNotContain(store.Snapshot().Features, feature => feature.Id.Value == "created");
    }

    [Fact]
    public async Task Unhandled_Rule_Exception_Rolls_Back_And_Becomes_Explicit_Failure()
    {
        var store = NewStore();
        var control = new InMemoryCadControlService(store);
        var context = Context(store);

        var result = await new EngineeringRuleRuntime().ExecuteAsync(
            new ThrowingRule(),
            context,
            new EngineeringServices(control));

        Assert.Equal(EngineeringRuleOutcomeKind.Failed, result.Outcome);
        Assert.Contains(
            result.Diagnostics,
            diagnostic => diagnostic.Code == "ENGINEERING_RULE_UNHANDLED_EXCEPTION");
        Assert.DoesNotContain(store.Snapshot().Features, feature => feature.Id.Value == "created");
    }


    [Fact]
    public void Rule_Cad_Interface_Does_Not_Expose_Transaction_Control()
    {
        var methods = typeof(ICadControlService)
            .GetMethods()
            .Select(method => method.Name)
            .OrderBy(name => name, StringComparer.Ordinal)
            .ToArray();

        Assert.Equal(
            ["AddFeature", "RemoveFeature", "RenameFeature"],
            methods);
    }


    [Fact]
    public async Task Rule_Can_Query_Context_Call_Simulation_And_Change_Cad()
    {
        var store = NewStore();
        var control = new InMemoryCadControlService(store);
        var simulation = new RecordingSimulationService();
        var context = Context(store);

        var result = await new EngineeringRuleRuntime().ExecuteAsync(
            new SimulationDrivenRule(),
            context,
            new EngineeringServices(control, simulation));

        Assert.Equal(EngineeringRuleOutcomeKind.ApplyChange, result.Outcome);
        Assert.Equal(1, simulation.ExecutionCount);
        Assert.Equal("created-after-simulation", Assert.Single(store.Snapshot().Features).Id.Value);
    }

    [Fact]
    public void EngineeringContext_Copies_FeatureIndex()
    {
        var store = NewStore();
        var snapshot = store.Snapshot();
        var source = snapshot.Features.ToDictionary(feature => feature.Id);

        var context = new EngineeringContext(snapshot, source);
        source.Clear();

        Assert.True(context.TryGetFeature(new CadId("created-after-simulation"), out _ ) is false);
        Assert.Empty(context.FeatureIndex);
    }

    private static CadDocumentStore NewStore() =>
        new(
            new CadDocumentDefinition(
                new CadId("document-1"),
                "Test",
                []));

    private static EngineeringContext Context(CadDocumentStore store)
    {
        var snapshot = store.Snapshot();
        var index = snapshot.Features.ToDictionary(feature => feature.Id);
        return new EngineeringContext(snapshot, index);
    }

    private sealed class ChangeRule(
        EngineeringRuleIdentity identity,
        EngineeringRuleOutcomeKind outcome) : IEngineeringRule
    {
        public EngineeringRuleIdentity Identity { get; } = identity;

        public ValueTask<EngineeringRuleResult> ExecuteAsync(
            EngineeringContext context,
            IEngineeringServices services,
            CancellationToken cancellationToken = default)
        {
            var add = services.Cad.AddFeature(
                new CadFeatureDefinition(
                    new CadId("created"),
                    CadFeatureKind.Feature,
                    "Created by Rule"));

            if (add.Status is not CadCommandStatus.Accepted)
                throw new InvalidOperationException(add.Message);

            return ValueTask.FromResult(
                new EngineeringRuleResult(
                    outcome,
                    outcome == EngineeringRuleOutcomeKind.Reject
                        ? [new EngineeringDiagnostic("RULE_REJECTED", "Rejected by test rule.", [])]
                        : []));
        }
    }

    private sealed class ThrowingRule : IEngineeringRule
    {
        public EngineeringRuleIdentity Identity { get; } =
            new("rule.throw", "1.0.0");

        public ValueTask<EngineeringRuleResult> ExecuteAsync(
            EngineeringContext context,
            IEngineeringServices services,
            CancellationToken cancellationToken = default)
        {
            services.Cad.AddFeature(
                new CadFeatureDefinition(
                    new CadId("created"),
                    CadFeatureKind.Feature,
                    "Should Roll Back"));

            throw new InvalidOperationException("intentional");
        }
    }

    private sealed class EmptySimulationService : IPhenomenaSimulationService
    {
        public Task<SimulationResult> GetOrRunAsync(
            SimulationRequest request,
            CancellationToken cancellationToken = default) =>
            Task.FromResult(new SimulationResult(
                request,
                SimulationResultStatus.Completed,
                [],
                []));
    }


    private sealed class SimulationDrivenRule : IEngineeringRule
    {
        public EngineeringRuleIdentity Identity { get; } =
            new("rule.simulation-driven", "1.0.0");

        public async ValueTask<EngineeringRuleResult> ExecuteAsync(
            EngineeringContext context,
            IEngineeringServices services,
            CancellationToken cancellationToken = default)
        {
            var result = await services.Simulation.GetOrRunAsync(
                new SimulationRequest(
                    PhenomenonKind.Thermal,
                    "thermal-model-1",
                    "geometry-1",
                    "material-1",
                    "process-1",
                    "bc-1",
                    "environment-1",
                    "config-1",
                    "numerical-1",
                    "uncertainty-1"),
                cancellationToken);

            if (!result.IsReusable(
                    new SimulationRequest(
                        PhenomenonKind.Thermal,
                        "thermal-model-1",
                        "geometry-1",
                        "material-1",
                        "process-1",
                        "bc-1",
                        "environment-1",
                        "config-1",
                        "numerical-1",
                        "uncertainty-1")))
            {
                return new EngineeringRuleResult(
                    EngineeringRuleOutcomeKind.Indeterminate,
                    [new EngineeringDiagnostic(
                        "SIMULATION_NOT_REUSABLE",
                        "Simulation result was not valid for the requested identity.",
                        [])]);
            }

            services.Cad.AddFeature(
                new CadFeatureDefinition(
                    new CadId("created-after-simulation"),
                    CadFeatureKind.Feature,
                    "Created after simulation"));

            return new EngineeringRuleResult(
                EngineeringRuleOutcomeKind.ApplyChange,
                []);
        }
    }

    private sealed class RecordingSimulationService : IPhenomenaSimulationService
    {
        public int ExecutionCount { get; private set; }

        public Task<SimulationResult> GetOrRunAsync(
            SimulationRequest request,
            CancellationToken cancellationToken = default)
        {
            ExecutionCount++;
            return Task.FromResult(
                new SimulationResult(
                    request,
                    SimulationResultStatus.Completed,
                    [],
                    []));
        }
    }

}
