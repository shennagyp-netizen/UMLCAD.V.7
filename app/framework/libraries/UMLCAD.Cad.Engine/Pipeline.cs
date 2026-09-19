using System.Collections.ObjectModel;
using System.Security.Cryptography;
using System.Text;
using UMLCAD.Cad.Contracts;
using UMLCAD.Cad.Semantics;

namespace UMLCAD.Cad.Engine;

public sealed record CadEvaluationPlan(IReadOnlyList<CadId> OperationIds);

public sealed class CadDependencyGraph
{
    private readonly IReadOnlyDictionary<CadId, IReadOnlyList<CadId>> _deps;
    private readonly IReadOnlyDictionary<CadId, IReadOnlyList<CadId>> _dependents;

    public CadDependencyGraph(CadPart part)
    {
        var ops = part.Operations.ToDictionary(x => x.Id);
        var deps = new Dictionary<CadId, IReadOnlyList<CadId>>();

        foreach (var op in ops.Values)
        {
            var inputs = op.InputOperationIds
                .Distinct()
                .OrderBy(x => x.Value, StringComparer.Ordinal)
                .ToArray();

            foreach (var input in inputs)
            {
                if (!ops.ContainsKey(input))
                {
                    throw new InvalidOperationException(
                        $"Operation '{op.Id.Value}' consumes missing operation '{input.Value}'.");
                }
            }

            deps[op.Id] = inputs;
        }

        var reverse = ops.Keys.ToDictionary(x => x, _ => new List<CadId>());

        foreach (var pair in deps)
        {
            foreach (var dependency in pair.Value)
            {
                reverse[dependency].Add(pair.Key);
            }
        }

        _deps = deps;
        _dependents = reverse.ToDictionary(
            x => x.Key,
            x => (IReadOnlyList<CadId>)x.Value
                .OrderBy(y => y.Value, StringComparer.Ordinal)
                .ToArray());
    }

    public IReadOnlyList<CadId> DependenciesOf(CadId id) =>
        _deps.TryGetValue(id, out var dependencies)
            ? dependencies
            : Array.Empty<CadId>();

    public IReadOnlySet<CadId> InvalidationClosure(IEnumerable<CadId> changed)
    {
        var result = new HashSet<CadId>();
        var queue = new Queue<CadId>(
            changed
                .Distinct()
                .OrderBy(x => x.Value, StringComparer.Ordinal));

        while (queue.Count > 0)
        {
            var id = queue.Dequeue();
            if (!result.Add(id))
                continue;

            if (_dependents.TryGetValue(id, out var next))
            {
                foreach (var dependent in next)
                {
                    queue.Enqueue(dependent);
                }
            }
        }

        return result;
    }

    public CadEvaluationPlan Plan()
    {
        var indegree = _deps.ToDictionary(x => x.Key, x => x.Value.Count);
        var outEdges = _deps.Keys.ToDictionary(x => x, _ => new List<CadId>());

        foreach (var pair in _deps)
        {
            foreach (var dependency in pair.Value)
            {
                outEdges[dependency].Add(pair.Key);
            }
        }

        var ready = new SortedSet<CadId>(
            Comparer<CadId>.Create(
                (a, b) => StringComparer.Ordinal.Compare(a.Value, b.Value)));

        foreach (var pair in indegree.Where(x => x.Value == 0))
        {
            ready.Add(pair.Key);
        }

        var order = new List<CadId>();

        while (ready.Count > 0)
        {
            var current = ready.Min;
            ready.Remove(current);
            order.Add(current);

            foreach (var dependent in outEdges[current]
                         .OrderBy(x => x.Value, StringComparer.Ordinal))
            {
                if (--indegree[dependent] == 0)
                {
                    ready.Add(dependent);
                }
            }
        }

        if (order.Count != _deps.Count)
        {
            throw new InvalidOperationException(
                "CAD semantic operation graph contains a cycle.");
        }

        return new CadEvaluationPlan(
            new ReadOnlyCollection<CadId>(order));
    }
}

/// <summary>
/// Immutable identity for one .NET-owned authoritative evaluation history snapshot.
/// This is application state; it is not kernel state and is never supplied by the
/// mathematical kernel.
/// </summary>
public readonly record struct CadHistoryIdentity(string Value);

