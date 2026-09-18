using UMLCAD.Cad.Contracts;

namespace UMLCAD.Engineering.Tests;

public sealed class AuthoritativeCadReferenceResolverTests
{
    [Fact]
    public void ResolvesSemanticPlanarFaceByEvidenceNotArrayOrder()
    {
        var producer = new CadId("box");
        var result = Result(producer, reverse: true);

        var reference = Reference(producer);

        var resolution = new UMLCAD.Cad.Engine.AuthoritativeCadReferenceResolver()
            .Resolve(reference, result);

        Assert.Equal(ReferenceResolutionStatus.Resolved, resolution.Status);
        Assert.Equal("f_top", resolution.Candidates.Single().Value);
    }

    [Fact]
    public void ExpectedResultMismatchIsIndeterminate()
    {
        var producer = new CadId("box");
        var result = Result(producer);

        var reference = Reference(
            producer,
            new ReferenceContext(CadFrameKind.Part, "default", new CadResultId("different")));

        var resolution = new UMLCAD.Cad.Engine.AuthoritativeCadReferenceResolver()
            .Resolve(reference, result);

        Assert.Equal(ReferenceResolutionStatus.Indeterminate, resolution.Status);
        Assert.Equal("REFERENCE_RESULT_MISMATCH", resolution.DiagnosticCode);
    }

    [Fact]
    public void WrongProducerProvenanceFailsClosed()
    {
        var result = Result(new CadId("actual"));
        var reference = Reference(new CadId("requested"));

        var resolution = new UMLCAD.Cad.Engine.AuthoritativeCadReferenceResolver()
            .Resolve(reference, result);

        Assert.Equal(ReferenceResolutionStatus.Indeterminate, resolution.Status);
        Assert.Equal("REFERENCE_TARGET_MISMATCH", resolution.DiagnosticCode);
    }

    [Fact]
    public void UnsupportedSelectorIsNotGuessed()
    {
        var producer = new CadId("box");
        var result = Result(producer);

        var reference = new CadReference(
            new CadId("support"),
            ReferenceKind.Support,
            producer,
            new TopologySelector(
                TopologyEntityKind.Face,
                "viewer-face-index",
                new Dictionary<string, string> { ["index"] = "5" }),
            new ReferenceContext(CadFrameKind.Part, "default"));

        var resolution = new UMLCAD.Cad.Engine.AuthoritativeCadReferenceResolver()
            .Resolve(reference, result);

        Assert.Equal(ReferenceResolutionStatus.Unsupported, resolution.Status);
    }

    [Fact]
    public void AmbiguousEvidenceFailsClosed()
    {
        var producer = new CadId("box");
        var result = Result(producer, duplicateTop: true);

        var resolution = new UMLCAD.Cad.Engine.AuthoritativeCadReferenceResolver()
            .Resolve(Reference(producer), result);

        Assert.Equal(ReferenceResolutionStatus.Ambiguous, resolution.Status);
        Assert.Equal(
            new[] { "f_top", "f_top_duplicate" },
            resolution.Candidates.Select(x => x.Value).OrderBy(x => x, StringComparer.Ordinal).ToArray());
    }

    private static CadReference Reference(
        CadId producer,
        ReferenceContext? context = null) =>
        new(
            new CadId("support"),
            ReferenceKind.Support,
            producer,
            TopologySelector.PlanarFaceByNormalAndPoint(
                new CadVector3(0, 0, 1),
                new CadVector3(1, 1.5, 4)),
            context ?? new ReferenceContext(CadFrameKind.Part, "default"));

    private static AuthoritativeCadResult Result(
        CadId producer,
        bool reverse = false,
        bool duplicateTop = false)
    {
        var faces = new List<TopologyEntityResult>
        {
            Face("f_bottom", 0, 0, -1, 1, 1.5, 0),
            Face("f_top", 0, 0, 1, 1, 1.5, 4),
            Face("f_back", 0, -1, 0, 1, 0, 2),
            Face("f_front", 0, 1, 0, 1, 3, 2),
            Face("f_left", -1, 0, 0, 0, 1.5, 2),
            Face("f_right", 1, 0, 0, 2, 1.5, 2),
        };

        if (duplicateTop)
            faces.Add(Face("f_top_duplicate", 0, 0, 1, 1, 1.5, 4));
        if (reverse)
            faces.Reverse();

        return new AuthoritativeCadResult(
            new CadResultId("solid:box"),
            CadContractVersions.KernelEvaluation,
            new CadBoundingBox3(0, 0, 0, 2, 3, 4),
            24,
            52,
            new TopologySnapshot(
                CadContractVersions.KernelEvaluation,
                faces));
    }

    private static TopologyEntityResult Face(
        string id,
        double nx, double ny, double nz,
        double x, double y, double z) =>
        new(
            new TopologyEntityId(id),
            TopologyEntityKind.Face,
            new CadVector3(nx, ny, nz),
            new CadVector3(x, y, z),
            1,
            4,
            new TopologyProvenance(
                new CadId("box"),
                new CadResultId("solid:box"),
                "Created",
                Array.Empty<TopologyEntityId>()));
}
