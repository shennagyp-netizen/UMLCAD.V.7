using UMLCAD.Cad.Contracts;
using UMLCAD.Cad.Semantics;

namespace UMLCAD.Cad.Engine;

public sealed record FeatureEvaluationOptions(
    string ConfigurationContext,
    KernelTolerance Tolerance,
    string TolerancePolicy,
    string RepresentationPolicy)
{
    public FeatureEvaluationOptions
    {
        if (string.IsNullOrWhiteSpace(ConfigurationContext))
            throw new ArgumentException("Configuration context is required.", nameof(ConfigurationContext));
        if (string.IsNullOrWhiteSpace(TolerancePolicy))
            throw new ArgumentException("Tolerance policy is required.", nameof(TolerancePolicy));
        if (string.IsNullOrWhiteSpace(RepresentationPolicy))
            throw new ArgumentException("Representation policy is required.", nameof(RepresentationPolicy));
    }
}

public sealed class FeatureSpecificationCatalog
{
    private readonly IReadOnlyDictionary<SemanticId, FeatureSpecification> _specifications;

    public FeatureSpecificationCatalog(IEnumerable<FeatureSpecification> specifications)
    {
        ArgumentNullException.ThrowIfNull(specifications);

        var values = specifications.ToArray();
        if (values.Any(x => x is null))
            throw new ArgumentException("Feature specifications cannot contain null values.", nameof(specifications));

        if (values.GroupBy(x => x.FeatureId).Any(group => group.Count() != 1))
            throw new ArgumentException("Feature identifiers must be unique.", nameof(specifications));

        _specifications = values.ToDictionary(x => x.FeatureId);
    }

    public FeatureSpecification Get(SemanticId featureId)
    {
        if (!_specifications.TryGetValue(featureId, out var specification))
            throw new InvalidOperationException(
                $"No feature specification exists for '{featureId}'.");

        return specification;
    }
}

public interface IAuthoritativeFeatureEvaluator
{
    string OperationKind { get; }

    Task<AuthoritativeCadResult> EvaluateAsync(
        FeatureSpecification specification,
        FeatureEvaluationOptions options,
        CancellationToken cancellationToken = default);
}

public sealed class AxisAlignedBoxFeatureEvaluator : IAuthoritativeFeatureEvaluator
{
    private readonly AxisAlignedBoxSolidEvaluator _evaluator;

    public AxisAlignedBoxFeatureEvaluator(
        IAuthoritativeGeometryService geometry)
    {
        _evaluator = new AxisAlignedBoxSolidEvaluator(geometry);
    }

    public string OperationKind => "PartDesign.AxisAlignedBoxSolid";

    public Task<AuthoritativeCadResult> EvaluateAsync(
        FeatureSpecification specification,
        FeatureEvaluationOptions options,
        CancellationToken cancellationToken = default)
    {
        ArgumentNullException.ThrowIfNull(specification);
        ArgumentNullException.ThrowIfNull(options);

        if (specification is not AxisAlignedBoxSolidSpecification box)
            throw new ArgumentException(
                $"Operation '{OperationKind}' requires '{nameof(AxisAlignedBoxSolidSpecification)}'.",
                nameof(specification));

        return _evaluator.EvaluateAsync(
            box,
            options.ConfigurationContext,
            options.Tolerance,
            cancellationToken);
    }
}

public sealed class ConvexProfileExtrusionFeatureEvaluator : IAuthoritativeFeatureEvaluator
{
    private readonly ConvexProfileExtrusionEvaluator _evaluator;

    public ConvexProfileExtrusionFeatureEvaluator(IExtrusionGeometryService geometry)
    {
        _evaluator = new ConvexProfileExtrusionEvaluator(geometry);
    }

    public string OperationKind => "PartDesign.ExtrudeConvexPlanarProfile";

    public Task<AuthoritativeCadResult> EvaluateAsync(
        FeatureSpecification specification,
        FeatureEvaluationOptions options,
        CancellationToken cancellationToken = default)
    {
        ArgumentNullException.ThrowIfNull(specification);
        ArgumentNullException.ThrowIfNull(options);

        if (specification is not ExtrusionFeatureSpecification extrusion)
            throw new ArgumentException(
                $"Operation '{OperationKind}' requires '{nameof(ExtrusionFeatureSpecification)}'.",
                nameof(specification));

        return _evaluator.EvaluateAsync(
            extrusion,
            options.Tolerance,
            cancellationToken);
    }
}

