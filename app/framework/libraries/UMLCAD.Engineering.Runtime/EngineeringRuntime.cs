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
    IReadOnlyList<CadId> AffectedEntities)
{
    public EngineeringDiagnostic
    {
        if (string.IsNullOrWhiteSpace(Code))
            throw new ArgumentException("Diagnostic code cannot be empty.", nameof(Code));

        if (string.IsNullOrWhiteSpace(Message))
            throw new ArgumentException("Diagnostic message cannot be empty.", nameof(Message));

        ArgumentNullException.ThrowIfNull(AffectedEntities);
    }
}

public sealed record EngineeringRuleResult(
    EngineeringRuleOutcomeKind Outcome,
    IReadOnlyList<EngineeringDiagnostic> Diagnostics)
{
    public EngineeringRuleResult
    {
        ArgumentNullException.ThrowIfNull(Diagnostics);
    }

    public bool IsBuildAcceptable =>
        Outcome is EngineeringRuleOutcomeKind.Pass
            or EngineeringRuleOutcomeKind.Warn
            or EngineeringRuleOutcomeKind.ApplyChange
            or EngineeringRuleOutcomeKind.RequestRecompute;

    public bool RequiresRollback =>
        Outcome is EngineeringRuleOutcomeKind.Reject
            or EngineeringRuleOutcomeKind.Indeterminate
            or EngineeringRuleOutcomeKind.Unsupported
            or EngineeringRuleOutcomeKind.Failed;
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

public sealed class EngineeringServices(ICadControlService cad) : IEngineeringServices
{
    public ICadControlService Cad { get; } =
        cad ?? throw new ArgumentNullException(nameof(cad));
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

        EngineeringRuleResult result;
        try
        {
            result = await rule.ExecuteAsync(context, services, cancellationToken);
        }
        catch (OperationCanceledException) when (cancellationToken.IsCancellationRequested)
        {
            services.Cad.Rollback();
            throw;
        }
        catch (Exception exception)
        {
            services.Cad.Rollback();

            return new EngineeringRuleResult(
                EngineeringRuleOutcomeKind.Failed,
                [
                    new EngineeringDiagnostic(
                        "ENGINEERING_RULE_UNHANDLED_EXCEPTION",
                        $"Rule {rule.Identity.RuleId} failed: {exception.Message}",
                        [])
                ]);
        }

        if (result.RequiresRollback)
        {
            services.Cad.Rollback();
            return result;
        }

        var commit = services.Cad.Commit();
        if (commit.Status is CadCommandStatus.Accepted)
            return result;

        services.Cad.Rollback();

        return new EngineeringRuleResult(
            EngineeringRuleOutcomeKind.Failed,
            [
                new EngineeringDiagnostic(
                    commit.Code ?? "CAD_TRANSACTION_COMMIT_FAILED",
                    commit.Message ?? "CAD transaction commit failed.",
                    commit.TargetId is CadId target ? [target] : [])
            ]);
    }
}
