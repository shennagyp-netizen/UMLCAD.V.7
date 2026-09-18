using UMLCAD.Cad.Contracts;
using UMLCAD.Cad.Engine;

namespace UMLCAD.Engineering.Tests;

public sealed class CadEvaluationVerticalSliceTests
{
    [Fact]
    public async Task Specification_evaluation_result_representation_slice_is_deterministic()
    {
        var kernel = new ContractTestKernel();
        var engine = new CadEvaluationEngine(kernel);
        var result = await engine.RecomputeAsync(CreateVerticalSliceDocument());

        Assert.True(result.Succeeded, FailureText(result));
        Assert.Equal(CadContractVersions.Evaluation, result.ContractVersion);
        Assert.Equal(RecomputeMode.Full, result.Mode);
        Assert.Equal(new[] { "base", "sketch", "feature-a", "feature-b" },
            result.EvaluationPlan.Select(x => x.Value));

        var sketch = Assert.Single(result.Features, x => x.FeatureId.Value == "sketch");
        Assert.True(sketch.Succeeded);
        Assert.True(Assert.Single(sketch.References).IsResolved);

        var extrusionA = Assert.Single(result.Features, x => x.FeatureId.Value == "feature-a");
        var extrusionB = Assert.Single(result.Features, x => x.FeatureId.Value == "feature-b");
        Assert.True(extrusionA.Succeeded);
        Assert.True(extrusionB.Succeeded);

        var authoritative = result.FinalAuthoritativeResult;
        Assert.NotNull(authoritative);
        Assert.Equal(CadContractVersions.KernelEvaluation, authoritative!.KernelContractVersion);
        Assert.Equal(6, authoritative.Topology.Faces.Count);
        Assert.NotNull(result.Representation);
        Assert.Equal(authoritative.ResultId, result.Representation!.SourceResultId);
    }

    [Fact]
    public async Task Face_reference_is_semantic_and_does_not_depend_on_face_array_position()
    {
        var kernel = new ContractTestKernel { ReverseFaceOrder = true };
        var result = await new CadEvaluationEngine(kernel).RecomputeAsync(CreateVerticalSliceDocument());

        Assert.True(result.Succeeded, FailureText(result));
        var resolution = Assert.Single(result.Features.Single(x => x.FeatureId.Value == "sketch").References);
        Assert.Equal(ReferenceResolutionStatus.Resolved, resolution.Status);
        Assert.Equal("face:+Z", resolution.Candidates.Single().Value);
    }

    [Fact]
    public async Task Ambiguous_support_reference_fails_closed_before_feature_evaluation()
    {
        var kernel = new ContractTestKernel { DuplicateTopFaces = true };
        var result = await new CadEvaluationEngine(kernel).RecomputeAsync(CreateVerticalSliceDocument());

        Assert.False(result.Succeeded);
        Assert.Equal(CadEvaluationStatus.AmbiguousReference, result.Failure!.Status);
        Assert.Equal("REFERENCE_AMBIGUOUS", result.Failure.Code);
        Assert.Equal(2, result.Features.Count);
        Assert.DoesNotContain("sketch", kernel.EvaluatedFeatureIds);
    }

    [Fact]
    public async Task Full_and_incremental_recompute_produce_the_same_semantic_final_result()
    {
        var kernel = new ContractTestKernel();
        var engine = new CadEvaluationEngine(kernel);

        var initial = await engine.RecomputeAsync(CreateVerticalSliceDocument());
        Assert.True(initial.Succeeded, FailureText(initial));

        var changed = await engine.RecomputeAsync(
            CreateVerticalSliceDocument(12d),
            new CadChangeSet(new HashSet<CadId> { new("feature-b") }));

        Assert.True(changed.Succeeded, FailureText(changed));
        Assert.Equal(RecomputeMode.Incremental, changed.Mode);
        Assert.Equal(new[] { "feature-b" },
            changed.InvalidatedFeatures.OrderBy(x => x.Value, StringComparer.Ordinal).Select(x => x.Value));

        var freshKernel = new ContractTestKernel();
        var fresh = await new CadEvaluationEngine(freshKernel).RecomputeAsync(CreateVerticalSliceDocument(12d));

        Assert.True(fresh.Succeeded, FailureText(fresh));
        Assert.Equal(changed.FinalAuthoritativeResult, fresh.FinalAuthoritativeResult);
        Assert.Equal(1, kernel.EvaluatedFeatureIds.Count(x => x == "base"));
        Assert.Equal(1, kernel.EvaluatedFeatureIds.Count(x => x == "sketch"));
        Assert.Equal(1, kernel.EvaluatedFeatureIds.Count(x => x == "feature-a"));
        Assert.Equal(2, kernel.EvaluatedFeatureIds.Count(x => x == "feature-b"));
    }

