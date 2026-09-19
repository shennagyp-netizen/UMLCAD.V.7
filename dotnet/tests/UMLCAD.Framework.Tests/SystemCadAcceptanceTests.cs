using System.Globalization;
using System.Text.Json;
using UMLCAD.Framework;
using UMLCAD.Framework.Semantics;
using Xunit;

namespace UMLCAD.Framework.Tests;

public sealed class SystemCadAcceptanceTests
{
    [Fact]
    public void End_to_end_part_assembly_drawing_path_keeps_one_semantic_identity_flow()
    {
        var builder = CadApplication.CreateBuilder();
        builder.ApplicationId = "system-cad";
        builder.Version = "1.0.0";

        builder.AddPart("bracket", "mechanical-part", part =>
        {
            part.PartNumber("BR-001")
                .Material("Aluminium")
                .Geometry("base-edge", "line", new Dictionary<string, string>
                {
                    ["start"] = "0,0",
                    ["end"] = "100,0"
                })
                .Constraint("horizontal", "horizontal", ["base-edge"], new Dictionary<string, string>());
        });

        builder.AddAssembly("machine", "Machine", assembly =>
            assembly.Part("bracket-1", "bracket", "Bracket:1", occurrence =>
            {
                occurrence.Configuration("Default")
                    .Quantity(1)
                    .Grounded();
            }));

        builder.AddDrawing("bracket-drawing", "Bracket Drawing", drawing =>
            drawing.PartReference("bracket")
                .Sheet("sheet-1", "A3"));

        using var app = builder.Build();
        var references = app.GetRequiredService<ISemanticReferenceService>();
        var manifest = app.CreateCompiledModelManifest();

        var geometry = references.Resolve(app.Semantic,
            new SemanticReference("bracket", "base-edge", "geometry"));
        var occurrence = references.Resolve(app.Semantic,
            new SemanticReference("machine", "bracket-1", "occurrence"));
        var sheet = references.Resolve(app.Semantic,
            new SemanticReference("bracket-drawing", "sheet-1", "sheet"));

        Assert.True(geometry.IsResolved);
        Assert.True(occurrence.IsResolved);
        Assert.True(sheet.IsResolved);

        Assert.Contains(manifest.Nodes, x => x.Id == "definition:part:bracket");
        Assert.Contains(manifest.Nodes, x => x.Id == "definition:assembly:machine");
        Assert.Contains(manifest.Nodes, x => x.Id == "occurrence:machine/bracket-1");
        Assert.Contains(manifest.Nodes, x => x.Id == "definition:drawing:bracket-drawing");
        Assert.Contains(manifest.Nodes, x => x.Id == "definition:drawing:bracket-drawing/sheet:sheet-1");
    }

    [Fact]
    public void Compiled_constraint_reference_and_semantic_reference_resolve_to_the_same_target()
    {
        var builder = CadApplication.CreateBuilder();
        builder.AddPart("part", "part", part =>
            part.Geometry("edge", "line", new Dictionary<string, string>())
                .Constraint("constraint", "horizontal", ["edge"], new Dictionary<string, string>()));

        using var app = builder.Build();

        var semantic = app.GetRequiredService<ISemanticReferenceService>()
            .Resolve(app.Semantic, new SemanticReference("part", "edge", "geometry"));
        var manifest = app.CreateCompiledModelManifest();

        var constraint = manifest.Nodes.Single(x => x.Id.EndsWith("/constraint:constraint", StringComparison.Ordinal));
        var relationship = manifest.Relationships.Single(x => x.SourceId == constraint.Id && x.Kind == "references");

        Assert.True(semantic.IsResolved);
        Assert.Equal("definition:part:part/geometry:edge", relationship.TargetIds.Single());
    }

