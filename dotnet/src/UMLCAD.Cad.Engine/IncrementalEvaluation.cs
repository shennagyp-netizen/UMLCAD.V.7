using UMLCAD.Cad.Semantics;

namespace UMLCAD.Cad.Engine;

public sealed record IncrementalEvaluationPlan(
    EvaluationPlan Plan,
    IReadOnlySet<SemanticId> RecomputeStepIds)
{
    public IncrementalEvaluationPlan
    {
        ArgumentNullException.ThrowIfNull(Plan);
        ArgumentNullException.ThrowIfNull(RecomputeStepIds);
        RecomputeStepIds = new HashSet<SemanticId>(RecomputeStepIds);
    }
}

public static class IncrementalEvaluationPlanner
{
    public static IncrementalEvaluationPlan Create(
        EvaluationPlan plan,
        CadChangeSet changeSet)
    {
        ArgumentNullException.ThrowIfNull(plan);
        ArgumentNullException.ThrowIfNull(changeSet);

        var recompute = changeSet.Mode switch
        {
            RecomputeMode.Full => plan.Steps
                .Select(step => step.StepId)
                .ToHashSet(),
            RecomputeMode.Incremental => plan.AffectedBy(changeSet.ChangedFeatureIds),
            _ => throw new InvalidOperationException(
                $"Unknown recompute mode '{changeSet.Mode}'."),
        };

        return new IncrementalEvaluationPlan(plan, recompute);
    }
}
