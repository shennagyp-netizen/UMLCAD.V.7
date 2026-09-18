using System.Globalization;
using System.Security.Cryptography;
using System.Text;
using UMLCAD.Cad.Contracts;

namespace UMLCAD.Cad.Engine;

public sealed class SpecificationGraph
{
    private readonly IReadOnlyDictionary<CadId, CadFeatureSpecification> _features;
    private readonly IReadOnlyDictionary<CadId, IReadOnlyList<CadId>> _dependencies;
    private readonly IReadOnlyDictionary<CadId, IReadOnlyList<CadId>> _dependents;

    private SpecificationGraph(
        IReadOnlyDictionary<CadId, CadFeatureSpecification> features,
        IReadOnlyDictionary<CadId, IReadOnlyList<CadId>> dependencies,
        IReadOnlyDictionary<CadId, IReadOnlyList<CadId>> dependents)
    {
        _features = features;
        _dependencies = dependencies;
        _dependents = dependents;
    }

    public IReadOnlyDictionary<CadId, CadFeatureSpecification> Features => _features;

    public IReadOnlyList<CadId> DependenciesOf(CadId featureId) =>
        _dependencies.TryGetValue(featureId, out var dependencies) ? dependencies : Array.Empty<CadId>();

    public static SpecificationGraph Build(CadDocumentSpecification document)
    {
        document.Validate();
        var features = document.Features.ToDictionary(x => x.Id, x => x);
        var dependencies = new Dictionary<CadId, IReadOnlyList<CadId>>();

        foreach (var feature in document.Features)
        {
            var ids = feature switch
            {
                BoxFeatureSpecification => Array.Empty<CadId>(),
                SketchFeatureSpecification sketch => new[] { sketch.Support.TargetSpecificationId },
                ExtrusionFeatureSpecification extrusion =>
                    new[] { extrusion.ProfileSketchId, extrusion.Support.TargetSpecificationId },
                _ => throw new NotSupportedException($"Unsupported CAD feature type '{feature.GetType().Name}'.")
            };

            foreach (var dependency in ids.Distinct())
            {
                if (!features.ContainsKey(dependency))
                    throw new InvalidOperationException($"Feature '{feature.Id}' depends on missing feature '{dependency}'.");
                if (dependency == feature.Id)
                    throw new InvalidOperationException($"Feature '{feature.Id}' cannot depend on itself.");
            }

            dependencies[feature.Id] = ids.Distinct()
                .OrderBy(x => x.Value, StringComparer.Ordinal)
                .ToArray();
        }

        var dependentSets = dependencies.Keys.ToDictionary(
            key => key,
            _ => new SortedSet<CadId>(Comparer<CadId>.Create(
                (left, right) => StringComparer.Ordinal.Compare(left.Value, right.Value))));

        foreach (var pair in dependencies)
            foreach (var dependency in pair.Value)
                dependentSets[dependency].Add(pair.Key);

        return new SpecificationGraph(
            features,
            dependencies,
            dependentSets.ToDictionary(x => x.Key, x => (IReadOnlyList<CadId>)x.Value.ToArray()));
    }

    public IReadOnlySet<CadId> ComputeInvalidation(IEnumerable<CadId> changedFeatureIds)
    {
        var invalidated = new HashSet<CadId>();
        var queue = new Queue<CadId>(
            changedFeatureIds.Distinct().OrderBy(x => x.Value, StringComparer.Ordinal));

        while (queue.Count != 0)
        {
            var current = queue.Dequeue();
            if (!invalidated.Add(current))
                continue;

            if (_dependents.TryGetValue(current, out var dependents))
                foreach (var dependent in dependents)
                    queue.Enqueue(dependent);
        }

        return invalidated;
    }
}

public sealed record CadChangeSet(IReadOnlySet<CadId> ChangedFeatureIds)
{
    public static CadChangeSet Empty { get; } = new(new HashSet<CadId>());
}

public sealed record EvaluationPlan(IReadOnlyList<CadId> FeatureIds);

