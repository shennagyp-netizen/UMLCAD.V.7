using System.Text.Json;
using UMLCAD.Framework;
using UMLCAD.Framework.Semantics;
using Xunit;

namespace UMLCAD.Framework.Tests;

public sealed class SystemCadRedTeamTests
{
    [Fact]
    public void Duplicate_geometry_identity_is_ambiguous_and_never_first_match()
    {
        using var app = Build(builder => builder.AddPart("part", "part", part =>
        {
            part.Geometry("edge", "line", new Dictionary<string, string>());
            part.Geometry("edge", "circle", new Dictionary<string, string>());
        }));

        var result = Resolve(app, "part", "edge", "geometry");

        Assert.Equal(SemanticReferenceStatus.Ambiguous, result.Status);
        Assert.False(result.IsResolved);
        Assert.Equal(2, result.Candidates.Count);
    }

    [Fact]
    public void Duplicate_constraint_identity_is_ambiguous()
    {
        using var app = Build(builder => builder.AddPart("part", "part", part =>
        {
            part.Geometry("edge", "line", new Dictionary<string, string>());
            part.Constraint("c", "horizontal", ["edge"], new Dictionary<string, string>());
            part.Constraint("c", "vertical", ["edge"], new Dictionary<string, string>());
        }));

        var result = Resolve(app, "part", "c", "constraint");

        Assert.Equal(SemanticReferenceStatus.Ambiguous, result.Status);
        Assert.Equal("REFERENCE_AMBIGUOUS", result.DiagnosticCode);
    }

    [Fact]
    public void Duplicate_component_identity_is_ambiguous()
    {
        var builder = CadApplication.CreateBuilder();
        builder.AddPart("part", "part", part =>
        {
            part.Component("component");
            part.Component("component");
        });

        using var app = builder.Build();
        var result = Resolve(app, "part", "component", "component");

        Assert.Equal(SemanticReferenceStatus.Ambiguous, result.Status);
        Assert.Equal(2, result.Candidates.Count);
    }

    [Fact]
    public void Face_publication_without_matching_binding_is_rejected_at_build()
    {
        var builder = CadApplication.CreateBuilder();
        builder.AddPart("part", "solid", part =>
            part.Publication("front", "face", "front-face", "result-r1", "missing-binding"));

        var exception = Assert.Throws<InvalidOperationException>(() => builder.Build());
        Assert.Contains("publication", exception.Message, StringComparison.OrdinalIgnoreCase);
    }

    [Fact]
    public void Face_publication_with_mismatched_binding_cannot_become_a_valid_snapshot()
    {
        var builder = CadApplication.CreateBuilder();
        builder.AddPart("part", "solid", part =>
        {
            part.TopologyBinding("binding", "face", "other-face", "result-r1", "brep-face-2");
            part.Publication("front", "face", "front-face", "result-r1", "binding");
        });

        Assert.Throws<InvalidOperationException>(() => builder.Build());
    }

    [Fact]
    public void Duplicate_publication_identity_is_rejected()
    {
        var builder = CadApplication.CreateBuilder();
        builder.AddPart("part", "solid", part =>
        {
            part.TopologyBinding("binding", "face", "front-face", "result-r1", "brep-face-1");
            part.Publication("front", "face", "front-face", "result-r1", "binding");
            part.Publication("front", "face", "front-face", "result-r1", "binding");
        });

        Assert.Throws<InvalidOperationException>(() => builder.Build());
    }

    [Fact]
    public void Duplicate_topology_binding_identity_is_rejected()
    {
        var builder = CadApplication.CreateBuilder();
        builder.AddPart("part", "solid", part =>
        {
            part.TopologyBinding("binding", "face", "front-face", "result-r1", "brep-face-1");
            part.TopologyBinding("binding", "face", "back-face", "result-r1", "brep-face-2");
        });

        Assert.Throws<InvalidOperationException>(() => builder.Build());
    }

