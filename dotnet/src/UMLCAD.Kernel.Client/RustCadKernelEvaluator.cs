using System.Globalization;
using UMLCAD.Cad.Contracts;

namespace UMLCAD.Kernel.Client;

/// <summary>
/// Production bridge from the system-CAD evaluation contract to the existing
/// certified geometry transport services. It owns transport/result adaptation,
/// not geometric mathematics or CAD semantic ownership.
/// </summary>
public sealed class RustCadKernelEvaluator : ICadKernelEvaluator
{
    private const double SelectorAbsoluteTolerance = 1e-9;
    private const double SelectorRelativeTolerance = 1e-9;

    private readonly IAuthoritativeGeometryService _boxGeometry;
    private readonly IExtrusionGeometryService _extrusionGeometry;

    public RustCadKernelEvaluator(
        IAuthoritativeGeometryService boxGeometry,
        IExtrusionGeometryService extrusionGeometry)
    {
        _boxGeometry = boxGeometry ?? throw new ArgumentNullException(nameof(boxGeometry));
        _extrusionGeometry = extrusionGeometry ?? throw new ArgumentNullException(nameof(extrusionGeometry));
    }

    public Task<ReferenceResolution> ResolveReferenceAsync(
        CadReference reference,
        AuthoritativeCadResult result,
        CancellationToken cancellationToken = default)
    {
        ArgumentNullException.ThrowIfNull(reference);
        ArgumentNullException.ThrowIfNull(result);
        reference.Validate();
        result.Validate();
        cancellationToken.ThrowIfCancellationRequested();

        if (reference.Context.ExpectedResultId is { } expected &&
            expected != result.ResultId)
        {
            return Task.FromResult(new ReferenceResolution(
                reference,
                ReferenceResolutionStatus.Indeterminate,
                Array.Empty<TopologyEntityId>(),
                "REFERENCE_RESULT_MISMATCH",
                $"Reference expected result '{expected.Value}', but received '{result.ResultId.Value}'."));
        }

        if (result.Topology.KernelContractVersion != CadContractVersions.KernelEvaluation)
        {
            return Task.FromResult(new ReferenceResolution(
                reference,
                ReferenceResolutionStatus.Unsupported,
                Array.Empty<TopologyEntityId>(),
                "REFERENCE_KERNEL_CONTRACT_UNSUPPORTED",
                $"Result uses unsupported kernel contract '{result.Topology.KernelContractVersion}'."));
        }

        if (!string.Equals(
                reference.Selector.SelectorKind,
                "planar-face",
                StringComparison.Ordinal) ||
            reference.Selector.EntityKind != TopologyEntityKind.Face)
        {
            return Task.FromResult(new ReferenceResolution(
                reference,
                ReferenceResolutionStatus.Unsupported,
                Array.Empty<TopologyEntityId>(),
                "REFERENCE_SELECTOR_UNSUPPORTED",
                $"Selector '{reference.Selector.SelectorKind}' is not supported by the first certified adapter subset."));
        }

        if (!TryReadPlanarSelector(reference.Selector, out var normal, out var point))
        {
            return Task.FromResult(new ReferenceResolution(
                reference,
                ReferenceResolutionStatus.Indeterminate,
                Array.Empty<TopologyEntityId>(),
                "REFERENCE_SELECTOR_INVALID",
                "Planar-face selector does not contain a complete finite normal/point predicate."));
        }

        if (result.Status != CadEvaluationStatus.SucceededEquivalent())
        {
            return Task.FromResult(new ReferenceResolution(
                reference,
                ReferenceResolutionStatus.Indeterminate,
                Array.Empty<TopologyEntityId>(),
                "REFERENCE_RESULT_NOT_AUTHORITATIVE",
                "A reference cannot resolve against a non-successful authoritative result."));
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
            0 => Task.FromResult(new ReferenceResolution(
                reference,
                ReferenceResolutionStatus.Missing,
                candidates,
                "REFERENCE_MISSING",
                "No authoritative planar face satisfies the semantic normal/point predicate.")),
            1 => Task.FromResult(new ReferenceResolution(
                reference,
                ReferenceResolutionStatus.Resolved,
                candidates,
                null,
                "Unique authoritative planar face resolved by semantic evidence.")),
            _ => Task.FromResult(new ReferenceResolution(
                reference,
                ReferenceResolutionStatus.Ambiguous,
                candidates,
                "REFERENCE_AMBIGUOUS",
                "More than one authoritative face satisfies the semantic selector."))
        };
    }

    public async Task<KernelEvaluationResponse> EvaluateAsync(
        KernelEvaluationRequest request,
        CancellationToken cancellationToken = default)
    {
        ArgumentNullException.ThrowIfNull(request);
        request.Validate();

        return request.Feature switch
        {
            BoxFeatureSpecification box =>
                await EvaluateBoxAsync(request, box, cancellationToken),
            _ => UnsupportedFeature(request.Feature)
        };
    }

