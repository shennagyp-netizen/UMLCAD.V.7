using UMLCAD.Cad.Contracts;
using UMLCAD.Cad.Engine;
using UMLCAD.Cad.Semantics;

namespace UMLCAD.Engineering.Runtime;

public sealed record EngineeringRuleIdentity(
    string RuleId,
    string ImplementationVersion)
{
    public EngineeringRuleIdentity
    {
        if (string.IsNullOrWhiteSpace(RuleId))
            throw new ArgumentException("Rule ID cannot be empty.", nameof(RuleId));

        if (string.IsNullOrWhiteSpace(ImplementationVersion))
            throw new ArgumentException(
                "Rule implementation version cannot be empty.",
                nameof(ImplementationVersion));
    }
}

public enum EngineeringRuleOutcomeKind
{
    Pass,
    Warn,
    Reject,
    ApplyChange,
    RequestRecompute,
    Indeterminate,
    Unsupported,
    Failed
}

public sealed record EngineeringDiagnostic(
    string Code,
    string Message,
    IReadOnlyList<CadId> AffectedEntities);

public sealed record EngineeringRuleResult(
    EngineeringRuleOutcomeKind Outcome,
    IReadOnlyList<EngineeringDiagnostic> Diagnostics)
{
    public bool IsBuildAcceptable =>
        Outcome is EngineeringRuleOutcomeKind.Pass
            or EngineeringRuleOutcomeKind.Warn;
}

public interface IEngineeringRule
{
    EngineeringRuleIdentity Identity { get; }

    ValueTask<EngineeringRuleResult> ExecuteAsync(
        EngineeringContext context,
        IEngineeringServices services,
        CancellationToken cancellationToken = default);
}

public interface IEngineeringServices
{
    ICadControlService Cad { get; }
}

public sealed class EngineeringContext
{
    public EngineeringContext(
        CadDocumentSnapshot document,
        IReadOnlyDictionary<CadId, CadFeatureDefinition> featureIndex)
    {
        Document = document ?? throw new ArgumentNullException(nameof(document));
        FeatureIndex = featureIndex ?? throw new ArgumentNullException(nameof(featureIndex));
    }

    public CadDocumentSnapshot Document { get; }

    public IReadOnlyDictionary<CadId, CadFeatureDefinition> FeatureIndex { get; }

    public bool TryGetFeature(CadId id, out CadFeatureDefinition? feature) =>
        FeatureIndex.TryGetValue(id, out feature);
}

public sealed class EngineeringRuleRuntime
{
    public async ValueTask<EngineeringRuleResult> ExecuteAsync(
        IEngineeringRule rule,
        EngineeringContext context,
        IEngineeringServices services,
        CancellationToken cancellationToken = default)
    {
        ArgumentNullException.ThrowIfNull(rule);
        ArgumentNullException.ThrowIfNull(context);
        ArgumentNullException.ThrowIfNull(services);

        try
        {
            return await rule.ExecuteAsync(context, services, cancellationToken);
        }
        catch (OperationCanceledException) when (cancellationToken.IsCancellationRequested)
        {
            throw;
        }
        catch (Exception exception)
        {
            return new EngineeringRuleResult(
                EngineeringRuleOutcomeKind.Failed,
                [
                    new EngineeringDiagnostic(
                        "ENGINEERING_RULE_UNHANDLED_EXCEPTION",
                        $"Rule {rule.Identity.RuleId} failed: {exception.Message}",
                        [])
                ]);
        }
    }
}
