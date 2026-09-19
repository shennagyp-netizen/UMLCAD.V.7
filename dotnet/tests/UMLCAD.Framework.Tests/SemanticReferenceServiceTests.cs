using Microsoft.Extensions.DependencyInjection;
using UMLCAD.Framework;
using UMLCAD.Framework.Semantics;
using Xunit;

namespace UMLCAD.Framework.Tests;

public sealed class SemanticReferenceServiceTests
{
    [Fact]
    public void Resolves_unique_part_geometry_by_semantic_identity()
    {
        var builder = CadApplication.CreateBuilder();
        builder.AddPart("part", "part", part =>
            part.Geometry("edge", "line", new Dictionary<string, string>
            {
                ["start"] = "0,0",
                ["end"] = "10,0"
            }));

        using var app = builder.Build();
        var service = app.GetRequiredService<ISemanticReferenceService>();

        var result = service.Resolve(app.Semantic,
            new SemanticReference("part", "edge", "geometry"));

        Assert.Equal(SemanticReferenceStatus.Resolved, result.Status);
        Assert.True(result.IsResolved);
        Assert.Equal("edge", result.Target!.TargetId);
        Assert.Equal("geometry", result.Target.TargetKind);
    }

    [Fact]
    public void Reports_stale_result_identity_as_indeterminate()
    {
        var builder = CadApplication.CreateBuilder();
        builder.AddPart("part", "solid", part =>
        {
            part.TopologyBinding("topology-front", "face", "front-face", "result-r1", "brep-face-6");
            part.Publication("front", "face", "front-face", "result-r1", "topology-front");
        });

        using var app = builder.Build();
        var service = app.GetRequiredService<ISemanticReferenceService>();

        var result = service.Resolve(app.Semantic,
            new SemanticReference("part", "front-face", "face", "stale-build"));

        Assert.Equal(SemanticReferenceStatus.Indeterminate, result.Status);
        Assert.Equal("REFERENCE_STALE_RESULT", result.DiagnosticCode);
        Assert.False(result.IsResolved);
    }

    [Fact]
    public void Matching_authoritative_result_identity_resolves_published_face()
    {
        var builder = CadApplication.CreateBuilder();
        builder.AddPart("part", "solid", part =>
        {
            part.TopologyBinding("topology-front", "face", "front-face", "result-r1", "brep-face-6");
            part.Publication("front", "face", "front-face", "result-r1", "topology-front");
        });

        using var app = builder.Build();
        var result = app.GetRequiredService<ISemanticReferenceService>()
            .Resolve(app.Semantic, new SemanticReference("part", "front-face", "face", "result-r1"));

        Assert.Equal(SemanticReferenceStatus.Resolved, result.Status);
        Assert.Equal("result-r1", result.Target!.ResultIdentity);
    }

    [Fact]
    public void Reports_duplicate_geometry_identity_as_ambiguous_instead_of_selecting_one()
    {
        var builder = CadApplication.CreateBuilder();
        builder.AddPart("part", "part", part =>
        {
            part.Geometry("edge", "line", new Dictionary<string, string>());
            part.Geometry("edge", "circle", new Dictionary<string, string>());
        });

        using var app = builder.Build();
        var service = app.GetRequiredService<ISemanticReferenceService>();

        var result = service.Resolve(app.Semantic,
            new SemanticReference("part", "edge", "geometry"));

        Assert.Equal(SemanticReferenceStatus.Ambiguous, result.Status);
        Assert.Equal("REFERENCE_AMBIGUOUS", result.DiagnosticCode);
        Assert.Equal(2, result.Candidates.Count);
        Assert.False(result.IsResolved);
    }

    [Fact]
    public void Resolves_published_face_only_when_publication_and_topology_provenance_match()
    {
        var builder = CadApplication.CreateBuilder();
        builder.AddPart("part", "solid", part =>
        {
            part.Geometry("body", "solid", new Dictionary<string, string>());
            part.TopologyBinding("topology-front", "face", "front-face", "result-r1", "brep-face-6");
            part.Publication("front", "face", "front-face", "result-r1", "topology-front");
        });

        using var app = builder.Build();
        var service = app.GetRequiredService<ISemanticReferenceService>();

        var result = service.Resolve(app.Semantic,
            new SemanticReference("part", "front-face", "face", "result-r1"));

        Assert.Equal(SemanticReferenceStatus.Resolved, result.Status);
        Assert.Equal("REFERENCE_RESOLVED", result.DiagnosticCode);
        Assert.Equal("result-r1", result.Target!.ResultIdentity);
        Assert.Equal("topology-front", result.Target.ProvenanceId);
    }

