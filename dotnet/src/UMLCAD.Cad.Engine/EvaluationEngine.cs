using UMLCAD.Cad.Semantics;

namespace UMLCAD.Cad.Engine;

public enum EvaluationOutcomeStatus
{
    Succeeded,
    Failed,
    Unsupported,
    Ambiguous,
    Indeterminate,
}

public sealed record EvaluationOutcome(
    SemanticId StepId,
    EvaluationIdentity Identity,
    EvaluationOutcomeStatus Status,
    AuthoritativeResultIdentity? ResultIdentity,
    IReadOnlyList<string> Diagnostics)
{
    public AuthoritativeCadResult? Result { get; init; }
    public EvaluationOutcome
    {
        if (StepId.Value == Guid.Empty)
            throw new ArgumentException("StepId is required.", nameof(StepId));

        Diagnostics = Diagnostics?.ToArray() ??
            throw new ArgumentNullException(nameof(Diagnostics));

        if (Status == EvaluationOutcomeStatus.Succeeded && ResultIdentity is null)
            throw new ArgumentException(
                "A successful evaluation requires an authoritative result identity.",
                nameof(ResultIdentity));
    }
}

public interface IEvaluationStepExecutor
{
    string OperationKind { get; }

    EvaluationOutcome Execute(
        EvaluationStep step,
        EvaluationIdentity identity,
        IReadOnlyDictionary<SemanticId, EvaluationOutcome> completed);
}

public sealed class EvaluationEngine
{
    private readonly IReadOnlyDictionary<string, IEvaluationStepExecutor> _executors;

    public EvaluationEngine(IEnumerable<IEvaluationStepExecutor> executors)
    {
        ArgumentNullException.ThrowIfNull(executors);

        var ordered = executors
            .Where(x => x is not null)
            .OrderBy(x => x.OperationKind, StringComparer.Ordinal)
            .ToArray();

        if (ordered.Length == 0)
            throw new ArgumentException("At least one evaluation executor is required.", nameof(executors));

        if (ordered.GroupBy(x => x.OperationKind, StringComparer.Ordinal).Any(g => g.Count() != 1))
            throw new ArgumentException("Evaluation operation kinds must be unique.", nameof(executors));

        _executors = ordered.ToDictionary(x => x.OperationKind, StringComparer.Ordinal);
    }

    public IReadOnlyList<EvaluationOutcome> Evaluate(EvaluationPlan plan)
    {
        ArgumentNullException.ThrowIfNull(plan);

        var outcomes = new Dictionary<SemanticId, EvaluationOutcome>();
        foreach (var step in plan.Steps)
        {
            if (!_executors.TryGetValue(step.OperationKind, out var executor))
            {
                throw new NotSupportedException(
                    $"No evaluator is registered for operation kind '{step.OperationKind}'.");
            }

            foreach (var dependency in step.Dependencies)
            {
                if (!outcomes.ContainsKey(dependency))
                    throw new InvalidOperationException(
                        $"Dependency '{dependency}' was not evaluated before step '{step.StepId}'.");
            }

            var identity = EvaluationIdentityBuilder.Build(step);
            var outcome = executor.Execute(step, identity, outcomes);

            if (outcome.StepId != step.StepId || outcome.Identity != identity)
            {
                throw new InvalidOperationException(
                    $"Executor '{executor.OperationKind}' returned an outcome for the wrong evaluation step or identity.");
            }

            outcomes.Add(step.StepId, outcome);

            if (outcome.Status != EvaluationOutcomeStatus.Succeeded)
                break;
        }

        return plan.Steps
            .Where(x => outcomes.ContainsKey(x.StepId))
            .Select(x => outcomes[x.StepId])
            .ToArray();
    }
}
