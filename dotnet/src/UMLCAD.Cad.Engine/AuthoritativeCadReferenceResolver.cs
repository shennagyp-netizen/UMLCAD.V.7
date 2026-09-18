using System.Globalization;
using UMLCAD.Cad.Contracts;

namespace UMLCAD.Cad.Engine;

public interface ICadReferenceResolver
{
    ReferenceResolution Resolve(
        CadReference reference,
        AuthoritativeCadResult result);
}

public sealed class AuthoritativeCadReferenceResolver : ICadReferenceResolver
{
    private const double AbsoluteTolerance = 1e-9;
    private const double RelativeTolerance = 1e-9;

    public ReferenceResolution Resolve(
        CadReference reference,
        AuthoritativeCadResult result)
    {
        ArgumentNullException.ThrowIfNull(reference);
        ArgumentNullException.ThrowIfNull(result);
        reference.Validate();
        result.Validate();

        if (reference.Context.ExpectedResultId is { } expected &&
            expected != result.ResultId)
        {
            return new ReferenceResolution(
                reference,
                ReferenceResolutionStatus.Indeterminate,
                Array.Empty<TopologyEntityId>(),
                "REFERENCE_RESULT_MISMATCH",
                $"Reference expected result '{expected.Value}', but received '{result.ResultId.Value}'.");
        }

        var producerIds = result.Topology.Entities
            .Select(entity => entity.Provenance.ProducingFeatureId)
            .Where(id => id.IsValid)
            .Distinct()
            .ToArray();

        if (producerIds.Length != 1 ||
            producerIds[0] != reference.TargetSpecificationId)
        {
            return new ReferenceResolution(
                reference,
                ReferenceResolutionStatus.Indeterminate,
                Array.Empty<TopologyEntityId>(),
                "REFERENCE_TARGET_MISMATCH",
                "The authoritative result provenance does not match the reference target specification.");
        }

        if (reference.Selector.EntityKind != TopologyEntityKind.Face ||
            !string.Equals(reference.Selector.SelectorKind, "planar-face", StringComparison.Ordinal))
        {
            return new ReferenceResolution(
                reference,
                ReferenceResolutionStatus.Unsupported,
                Array.Empty<TopologyEntityId>(),
                "REFERENCE_SELECTOR_UNSUPPORTED",
                $"Selector '{reference.Selector.SelectorKind}' is not supported by the certified resolver.");
        }

        if (!TryReadPlanarSelector(reference.Selector, out var normal, out var point))
        {
            return new ReferenceResolution(
                reference,
                ReferenceResolutionStatus.Indeterminate,
                Array.Empty<TopologyEntityId>(),
                "REFERENCE_SELECTOR_INVALID",
                "Planar-face selector does not contain a complete finite normal/point predicate.");
        }

        var candidates = result.Topology.Faces
            .Where(face =>
                face.Normal is { } faceNormal &&
                face.Point is { } facePoint &&
                NearlyEqualVector(faceNormal, normal) &&
                NearlyEqualVector(facePoint, point))
            .OrderBy(face => face.Id.Value, StringComparer.Ordinal)
            .Select(face => face.Id)
            .ToArray();

        return candidates.Length switch
        {
            0 => new ReferenceResolution(
                reference,
                ReferenceResolutionStatus.Missing,
                candidates,
                "REFERENCE_MISSING",
                "No authoritative planar face satisfies the semantic normal/point predicate."),
            1 => new ReferenceResolution(
                reference,
                ReferenceResolutionStatus.Resolved,
                candidates,
                null,
                "Unique authoritative planar face resolved by semantic evidence."),
            _ => new ReferenceResolution(
                reference,
                ReferenceResolutionStatus.Ambiguous,
                candidates,
                "REFERENCE_AMBIGUOUS",
                "More than one authoritative face satisfies the semantic selector.")
        };
    }

    private static bool TryReadPlanarSelector(
        TopologySelector selector,
        out CadVector3 normal,
        out CadVector3 point)
    {
        normal = default;
        point = default;

        if (!TryReadFinite(selector.Parameters, "normalX", out var nx) ||
            !TryReadFinite(selector.Parameters, "normalY", out var ny) ||
            !TryReadFinite(selector.Parameters, "normalZ", out var nz) ||
            !TryReadFinite(selector.Parameters, "pointX", out var px) ||
            !TryReadFinite(selector.Parameters, "pointY", out var py) ||
            !TryReadFinite(selector.Parameters, "pointZ", out var pz))
            return false;

        try
        {
            normal = new CadVector3(nx, ny, nz).Normalize("Reference normal");
        }
        catch (ArgumentException)
        {
            return false;
        }

        point = new CadVector3(px, py, pz);
        return true;
    }

    private static bool TryReadFinite(
        IReadOnlyDictionary<string, string> parameters,
        string key,
        out double value) =>
        parameters.TryGetValue(key, out var raw) &&
        double.TryParse(
            raw,
            NumberStyles.Float | NumberStyles.AllowLeadingSign,
            CultureInfo.InvariantCulture,
            out value) &&
        double.IsFinite(value);

    private static bool NearlyEqualVector(CadVector3 left, CadVector3 right) =>
        NearlyEqual(left.X, right.X) &&
        NearlyEqual(left.Y, right.Y) &&
        NearlyEqual(left.Z, right.Z);

    private static bool NearlyEqual(double left, double right)
    {
        var scale = Math.Max(1d, Math.Max(Math.Abs(left), Math.Abs(right)));
        return Math.Abs(left - right) <= AbsoluteTolerance + RelativeTolerance * scale;
    }
}