    private async Task<KernelEvaluationResponse> EvaluateBoxAsync(
        KernelEvaluationRequest request,
        BoxFeatureSpecification specification,
        CancellationToken cancellationToken)
    {
        var min = new KernelVector3(
            specification.Frame.OriginX,
            specification.Frame.OriginY,
            specification.Frame.OriginZ);

        var max = new KernelVector3(
            specification.Frame.OriginX + specification.Width,
            specification.Frame.OriginY + specification.Depth,
            specification.Frame.OriginZ + specification.Height);

        AxisAlignedBoxSolidKernelResult kernelResult;
        try
        {
            kernelResult = await _boxGeometry.BuildAxisAlignedBoxSolidAsync(
                new AxisAlignedBoxSolidRequest(
                    request.EvaluationId.Value,
                    min,
                    max,
                    new KernelTolerance(
                        SelectorAbsoluteTolerance,
                        SelectorRelativeTolerance),
                    new ContractVersion("1.0")),
                cancellationToken);
        }
        catch (OperationCanceledException)
        {
            throw;
        }
        catch (Exception exception)
        {
            return new KernelEvaluationResponse(
                CadEvaluationStatus.KernelFailure,
                null,
                new[]
                {
                    new CadDiagnostic(
                        "KERNEL_BOX_TRANSPORT",
                        CadEvaluationStatus.KernelFailure,
                        exception.Message)
                });
        }

        var status = kernelResult.Status switch
        {
            GeometryKernelStatus.Succeeded => CadEvaluationStatus.Succeeded,
            GeometryKernelStatus.Failed => CadEvaluationStatus.KernelFailure,
            GeometryKernelStatus.Unsupported => CadEvaluationStatus.Unsupported,
            GeometryKernelStatus.Ambiguous => CadEvaluationStatus.AmbiguousEvaluation,
            GeometryKernelStatus.Indeterminate => CadEvaluationStatus.Indeterminate,
            _ => throw new InvalidOperationException(
                $"Unknown geometry-kernel status '{kernelResult.Status}'.")
        };

        if (status != CadEvaluationStatus.Succeeded)
        {
            return new KernelEvaluationResponse(
                status,
                null,
                kernelResult.Diagnostics
                    .Select(message => new CadDiagnostic(
                        "KERNEL_BOX_RESULT",
                        status,
                        message))
                    .DefaultIfEmpty(
                        new CadDiagnostic(
                            "KERNEL_BOX_RESULT",
                            status,
                            $"The box-solid kernel operation returned '{kernelResult.Status}'."))
                    .ToArray());
        }

        if (kernelResult.ResultId is null ||
            kernelResult.EvidenceHash is null ||
            kernelResult.Volume is null ||
            kernelResult.SurfaceArea is null ||
            kernelResult.Centroid is null)
        {
            return new KernelEvaluationResponse(
                CadEvaluationStatus.KernelFailure,
                null,
                new[]
                {
                    new CadDiagnostic(
                        "KERNEL_BOX_RESULT_INCOMPLETE",
                        CadEvaluationStatus.KernelFailure,
                        "Successful box-solid transport result is missing authoritative identity or metrics.")
                });
        }

        if (!TryBuildBoxTopology(
                specification,
                kernelResult,
                out var topology,
                out var topologyDiagnostic))
        {
            return new KernelEvaluationResponse(
                CadEvaluationStatus.KernelFailure,
                null,
                new[]
                {
                    new CadDiagnostic(
                        "KERNEL_BOX_TOPOLOGY_INVALID",
                        CadEvaluationStatus.KernelFailure,
                        topologyDiagnostic)
                });
        }

        var authoritative = new AuthoritativeCadResult(
            kernelResult.ResultId.ValueAsCadResultId(),
            CadContractVersions.KernelEvaluation,
            new CadBoundingBox3(
                min.X, min.Y, min.Z,
                max.X, max.Y, max.Z),
            kernelResult.Volume.Value,
            kernelResult.SurfaceArea.Value,
            new TopologySnapshot(
                CadContractVersions.KernelEvaluation,
                topology));

        authoritative.Validate();

        return new KernelEvaluationResponse(
            CadEvaluationStatus.Succeeded,
            authoritative,
            Array.Empty<CadDiagnostic>());
    }

    private static KernelEvaluationResponse UnsupportedFeature(CadFeatureSpecification feature) =>
        new(
            CadEvaluationStatus.Unsupported,
            null,
            new[]
            {
                new CadDiagnostic(
                    "KERNEL_FEATURE_UNSUPPORTED",
                    CadEvaluationStatus.Unsupported,
                    $"The production Rust CAD adapter currently certifies only '{nameof(BoxFeatureSpecification)}'. " +
                    $"Feature '{feature.GetType().Name}' requires an explicit mathematical transport contract.")
            });

