using System.Collections.ObjectModel;
using System.Security.Cryptography;
using System.Text;
using UMLCAD.Cad.Contracts;
using UMLCAD.Cad.Semantics;

namespace UMLCAD.Cad.Engine;

public sealed record CadEvaluationPlan(IReadOnlyList<CadId> OperationIds);

public sealed class CadDependencyGraph
{
    private readonly IReadOnlyDictionary<CadId, IReadOnlyList<CadId>> _dependencies;
    private readonly IReadOnlyDictionary<CadId, IReadOnlyList<CadId>> _dependents;

    public CadDependencyGraph(CadPartDefinition part)
    {
        ArgumentNullException.ThrowIfNull(part);

        var operations = part.Operations.ToDictionary(x => x.Id);
        var dependencies = new Dictionary<CadId, IReadOnlyList<CadId>>();

        foreach (var operation in operations.Values)
        {
            var dependenciesOfOperation = operation.InputOperationIds
                .Distinct()
                .OrderBy(x => x.Value, StringComparer.Ordinal)
                .ToArray();

            foreach (var dependency in dependenciesOfOperation)
            {
                if (!operations.ContainsKey(dependency))
                    throw new InvalidOperationException(
                        $"Operation '{operation.Id}' consumes missing operation '{dependency}'.");
                if (dependency == operation.Id)
                    throw new InvalidOperationException(
                        $"Operation '{operation.Id}' cannot consume itself.");
            }

            dependencies[operation.Id] = dependenciesOfOperation;
        }

        var dependents = operations.Keys.ToDictionary(
            x => x,
            _ => new List<CadId>());

        foreach (var pair in dependencies)
            foreach (var dependency in pair.Value)
                dependents[dependency].Add(pair.Key);

        _dependencies = dependencies;
        _dependents = dependents.ToDictionary(
            x => x.Key,
            x => (IReadOnlyList<CadId>)x.Value
                .OrderBy(y => y.Value, StringComparer.Ordinal)
                .ToArray());
    }

    public IReadOnlyList<CadId> DependenciesOf(CadId operationId) =>
        _dependencies.TryGetValue(operationId, out var value)
            ? value
            : Array.Empty<CadId>();

    public IReadOnlySet<CadId> InvalidationClosure(
        IEnumerable<CadId> changedOperationIds)
    {
        ArgumentNullException.ThrowIfNull(changedOperationIds);

        var affected = new HashSet<CadId>();
        var queue = new Queue<CadId>(
            changedOperationIds
                .Distinct()
                .OrderBy(x => x.Value, StringComparer.Ordinal));

        while (queue.Count > 0)
        {
            var current = queue.Dequeue();
            if (!affected.Add(current))
                continue;

            if (_dependents.TryGetValue(current, out var dependents))
                foreach (var dependent in dependents)
                    queue.Enqueue(dependent);
        }

        return affected;
    }

    public CadEvaluationPlan Plan()
    {
        var indegree = _dependencies.ToDictionary(
            x => x.Key,
            x => x.Value.Count);

        var outgoing = _dependencies.Keys.ToDictionary(
            x => x,
            _ => new List<CadId>());

        foreach (var pair in _dependencies)
            foreach (var dependency in pair.Value)
                outgoing[dependency].Add(pair.Key);

        var ready = new SortedSet<CadId>(
            Comparer<CadId>.Create(
                static (left, right) =>
                    StringComparer.Ordinal.Compare(left.Value, right.Value)));

        foreach (var pair in indegree.Where(x => x.Value == 0))
            ready.Add(pair.Key);

        var order = new List<CadId>(_dependencies.Count);

        while (ready.Count > 0)
        {
            var current = ready.Min;
            ready.Remove(current);
            order.Add(current);

            foreach (var dependent in outgoing[current]
                         .OrderBy(x => x.Value, StringComparer.Ordinal))
            {
                indegree[dependent]--;

                if (indegree[dependent] == 0)
                    ready.Add(dependent);
            }
        }

        if (order.Count != _dependencies.Count)
            throw new InvalidOperationException(
                "CAD semantic operation graph contains a cycle.");

        return new CadEvaluationPlan(
            new ReadOnlyCollection<CadId>(order));
    }
}

public sealed record CadEvaluationOptions(
    string ConfigurationIdentity = "default",
    string TolerancePolicyIdentity = "default",
    string RepresentationPolicyIdentity = "default");

