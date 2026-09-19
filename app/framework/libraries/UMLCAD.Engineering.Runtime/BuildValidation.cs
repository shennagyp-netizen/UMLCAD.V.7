using System.Collections.ObjectModel;
using UMLCAD.Cad.Engine;
using UMLCAD.Cad.Semantics;

namespace UMLCAD.Engineering.Runtime;

public sealed record EngineeringBuildValidationResult(
    bool Accepted,
    IReadOnlyList<EngineeringRuleResult> RuleResults,
    CadDocumentSnapshot FinalState)
{
    public EngineeringBuildValidationResult
    {
        ArgumentNullException.ThrowIfNull(RuleResults);
        ArgumentNullException.ThrowIfNull(FinalState);
        RuleResults = new ReadOnlyCollection<EngineeringRuleResult>(RuleResults.ToArray());
    }
}

public sealed class EngineeringBuildValidator
{
    private readonly EngineeringRuleRuntime _ruleRuntime;

    public EngineeringBuildValidator(EngineeringRuleRuntime? ruleRuntime = null)
    {
        _ruleRuntime = ruleRuntime ?? new EngineeringRuleRuntime();
    }

    public async Task<EngineeringBuildValidationResult> ValidateAsync(
        CadDocumentStore store,
        IEnumerable<IEngineeringRule> rules,
        IPhenomenaSimulationService simulation,
        CancellationToken cancellationToken = default)
    {
        ArgumentNullException.ThrowIfNull(store);
        ArgumentNullException.ThrowIfNull(rules);
        ArgumentNullException.ThrowIfNull(simulation);

        var orderedRules = rules.ToArray();
        var initialState = store.Snapshot();
        var results = new List<EngineeringRuleResult>(orderedRules.Length);

        foreach (var rule in orderedRules)
        {
            cancellationToken.ThrowIfCancellationRequested();

            var currentState = store.Snapshot();
            var context = new EngineeringContext(
                currentState,
                currentState.Features.ToDictionary(feature => feature.Id));

            var control = new InMemoryCadControlService(store);
            var services = new EngineeringServices(control, simulation);
            var result = await _ruleRuntime.ExecuteAsync(
                rule,
                context,
                services,
                cancellationToken);

            results.Add(result);

            if (result.RequiresRollback)
            {
                store.Restore(initialState);
                return new EngineeringBuildValidationResult(
                    false,
                    results,
                    store.Snapshot());
            }
        }

        return new EngineeringBuildValidationResult(
            true,
            results,
            store.Snapshot());
    }
}
