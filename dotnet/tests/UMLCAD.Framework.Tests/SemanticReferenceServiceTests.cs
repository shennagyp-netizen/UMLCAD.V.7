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
            new SemanticReference("part", "edge", "geometry", app.Semantic.BuildIdentity));

        Assert.Equal(SemanticReferenceStatus.Resolved, result.Status);
        Assert.True(result.IsResolved);
        Assert.Equal("edge", result.Target!.TargetId);
        Assert.Equal("geometry", result.Target.TargetKind);
    }

    [Fact]
    public void Reports_stale_result_identity_as_indeterminate()
    {
        var builder = CadApplication.CreateBuilder();
        builder.AddPart("part", "part", part => part.Geometry("edge", "line", new Dictionary<string, string>()));

        using var app = builder.Build();
        var service = app.GetRequiredService<ISemanticReferenceService>();

        var result = service.Resolve(app.Semantic,
            new SemanticReference("part", "edge", "geometry", "stale-build"));

        Assert.Equal(SemanticReferenceStatus.Indeterminate, result.Status);
        Assert.Equal("REFERENCE_STALE_RESULT", result.DiagnosticCode);
        Assert.False(result.IsResolved);
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
