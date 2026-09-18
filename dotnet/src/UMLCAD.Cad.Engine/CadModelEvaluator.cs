using UMLCAD.Cad.Contracts;
using UMLCAD.Cad.Semantics;

namespace UMLCAD.Cad.Engine;

public sealed record CadEvaluationSnapshot(
    EvaluationPlan Plan,
    IReadOnlyList<EvaluationOutcome> Outcomes)
{
    public CadEvaluationSnapshot
    {
        ArgumentNullException.ThrowIfNull(Plan);
        Outcomes = Outcomes?.ToArray() ??
            throw new ArgumentNullException(nameof(Outcomes));
    }

    public IReadOnlyList<AuthoritativeCadResult> SuccessfulResults =>
        Outcomes
            .Where(outcome =>
                outcome.Status == EvaluationOutcomeStatus.Succeeded &&
                outcome.Result is not null)
            .Select(outcome => outcome.Result!)
            .ToArray();
}

public sealed class CadModelEvaluator
{
    private readonly IReadOnlyList<IAuthoritativeFeatureEvaluator> _evaluators;

    public CadModelEvaluator(IEnumerable<IAuthoritativeFeatureEvaluator> evaluators)
    {
        ArgumentNullException.ThrowIfNull(evaluators);

        _evaluators = evaluators
            .Where(x => x is not null)
            .OrderBy(x => x.OperationKind, StringComparer.Ordinal)
            .ToArray();

        if (_evaluators.Count == 0)
            throw new ArgumentException(
                "At least one authoritative feature evaluator is required.",
                nameof(evaluators));

        if (_evaluators
            .GroupBy(x => x.OperationKind, StringComparer.Ordinal)
            .Any(group => group.Count() != 1))
        {
            throw new ArgumentException(
                "Feature evaluator operation kinds must be unique.",
                nameof(evaluators));
        }
    }

    public async Task<CadEvaluationSnapshot> EvaluateAsync(
        IEnumerable<FeatureSpecification> specifications,
        FeatureEvaluationOptions options,
        CancellationToken cancellationToken = default)
    {
        ArgumentNullException.ThrowIfNull(specifications);
        ArgumentNullException.ThrowIfNull(options);

        var specificationSnapshot = specifications
            .Where(x => x is not null)
            .OrderBy(x => x.FeatureId.Value)
            .ToArray();

        if (specificationSnapshot.Length == 0)
            throw new ArgumentException(
                "At least one feature specification is required.",
                nameof(specifications));

        var steps = specificationSnapshot
            .Select(specification =>
                EvaluationStepFactory.Create(
                    specification,
                    options.ConfigurationContext,
                    options.TolerancePolicy,
                    options.RepresentationPolicy))
            .ToArray();

        var plan = EvaluationPlanner.Plan(steps);
        var executors = FeatureEvaluationExecutorFactory.Create(
            specificationSnapshot,
            _evaluators,
            options);

        var outcomes = await new AsyncEvaluationEngine(executors).EvaluateAsync(
            plan,
            cancellationToken);

        return new CadEvaluationSnapshot(plan, outcomes);
    }
}