    [Fact]
    public void Evaluation_identity_changes_when_direction_changes()
    {
        var support = SupportReference();
        var sketch = new SketchFeatureSpecification(
            new CadId("sketch"), support,
            new CadFrame(new CadId("sketch-frame"), CadFrameKind.Sketch, 0, 0, 10),
            new[]
            {
                new SketchCircle(new CadId("circle-a"), 20, 20, 5),
                new SketchCircle(new CadId("circle-b"), 70, 30, 4)
            },
            new[]
            {
                new SketchConstraintSpecification(new CadId("fixed-a"), SketchConstraintKind.Fixed, new CadId("circle-a"))
            });

        var positive = new ExtrusionFeatureSpecification(
            new CadId("feature-a"), sketch.Id, support,
            new CadVector3(0, 0, 1), 10, FeatureBooleanOperation.Add);
        var negative = positive with { Direction = new CadVector3(0, 0, -1) };

        var first = EvaluationIdentity.Compute(positive, Array.Empty<ReferenceResolution>(),
            Array.Empty<CadFeatureEvaluationResult>(), "semantic-default", "A|mm");
        var second = EvaluationIdentity.Compute(negative, Array.Empty<ReferenceResolution>(),
            Array.Empty<CadFeatureEvaluationResult>(), "semantic-default", "A|mm");

        Assert.NotEqual(first, second);
    }

    [Fact]
    public void Specification_cycles_are_rejected_before_kernel_calls()
    {
        var support = SupportReference();
        var baseFeature = new BoxFeatureSpecification(
            new CadId("base"), new CadFrame(new CadId("part"), CadFrameKind.Part, 0, 0, 0), 10, 10, 10);

        var a = new ExtrusionFeatureSpecification(
            new CadId("a"), new CadId("b"), support, new CadVector3(0, 0, 1), 5, FeatureBooleanOperation.Add);
        var b = new ExtrusionFeatureSpecification(
            new CadId("b"), new CadId("a"), support, new CadVector3(0, 0, 1), 5, FeatureBooleanOperation.Add);

        var document = new CadDocumentSpecification(
            new CadId("cycle"), "A", "mm", new CadFeatureSpecification[] { baseFeature, a, b });

        Assert.Throws<InvalidOperationException>(() => EvaluationPlanner.Plan(SpecificationGraph.Build(document)));
    }

    private static CadDocumentSpecification CreateVerticalSliceDocument(double featureBDistance = 8d)
    {
        var part = new CadFrame(new CadId("part"), CadFrameKind.Part, 0, 0, 0);
        var sketchFrame = new CadFrame(new CadId("sketch-frame"), CadFrameKind.Sketch, 0, 0, 10);
        var support = SupportReference();

        var baseFeature = new BoxFeatureSpecification(new CadId("base"), part, 100, 80, 10);
        var sketch = new SketchFeatureSpecification(
            new CadId("sketch"), support, sketchFrame,
            new[]
            {
                new SketchCircle(new CadId("circle-a"), 20, 20, 5),
                new SketchCircle(new CadId("circle-b"), 70, 30, 4)
            },
            new[]
            {
                new SketchConstraintSpecification(new CadId("circle-a-fixed"), SketchConstraintKind.Fixed, new CadId("circle-a")),
                new SketchConstraintSpecification(new CadId("circle-b-fixed"), SketchConstraintKind.Fixed, new CadId("circle-b"))
            });

        var featureA = new ExtrusionFeatureSpecification(
            new CadId("feature-a"), sketch.Id, support, new CadVector3(0, 0, 1), 10, FeatureBooleanOperation.Add);
        var featureB = new ExtrusionFeatureSpecification(
            new CadId("feature-b"), sketch.Id, support, new CadVector3(0, 0, -1), featureBDistance, FeatureBooleanOperation.Remove);

        return new CadDocumentSpecification(
            new CadId("vertical-slice"), "A", "mm",
            new CadFeatureSpecification[] { baseFeature, sketch, featureA, featureB });
    }