    [Fact]
    public void Same_face_target_across_two_results_is_ambiguous_without_result_selector()
    {
        var builder = CadApplication.CreateBuilder();
        builder.AddPart("part", "solid", part =>
        {
            part.TopologyBinding("binding-a", "face", "front-face", "result-a", "brep-face-a");
            part.TopologyBinding("binding-b", "face", "front-face", "result-b", "brep-face-b");
            part.Publication("front-a", "face", "front-face", "result-a", "binding-a");
            part.Publication("front-b", "face", "front-face", "result-b", "binding-b");
        });

        using var app = builder.Build();
        var service = app.GetRequiredService<ISemanticReferenceService>();

        var ambiguous = service.Resolve(
            app.Semantic,
            new SemanticReference("part", "front-face", "face"));

        var selected = service.Resolve(
            app.Semantic,
            new SemanticReference("part", "front-face", "face", "result-b"));

        Assert.Equal(SemanticReferenceStatus.Ambiguous, ambiguous.Status);
        Assert.Equal(2, ambiguous.Candidates.Count);
        Assert.Equal(SemanticReferenceStatus.Resolved, selected.Status);
        Assert.Equal("result-b", selected.Target!.ResultIdentity);
        Assert.Equal("binding-b", selected.Target.ProvenanceId);
    }

    [Fact]
    public void Face_reference_with_missing_publication_is_indeterminate_not_geometry()
    {
        var builder = CadApplication.CreateBuilder();
        builder.AddPart("part", "solid", part =>
        {
            part.Geometry("body", "solid", new Dictionary<string, string>());
            part.TopologyBinding("topology-front", "face", "front-face", "result-r1", "brep-face-6");
        });

        using var app = builder.Build();
        var result = app.GetRequiredService<ISemanticReferenceService>()
            .Resolve(app.Semantic, new SemanticReference("part", "front-face", "face", "result-r1"));

        Assert.Equal(SemanticReferenceStatus.Indeterminate, result.Status);
        Assert.Equal("REFERENCE_PUBLICATION_MISSING", result.DiagnosticCode);
    }

    [Fact]
    public void Face_reference_with_mismatched_topology_provenance_is_indeterminate()
    {
        var builder = CadApplication.CreateBuilder();
        builder.AddPart("part", "solid", part =>
        {
            part.TopologyBinding("topology-front", "face", "front-face", "result-r1", "brep-face-6");
            part.Publication("front", "face", "front-face", "result-r2", "topology-front");
        });

        using var app = builder.Build();
        var result = app.GetRequiredService<ISemanticReferenceService>()
            .Resolve(app.Semantic, new SemanticReference("part", "front-face", "face", "result-r2"));

        Assert.Equal(SemanticReferenceStatus.Indeterminate, result.Status);
        Assert.Equal("REFERENCE_PROVENANCE_MISMATCH", result.DiagnosticCode);
    }

    [Fact]
    public void Reports_unsupported_face_reference_without_inventing_topology()
    {
        var builder = CadApplication.CreateBuilder();
        builder.AddPart("part", "part", part => part.Geometry("body", "solid", new Dictionary<string, string>()));

        using var app = builder.Build();
        var service = app.GetRequiredService<ISemanticReferenceService>();

        var result = service.Resolve(app.Semantic,
            new SemanticReference("part", "1", "face"));

        Assert.Equal(SemanticReferenceStatus.Unsupported, result.Status);
        Assert.Equal("REFERENCE_UNSUPPORTED_TARGET_KIND", result.DiagnosticCode);
        Assert.False(result.IsResolved);
    }

    [Fact]
    public void Reports_missing_producer_fail_closed()
    {
        var builder = CadApplication.CreateBuilder();
        builder.AddPart("part", "part");
        using var app = builder.Build();
        var service = app.GetRequiredService<ISemanticReferenceService>();

        var result = service.Resolve(app.Semantic,
            new SemanticReference("missing-producer", "edge", "geometry"));

        Assert.Equal(SemanticReferenceStatus.Missing, result.Status);
        Assert.Equal("REFERENCE_PRODUCER_MISSING", result.DiagnosticCode);
    }

    [Fact]
    public void Reports_duplicate_sheet_identity_as_ambiguous()
    {
        var builder = CadApplication.CreateBuilder();
        builder.AddDrawing("drawing", "Drawing", drawing =>
            drawing.Sheet("sheet", "A4")
                   .Sheet("sheet", "A4 duplicate"));

        using var app = builder.Build();
        var service = app.GetRequiredService<ISemanticReferenceService>();

        var result = service.Resolve(app.Semantic,
            new SemanticReference("drawing", "sheet", "sheet"));

        Assert.Equal(SemanticReferenceStatus.Ambiguous, result.Status);
        Assert.Equal("REFERENCE_AMBIGUOUS", result.DiagnosticCode);
        Assert.Equal(2, result.Candidates.Count);
    }
}