public sealed record CadReferenceResolution(
    CadReference Reference,
    CadEvaluationStatus Status,
    CadResultId? ResolvedResult,
    string? Diagnostic);

public sealed record CadEvaluationOutcome(
    CadId OperationId,
    string OperationKind,
    CadEvaluationIdentity EvaluationIdentity,
    CadEvaluationStatus Status,
    CadResultEnvelope? Result,
    IReadOnlyList<CadReferenceResolution> References,
    IReadOnlyList<CadDiagnostic> Diagnostics)
{
    public bool Succeeded => Status == CadEvaluationStatus.Succeeded;
}

public sealed record CadEvaluationSnapshot(
    CadPartDefinition Part,
    CadEvaluationPlan Plan,
    KernelEvaluationMode Mode,
    IReadOnlySet<CadId> EvaluatedOperationIds,
    IReadOnlyDictionary<CadId, CadEvaluationOutcome> Outcomes,
    IReadOnlyDictionary<CadId, CadResultId?> CurrentBodyResults,
    IReadOnlyList<CadDiagnostic> BodyDiagnostics)
{
    public bool Succeeded =>
        Outcomes.Count == Part.Operations.Count &&
        Outcomes.Values.All(x => x.Succeeded) &&
        BodyDiagnostics.Count == 0;

    public CadResultId? CurrentBody(CadId bodyId) =>
        CurrentBodyResults.TryGetValue(bodyId, out var result)
            ? result
            : null;
}

public interface ICadOperationCache
{
    bool TryGet(
        CadEvaluationIdentity identity,
        out CadEvaluationOutcome outcome);

    void Put(
        CadEvaluationIdentity identity,
        CadEvaluationOutcome outcome);
}

public sealed class InMemoryCadOperationCache : ICadOperationCache
{
    private readonly Dictionary<string, CadEvaluationOutcome> _values =
        new(StringComparer.Ordinal);

    public bool TryGet(
        CadEvaluationIdentity identity,
        out CadEvaluationOutcome outcome) =>
        _values.TryGetValue(identity.Value, out outcome!);

    public void Put(
        CadEvaluationIdentity identity,
        CadEvaluationOutcome outcome) =>
        _values[identity.Value] = outcome;
}

public static class CadEvaluationIdentityBuilder
{
    public static CadEvaluationIdentity Build(
        CadPartDefinition part,
        CadOperation operation,
        IReadOnlyList<CadResultEnvelope> upstream,
        IReadOnlyList<CadReferenceResolution> references,
        CadEvaluationOptions options)
    {
        var builder = new StringBuilder()
            .Append("contract=").Append(CadContractVersions.Semantic)
            .Append("|part=").Append(part.Id.Value)
            .Append("|operation=").Append(operation.CanonicalDefinition())
            .Append("|configuration=").Append(options.ConfigurationIdentity)
            .Append("|tolerance=").Append(options.TolerancePolicyIdentity)
            .Append("|representation=").Append(options.RepresentationPolicyIdentity);

        foreach (var reference in references
                     .OrderBy(x => x.Reference.Id.Value, StringComparer.Ordinal))
        {
            builder.Append("|reference=")
                .Append(reference.Reference.Id.Value)
                .Append(':')
                .Append(reference.Status)
                .Append(':')
                .Append(reference.ResolvedResult?.Value ?? "-")
                .Append(':')
                .Append(reference.Diagnostic ?? "-");
        }

        foreach (var parameter in part.Parameters
                     .Where(x => operation.ParameterNames.Contains(x.Name))
                     .OrderBy(x => x.Name, StringComparer.Ordinal))
            builder.Append("|parameter=").Append(parameter.Identity);

        foreach (var result in upstream
                     .OrderBy(x => x.Id.Value, StringComparer.Ordinal))
            builder.Append("|upstream=").Append(result.Id.Value)
                .Append(':').Append(result.EvidenceHash);

        return new CadEvaluationIdentity(
            "sha256:" + Convert.ToHexString(
                SHA256.HashData(
                    Encoding.UTF8.GetBytes(builder.ToString())))
                .ToLowerInvariant());
    }
}

public sealed class CadEvaluationEngine
{
    private readonly IKernelGateway _kernel;
    private readonly ICadOperationCache _cache;
    private readonly Dictionary<CadId, CadEvaluationOutcome> _previous =
        new();

