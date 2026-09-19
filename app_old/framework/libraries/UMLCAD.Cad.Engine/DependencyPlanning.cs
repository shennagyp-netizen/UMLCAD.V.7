using System.Collections.ObjectModel;
using UMLCAD.Cad.Contracts;
using UMLCAD.Cad.Semantics;

namespace UMLCAD.Cad.Engine;

public enum CadEvaluationPlanStatus
{
    Ready,
    CycleDetected
}

public sealed record CadEvaluationPlan
{
    public CadEvaluationPlan(
        CadEvaluationPlanStatus status,
        IReadOnlyList<CadId> evaluationOrder,
        IReadOnlyList<CadId> cyclePath)
    {
        ArgumentNullException.ThrowIfNull(evaluationOrder);
        ArgumentNullException.ThrowIfNull(cyclePath);

        Status = status;
        EvaluationOrder = new ReadOnlyCollection<CadId>(evaluationOrder.ToArray());
        CyclePath = new ReadOnlyCollection<CadId>(cyclePath.ToArray());
    }

    public CadEvaluationPlanStatus Status { get; }

    public IReadOnlyList<CadId> EvaluationOrder { get; }

    public IReadOnlyList<CadId> CyclePath { get; }

    public bool IsValid => Status == CadEvaluationPlanStatus.Ready;
}

public sealed class CadDependencyGraph
{
    private readonly Dictionary<CadId, IReadOnlyList<CadId>> _dependencies;

    public CadDependencyGraph(CadDocumentDefinition document)
    {
        ArgumentNullException.ThrowIfNull(document);

        _dependencies = document.Features.ToDictionary(
            feature => feature.Id,
            feature => (IReadOnlyList<CadId>)feature.Dependencies
                .OrderBy(id => id.Value, StringComparer.Ordinal)
                .ToArray());
    }

    public IReadOnlyList<CadId> GetAffectedByChanges(IEnumerable<CadId> changedIds)
    {
        ArgumentNullException.ThrowIfNull(changedIds);

        var changed = changedIds
            .Distinct()
            .OrderBy(id => id.Value, StringComparer.Ordinal)
            .ToArray();

        var reverse = _dependencies.Keys.ToDictionary(id => id, _ => new List<CadId>());
        var unknown = changed.Where(id => !reverse.ContainsKey(id)).ToArray();
        if (unknown.Length > 0)
        {
            throw new ArgumentException(
                "Changed feature IDs are not present in the document: " +
                string.Join(", ", unknown.Select(id => id.Value)),
                nameof(changedIds));
        }

        foreach (var (dependent, dependencies) in _dependencies)
        {
            foreach (var dependency in dependencies)
                reverse[dependency].Add(dependent);
        }

        var affected = new HashSet<CadId>(changed);
        var queue = new Queue<CadId>(changed);

        while (queue.Count > 0)
        {
            var current = queue.Dequeue();

            foreach (var dependent in reverse[current].OrderBy(id => id.Value, StringComparer.Ordinal))
            {
                if (affected.Add(dependent))
                    queue.Enqueue(dependent);
            }
        }

        return new ReadOnlyCollection<CadId>(
            affected
                .OrderBy(id => id.Value, StringComparer.Ordinal)
                .ToArray());
    }

    public CadEvaluationPlan CreatePlan()
    {
        var state = _dependencies.Keys.ToDictionary(id => id, _ => VisitState.Unvisited);
        var stack = new List<CadId>();
        var order = new List<CadId>();

        foreach (var node in _dependencies.Keys.OrderBy(id => id.Value, StringComparer.Ordinal))
        {
            if (state[node] != VisitState.Unvisited)
                continue;

            var cycle = Visit(node, state, stack, order);
            if (cycle.Count > 0)
            {
                return new CadEvaluationPlan(
                    CadEvaluationPlanStatus.CycleDetected,
                    [],
                    cycle);
            }
        }

        return new CadEvaluationPlan(
            CadEvaluationPlanStatus.Ready,
            order,
            []);
    }

    private List<CadId> Visit(
        CadId node,
        Dictionary<CadId, VisitState> state,
        List<CadId> stack,
        List<CadId> order)
    {
        state[node] = VisitState.Visiting;
        stack.Add(node);

        foreach (var dependency in _dependencies[node])
        {
            if (state[dependency] == VisitState.Visiting)
            {
                var start = stack.IndexOf(dependency);
                return [.. stack[start..], dependency];
            }

            if (state[dependency] == VisitState.Unvisited)
            {
                var cycle = Visit(dependency, state, stack, order);
                if (cycle.Count > 0)
                    return cycle;
            }
        }

        stack.RemoveAt(stack.Count - 1);
        state[node] = VisitState.Visited;
        order.Add(node);
        return [];
    }

    private enum VisitState
    {
        Unvisited,
        Visiting,
        Visited
    }
}
