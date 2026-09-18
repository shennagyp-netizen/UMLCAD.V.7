using UMLCAD.Cad.Contracts;

namespace UMLCAD.Engineering.Tests;

public sealed class ExtrusionProfileSelectionTests
{
    [Fact]
    public void Extrusion_requires_an_explicit_profile_geometry_identity()
    {
        var support = new CadReference(
            new CadId("support-face"),
            ReferenceKind.Support,
            new CadId("base"),
            TopologySelector.PlanarFaceByNormalAndPoint(
                new CadVector3(0d, 0d, 1d),
                new CadVector3(50d, 40d, 10d)),
            new ReferenceContext(CadFrameKind.Part, "default"));

        var feature = new ExtrusionFeatureSpecification(
            new CadId("feature-a"),
            new CadId("sketch"),
            new CadId("circle-a"),
            support,
            new CadVector3(0d, 0d, 1d),
            10d,
            FeatureBooleanOperation.Add);

        Assert.Equal(new CadId("circle-a"), feature.ProfileGeometryId);
    }

    [Fact]
    public void Profile_geometry_identity_changes_extrusion_evaluation_identity()
    {
        var support = new CadReference(
            new CadId("support-face"),
            ReferenceKind.Support,
            new CadId("base"),
            TopologySelector.PlanarFaceByNormalAndPoint(
                new CadVector3(0d, 0d, 1d),
                new CadVector3(50d, 40d, 10d)),
            new ReferenceContext(CadFrameKind.Part, "default"));

        var first = new ExtrusionFeatureSpecification(
            new CadId("feature-a"),
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

    [Fact]
    public void Evaluation_graph_rejects_a_profile_geometry_not_owned_by_the_profile_sketch()
    {
        var support = new CadReference(
            new CadId("support-face"),
            ReferenceKind.Support,
            new CadId("base"),
            TopologySelector.PlanarFaceByNormalAndPoint(
                new CadVector3(0d, 0d, 1d),
                new CadVector3(50d, 40d, 10d)),
            new ReferenceContext(CadFrameKind.Part, "default"));

        var baseFeature = new BoxFeatureSpecification(
            new CadId("base"),
            new CadFrame(new CadId("part"), CadFrameKind.Part, 0d, 0d, 0d),
            100d,
            80d,
            10d);

        var sketch = new SketchFeatureSpecification(
            new CadId("sketch"),
            support,
            new CadFrame(new CadId("sketch-frame"), CadFrameKind.Sketch, 50d, 40d, 10d),
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

        var extrusion = new ExtrusionFeatureSpecification(
            new CadId("feature-a"),
            sketch.Id,
            new CadId("circle-missing"),
            support,
            new CadVector3(0d, 0d, 1d),
            10d,
            FeatureBooleanOperation.Add);

        Assert.Throws<InvalidOperationException>(() =>
            SpecificationGraph.Build(
                new CadDocumentSpecification(
                    new CadId("document"),
                    "A",
                    "default",
                    new CadFeatureSpecification[]
                    {
                        baseFeature,
                        sketch,
                        extrusion
                    })));
    }
}