/// <summary>
/// One immutable commitment in the .NET evaluation history. A successful entry
/// identifies the authoritative result and its evidence. Failed entries retain
/// their status so the history also records why a semantic state is incomplete.
/// </summary>
public sealed record CadHistoryEntry(
    int Sequence,
    CadId OperationId,
    CadEvaluationIdentity EvaluationIdentity,
    CadEvaluationStatus Status,
    CadResultId? ResultId,
    string? EvidenceHash);

/// <summary>
/// Immutable .NET-owned history of the current semantic operation/result state.
/// A new evaluation produces a new history value; no existing history instance is
/// mutated and the mathematical kernel does not persist or advance this state.
/// </summary>
public sealed record CadEvaluationHistory(
    CadHistoryIdentity Identity,
    IReadOnlyList<CadHistoryEntry> Entries)
{
    public static CadEvaluationHistory Empty { get; } =
        Create(
            Array.Empty<CadId>(),
            new Dictionary<CadId, CadEvaluationOutcome>());

    public CadHistoryEntry? Find(CadId operationId) =>
        Entries.FirstOrDefault(x => x.OperationId == operationId);

    public static CadEvaluationHistory Create(
        CadEvaluationPlan plan,
        IReadOnlyDictionary<CadId, CadEvaluationOutcome> outcomes)
    {
        var entries = new List<CadHistoryEntry>(plan.OperationIds.Count);

        foreach (var (operationId, sequence) in plan.OperationIds.Select((id, index) => (id, index)))
        {
            if (!outcomes.TryGetValue(operationId, out var outcome))
            {
                throw new InvalidOperationException(
                    $"Cannot create history because operation '{operationId.Value}' has no outcome.");
            }

            entries.Add(
                new CadHistoryEntry(
                    sequence,
                    operationId,
                    outcome.EvaluationIdentity,
                    outcome.Status,
                    outcome.Result?.Id,
                    outcome.Result?.EvidenceHash));
        }

        var identity = BuildIdentity(entries);
        return new CadEvaluationHistory(
            identity,
            new ReadOnlyCollection<CadHistoryEntry>(entries));
    }

    private static CadHistoryIdentity BuildIdentity(
        IReadOnlyList<CadHistoryEntry> entries)
    {
        var chain = "genesis";

        foreach (var entry in entries)
        {
            var canonical = string.Join(
                "|",
                entry.Sequence.ToString(System.Globalization.CultureInfo.InvariantCulture),
                entry.OperationId.Value,
                entry.EvaluationIdentity.Value,
                entry.Status.ToString(),
                entry.ResultId?.Value ?? string.Empty,
                entry.EvidenceHash ?? string.Empty);

            chain = Convert.ToHexString(
                    SHA256.HashData(
                        Encoding.UTF8.GetBytes(chain + "|" + canonical)))
                .ToLowerInvariant();
        }

        return new CadHistoryIdentity("sha256:" + chain);
    }
}

public sealed record CadEvaluationOptions(
    string ConfigurationIdentity = "default",
    string TolerancePolicyIdentity = "default");

public sealed record CadEvaluationOutcome(
    CadId OperationId,
    string OperationKind,
    CadEvaluationIdentity EvaluationIdentity,
    CadEvaluationStatus Status,
    CadResult? Result,
    IReadOnlyList<CadDiagnostic> Diagnostics);

public sealed record CadEvaluationSnapshot(
    CadPart Part,
    CadEvaluationPlan Plan,
    KernelEvaluationMode Mode,
    CadEvaluationHistory History,
    IReadOnlySet<CadId> InvalidatedOperationIds,
    IReadOnlyDictionary<CadId, CadEvaluationOutcome> Outcomes,
    IReadOnlyDictionary<CadId, CadResultId?> CurrentBodyResults)
{
    public CadHistoryIdentity HistoryIdentity => History.Identity;

    public bool Succeeded =>
        Outcomes.Count == Part.Operations.Count &&
        Outcomes.Values.All(x => x.Status == CadEvaluationStatus.Succeeded);

    public CadResultId? CurrentBody(CadId bodyId) =>
        CurrentBodyResults.TryGetValue(bodyId, out var id)
            ? id
            : null;
}