    public CadEvaluationEngine(
        IKernelGateway kernel,
        ICadOperationCache? cache = null)
    {
        _kernel = kernel ?? throw new ArgumentNullException(nameof(kernel));
        _cache = cache ?? new InMemoryCadOperationCache();
    }

    public Task<CadEvaluationSnapshot> EvaluateAsync(
        CadPartDefinition part,
        CadEvaluationOptions? options = null,
        CancellationToken cancellationToken = default) =>
        EvaluateCoreAsync(
            part,
            null,
            options,
            cancellationToken);

    public Task<CadEvaluationSnapshot> EvaluateAsync(
        CadPartDefinition part,
        IReadOnlySet<CadId> changedOperationIds,
        CadEvaluationOptions? options = null,
        CancellationToken cancellationToken = default)
    {
        ArgumentNullException.ThrowIfNull(changedOperationIds);

        return EvaluateCoreAsync(
            part,
            changedOperationIds,
            options,
            cancellationToken);
    }

    private async Task<CadEvaluationSnapshot> EvaluateCoreAsync(
        CadPartDefinition part,
        IReadOnlySet<CadId>? changedOperationIds,
        CadEvaluationOptions? options,
        CancellationToken cancellationToken)
    {
        ArgumentNullException.ThrowIfNull(part);
        options ??= new CadEvaluationOptions();

        var graph = new CadDependencyGraph(part);
        var plan = graph.Plan();
        var operationById = part.Operations.ToDictionary(x => x.Id);

        var incremental = changedOperationIds is not null;
        var invalidated = incremental
            ? graph.InvalidationClosure(changedOperationIds!)
            : part.Operations.Select(x => x.Id).ToHashSet();

        var outcomes = new Dictionary<CadId, CadEvaluationOutcome>();

        foreach (var operationId in plan.OperationIds)
        {
            cancellationToken.ThrowIfCancellationRequested();

            var operation = operationById[operationId];
            var upstream = graph.DependenciesOf(operationId)
                .Select(id => outcomes[id].Result)
                .Where(x => x is not null)
                .Cast<CadResultEnvelope>()
                .ToArray();

            var references = ResolveReferences(
                part,
                operation,
                upstream,
                outcomes);

            var invalidReference = references.FirstOrDefault(
                x => x.Status != CadEvaluationStatus.Succeeded);

            var identity = CadEvaluationIdentityBuilder.Build(
                part,
                operation,
                upstream,
                references,
                options);

            if (invalidReference is not null)
            {
                var failed = new CadEvaluationOutcome(
                    operation.Id,
                    operation.OperationKind,
                    identity,
                    invalidReference.Status,
                    null,
                    references,
                    new[]
                    {
                        new CadDiagnostic(
                            "CAD_REFERENCE_RESOLUTION_FAILED",
                            invalidReference.Diagnostic ??
                            $"Reference '{invalidReference.Reference.Id}' failed resolution.",
                            invalidReference.Status,
                            operation.Id)
                    });

                outcomes[operationId] = failed;
                _cache.Put(identity, failed);
                break;
            }

            if (incremental &&
                !invalidated.Contains(operationId) &&
                _previous.TryGetValue(operationId, out var previous) &&
                previous.EvaluationIdentity == identity &&
                _cache.TryGet(identity, out var cached))
            {
                outcomes[operationId] = cached;
                continue;
            }

            var inputResults = upstream
                .Select(x => x.Id)
                .OrderBy(x => x.Value, StringComparer.Ordinal)
                .ToArray();

            var incrementalBase = inputResults.LastOrDefault();

            var request = new KernelOperationRequest(
                CadContractVersions.Kernel,
                identity,
                part.Id,
                operation.Id,
                operation.OperationKind,
                incremental
                    ? KernelEvaluationMode.Incremental
                    : KernelEvaluationMode.Full,
                incremental && incrementalBase.IsValid
                    ? incrementalBase
                    : null,
                inputResults,
                operation.Inputs);

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
                response = new KernelOperationResponse(
                    CadEvaluationStatus.Failed,
                    null,
                    null,
                    Array.Empty<KernelTopologyBinding>(),
                    new[]
                    {
                        new CadDiagnostic(
                            "KERNEL_EXCEPTION",
                            exception.Message,
                            CadEvaluationStatus.Failed,
                            operation.Id)
                    });
            }

            if (response.Status != CadEvaluationStatus.Succeeded ||
                response.AuthoritativeResultId is null ||
                string.IsNullOrWhiteSpace(response.EvidenceHash))
            {
                var failure = new CadEvaluationOutcome(
                    operation.Id,
                    operation.OperationKind,
                    identity,
                    response.Status,
                    null,
                    references,
                    response.Diagnostics);

                outcomes[operationId] = failure;
                _cache.Put(identity, failure);
                break;
            }

            var resultKind = operation switch
            {
                SketchOperation => CadResultKind.SketchProfile,
                ExtrusionOperation => CadResultKind.Body,
                HoleOperation => CadResultKind.Body,
                _ => throw new NotSupportedException(
                    $"No result kind exists for '{operation.GetType().Name}'.")
            };

            var result = new CadResultEnvelope(
                response.AuthoritativeResultId.Value,
                resultKind,
                operation.Id,
                inputResults,
                response.EvidenceHash,
                response.Topology,
                response.Status);

            var outcome = new CadEvaluationOutcome(
                operation.Id,
                operation.OperationKind,
                identity,
                response.Status,
                result,
                references,
                response.Diagnostics);

            outcomes[operationId] = outcome;
            _cache.Put(identity, outcome);
        }