public static class EvaluationPlanner
{
    public static EvaluationPlan Plan(SpecificationGraph graph)
    {
        var remaining = graph.Features.Keys.ToDictionary(x => x, _ => 0);
        foreach (var feature in graph.Features.Keys)
            remaining[feature] = graph.DependenciesOf(feature).Count;

        var ready = new SortedSet<CadId>(Comparer<CadId>.Create(
            (left, right) => StringComparer.Ordinal.Compare(left.Value, right.Value)));

        foreach (var pair in remaining)
            if (pair.Value == 0)
                ready.Add(pair.Key);

        var result = new List<CadId>(graph.Features.Count);
        while (ready.Count != 0)
        {
            var current = ready.Min!;
            ready.Remove(current);
            result.Add(current);

            foreach (var dependent in graph.Features.Keys
                         .Where(x => graph.DependenciesOf(x).Contains(current))
                         .OrderBy(x => x.Value, StringComparer.Ordinal))
            {
                remaining[dependent]--;
                if (remaining[dependent] == 0)
                    ready.Add(dependent);
            }
        }

        if (result.Count != graph.Features.Count)
            throw new InvalidOperationException("CAD specification graph contains an evaluation cycle.");

        return new EvaluationPlan(result);
    }
}

public interface IEvaluationCache
{
    bool TryGet(string evaluationIdentity, out CadFeatureEvaluationResult result);
    void Put(CadFeatureEvaluationResult result);
    void Clear();
}

public sealed class InMemoryEvaluationCache : IEvaluationCache
{
    private readonly Dictionary<string, CadFeatureEvaluationResult> _values = new(StringComparer.Ordinal);

    public bool TryGet(string evaluationIdentity, out CadFeatureEvaluationResult result) =>
        _values.TryGetValue(evaluationIdentity, out result!);

    public void Put(CadFeatureEvaluationResult result) =>
        _values[result.EvaluationIdentity] = result;

    public void Clear() => _values.Clear();
}

public static class EvaluationIdentity
{
    public static string Compute(
        CadFeatureSpecification feature,
        IReadOnlyList<ReferenceResolution> references,
        IReadOnlyList<CadFeatureEvaluationResult> upstream,
        string representationPolicy,
        string evaluationContext = "",
        string kernelContractVersion = CadContractVersions.KernelEvaluation)
    {
        var builder = new StringBuilder();
        Append(builder, "kernelContract", kernelContractVersion);
        Append(builder, "representationPolicy", representationPolicy);
        Append(builder, "evaluationContext", evaluationContext);
        AppendFeature(builder, feature);

        foreach (var reference in references.OrderBy(x => x.Reference.ReferenceId.Value, StringComparer.Ordinal))
        {
            Append(builder, "reference.id", reference.Reference.ReferenceId.Value);
            Append(builder, "reference.kind", reference.Reference.Kind.ToString());
            Append(builder, "reference.target", reference.Reference.TargetSpecificationId.Value);
            Append(builder, "reference.status", reference.Status.ToString());
            foreach (var candidate in reference.Candidates.OrderBy(x => x.Value, StringComparer.Ordinal))
                Append(builder, "reference.candidate", candidate.Value);
        }

        foreach (var evaluation in upstream.OrderBy(x => x.FeatureId.Value, StringComparer.Ordinal))
        {
            Append(builder, "upstream.feature", evaluation.FeatureId.Value);
            Append(builder, "upstream.identity", evaluation.EvaluationIdentity);
            Append(builder, "upstream.result", evaluation.ResultId?.Value ?? "null");
        }

        return Convert.ToHexString(SHA256.HashData(Encoding.UTF8.GetBytes(builder.ToString()))).ToLowerInvariant();
    }

    public static string ComputeDocument(
        CadDocumentSpecification document,
        EvaluationPlan plan,
        IEnumerable<CadFeatureEvaluationResult> evaluations,
        string representationPolicy)
    {
        var builder = new StringBuilder();
        Append(builder, "document", document.DocumentId.Value);
        Append(builder, "revision", document.Revision);
        Append(builder, "configuration", document.Configuration);
        Append(builder, "representationPolicy", representationPolicy);

        foreach (var id in plan.FeatureIds)
            Append(builder, "plan", id.Value);

        foreach (var evaluation in evaluations.OrderBy(x => x.FeatureId.Value, StringComparer.Ordinal))
        {
            Append(builder, "feature", evaluation.FeatureId.Value);
            Append(builder, "identity", evaluation.EvaluationIdentity);
            Append(builder, "status", evaluation.Status.ToString());
            Append(builder, "result", evaluation.ResultId?.Value ?? "null");
        }

        return Convert.ToHexString(SHA256.HashData(Encoding.UTF8.GetBytes(builder.ToString()))).ToLowerInvariant();
    }