public interface ICadOperationCache
{
    bool TryGet(
        CadEvaluationIdentity id,
        out CadEvaluationOutcome outcome);

    void Put(
        CadEvaluationIdentity id,
        CadEvaluationOutcome outcome);
}

public sealed class InMemoryCadOperationCache : ICadOperationCache
{
    private readonly Dictionary<string, CadEvaluationOutcome> _cache =
        new(StringComparer.Ordinal);

    public bool TryGet(
        CadEvaluationIdentity id,
        out CadEvaluationOutcome outcome) =>
        _cache.TryGetValue(id.Value, out outcome!);

    public void Put(
        CadEvaluationIdentity id,
        CadEvaluationOutcome outcome) =>
        _cache[id.Value] = outcome;
}

public static class CadEvaluationIdentityBuilder
{
    public static CadEvaluationIdentity Build(
        string partId,
        CadOperation operation,
        IReadOnlyList<CadResult> upstream,
        CadEvaluationOptions options)
    {
        var builder = new StringBuilder(partId)
            .Append('|')
            .Append(operation.CanonicalDefinition())
            .Append("|configuration=")
            .Append(options.ConfigurationIdentity)
            .Append("|tolerance=")
            .Append(options.TolerancePolicyIdentity);

        foreach (var result in upstream.OrderBy(
                     x => x.Id.Value,
                     StringComparer.Ordinal))
        {
            builder
                .Append("|upstream=")
                .Append(result.Id.Value)
                .Append(':')
                .Append(result.EvidenceHash);
        }

        return new CadEvaluationIdentity(
            "sha256:" +
            Convert.ToHexString(
                    SHA256.HashData(
                        Encoding.UTF8.GetBytes(builder.ToString())))
                .ToLowerInvariant());
    }
}

public sealed class CadEvaluationEngine
{
    private readonly IKernelGateway _kernel;
    private readonly ICadOperationCache _cache;
    private readonly Dictionary<CadId, CadEvaluationOutcome> _previous = new();
    private CadEvaluationHistory _previousHistory = CadEvaluationHistory.Empty;

    public CadEvaluationEngine(
        IKernelGateway kernel,
        ICadOperationCache? cache = null)
    {
        _kernel = kernel ?? throw new ArgumentNullException(nameof(kernel));
        _cache = cache ?? new InMemoryCadOperationCache();
    }

    public CadEvaluationHistory PreviousHistory => _previousHistory;

    public Task<CadEvaluationSnapshot> EvaluateAsync(
        CadPart part,
        CadEvaluationOptions? options = null,
        CancellationToken cancellationToken = default) =>
        EvaluateCoreAsync(
            part,
            changed: null,
            options,
            cancellationToken);

    public Task<CadEvaluationSnapshot> RebuildAsync(
        CadPart part,
        IReadOnlySet<CadId> changed,
        CadEvaluationOptions? options = null,
        CancellationToken cancellationToken = default) =>
        EvaluateCoreAsync(
            part,
            changed,
            options,
            cancellationToken);