    [Fact]
    public void Wrong_target_kind_cannot_cross_reinterpretation_boundaries()
    {
        using var app = Build(builder => builder.AddPart("part", "part", part =>
            part.Geometry("edge", "line", new Dictionary<string, string>())));

        foreach (var kind in new[] { "constraint", "component", "occurrence", "sheet", "face", "vertex", "body" })
        {
            var result = Resolve(app, "part", "edge", kind);
            Assert.NotEqual(SemanticReferenceStatus.Resolved, result.Status);
        }
    }

    [Fact]
    public void Producer_identity_is_exact_and_does_not_trim_or_case_fold()
    {
        using var app = Build(builder => builder.AddPart("part", "part"));

        var whitespace = Resolve(app, " part", "part", "part");
        var caseChanged = Resolve(app, "PART", "part", "part");

        Assert.Equal(SemanticReferenceStatus.Missing, whitespace.Status);
        Assert.Equal(SemanticReferenceStatus.Missing, caseChanged.Status);
    }

    [Fact]
    public void Target_identity_is_exact_and_does_not_decode_or_normalize_paths()
    {
        using var app = Build(builder => builder.AddPart("part", "part", part =>
            part.Geometry("edge/1", "line", new Dictionary<string, string>())));

        var exact = Resolve(app, "part", "edge/1", "geometry");
        var encoded = Resolve(app, "part", "edge%2F1", "geometry");

        Assert.Equal(SemanticReferenceStatus.Resolved, exact.Status);
        Assert.Equal(SemanticReferenceStatus.Missing, encoded.Status);
    }

    [Fact]
    public void Unicode_semantic_identity_is_supported_without_ascii_fallback()
    {
        const string producer = "pièce-α";
        const string target = "arête-β";

        using var app = Build(builder => builder.AddPart(producer, "part", part =>
            part.Geometry(target, "line", new Dictionary<string, string>())));

        var result = Resolve(app, producer, target, "geometry");

        AssertResolved(result, producer, target, "geometry");
    }

    [Fact]
    public void Stale_result_identity_cannot_be_bypassed_by_a_valid_target()
    {
        using var app = Build(builder => builder.AddPart("part", "part", part =>
            part.Geometry("edge", "line", new Dictionary<string, string>())));

        var result = Resolve(app, "part", "edge", "geometry", "old-result");

        Assert.Equal(SemanticReferenceStatus.Indeterminate, result.Status);
        Assert.Equal("REFERENCE_STALE_RESULT", result.DiagnosticCode);
        Assert.Empty(result.Candidates);
    }

    [Fact]
    public void Unsupported_reference_returns_no_candidate_that_could_be_misused()
    {
        using var app = Build(builder => builder.AddPart("part", "part"));

        var result = Resolve(app, "part", "face-1", "face");

        Assert.Equal(SemanticReferenceStatus.Unsupported, result.Status);
        Assert.Empty(result.Candidates);
        Assert.Null(result.Target);
    }

    [Fact]
    public void Malformed_reference_does_not_fall_back_to_lookup()
    {
        using var app = Build(builder => builder.AddPart("part", "part"));

        var result = Resolve(app, "part", "target", " ", " ");

        Assert.Equal(SemanticReferenceStatus.Indeterminate, result.Status);
        Assert.Equal("REFERENCE_INVALID", result.DiagnosticCode);
    }

    [Fact]
    public void Null_values_at_the_public_boundary_fail_closed()
    {
        using var app = Build(builder => builder.AddPart("part", "part"));

        Assert.Throws<ArgumentNullException>(() =>
            Resolve(app, null!, "target", "geometry"));
        Assert.Throws<ArgumentNullException>(() =>
            Resolve(app, "part", null!, "geometry"));
        Assert.Throws<ArgumentNullException>(() =>
            Resolve(app, "part", "target", null!));
    }

