using UMLCAD.Cad.Contracts;
using UMLCAD.Cad.Engine;
using UMLCAD.Science;

namespace UMLCAD.Engineering.Runtime;

public readonly record struct EngineeringRuleId(string Value)
{
    public EngineeringRuleId
    {
        if (string.IsNullOrWhiteSpace(Value))
            throw new ArgumentException("Rule ID is required.", nameof(Value));
    }
}

public enum RuleOutcome
{
    Pass,
    Warn,
    Reject,
    Indeterminate,
    Failed
}

public sealed record EngineeringContext(
    CadEvaluationSnapshot Cad,
    IPhenomenaSimulationService Simulation);

public sealed record EngineeringRuleResult(
    EngineeringRuleId RuleId,
    RuleOutcome Outcome,
    IReadOnlyList<CadDiagnostic> Diagnostics)
{
    public bool BuildAcceptable =>
        Outcome is RuleOutcome.Pass or RuleOutcome.Warn;
}

public interface IEngineeringRule
{
    EngineeringRuleId Id { get; }

    ValueTask<EngineeringRuleResult> EvaluateAsync(
        EngineeringContext context,
        CancellationToken cancellationToken = default);
}

public sealed class EngineeringRuleRuntime
{
    public async ValueTask<IReadOnlyList<EngineeringRuleResult>> EvaluateAsync(
        EngineeringContext context,
        IEnumerable<IEngineeringRule> rules,
        CancellationToken cancellationToken = default)
    {
        ArgumentNullException.ThrowIfNull(context);
        ArgumentNullException.ThrowIfNull(rules);

        var ordered = rules
            .OrderBy(x => x.Id.Value, StringComparer.Ordinal)
            .ToArray();

        var results = new List<EngineeringRuleResult>(ordered.Length);

        foreach (var rule in ordered)
        {
            cancellationToken.ThrowIfCancellationRequested();

            var result = await rule.EvaluateAsync(
                context,
                cancellationToken);

            results.Add(result);

            if (result.Outcome is RuleOutcome.Reject or RuleOutcome.Failed)
                break;
        }

        return results;
    }
}