    private async Task<CadEvaluationSnapshot> EvaluateCoreAsync(
        CadPart part,
        IReadOnlySet<CadId>? changed,
        CadEvaluationOptions? options,
        CancellationToken cancellationToken)
    {
        ArgumentNullException.ThrowIfNull(part);
        options ??= new CadEvaluationOptions();

        var graph = new CadDependencyGraph(part);
        var plan = graph.Plan();
        var incremental = changed is not null;

        var invalidated = incremental
            ? graph.InvalidationClosure(changed!)
            : part.Operations.Select(x => x.Id).ToHashSet();

        var byId = part.Operations.ToDictionary(x => x.Id);
        var outcomes = new Dictionary<CadId, CadEvaluationOutcome>();
        var bodies = part.Bodies.ToDictionary(x => x.Id, _ => (CadResultId?)null);

        foreach (var id in plan.OperationIds)
        {
            cancellationToken.ThrowIfCancellationRequested();

            var operation = byId[id];
            var upstream = graph
                .DependenciesOf(id)
                .Select(x => outcomes[x])
                .ToArray();

            if (upstream.Any(x => x.Status != CadEvaluationStatus.Succeeded))
            {
                outcomes[id] =
                    new CadEvaluationOutcome(
                        id,
                        operation.OperationKind,
                        new CadEvaluationIdentity("blocked:" + id.Value),
                        CadEvaluationStatus.Failed,
                        null,
                        new[]
                        {
                            new CadDiagnostic(
                                "DEPENDENCY_FAILED",
                                "Unsuccessful upstream operation blocks this operation.",
                                CadEvaluationStatus.Failed,
                                id)
                        });
                continue;
            }

            var upstreamResults = upstream
                .Select(x => x.Result!)
                .ToArray();

            var identity = CadEvaluationIdentityBuilder.Build(
                part.Id.Value,
                operation,
                upstreamResults,
                options);

            if (incremental &&
                !invalidated.Contains(id) &&
                _previous.TryGetValue(id, out var previous) &&
                previous.EvaluationIdentity == identity &&
                _cache.TryGet(identity, out var cached))
            {
                outcomes[id] = cached;

                if (cached.Result?.Kind == CadResultKind.Body)
                {
                    bodies[operation.BodyId] = cached.Result.Id;
                }

                continue;
            }

            var inputResults = upstreamResults
                .Select(x => x.Id)
                .ToArray();

            var previousResultId =
                incremental &&
                _previous.TryGetValue(id, out var previousForOperation)
                    ? previousForOperation.Result?.Id
                    : null;

            var request = new KernelOperationRequest(
                CadContractVersions.KernelOperation,
                identity,
                part.Id.Value,
                operation.Id,
                operation.OperationKind,
                incremental
                    ? KernelEvaluationMode.Incremental
                    : KernelEvaluationMode.Full,
                previousResultId,
                inputResults,
                operation.SemanticInputs);

            KernelOperationResponse response;

            try
            {
                response = await _kernel.EvaluateAsync(
                    request,
                    cancellationToken);
            }
            catch (OperationCanceledException)
            {
                throw;
            }
            catch (Exception exception)
            {
                response = KernelOperationResponse.Failure(
                    request,
                    "KERNEL_EXCEPTION",
                    exception.Message);
            }

            if (response.ContractVersion != request.ContractVersion ||
                response.EvaluationIdentity != request.EvaluationIdentity ||
                response.OperationId != request.OperationId)
            {
                outcomes[id] =
                    new CadEvaluationOutcome(
                        id,
                        operation.OperationKind,
                        identity,
                        CadEvaluationStatus.Failed,
                        null,
                        new[]
                        {
                            new CadDiagnostic(
                                "KERNEL_RESPONSE_IDENTITY_MISMATCH",
                                "Kernel response identity does not match the submitted operation.",
                                CadEvaluationStatus.Failed,
                                id)
                        });
                continue;
            }

            if (response.Status != CadEvaluationStatus.Succeeded ||
                response.AuthoritativeResultId is null ||
                string.IsNullOrWhiteSpace(response.EvidenceHash))
            {
                outcomes[id] =
                    new CadEvaluationOutcome(
                        id,
                        operation.OperationKind,
                        identity,
                        response.Status,
                        null,
                        response.Diagnostics);
                continue;
            }

            var kind = operation is SketchOperation
                ? CadResultKind.SketchProfile
                : CadResultKind.Body;

            var result = new CadResult(
                response.AuthoritativeResultId.Value,
                kind,
                id,
                inputResults,
                response.EvidenceHash!,
                response.Topology);

            var outcome = new CadEvaluationOutcome(
                id,
                operation.OperationKind,
                identity,
                response.Status,
                result,
                response.Diagnostics);

            outcomes[id] = outcome;
            _cache.Put(identity, outcome);

            if (kind == CadResultKind.Body)
            {
                bodies[operation.BodyId] = result.Id;
            }
        }

        var history = CadEvaluationHistory.Create(plan, outcomes);

        _previous.Clear();
        foreach (var outcome in outcomes)
        {
            _previous[outcome.Key] = outcome.Value;
        }

        _previousHistory = history;

        return new CadEvaluationSnapshot(
            part,
            plan,
            incremental
                ? KernelEvaluationMode.Incremental
                : KernelEvaluationMode.Full,
            history,
            invalidated,
            new ReadOnlyDictionary<CadId, CadEvaluationOutcome>(outcomes),
            new ReadOnlyDictionary<CadId, CadResultId?>(bodies));
    }
}
