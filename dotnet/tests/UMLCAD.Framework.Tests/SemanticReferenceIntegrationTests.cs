using Microsoft.Extensions.DependencyInjection;
using UMLCAD.Framework;
using UMLCAD.Framework.Semantics;
using Xunit;

namespace UMLCAD.Framework.Tests;

public sealed class SemanticReferenceIntegrationTests
{
    [Fact]
    public void Resolves_top_level_part_definition()
    {
        using var app = Build(builder => builder.AddPart("part", "mechanical"));
        var result = Resolve(app, app.Semantic.Id, "part", "part");

        AssertResolved(result, "part", "part", app.Semantic.Id);
    }

    [Fact]
    public void Resolves_top_level_assembly_definition()
    {
        using var app = Build(builder => builder.AddAssembly("assembly", "Assembly"));
        var result = Resolve(app, app.Semantic.Id, "assembly", "assembly");

        AssertResolved(result, "assembly", "assembly", app.Semantic.Id);
    }

    [Fact]
    public void Resolves_top_level_drawing_definition()
    {
        using var app = Build(builder => builder.AddDrawing("drawing", "Drawing"));
        var result = Resolve(app, app.Semantic.Id, "drawing", "drawing");

        AssertResolved(result, "drawing", "drawing", app.Semantic.Id);
    }

    [Fact]
    public void Resolves_part_geometry_constraint_and_component_targets()
    {
        using var app = Build(builder => builder.AddPart("part", "part", part =>
        {
            part.Geometry("edge", "line", new Dictionary<string, string>());
            part.Constraint("constraint", "horizontal", ["edge"], new Dictionary<string, string>());
            part.Component("component-1");
        }));

        AssertResolved(Resolve(app, "part", "edge", "geometry"), "edge", "geometry", "part");
        AssertResolved(Resolve(app, "part", "constraint", "constraint"), "constraint", "constraint", "part");
        AssertResolved(Resolve(app, "part", "component-1", "component"), "component-1", "component", "part");
    }

    [Fact]
    public void Resolves_assembly_occurrence_target_integration_boundary()
    {
        using var app = Build(builder =>
        {
            builder.AddPart("part", "part");
            builder.AddAssembly("assembly", "Assembly", assembly =>
                assembly.Part("occurrence-1", "part"));
        });

        var result = Resolve(app, "assembly", "occurrence-1", "occurrence");

        AssertResolved(result, "occurrence-1", "occurrence", "assembly");
    }

    [Fact]
    public void Resolves_drawing_sheet_target_integration_boundary()
    {
        using var app = Build(builder =>
            builder.AddDrawing("drawing", "Drawing", drawing =>
                drawing.Sheet("sheet-1", "A3")));

        var result = Resolve(app, "drawing", "sheet-1", "sheet");

        AssertResolved(result, "sheet-1", "sheet", "drawing");
    }

    [Fact]
    public void Missing_target_is_distinct_from_missing_producer()
    {
        using var app = Build(builder => builder.AddPart("part", "part"));

        var targetMissing = Resolve(app, "part", "missing", "geometry");
        var producerMissing = Resolve(app, "missing-producer", "missing", "geometry");

        Assert.Equal(SemanticReferenceStatus.Missing, targetMissing.Status);
        Assert.Equal("REFERENCE_TARGET_MISSING", targetMissing.DiagnosticCode);
        Assert.Equal(SemanticReferenceStatus.Missing, producerMissing.Status);
        Assert.Equal("REFERENCE_PRODUCER_MISSING", producerMissing.DiagnosticCode);
        Assert.NotEqual(targetMissing.DiagnosticCode, producerMissing.DiagnosticCode);
    }

    [Fact]
    public void Cross_scope_reference_does_not_escape_its_declared_producer()
    {
        using var app = Build(builder =>
        {
            builder.AddPart("part-a", "part", part =>
                part.Geometry("edge-a", "line", new Dictionary<string, string>()));
            builder.AddPart("part-b", "part", part =>
                part.Geometry("edge-b", "line", new Dictionary<string, string>()));
        });

        var result = Resolve(app, "part-a", "edge-b", "geometry");

        Assert.Equal(SemanticReferenceStatus.Missing, result.Status);
        Assert.Equal("REFERENCE_TARGET_MISSING", result.DiagnosticCode);
    }

    [Fact]
    public void Unsupported_target_kind_is_not_reinterpreted_as_geometry()
    {
        using var app = Build(builder => builder.AddPart("part", "part", part =>
            part.Geometry("edge", "line", new Dictionary<string, string>())));

        var result = Resolve(app, "part", "edge", "face");

        Assert.Equal(SemanticReferenceStatus.Unsupported, result.Status);
        Assert.Equal("REFERENCE_UNSUPPORTED_TARGET_KIND", result.DiagnosticCode);
        Assert.Empty(result.Candidates);
    }

    [Fact]
    public void Application_target_kind_is_checked_against_application_scope()
    {
        using var app = Build(builder => builder.AddPart("part", "part"));

        var result = Resolve(app, app.Semantic.Id, "part", "geometry");

        Assert.Equal(SemanticReferenceStatus.Unsupported, result.Status);
        Assert.Equal("REFERENCE_UNSUPPORTED_TARGET_KIND", result.DiagnosticCode);
    }