    private static bool TryBuildBoxTopology(
        BoxFeatureSpecification specification,
        AxisAlignedBoxSolidKernelResult kernelResult,
        out IReadOnlyList<TopologyEntityResult> topology,
        out string diagnostic)
    {
        var expected = new Dictionary<string, (CadVector3 Normal, CadVector3 Point, double Measure)>(StringComparer.Ordinal)
        {
            ["f_bottom"] = (
                new CadVector3(0d, 0d, -1d),
                new CadVector3(
                    specification.Frame.OriginX + specification.Width / 2d,
                    specification.Frame.OriginY + specification.Depth / 2d,
                    specification.Frame.OriginZ),
                specification.Width * specification.Depth),
            ["f_top"] = (
                new CadVector3(0d, 0d, 1d),
                new CadVector3(
                    specification.Frame.OriginX + specification.Width / 2d,
                    specification.Frame.OriginY + specification.Depth / 2d,
                    specification.Frame.OriginZ + specification.Height),
                specification.Width * specification.Depth),
            ["f_back"] = (
                new CadVector3(0d, -1d, 0d),
                new CadVector3(
                    specification.Frame.OriginX + specification.Width / 2d,
                    specification.Frame.OriginY,
                    specification.Frame.OriginZ + specification.Height / 2d),
                specification.Width * specification.Height),
            ["f_front"] = (
                new CadVector3(0d, 1d, 0d),
                new CadVector3(
                    specification.Frame.OriginX + specification.Width / 2d,
                    specification.Frame.OriginY + specification.Depth,
                    specification.Frame.OriginZ + specification.Height / 2d),
                specification.Width * specification.Height),
            ["f_left"] = (
                new CadVector3(-1d, 0d, 0d),
                new CadVector3(
                    specification.Frame.OriginX,
                    specification.Frame.OriginY + specification.Depth / 2d,
                    specification.Frame.OriginZ + specification.Height / 2d),
                specification.Depth * specification.Height),
            ["f_right"] = (
                new CadVector3(1d, 0d, 0d),
                new CadVector3(
                    specification.Frame.OriginX + specification.Width,
                    specification.Frame.OriginY + specification.Depth / 2d,
                    specification.Frame.OriginZ + specification.Height / 2d),
                specification.Depth * specification.Height)
        };

        var actualKeys = kernelResult.Topology
            .Where(x => string.Equals(x.Kind, "Face", StringComparison.Ordinal))
            .Select(x => x.Key)
            .ToHashSet(StringComparer.Ordinal);

        if (kernelResult.Topology.Count != expected.Count ||
            actualKeys.Count != expected.Count ||
            !actualKeys.SetEquals(expected.Keys))
        {
            topology = Array.Empty<TopologyEntityResult>();
            diagnostic =
                $"Kernel returned an uncertified box topology set. Expected exactly {{{string.Join(", ", expected.Keys.OrderBy(x => x, StringComparer.Ordinal))}}}.";
            return false;
        }

        topology = kernelResult.Topology
            .OrderBy(x => x.Key, StringComparer.Ordinal)
            .Select(entry =>
            {
                var evidence = expected[entry.Key];
                return new TopologyEntityResult(
                    new TopologyEntityId(entry.Key),
                    TopologyEntityKind.Face,
                    evidence.Normal,
                    evidence.Point,
                    evidence.Measure,
                    4,
                    new TopologyProvenance(
                        specification.Id,
                        new CadResultId(kernelResult.ResultId!.Value.Value),
                        "Created",
                        Array.Empty<TopologyEntityId>()));
            })
            .ToArray();

        diagnostic = string.Empty;
        return true;
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

        var candidateNormal = new CadVector3(nx, ny, nz);
        try
        {
            normal = candidateNormal.Normalize("Reference normal");
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
        out double value)
    {
        return parameters.TryGetValue(key, out var raw) &&
               double.TryParse(
                   raw,
                   NumberStyles.Float | NumberStyles.AllowLeadingSign,
                   CultureInfo.InvariantCulture,
                   out value) &&
               double.IsFinite(value);
    }

    private static bool NearlyEqualVector(CadVector3 left, CadVector3 right)
    {
        return NearlyEqual(left.X, right.X) &&
               NearlyEqual(left.Y, right.Y) &&
               NearlyEqual(left.Z, right.Z);
    }

    private static bool NearlyEqual(double left, double right)
    {
        var scale = Math.Max(1d, Math.Max(Math.Abs(left), Math.Abs(right)));
        return Math.Abs(left - right) <=
               SelectorAbsoluteTolerance + (SelectorRelativeTolerance * scale);
    }

    private static CadResultId ValueAsCadResultId(this ContractResultId resultId) =>
        new(resultId.Value);
}

internal static class CadEvaluationStatusExtensions
{
    public static bool SucceededEquivalent(this CadEvaluationStatus status) =>
        status == CadEvaluationStatus.Succeeded;
}