    [Fact]
    public void Assembly_occurrence_transform_is_instance_state_and_does_not_mutate_source_part()
    {
        var builder = CadApplication.CreateBuilder();
        builder.AddPart("part", "part", part =>
            part.Geometry("edge", "line", new Dictionary<string, string>
            {
                ["start"] = "0,0",
                ["end"] = "10,0"
            }));
        builder.AddAssembly("assembly", "Assembly", assembly =>
            assembly.Part("instance", "part", configure: occurrence =>
                occurrence.Transform([
                    1, 0, 0, 100,
                    0, 1, 0, 200,
                    0, 0, 1, 300,
                    0, 0, 0, 1
                ]));

        using var app = builder.Build();

        var part = app.Semantic.Parts.Single(x => x.Id == "part");
        var occurrence = app.Semantic.Assemblies.Single(x => x.Id == "assembly").Occurrences.Single();

        Assert.Equal([0d, 0d], ParsePoint(part.Geometry.Single().Properties["start"]));
        Assert.Equal(100d, occurrence.Transform.Matrix[3]);
        Assert.Equal(200d, occurrence.Transform.Matrix[7]);
        Assert.Equal(300d, occurrence.Transform.Matrix[11]);
    }

    [Fact]
    public void Source_definition_reference_and_occurrence_context_are_distinct()
    {
        var builder = CadApplication.CreateBuilder();
        builder.AddPart("part", "part");
        builder.AddAssembly("assembly", "Assembly", assembly =>
            assembly.Part("instance", "part"));

        using var app = builder.Build();

        var definition = app.CreateCompiledModelManifest().Nodes.Single(x => x.Id == "definition:part:part");
        var occurrence = app.CreateCompiledModelManifest().Nodes.Single(x => x.Id == "occurrence:assembly/instance");

        Assert.NotEqual(definition.Id, occurrence.Id);
        Assert.Equal("definition:part:part", occurrence.Metadata["definitionId"].GetString());
    }

    [Fact]
    public void Equivalent_semantic_inputs_compile_to_identical_manifests()
    {
        static CompiledModelManifest Build(bool reverse)
        {
            var builder = CadApplication.CreateBuilder();
            builder.ApplicationId = "deterministic";
            if (reverse)
            {
                builder.AddPart("b", "part");
                builder.AddPart("a", "part");
            }
            else
            {
                builder.AddPart("a", "part");
                builder.AddPart("b", "part");
            }

            using var app = builder.Build();
            return app.CreateCompiledModelManifest();
        }

        var first = Build(false);
        var second = Build(true);

        Assert.Equal(first.BuildIdentity, second.BuildIdentity);
        Assert.Equal(
            JsonSerializer.Serialize(first),
            JsonSerializer.Serialize(second));
    }

    [Fact]
    public void Missing_drawing_part_reference_is_rejected_before_a_drawing_can_become_authoritative()
    {
        var builder = CadApplication.CreateBuilder();
        builder.AddDrawing("drawing", "Drawing", drawing =>
            drawing.PartReference("does-not-exist"));

        Assert.Throws<InvalidOperationException>(() => builder.Build());
    }

    [Fact]
    public void Missing_assembly_occurrence_definition_is_rejected_before_compilation()
    {
        var builder = CadApplication.CreateBuilder();
        builder.AddAssembly("assembly", "Assembly", assembly =>
            assembly.Occurrence("missing", "Missing", "no-such-definition", "part"));

        Assert.Throws<InvalidOperationException>(() => builder.Build());
    }

    [Fact]
    public void Duplicate_drawing_sheet_id_is_not_silently_selected()
    {
        var builder = CadApplication.CreateBuilder();
        builder.AddDrawing("drawing", "Drawing", drawing =>
            drawing.Sheet("sheet", "A4")
                .Sheet("sheet", "A4 duplicate"));

        using var app = builder.Build();
        var result = app.GetRequiredService<ISemanticReferenceService>()
            .Resolve(app.Semantic, new SemanticReference("drawing", "sheet", "sheet"));

        Assert.Equal(SemanticReferenceStatus.Ambiguous, result.Status);
        Assert.False(result.IsResolved);
    }

    [Fact]
    public void Drawing_definition_must_not_be_reconstructed_from_compiled_render_data()
    {
        var builder = CadApplication.CreateBuilder();
        builder.AddDrawing("drawing", "Drawing", drawing =>
            drawing.PartReference("part").Sheet("sheet", "A4"));
        builder.AddPart("part", "part");

        using var app = builder.Build();
        var manifest = app.CreateCompiledModelManifest();

        Assert.Empty(manifest.Representations);
        Assert.Empty(manifest.TopologyBindings);
        Assert.Contains(manifest.Nodes, x => x.Id == "definition:drawing:drawing");
    }

    private static double[] ParsePoint(string value) =>
        value.Split(',', StringSplitOptions.RemoveEmptyEntries)
            .Select(value => double.Parse(value, CultureInfo.InvariantCulture))
            .ToArray();
}