    [Fact]
    public void Resolution_is_case_sensitive_for_semantic_identity()
    {
        using var app = Build(builder => builder.AddPart("part", "part", part =>
            part.Geometry("Edge-A", "line", new Dictionary<string, string>())));

        var result = Resolve(app, "part", "edge-a", "geometry");

        Assert.Equal(SemanticReferenceStatus.Missing, result.Status);
        Assert.Equal("REFERENCE_TARGET_MISSING", result.DiagnosticCode);
    }

    [Fact]
    public void Null_application_and_reference_are_rejected_at_service_boundary()
    {
        using var app = Build(builder => builder.AddPart("part", "part"));
        var service = app.GetRequiredService<ISemanticReferenceService>();

        Assert.Throws<ArgumentNullException>(() =>
            service.Resolve(null!, new SemanticReference("part", "part", "part")));
        Assert.Throws<ArgumentNullException>(() =>
            service.Resolve(app.Semantic, null!));
    }

    [Theory]
    [InlineData("", "target", "geometry", "REFERENCE_INVALID")]
    [InlineData("producer", "", "geometry", "REFERENCE_INVALID")]
    [InlineData("producer", "target", "", "REFERENCE_INVALID")]
    [InlineData("producer", "target", " ", "REFERENCE_INVALID")]
    public void Invalid_reference_identity_fields_fail_closed(
        string producerId,
        string targetId,
        string targetKind,
        string diagnosticCode)
    {
        using var app = Build(builder => builder.AddPart("producer", "part"));
        var result = Resolve(app, producerId, targetId, targetKind);

        Assert.Equal(SemanticReferenceStatus.Indeterminate, result.Status);
        Assert.Equal(diagnosticCode, result.DiagnosticCode);
        Assert.False(result.IsResolved);
    }

    [Fact]
    public void Whitespace_only_expected_result_identity_is_invalid()
    {
        using var app = Build(builder => builder.AddPart("part", "part"));
        var result = Resolve(app, "part", "part", "part", " ");

        Assert.Equal(SemanticReferenceStatus.Indeterminate, result.Status);
        Assert.Equal("REFERENCE_INVALID", result.DiagnosticCode);
    }

    [Fact]
    public void Matching_result_identity_resolves()
    {
        using var app = Build(builder => builder.AddPart("part", "part"));
        var result = Resolve(app, "part", "part", "part", app.Semantic.BuildIdentity);

        AssertResolved(result, "part", "part", app.Semantic.Id);
    }

    [Fact]
    public void Wrong_result_identity_is_indeterminate_even_when_target_exists()
    {
        using var app = Build(builder => builder.AddPart("part", "part"));
        var result = Resolve(app, "part", "part", "part", "different-result");

        Assert.Equal(SemanticReferenceStatus.Indeterminate, result.Status);
        Assert.Equal("REFERENCE_STALE_RESULT", result.DiagnosticCode);
        Assert.Empty(result.Candidates);
    }

    [Fact]
    public void All_resolved_candidates_preserve_the_declared_producer_and_kind()
    {
        using var app = Build(builder => builder.AddAssembly("assembly", "Assembly", assembly =>
            assembly.Part("occurrence", "part")));
        
        var result = Resolve(app, "assembly", "occurrence", "occurrence");

        Assert.True(result.IsResolved);
        Assert.Single(result.Candidates);
        var candidate = result.Candidates[0];
        Assert.Equal("assembly", candidate.ProducerId);
        Assert.Equal("occurrence", candidate.TargetId);
        Assert.Equal("occurrence", candidate.TargetKind);
    }

    [Fact]
    public void Equivalent_reference_resolution_is_deterministic_across_repeated_calls()
    {
        using var app = Build(builder => builder.AddPart("part", "part", part =>
            part.Geometry("edge", "line", new Dictionary<string, string>())));

        var service = app.GetRequiredService<ISemanticReferenceService>();
        var reference = new SemanticReference("part", "edge", "geometry");

        var first = service.Resolve(app.Semantic, reference);
        var second = service.Resolve(app.Semantic, reference);

        Assert.Equal(first.Status, second.Status);
        Assert.Equal(first.DiagnosticCode, second.DiagnosticCode);
        Assert.Equal(first.Message, second.Message);
        Assert.Equal(first.Candidates, second.Candidates);
    }

    private static CadApplication Build(Action<CadApplicationBuilder> configure)
    {
        var builder = CadApplication.CreateBuilder();
        configure(builder);
        return builder.Build();
    }

    private static SemanticReferenceResolution Resolve(
        CadApplication app,
        string producerId,
        string targetId,
        string targetKind,
        string? expectedResultIdentity = null) =>
        app.GetRequiredService<ISemanticReferenceService>().Resolve(
            app.Semantic,
            new SemanticReference(producerId, targetId, targetKind, expectedResultIdentity));

    private static void AssertResolved(
        SemanticReferenceResolution result,
        string targetId,
        string targetKind,
        string producerId)
    {
        Assert.Equal(SemanticReferenceStatus.Resolved, result.Status);
        Assert.True(result.IsResolved);
        Assert.NotNull(result.Target);
        Assert.Equal(producerId, result.Target!.ProducerId);
        Assert.Equal(targetId, result.Target.TargetId);
        Assert.Equal(targetKind, result.Target.TargetKind);
    }
}
