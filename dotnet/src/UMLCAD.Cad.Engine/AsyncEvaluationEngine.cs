using UMLCAD.Cad.Semantics;

namespace UMLCAD.Cad.Engine;

public interface IAsyncEvaluationStepExecutor
{
    string OperationKind { get; }

    Task<EvaluationOutcome> ExecuteAsync(
        EvaluationStep step,
        EvaluationIdentity identity,
        IReadOnlyDictionary<SemanticId, EvaluationOutcome> completed,
        CancellationToken cancellationToken);
}

public sealed class AsyncEvaluationEngine
{
    private readonly IReadOnlyDictionary<string, IAsyncEvaluationStepExecutor> _executors;

    public AsyncEvaluationEngine(IEnumerable<IAsyncEvaluationStepExecutor> executors)
    {
        ArgumentNullException.ThrowIfNull(executors);

        var ordered = executors
            .Where(x => x is not null)
            .OrderBy(x => x.OperationKind, StringComparer.Ordinal)
            .ToArray();

        if (ordered.Length == 0)
            throw new ArgumentException(
                "At least one asynchronous evaluation executor is required.",
                nameof(executors));

        if (ordered.GroupBy(x => x.OperationKind, StringComparer.Ordinal).Any(g => g.Count() != 1))
            throw new ArgumentException(
                "Evaluation operation kinds must be unique.",
                nameof(executors));

        _executors = ordered.ToDictionary(x => x.OperationKind, StringComparer.Ordinal);
    }

    public async Task<IReadOnlyList<EvaluationOutcome>> EvaluateAsync(
        EvaluationPlan plan,
        CancellationToken cancellationToken = default)
    {
        ArgumentNullException.ThrowIfNull(plan);

        var outcomes = new Dictionary<SemanticId, EvaluationOutcome>();

        foreach (var step in plan.Steps)
        {
            cancellationToken.ThrowIfCancellationRequested();

            if (!_executors.TryGetValue(step.OperationKind, out var executor))
            {
                throw new NotSupportedException(
                    $"No asynchronous evaluator is registered for operation kind '{step.OperationKind}'.");
            }

            foreach (var dependency in step.Dependencies)
            {
                if (!outcomes.TryGetValue(dependency, out var dependencyOutcome))
                {
                    throw new InvalidOperationException(
                        $"Dependency '{dependency}' was not evaluated before step '{step.StepId}'.");
                }

                if (dependencyOutcome.Status != EvaluationOutcomeStatus.Succeeded)
                {
                    var blockedIdentity = EvaluationIdentityBuilder.Build(step);
                    outcomes[step.StepId] = new EvaluationOutcome(
                        step.StepId,
                        blockedIdentity,
                        EvaluationOutcomeStatus.Failed,
                        null,
                        [$"Evaluation blocked by unsuccessful dependency '{dependency}'."]);
                    return outcomes.Values.ToArray();
                }
            }

            var identity = EvaluationIdentityBuilder.Build(step);
            var outcome = await executor.ExecuteAsync(
                step,
                identity,
                outcomes,
                cancellationToken);

            if (outcome.StepId != step.StepId || outcome.Identity != identity)
            {
                throw new InvalidOperationException(
                    $"Executor '{executor.OperationKind}' returned an outcome for the wrong evaluation step or identity.");
            }

            if (outcome.Status == EvaluationOutcomeStatus.Succeeded)
            {
                if (outcome.Result is null)
                {
                    throw new InvalidOperationException(
                        $"Successful evaluation step '{step.StepId}' did not return an authoritative result.");
                }

                if (!string.Equals(
                        outcome.ResultIdentity?.Value,
                        outcome.Result.Identity.Value,
                        StringComparison.Ordinal))
                {
                    throw new InvalidOperationException(
                        $"Evaluation step '{step.StepId}' returned a mismatched authoritative result identity.");
                }
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