    private static CadReference SupportReference() =>
        new(
            new CadId("support-face"), ReferenceKind.Support, new CadId("base"),
            TopologySelector.PlanarFaceByNormalAndPoint(new CadVector3(0, 0, 1), new CadVector3(50, 40, 10)),
            new ReferenceContext(CadFrameKind.Part, "default"));

    private static string FailureText(CadEvaluationResult result) =>
        result.Failure is null
            ? string.Join("; ", result.Features.SelectMany(x => x.Diagnostics).Select(x => x.Code))
            : $"{result.Failure.Code}: {result.Failure.Message}";

    private sealed class ContractTestKernel : ICadKernelEvaluator
    {
        public bool ReverseFaceOrder { get; init; }
        public bool DuplicateTopFaces { get; init; }
        public List<string> EvaluatedFeatureIds { get; } = new();

        public Task<KernelEvaluationResponse> EvaluateAsync(
            KernelEvaluationRequest request,
            CancellationToken cancellationToken = default)
        {
            cancellationToken.ThrowIfCancellationRequested();
            EvaluatedFeatureIds.Add(request.Feature.Id.Value);

            var result = request.Feature switch
            {
                BoxFeatureSpecification box => CreateResult(
                    box.Id,
                    80_000d,
                    ReverseFaceOrder,
                    DuplicateTopFaces),
                SketchFeatureSpecification => null,
                ExtrusionFeatureSpecification extrusion => CreateResult(
                    extrusion.Id,
                    Math.Max(
                        1d,
                        (request.UpstreamResult?.Volume ?? 80_000d) +
                        (extrusion.Operation == FeatureBooleanOperation.Add ? extrusion.Distance : -extrusion.Distance))),
                _ => throw new NotSupportedException()
            };

            return Task.FromResult(new KernelEvaluationResponse(
                CadEvaluationStatus.Succeeded, result, Array.Empty<CadDiagnostic>()));
        }

        private static AuthoritativeCadResult CreateResult(
            CadId featureId,
            double volume,
            bool reverseFaceOrder = false,
            bool duplicateTopFaces = false)
        {
            var resultId = new CadResultId($"result:{featureId.Value}");
            var faces = new[]
            {
                Face("face:-X", featureId, resultId, -1, 0, 0, 0, 0, 0),
                Face("face:+X", featureId, resultId, 1, 0, 0, 100, 0, 0),
                Face("face:-Y", featureId, resultId, 0, -1, 0, 0, 0, 0),
                Face("face:+Y", featureId, resultId, 0, 1, 0, 0, 80, 0),
                Face("face:-Z", featureId, resultId, 0, 0, -1, 0, 0, 0),
                Face("face:+Z", featureId, resultId, 0, 0, 1, 50, 40, 10)
            };

            var faceList = faces.ToList();
            if (duplicateTopFaces)
                faceList.Add(Face(
                    "face:duplicate:+Z",
                    featureId,
                    resultId,
                    0, 0, 1,
                    50, 40, 10));
            if (reverseFaceOrder)
                faceList.Reverse();

            return new AuthoritativeCadResult(
                resultId,
                CadContractVersions.KernelEvaluation,
                new CadBoundingBox3(0, 0, 0, 100, 80, 10),
                volume,
                19_600d,
                new TopologySnapshot(CadContractVersions.KernelEvaluation, faceList));
        }

        private static TopologyEntityResult Face(
            string id,
            CadId producingFeatureId,
            CadResultId resultId,
            double nx, double ny, double nz,
            double x, double y, double z) =>
            new(
                new TopologyEntityId(id),
                TopologyEntityKind.Face,
                new CadVector3(nx, ny, nz),
                new CadVector3(x, y, z),
                1_000,
                4,
                new TopologyProvenance(
                    producingFeatureId,
                    resultId,
                    "OneToOne",
                    Array.Empty<TopologyEntityId>()));
    }
}
