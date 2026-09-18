using UMLCAD.Cad.Contracts;
using UMLCAD.Cad.Engine;

namespace UMLCAD.Engineering.Tests;

public sealed class S1ProfileBindingTddTests
{
    [Fact]
    public void Extrusions_must_bind_to_distinct_sketch_geometry()
    {
        var support = SupportReference();

        var sketch = new SketchFeatureSpecification(
            new CadId("sketch"),
            support,
            new CadFrame(
                new CadId("sketch-frame"),
                CadFrameKind.Sketch,
                50d,
                40d,
                10d),
            new[]
            {
                new SketchCircle(new CadId("circle-a"), 20d, 20d, 5d),
                new SketchCircle(new CadId("circle-b"), 70d, 30d, 4d)
            },
            new[]
            {
                new SketchConstraintSpecification(
                    new CadId("fixed-a"),
                    SketchConstraintKind.Fixed,
                    new CadId("circle-a")),
                new SketchConstraintSpecification(
                    new CadId("fixed-b"),
                    SketchConstraintKind.Fixed,
                    new CadId("circle-b"))
            });

        var additive = new ExtrusionFeatureSpecification(
            new CadId("add"),
            sketch.Id,
            new CadId("circle-a"),
            support,
            new CadVector3(0d, 0d, 1d),
            10d,
            FeatureBooleanOperation.Add);

        var subtractive = new ExtrusionFeatureSpecification(
            new CadId("remove"),
            sketch.Id,
            new CadId("circle-b"),
            support,
            new CadVector3(0d, 0d, -1d),
            8d,
            FeatureBooleanOperation.Remove);

        Assert.Equal(new CadId("circle-a"), additive.ProfileGeometryId);
        Assert.Equal(new CadId("circle-b"), subtractive.ProfileGeometryId);
        Assert.NotEqual(additive.ProfileGeometryId, subtractive.ProfileGeometryId);
    }

    [Fact]
    public void Specification_graph_must_reject_profile_geometry_not_owned_by_the_sketch()
    {
        var support = SupportReference();

        var baseFeature = new BoxFeatureSpecification(
            new CadId("base"),
            new CadFrame(new CadId("part"), CadFrameKind.Part, 0d, 0d, 0d),
            100d,
            80d,
            10d);

        var sketch = new SketchFeatureSpecification(
            new CadId("sketch"),
            support,
            new CadFrame(
                new CadId("sketch-frame"),
                CadFrameKind.Sketch,
                50d,
                40d,
                10d),
            new[]
            {
                new SketchCircle(new CadId("circle-a"), 20d, 20d, 5d)
            },
            new[]
            {
                new SketchConstraintSpecification(
                    new CadId("fixed-a"),
                    SketchConstraintKind.Fixed,
                    new CadId("circle-a"))
            });

        var extrusion = new ExtrusionFeatureSpecification(
            new CadId("remove"),
            sketch.Id,
            new CadId("circle-missing"),
            support,
            new CadVector3(0d, 0d, -1d),
            8d,
            FeatureBooleanOperation.Remove);

        var document = new CadDocumentSpecification(
            new CadId("profile-binding-red"),
            "A",
            "mm",
            new CadFeatureSpecification[]
            {
                baseFeature,
                sketch,
                extrusion
            });

        var exception = Assert.Throws<InvalidOperationException>(
            () => SpecificationGraph.Build(document));

        Assert.Contains("circle-missing", exception.Message, StringComparison.Ordinal);
    }

    [Fact]
    public void Evaluation_identity_must_include_selected_profile_geometry()
    {
        var support = SupportReference();

        var first = new ExtrusionFeatureSpecification(
            new CadId("extrusion"),
            new CadId("sketch"),
            new CadId("circle-a"),
            support,
            new CadVector3(0d, 0d, 1d),
            10d,
            FeatureBooleanOperation.Add);

        var second = first with
        {
            ProfileGeometryId = new CadId("circle-b")
        };

        var firstIdentity = EvaluationIdentity.Compute(
            first,
            Array.Empty<ReferenceResolution>(),
            Array.Empty<CadFeatureEvaluationResult>(),
            "semantic-default",
            "A|mm");

        var secondIdentity = EvaluationIdentity.Compute(
            second,
            Array.Empty<ReferenceResolution>(),
            Array.Empty<CadFeatureEvaluationResult>(),
            "semantic-default",
            "A|mm");

        Assert.NotEqual(firstIdentity, secondIdentity);
    }

    private static CadReference SupportReference() =>
        new(
            new CadId("support-face"),
            ReferenceKind.Support,
            new CadId("base"),
            TopologySelector.PlanarFaceByNormalAndPoint(
                new CadVector3(0d, 0d, 1d),
                new CadVector3(50d, 40d, 10d)),
            new ReferenceContext(CadFrameKind.Part, "default"));
}
