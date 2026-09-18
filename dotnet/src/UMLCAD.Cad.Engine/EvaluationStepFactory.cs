using UMLCAD.Cad.Semantics;

namespace UMLCAD.Cad.Engine;

public static class EvaluationStepFactory
{
    public static EvaluationStep Create(
        FeatureSpecification specification,
        string configurationContext,
        string tolerancePolicy,
        string representationPolicy,
        IReadOnlyList<EvaluationInputIdentity>? additionalInputs = null)
    {
        ArgumentNullException.ThrowIfNull(specification);

        if (string.IsNullOrWhiteSpace(configurationContext))
            throw new ArgumentException("ConfigurationContext is required.", nameof(configurationContext));
        if (string.IsNullOrWhiteSpace(tolerancePolicy))
            throw new ArgumentException("TolerancePolicy is required.", nameof(tolerancePolicy));
        if (string.IsNullOrWhiteSpace(representationPolicy))
            throw new ArgumentException("RepresentationPolicy is required.", nameof(representationPolicy));

        var inputs = new List<EvaluationInputIdentity>
        {
            new("feature", specification.FeatureId.Value.ToString("D")),
            new("part", specification.PartId.Value.ToString("D")),
        };

        if (additionalInputs is not null)
            inputs.AddRange(additionalInputs);

        return new EvaluationStep(
            specification.FeatureId,
            specification.OperationKind,
            specification.CanonicalDefinition,
            specification.Dependencies,
            inputs,
            configurationContext,
            tolerancePolicy,
            "kernel-contract-v1",
            representationPolicy);
    }
}
