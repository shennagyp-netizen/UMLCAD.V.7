using UMLCAD.Cad.Contracts;
using UMLCAD.Cad.Engine;
using UMLCAD.Cad.Semantics;
using UMLCAD.Engineering.Runtime;

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
            new EngineeringServices(control));

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
            new EngineeringServices(control));

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
}