        var (currentBodies, bodyDiagnostics) =
            ResolveCurrentBodies(part, outcomes);

        _previous.Clear();

        foreach (var pair in outcomes)
            _previous[pair.Key] = pair.Value;

        return new CadEvaluationSnapshot(
            part,
            plan,
            incremental
                ? KernelEvaluationMode.Incremental
                : KernelEvaluationMode.Full,
            outcomes.Keys.ToHashSet(),
            new ReadOnlyDictionary<CadId, CadEvaluationOutcome>(outcomes),
            new ReadOnlyDictionary<CadId, CadResultId?>(
                currentBodies),
            bodyDiagnostics);
    }
    private static (
        IReadOnlyDictionary<CadId, CadResultId?> Results,
        IReadOnlyList<CadDiagnostic> Diagnostics)
        ResolveCurrentBodies(
            CadPartDefinition part,
            IReadOnlyDictionary<CadId, CadEvaluationOutcome> outcomes)
    {
        var results = part.Bodies.ToDictionary(
            body => body.Id,
            _ => (CadResultId?)null);

        var diagnostics = new List<CadDiagnostic>();
        var operationsByBody = part.Operations
            .GroupBy(operation => operation.BodyId)
            .ToDictionary(
                group => group.Key,
                group => group.ToArray());

        foreach (var body in part.Bodies)
        {
            if (!operationsByBody.TryGetValue(body.Id, out var operations))
                continue;

            var bodyOperations = operations
                .Where(operation => operation.OutputKind == CadResultKind.Body)
                .ToArray();

            if (bodyOperations.Length == 0)
                continue;

            var consumedByBodyOperation = bodyOperations
                .SelectMany(operation => operation.InputOperationIds)
                .ToHashSet();

            var terminal = bodyOperations
                .Where(operation => !consumedByBodyOperation.Contains(operation.Id))
                .OrderBy(operation => operation.Id.Value, StringComparer.Ordinal)
                .ToArray();

            if (terminal.Length != 1)
            {
                diagnostics.Add(
                    new CadDiagnostic(
                        terminal.Length == 0
                            ? "CAD_BODY_PIPELINE_CYCLE_OR_NO_TERMINAL"
                            : "CAD_BODY_PIPELINE_MULTIPLE_TERMINALS",
                        $"Body '{body.Id}' does not have exactly one terminal body result.",
                        CadEvaluationStatus.Indeterminate,
                        body.Id));
                continue;
            }

            if (!outcomes.TryGetValue(
                    terminal[0].Id,
                    out var outcome) ||
                outcome.Result is null)
            {
                diagnostics.Add(
                    new CadDiagnostic(
                        "CAD_CURRENT_BODY_UNAVAILABLE",
                        $"Terminal body operation '{terminal[0].Id}' has no authoritative result.",
                        CadEvaluationStatus.Failed,
                        body.Id));
                continue;
            }

            if (outcome.Result.Kind != CadResultKind.Body)
            {
                diagnostics.Add(
                    new CadDiagnostic(
                        "CAD_CURRENT_BODY_KIND_MISMATCH",
                        $"Terminal operation '{terminal[0].Id}' did not produce a Body result.",
                        CadEvaluationStatus.Failed,
                        body.Id));
                continue;
            }

            results[body.Id] = outcome.Result.Id;
        }

        return (
            new ReadOnlyDictionary<CadId, CadResultId?>(results),
            diagnostics.AsReadOnly());
    }

    private static IReadOnlyList<CadReferenceResolution> ResolveReferences(
        CadPartDefinition part,
        CadOperation operation,
        IReadOnlyList<CadResultEnvelope> upstream,
        IReadOnlyDictionary<CadId, CadEvaluationOutcome> outcomes)
    {
        var resolutions = new List<CadReferenceResolution>();

        foreach (var reference in operation.References)
        {
            switch (reference.TargetKind)
            {
                case ReferenceTargetKind.Result:
                {
                    var result = outcomes.Values
                        .Select(x => x.Result)
                        .FirstOrDefault(x =>
                            x is not null &&
                            x.Id == reference.ResultId);

                    resolutions.Add(result is null
                        ? new CadReferenceResolution(
                            reference,
                            CadEvaluationStatus.Failed,
                            null,
                            "Referenced authoritative result is unavailable.")
                        : new CadReferenceResolution(
                            reference,
                            CadEvaluationStatus.Succeeded,
                            result.Id,
                            null));
                    break;
                }

                case ReferenceTargetKind.Topology:
                {
                    var result = outcomes.Values
                        .Select(x => x.Result)
                        .FirstOrDefault(x =>
                            x is not null &&
                            x.Id == reference.ResultId);

                    if (result is null)
                    {
                        resolutions.Add(
                            new CadReferenceResolution(
                                reference,
                                CadEvaluationStatus.Failed,
                                null,
                                "Referenced topology result is unavailable."));
                        break;
                    }

                    var matchCount = result.Topology.Count(x =>
                        string.Equals(
                            x.Kind,
                            reference.TopologyKind,
                            StringComparison.Ordinal) &&
                        string.Equals(
                            x.Key,
                            reference.TopologyKey,
                            StringComparison.Ordinal));

                    resolutions.Add(
                        matchCount switch
                        {
                            1 => new CadReferenceResolution(
                                reference,
                                CadEvaluationStatus.Succeeded,
                                result.Id,
                                null),
                            0 => new CadReferenceResolution(
                                reference,
                                CadEvaluationStatus.Failed,
                                null,
                                "Requested topology entity does not exist in the authoritative result."),
                            _ => new CadReferenceResolution(
                                reference,
                                CadEvaluationStatus.Ambiguous,
                                null,
                                "Requested topology entity is multiply bound.")
                        });
                    break;
                }

                case ReferenceTargetKind.Publication:
                {
                    var publication = part.Publications
                        .Where(x => x.Id == reference.PublicationId)
                        .ToArray();

                    if (publication.Length != 1)
                    {
                        resolutions.Add(
                            new CadReferenceResolution(
                                reference,
                                publication.Length == 0
                                    ? CadEvaluationStatus.Failed
                                    : CadEvaluationStatus.Ambiguous,
                                null,
                                "Publication cannot be resolved uniquely."));
                        break;
                    }

                    var source = outcomes.Values
                        .Select(x => x.Result)
                        .FirstOrDefault(x =>
                            x is not null &&
                            x.Id == publication[0].SourceResult);

                    resolutions.Add(source is null
                        ? new CadReferenceResolution(
                            reference,
                            CadEvaluationStatus.Failed,
                            null,
                            "Publication source result is unavailable.")
                        : new CadReferenceResolution(
                            reference,
                            CadEvaluationStatus.Succeeded,
                            source.Id,
                            null));
                    break;
                }

                case ReferenceTargetKind.Semantic:
                    resolutions.Add(
                        new CadReferenceResolution(
                            reference,
                            CadEvaluationStatus.Unsupported,
                            null,
                            "Generic semantic references require an explicit semantic catalog."));
                    break;

                default:
                    resolutions.Add(
                        new CadReferenceResolution(
                            reference,
                            CadEvaluationStatus.Unsupported,
                            null,
                            "Unknown reference target kind."));
                    break;
            }
        }

        return resolutions;
    }

}