    private static void AppendFeature(StringBuilder builder, CadFeatureSpecification feature)
    {
        switch (feature)
        {
            case BoxFeatureSpecification box:
                Append(builder, "feature.kind", "box");
                Append(builder, "feature.id", box.Id.Value);
                AppendFrame(builder, box.Frame);
                Append(builder, "width", box.Width);
                Append(builder, "depth", box.Depth);
                Append(builder, "height", box.Height);
                break;

            case SketchFeatureSpecification sketch:
                Append(builder, "feature.kind", "sketch");
                Append(builder, "feature.id", sketch.Id.Value);
                AppendFrame(builder, sketch.Frame);
                AppendReference(builder, sketch.Support);
                foreach (var circle in sketch.Circles.OrderBy(x => x.Id.Value, StringComparer.Ordinal))
                {
                    Append(builder, "circle.id", circle.Id.Value);
                    Append(builder, "circle.x", circle.X);
                    Append(builder, "circle.y", circle.Y);
                    Append(builder, "circle.radius", circle.Radius);
                }
                foreach (var constraint in sketch.Constraints.OrderBy(x => x.Id.Value, StringComparer.Ordinal))
                {
                    Append(builder, "constraint.id", constraint.Id.Value);
                    Append(builder, "constraint.kind", constraint.Kind.ToString());
                    Append(builder, "constraint.geometry", constraint.GeometryId.Value);
                }
                break;

            case ExtrusionFeatureSpecification extrusion:
                Append(builder, "feature.kind", "extrusion");
                Append(builder, "feature.id", extrusion.Id.Value);
                Append(builder, "profileSketch", extrusion.ProfileSketchId.Value);
                AppendReference(builder, extrusion.Support);
                Append(builder, "direction.x", extrusion.Direction.X);
                Append(builder, "direction.y", extrusion.Direction.Y);
                Append(builder, "direction.z", extrusion.Direction.Z);
                Append(builder, "distance", extrusion.Distance);
                Append(builder, "operation", extrusion.Operation.ToString());
                break;

            default:
                throw new NotSupportedException($"Unsupported CAD feature type '{feature.GetType().Name}'.");
        }
    }

    private static void AppendReference(StringBuilder builder, CadReference reference)
    {
        Append(builder, "support.reference", reference.ReferenceId.Value);
        Append(builder, "support.kind", reference.Kind.ToString());
        Append(builder, "support.target", reference.TargetSpecificationId.Value);
        Append(builder, "support.selector.kind", reference.Selector.SelectorKind);
        foreach (var parameter in reference.Selector.Parameters.OrderBy(x => x.Key, StringComparer.Ordinal))
        {
            Append(builder, "support.selector.parameter", parameter.Key);
            Append(builder, "support.selector.value", parameter.Value);
        }
        Append(builder, "support.context.frame", reference.Context.FrameKind.ToString());
        Append(builder, "support.context.configuration", reference.Context.Configuration);
        Append(builder, "support.context.expectedResult", reference.Context.ExpectedResultId?.Value ?? "null");
        Append(builder, "support.context.occurrencePath", reference.Context.OccurrencePath ?? "null");
    }

    private static void AppendFrame(StringBuilder builder, CadFrame frame)
    {
        Append(builder, "frame.id", frame.Id.Value);
        Append(builder, "frame.kind", frame.Kind.ToString());
        Append(builder, "frame.x", frame.OriginX);
        Append(builder, "frame.y", frame.OriginY);
        Append(builder, "frame.z", frame.OriginZ);
    }

    private static void Append(StringBuilder builder, string key, string value) =>
        builder.Append(key.Length).Append(':').Append(key).Append(value.Length).Append(':').Append(value).Append('|');

    private static void Append(StringBuilder builder, string key, double value) =>
        Append(builder, key, value.ToString("R", CultureInfo.InvariantCulture));
}

