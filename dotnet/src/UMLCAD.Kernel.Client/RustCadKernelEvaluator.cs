using UMLCAD.Cad.Contracts;

namespace UMLCAD.Kernel.Client;

/// <summary>
/// Production bridge from the system-CAD evaluation contract to the existing
/// certified geometry transport services. It owns transport/result adaptation,
/// not geometric mathematics or CAD semantic ownership.
/// </summary>
public sealed class RustCadKernelEvaluator : ICadKernelEvaluator
{
    private readonly IAuthoritativeGeometryService _boxGeometry;
    private readonly ISketchGeometryService? _sketchGeometry;

    public RustCadKernelEvaluator(IAuthoritativeGeometryService boxGeometry)
        : this(boxGeometry, null)
    {
    }

    public RustCadKernelEvaluator(
        IAuthoritativeGeometryService boxGeometry,
        ISketchGeometryService? sketchGeometry)
    {
        _boxGeometry = boxGeometry ?? throw new ArgumentNullException(nameof(boxGeometry));
        _sketchGeometry = sketchGeometry;
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
            SketchFeatureSpecification sketch =>
                await EvaluateSketchAsync(request, sketch, cancellationToken),
            _ => UnsupportedFeature(request.Feature)
        };
    }

    private async Task<KernelEvaluationResponse> EvaluateSketchAsync(
        KernelEvaluationRequest request,
        SketchFeatureSpecification specification,
        CancellationToken cancellationToken)
    {
        if (_sketchGeometry is null)
        {
            return UnsupportedFeature(specification);
        }

        SketchKernelResult kernelResult;
        try
        {
            kernelResult = await _sketchGeometry.SolveAsync(
                new SketchSolveRequest(
                    request.EvaluationId.Value,
                    specification.Id,
                    specification.Frame,
                    specification.Circles,
                    specification.Constraints,
                    request.Tolerance,
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
                        "KERNEL_SKETCH_TRANSPORT",
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
                $"Unknown sketch-kernel status '{kernelResult.Status}'.")
        };

        if (status != CadEvaluationStatus.Succeeded)
        {
            return new KernelEvaluationResponse(
                status,
                null,
                kernelResult.Diagnostics
                    .Select(message => new CadDiagnostic(
                        "KERNEL_SKETCH_RESULT",
                        status,
                        message))
                    .DefaultIfEmpty(
                        new CadDiagnostic(
                            "KERNEL_SKETCH_RESULT",
                            status,
                            $"The sketch solver returned '{kernelResult.Status}'."))
                    .ToArray());
        }

        if (kernelResult.ResultId is null ||
            kernelResult.EvidenceHash is null ||
            kernelResult.Converged != true ||
            kernelResult.Iterations is null ||
            kernelResult.FinalResidualNorm is null ||
            kernelResult.FinalScaledResidualNorm is null ||
            kernelResult.FinalStepNorm is null ||
            kernelResult.DegreesOfFreedom is null ||
            kernelResult.VariableCount is null ||
            kernelResult.EquationCount is null)
        {
            return new KernelEvaluationResponse(
                CadEvaluationStatus.KernelFailure,
                null,
                new[]
                {
                    new CadDiagnostic(
                        "KERNEL_SKETCH_RESULT_INCOMPLETE",
                        CadEvaluationStatus.KernelFailure,
                        "Successful sketch transport result is missing authoritative solver evidence.")
                });
        }

        if (kernelResult.Circles.Count != specification.Circles.Count)
        {
            return new KernelEvaluationResponse(
                CadEvaluationStatus.KernelFailure,
                null,
                new[]
                {
                    new CadDiagnostic(
                        "KERNEL_SKETCH_RESULT_GEOMETRY_MISMATCH",
                        CadEvaluationStatus.KernelFailure,
                        "Sketch solver returned a circle set different from the requested semantic sketch.")
                });
        }

        var expectedIds = specification.Circles
            .Select(x => x.Id.Value)
            .OrderBy(x => x, StringComparer.Ordinal)
            .ToArray();
        var actualIds = kernelResult.Circles
            .Select(x => x.Id)
            .OrderBy(x => x, StringComparer.Ordinal)
            .ToArray();

        if (!expectedIds.SequenceEqual(actualIds, StringComparer.Ordinal))
        {
            return new KernelEvaluationResponse(
                CadEvaluationStatus.KernelFailure,
                null,
                new[]
                {
                    new CadDiagnostic(
                        "KERNEL_SKETCH_RESULT_IDENTITY_MISMATCH",
                        CadEvaluationStatus.KernelFailure,
                        "Sketch solver returned geometry identities different from the semantic sketch.")
                });
        }

        var result = new CadSketchEvaluationResult(
            specification.Id,
            kernelResult.ResultId.ValueAsCadResultId(),
            CadContractVersions.KernelEvaluation,
            specification.Frame,
            kernelResult.Circles,
            true,
            kernelResult.Iterations.Value,
            kernelResult.FinalResidualNorm.Value,
            kernelResult.FinalScaledResidualNorm.Value,
            kernelResult.FinalStepNorm.Value,
            kernelResult.DegreesOfFreedom.Value,
            kernelResult.VariableCount.Value,
            kernelResult.EquationCount.Value,
            kernelResult.EvidenceHash);

        result.Validate();

        return new KernelEvaluationResponse(
            CadEvaluationStatus.Succeeded,
            null,
            Array.Empty<CadDiagnostic>())
        {
            SketchResult = result
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
                    request.Tolerance,
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

}