public sealed class AuthoritativeFeatureStepExecutor : IAsyncEvaluationStepExecutor
{
    private readonly FeatureSpecificationCatalog _catalog;
    private readonly IAuthoritativeFeatureEvaluator _evaluator;
    private readonly FeatureEvaluationOptions _options;

    public AuthoritativeFeatureStepExecutor(
        IEnumerable<FeatureSpecification> specifications,
        IAuthoritativeFeatureEvaluator evaluator,
        FeatureEvaluationOptions options)
    {
        _catalog = new FeatureSpecificationCatalog(specifications);
        _evaluator = evaluator ?? throw new ArgumentNullException(nameof(evaluator));
        _options = options ?? throw new ArgumentNullException(nameof(options));
    }

    public string OperationKind => _evaluator.OperationKind;

    public async Task<EvaluationOutcome> ExecuteAsync(
        EvaluationStep step,
        EvaluationIdentity identity,
        IReadOnlyDictionary<SemanticId, EvaluationOutcome> completed,
        CancellationToken cancellationToken)
    {
        ArgumentNullException.ThrowIfNull(step);
        ArgumentNullException.ThrowIfNull(completed);

        var specification = _catalog.Get(step.StepId);

        if (!string.Equals(specification.OperationKind, step.OperationKind, StringComparison.Ordinal))
        {
            return new EvaluationOutcome(
                step.StepId,
                identity,
                EvaluationOutcomeStatus.Failed,
                null,
                ["Specification operation kind does not match evaluation step operation kind."]);
        }

        var result = await _evaluator.EvaluateAsync(
            specification,
            _options,
            cancellationToken);

        var status = result.Status switch
        {
            AuthoritativeResultStatus.Succeeded => EvaluationOutcomeStatus.Succeeded,
            AuthoritativeResultStatus.Failed => EvaluationOutcomeStatus.Failed,
            AuthoritativeResultStatus.Unsupported => EvaluationOutcomeStatus.Unsupported,
            AuthoritativeResultStatus.Ambiguous => EvaluationOutcomeStatus.Ambiguous,
            AuthoritativeResultStatus.Indeterminate => EvaluationOutcomeStatus.Indeterminate,
            _ => throw new InvalidOperationException(
                $"Unknown authoritative result status '{result.Status}'."),
        };

        return new EvaluationOutcome(
            step.StepId,
            identity,
            status,
            status == EvaluationOutcomeStatus.Succeeded ? result.Identity : null,
            result.Evidence.Diagnostics)
        {
            Result = result,
        };
    }
}

public static class FeatureEvaluationExecutorFactory
{
    public static IReadOnlyList<IAsyncEvaluationStepExecutor> Create(
        IEnumerable<FeatureSpecification> specifications,
        IEnumerable<IAuthoritativeFeatureEvaluator> evaluators,
        FeatureEvaluationOptions options)
    {
        ArgumentNullException.ThrowIfNull(specifications);
        ArgumentNullException.ThrowIfNull(evaluators);
        ArgumentNullException.ThrowIfNull(options);

        var specificationSnapshot = specifications.ToArray();
        var evaluatorSnapshot = evaluators
            .Where(x => x is not null)
            .OrderBy(x => x.OperationKind, StringComparer.Ordinal)
            .ToArray();

        if (evaluatorSnapshot.Length == 0)
            throw new ArgumentException(
                "At least one authoritative feature evaluator is required.",
                nameof(evaluators));

        if (evaluatorSnapshot.GroupBy(x => x.OperationKind, StringComparer.Ordinal).Any(g => g.Count() != 1))
            throw new ArgumentException(
                "Feature evaluator operation kinds must be unique.",
                nameof(evaluators));

        return evaluatorSnapshot
            .Select(evaluator => (IAsyncEvaluationStepExecutor)
                new AuthoritativeFeatureStepExecutor(
                    specificationSnapshot,
                    evaluator,
                    options))
            .ToArray();
    }
}