public sealed class CadEvaluationEngine
{
    private readonly ICadKernelEvaluator _kernel;
    private readonly ICadReferenceResolver _referenceResolver;
    private readonly IEvaluationCache _cache;
    private readonly Dictionary<CadId, CadFeatureEvaluationResult> _lastResults = new();

    public CadEvaluationEngine(
        ICadKernelEvaluator kernel,
        IEvaluationCache? cache = null,
        ICadReferenceResolver? referenceResolver = null)
    {
        _kernel = kernel ?? throw new ArgumentNullException(nameof(kernel));
        _referenceResolver = referenceResolver ?? new AuthoritativeCadReferenceResolver();
        _cache = cache ?? new InMemoryEvaluationCache();
    }

    public async Task<CadEvaluationResult> RecomputeAsync(
        CadDocumentSpecification document,
        CadChangeSet? changes = null,
        string representationPolicy = "semantic-default",
        CancellationToken cancellationToken = default)
    {
        document.Validate();
        var graph = SpecificationGraph.Build(document);
        var plan = EvaluationPlanner.Plan(graph);

        var full = _lastResults.Count == 0 || changes is null;
        var mode = full ? RecomputeMode.Full : RecomputeMode.Incremental;
        var invalidated = full ? graph.Features.Keys.ToHashSet() : graph.ComputeInvalidation(changes!.ChangedFeatureIds);

        var current = new Dictionary<CadId, CadFeatureEvaluationResult>();
        AuthoritativeCadResult? finalResult = null;
        CadDiagnostic? failure = null;

        foreach (var featureId in plan.FeatureIds)
        {
            cancellationToken.ThrowIfCancellationRequested();
            var feature = graph.Features[featureId];
            var dependencies = graph.DependenciesOf(featureId).OrderBy(x => x.Value, StringComparer.Ordinal).ToArray();
            var upstream = dependencies.Where(current.ContainsKey).Select(x => current[x]).ToArray();
            var references = ResolveReferences(feature, current, graph);

            var identity = EvaluationIdentity.Compute(
                feature,
                references,
                upstream,
                representationPolicy,
                $"{document.Revision}|{document.Configuration}");

            var invalidReference = references.FirstOrDefault(x => !x.IsResolved);
            if (invalidReference is not null)
            {
                var status = invalidReference.Status switch
                {
                    ReferenceResolutionStatus.Missing => CadEvaluationStatus.MissingReference,
                    ReferenceResolutionStatus.Ambiguous => CadEvaluationStatus.AmbiguousReference,
                    ReferenceResolutionStatus.Indeterminate => CadEvaluationStatus.Indeterminate,
                    ReferenceResolutionStatus.Unsupported => CadEvaluationStatus.Unsupported,
                    _ => CadEvaluationStatus.Indeterminate
                };
                var diagnostic = new CadDiagnostic(
                    invalidReference.DiagnosticCode ?? "REFERENCE_RESOLUTION_FAILED",
                    status,
                    invalidReference.Evidence ?? $"Reference '{invalidReference.Reference.ReferenceId}' did not resolve uniquely.");

                var failed = new CadFeatureEvaluationResult(
                    featureId, identity, status, null, references, new[] { diagnostic }, null);
                current[featureId] = failed;
                _cache.Put(failed);
                failure = diagnostic;
                break;
            }

            if (!full &&
                !invalidated.Contains(featureId) &&
                _lastResults.TryGetValue(featureId, out var previous) &&
                previous.EvaluationIdentity == identity &&
                _cache.TryGet(identity, out var cached))
            {
                current[featureId] = cached;
                if (cached.AuthoritativeResult is not null)
                    finalResult = cached.AuthoritativeResult;
                continue;
            }

            var upstreamResult = FindReferenceTargetResult(feature, current);
            var request = new KernelEvaluationRequest(
                new CadId(identity),
                feature,
                upstreamResult,
                references,
                upstream);

            KernelEvaluationResponse response;
            try
            {
                request.Validate();
                response = await _kernel.EvaluateAsync(request, cancellationToken);
            }
            catch (OperationCanceledException)
            {
                throw;
            }
            catch (Exception exception)
            {
                response = new KernelEvaluationResponse(
                    CadEvaluationStatus.KernelFailure,
                    null,
                    new[]
                    {
                        new CadDiagnostic("KERNEL_EXCEPTION", CadEvaluationStatus.KernelFailure, exception.Message)
                    });
            }

            var featureResult = new CadFeatureEvaluationResult(
                featureId,
                identity,
                response.Status,
                response.AuthoritativeResult?.ResultId,
                references,
                response.Diagnostics,
                response.AuthoritativeResult);

            current[featureId] = featureResult;
            _cache.Put(featureResult);

            if (!featureResult.Succeeded)
            {
                failure = response.Diagnostics.FirstOrDefault()
                    ?? new CadDiagnostic("EVALUATION_FAILED", response.Status, $"Feature '{featureId}' failed evaluation.");
                break;
            }

            if (response.AuthoritativeResult is not null)
                finalResult = response.AuthoritativeResult;
        }

        _lastResults.Clear();
        foreach (var pair in current)
            _lastResults[pair.Key] = pair.Value;

        var documentIdentity = new CadResultId(
            EvaluationIdentity.ComputeDocument(document, plan, current.Values, representationPolicy));

        CadRepresentation? representation = null;
        if (failure is null && finalResult is not null)
            representation = BuildRepresentation(finalResult, representationPolicy);

        return new CadEvaluationResult(
            CadContractVersions.Evaluation,
            document.DocumentId,
            documentIdentity,
            mode,
            plan.FeatureIds,
            invalidated,
            current.Values.OrderBy(x => x.FeatureId.Value, StringComparer.Ordinal).ToArray(),
            finalResult,
            representation,
            failure);
    }

