using UMLCAD.Cad.Contracts;
using UMLCAD.Cad.Semantics;

namespace UMLCAD.Cad.Engine;

public enum CadEvaluationPlanStatus
{
    Ready,
    CycleDetected
}

public sealed record CadEvaluationPlan(
    CadEvaluationPlanStatus Status,
    IReadOnlyList<CadId> EvaluationOrder,
    IReadOnlyList<CadId> CyclePath)
{
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

        order.Reverse();

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