    [Fact]
    public void Application_identity_collision_with_definition_identity_is_rejected()
    {
        var builder = CadApplication.CreateBuilder();
        builder.ApplicationId = "same";
        builder.AddPart("same", "part");

        Assert.Throws<InvalidOperationException>(() => builder.Build());
    }

    [Fact]
    public void Compiled_node_identity_escapes_structural_path_delimiters()
    {
        using var app = Build(builder => builder.AddPart("part/a", "part", part =>
            part.Geometry("edge:b/c", "line", new Dictionary<string, string>())));

        var manifest = app.CreateCompiledModelManifest();

        Assert.Contains(manifest.Nodes, x => x.Id == "definition:part:part%2Fa");
        Assert.Contains(manifest.Nodes, x =>
            x.Id == "definition:part:part%2Fa/geometry:edge%3Ab%2Fc");
    }

    [Fact]
    public void Forged_reserved_metadata_cannot_replace_authoritative_occurrence_metadata()
    {
        using var app = Build(builder =>
        {
            builder.AddPart("part", "part");
            builder.AddAssembly("assembly", "Assembly", assembly =>
                assembly.Part("occurrence", "part", configure: occurrence =>
                {
                    occurrence.Property("definitionId", "forged");
                    occurrence.Property("configuration", "forged");
                    occurrence.Property("quantity", "NaN");
                    occurrence.Property("transform", JsonSerializer.Serialize(new[] { 99, 99, 99, 99 }));
                }));
        });

        var node = app.CreateCompiledModelManifest().Nodes.Single(x => x.Id == "occurrence:assembly/occurrence");

        Assert.Equal("definition:part:part", node.Metadata["definitionId"].GetString());
        Assert.Equal(1d, node.Metadata["quantity"].GetDouble());
        Assert.Equal("null", node.Metadata["configuration"].GetRawText());
        Assert.Contains("1", node.Metadata["transform"].GetRawText());
    }

    [Fact]
    public void Snapshot_isolation_prevents_builder_mutation_from_contaminating_an_existing_application()
    {
        var builder = CadApplication.CreateBuilder();
        builder.ApplicationId = "snapshot";
        builder.Configuration["state"] = "one";
        builder.AddPart("part", "part", part =>
            part.Geometry("edge", "line", new Dictionary<string, string>()));

        using var first = builder.Build();
        var firstIdentity = first.Semantic.BuildIdentity;

        builder.Configuration["state"] = "two";
        builder.AddPart("part-2", "part");
        using var second = builder.Build();

        Assert.Equal("one", first.Semantic.Configuration["state"]);
        Assert.Equal(firstIdentity, first.Semantic.BuildIdentity);
        Assert.Equal("two", second.Semantic.Configuration["state"]);
        Assert.NotEqual(first.Semantic.BuildIdentity, second.Semantic.BuildIdentity);
    }

    [Fact]
    public void Repeated_compilation_is_stable_for_the_same_snapshot()
    {
        using var app = Build(builder =>
        {
            builder.AddPart("part", "part", part =>
                part.Geometry("edge", "line", new Dictionary<string, string>()));
            builder.AddAssembly("assembly", "Assembly", assembly =>
                assembly.Part("instance", "part"));
        });

        var first = app.CreateCompiledModelManifest();
        var second = app.CreateCompiledModelManifest();

        Assert.Equal(first.BuildIdentity, second.BuildIdentity);
        Assert.Equal(JsonSerializer.Serialize(first), JsonSerializer.Serialize(second));
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
        string producerId,
        string targetId,
        string targetKind)
    {
        Assert.Equal(SemanticReferenceStatus.Resolved, result.Status);
        Assert.NotNull(result.Target);
        Assert.Equal(producerId, result.Target!.ProducerId);
        Assert.Equal(targetId, result.Target.TargetId);
        Assert.Equal(targetKind, result.Target.TargetKind);
    }

}