    private IReadOnlyList<ReferenceResolution> ResolveReferences(
        CadFeatureSpecification feature,
        IReadOnlyDictionary<CadId, CadFeatureEvaluationResult> current,
        SpecificationGraph graph)
    {
        var references = feature switch
        {
            SketchFeatureSpecification sketch => new[] { sketch.Support },
            ExtrusionFeatureSpecification extrusion => new[] { extrusion.Support },
            _ => Array.Empty<CadReference>()
        };

        var resolutions = new List<ReferenceResolution>(references.Length);
        foreach (var reference in references)
        {
            if (!graph.Features.ContainsKey(reference.TargetSpecificationId) ||
                !current.TryGetValue(reference.TargetSpecificationId, out var target) ||
                target.AuthoritativeResult is null)
            {
                resolutions.Add(new ReferenceResolution(
                    reference,
                    ReferenceResolutionStatus.Missing,
                    Array.Empty<TopologyEntityId>(),
                    "REFERENCE_TARGET_RESULT_MISSING",
                    "The semantic target has no authoritative result at this evaluation step."));
                continue;
            }

            resolutions.Add(_referenceResolver.Resolve(reference, target.AuthoritativeResult));
        }

        return resolutions;
    }

    private static AuthoritativeCadResult? FindReferenceTargetResult(
        CadFeatureSpecification feature,
        IReadOnlyDictionary<CadId, CadFeatureEvaluationResult> current)
    {
        var target = feature switch
        {
            SketchFeatureSpecification sketch => sketch.Support.TargetSpecificationId,
            ExtrusionFeatureSpecification extrusion => extrusion.Support.TargetSpecificationId,
            _ => (CadId?)null
        };

        return target is { } id && current.TryGetValue(id, out var evaluation)
            ? evaluation.AuthoritativeResult
            : null;
    }

    private static CadRepresentation BuildRepresentation(AuthoritativeCadResult result, string displayPolicy)
    {
        result.Validate();
        var material = string.Join(
            "|",
            result.ResultId.Value,
            result.Topology.Entities.Count.ToString(CultureInfo.InvariantCulture),
            displayPolicy);

        var hash = Convert.ToHexString(SHA256.HashData(Encoding.UTF8.GetBytes(material))).ToLowerInvariant();
        return new CadRepresentation(
            CadContractVersions.Representation,
            new CadId($"representation:{hash}"),
            result.ResultId,
            "authoritative-brep-derived",
            displayPolicy,
            hash);
    }
}
