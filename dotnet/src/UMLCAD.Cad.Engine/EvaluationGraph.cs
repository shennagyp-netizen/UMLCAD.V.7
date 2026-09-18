using UMLCAD.Cad.Semantics;

namespace UMLCAD.Cad.Engine;

public sealed record EvaluationStep(
    SemanticId StepId,
    string OperationKind,
    string NormalizedDefinition,
    IReadOnlyList<SemanticId> Dependencies,
    string KernelContractVersion)
{
    public EvaluationStep
    {
        if (StepId.Value == Guid.Empty)
            throw new ArgumentException("StepId is required.", nameof(StepId));
        if (string.IsNullOrWhiteSpace(OperationKind))
            throw new ArgumentException("OperationKind is required.", nameof(OperationKind));
        if (string.IsNullOrWhiteSpace(NormalizedDefinition))
            throw new ArgumentException("NormalizedDefinition is required.", nameof(NormalizedDefinition));
        if (string.IsNullOrWhiteSpace(KernelContractVersion))
            throw new ArgumentException("KernelContractVersion is required.", nameof(KernelContractVersion));

        Dependencies = Dependencies?.Distinct().ToArray() ??
            throw new ArgumentNullException(nameof(Dependencies));

        if (Dependencies.Contains(StepId))
            throw new ArgumentException("An evaluation step cannot depend on itself.", nameof(Dependencies));
    }
}

public sealed record EvaluationPlan(IReadOnlyList<EvaluationStep> Steps)
{
    public EvaluationPlan
    {
        Steps = Steps?.ToArray() ??
            throw new ArgumentNullException(nameof(Steps));

        if (Steps.Select(x => x.StepId).Distinct().Count() != Steps.Count)
            throw new ArgumentException("Evaluation step identities must be unique.", nameof(Steps));
    }

    public IReadOnlyList<SemanticId> OrderedStepIds =>
        Steps.Select(x => x.StepId).ToArray();
}

public static class EvaluationPlanner
{
    public static EvaluationPlan Plan(IEnumerable<EvaluationStep> steps)
    {
        ArgumentNullException.ThrowIfNull(steps);

        var input = steps.ToArray();
        var byId = input.ToDictionary(x => x.StepId);

        foreach (var step in input)
        {
            foreach (var dependency in step.Dependencies)
            {
                if (!byId.ContainsKey(dependency))
                {
                    throw new InvalidOperationException(
                        $"Evaluation step '{step.StepId}' references missing dependency '{dependency}'.");
                }
            }
        }

        var indegree = input.ToDictionary(x => x.StepId, _ => 0);
        var outgoing = input.ToDictionary(x => x.StepId, _ => new List<SemanticId>());

        foreach (var step in input)
        {
            foreach (var dependency in step.Dependencies)
            {
                indegree[step.StepId]++;
                outgoing[dependency].Add(step.StepId);
            }
        }

        var ready = new SortedSet<SemanticId>(Comparer<SemanticId>.Create(
            static (left, right) => left.Value.CompareTo(right.Value)));

        foreach (var pair in indegree)
        {
            if (pair.Value == 0)
                ready.Add(pair.Key);
        }

        var ordered = new List<SemanticId>(input.Length);
        while (ready.Count > 0)
        {
            var current = ready.Min;
            ready.Remove(current);
            ordered.Add(current);

            foreach (var dependent in outgoing[current].OrderBy(x => x.Value))
            {
                indegree[dependent]--;
                if (indegree[dependent] == 0)
                    ready.Add(dependent);
            }
        }

        if (ordered.Count != input.Length)
            throw new InvalidOperationException("The evaluation dependency graph contains a cycle.");

        return new EvaluationPlan(ordered.Select(id => byId[id]).ToArray());
    }
}

public readonly record struct EvaluationIdentity(string Value)
{
    public EvaluationIdentity
    {
        if (string.IsNullOrWhiteSpace(Value))
            throw new ArgumentException("Evaluation identity is required.", nameof(Value));
    }

    public override string ToString() => Value;
}
