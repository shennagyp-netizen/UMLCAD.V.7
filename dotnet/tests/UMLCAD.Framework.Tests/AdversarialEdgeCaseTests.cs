using System.Text.Json;
using UMLCAD.Framework;
using UMLCAD.Framework.Semantics;
using Xunit;

namespace UMLCAD.Framework.Tests;

public sealed class AdversarialEdgeCaseTests
{
    [Fact]
    public void Empty_application_is_a_valid_deterministic_build()
    {
        using var app = CadApplication.CreateBuilder().Build();
        Assert.Empty(app.Semantic.Parts);
        Assert.Empty(app.Semantic.Assemblies);
        Assert.Empty(app.Semantic.Drawings);
        Assert.False(string.IsNullOrWhiteSpace(app.Semantic.BuildIdentity));
        Assert.NotNull(app.CreateCompiledModelManifest());
    }

    [Theory]
    [InlineData("")]
    [InlineData(" ")]
    [InlineData("\t")]
    public void Required_builder_values_reject_whitespace(string value)
    {
        Assert.Throws<ArgumentException>(() => CadApplication.CreateBuilder().AddPart(value, "part"));
        Assert.Throws<ArgumentException>(() => CadApplication.CreateBuilder().AddAssembly("a", value));
        Assert.Throws<ArgumentException>(() => CadApplication.CreateBuilder().AddDrawing("d", value));
    }

    [Fact]
    public void Definition_ids_are_global_across_parts_and_assemblies()
    {
        var builder = CadApplication.CreateBuilder();
        builder.AddPart("same", "part");
        builder.AddAssembly("same", "Assembly");
        Assert.Throws<InvalidOperationException>(() => builder.Build());
    }

    [Fact]
    public void Duplicate_occurrence_ids_are_rejected_within_the_same_parent()
    {
        var builder = CadApplication.CreateBuilder();
        builder.AddPart("part", "part");
        builder.AddAssembly("assembly", "Assembly", a =>
        {
            a.Part("item", "part");
            a.Part("item", "part");
        });
        Assert.Throws<InvalidOperationException>(() => builder.Build());
    }

    [Fact]
    public void Repeated_shared_subassembly_can_be_instantiated_many_times()
    {
        var builder = CadApplication.CreateBuilder();
        builder.AddPart("p", "part");
        builder.AddAssembly("sub", "Sub", a => a.Part("p", "p"));
        builder.AddAssembly("root", "Root", a =>
        {
            for (var i = 0; i < 25; i++)
                a.Assembly($"sub-{i}", "sub");
        });

        using var app = builder.Build();
        var manifest = app.CreateCompiledModelManifest();
        var rootChildren = manifest.Nodes
            .Where(x => x.ParentId == "definition:assembly:root")
            .OrderBy(x => x.Id, StringComparer.Ordinal)
            .ToArray();

        Assert.Equal(25, rootChildren.Length);
        Assert.All(rootChildren, x => Assert.Equal("ComponentInstance", x.Kind));
        Assert.All(rootChildren, x => Assert.Equal("definition:assembly:sub", x.Metadata["definitionId"].GetString()));
    }

    [Fact]
    public void Deep_recursive_assembly_chain_preserves_full_occurrence_path()
    {
        const int depth = 250;
        var builder = CadApplication.CreateBuilder();
        builder.AddPart("leaf-part", "part");

        for (var i = depth - 1; i >= 0; i--)
        {
            var assemblyId = $"assembly-{i}";
            if (i == depth - 1)
            {
                builder.AddAssembly(assemblyId, assemblyId, a => a.Part("leaf", "leaf-part", "Leaf"));
            }
            else
            {
                var childId = $"assembly-{i + 1}";
                builder.AddAssembly(assemblyId, assemblyId, a => a.Assembly("child", childId));
            }
        }

        using var app = builder.Build();
        var manifest = app.CreateCompiledModelManifest();
        var current = manifest.Nodes.Single(x => x.Id == "definition:assembly:assembly-0");

        for (var i = 0; i < depth; i++)
        {
            Assert.Single(current.ChildIds);
            current = manifest.Nodes.Single(x => x.Id == current.ChildIds[0]);
        }

        Assert.Equal("ComponentInstance", current.Kind);
        Assert.Equal("Leaf", current.Name);
        Assert.Equal("definition:part:leaf-part", current.Metadata["definitionId"].GetString());
        Assert.Empty(current.ChildIds);
    }

    [Fact]
    public void Direct_self_cycle_is_rejected()
    {
        var builder = CadApplication.CreateBuilder();
        builder.AddAssembly("self", "Self", a => a.Assembly("self-instance", "self"));
        Assert.Throws<InvalidOperationException>(() => builder.Build());
    }

    [Fact]
    public void Invalid_definition_kind_is_rejected_before_compilation()
    {
        var builder = CadApplication.CreateBuilder();
        builder.AddAssembly("root", "Root", a => a.Occurrence("bad", "Bad", "missing", "widget"));
        Assert.Throws<InvalidOperationException>(() => builder.Build());
    }

    [Fact]
    public void Quantity_boundaries_and_non_finite_values_are_rejected()
    {
        foreach (var quantity in new[] { 0d, -1d, double.NaN, double.PositiveInfinity, double.NegativeInfinity })
        {
            var builder = CadApplication.CreateBuilder();
            builder.AddPart("p", "part");
            Assert.Throws<ArgumentOutOfRangeException>(() =>
                builder.AddAssembly("a", "A", a => a.Part("p1", "p", configure: o => o.Quantity(quantity))));
        }
    }

    [Fact]
    public void Transform_rejects_wrong_length_non_finite_and_zero_homogeneous_component()
    {
        Assert.Throws<ArgumentException>(() => TransformSemantic.FromArray(new double[15]));

        var nan = Enumerable.Repeat(0d, 16).ToArray();
        nan[0] = double.NaN;
        Assert.Throws<ArgumentException>(() => TransformSemantic.FromArray(nan));

        var infinity = Enumerable.Repeat(0d, 16).ToArray();
        infinity[0] = double.PositiveInfinity;
        Assert.Throws<ArgumentException>(() => TransformSemantic.FromArray(infinity));

        var zeroHomogeneous = TransformSemantic.Identity.Matrix.ToArray();
        zeroHomogeneous[15] = 0;
        Assert.Throws<ArgumentException>(() => TransformSemantic.FromArray(zeroHomogeneous));
    }

    [Fact]
    public void Occurrence_state_combinations_survive_compilation()
    {
        var builder = CadApplication.CreateBuilder();
        builder.AddPart("p", "part");
        builder.AddAssembly("a", "A", a => a.Part("x", "p", configure: o =>
        {
            o.Configuration("Service");
            o.Quantity(3.5);
            o.BomStructure("Reference");
            o.Visible(false);
            o.Suppressed(true);
            o.Grounded(true);
            o.Flexible(true);
            o.Property("role", "maintenance");
            o.Transform([
                1, 0, 0, 10,
                0, 1, 0, 20,
                0, 0, 1, 30,
                0, 0, 0, 1
            ]);
        }));

        using var app = builder.Build();
        var node = app.CreateCompiledModelManifest().Nodes.Single(x => x.Id == "occurrence:a/x");
        Assert.False(node.Capabilities.Visible);
        Assert.Equal("true", node.State["suppressed"]);
        Assert.Equal("true", node.State["grounded"]);
        Assert.Equal("true", node.State["flexible"]);
        Assert.Equal("maintenance", node.Metadata["role"].GetString());
        Assert.Equal(3.5, node.Metadata["quantity"].GetDouble());
        Assert.Equal("Service", node.Metadata["configuration"].GetString());
    }

    [Fact]
    public void Reserved_compiled_metadata_fields_are_authoritative()
    {
        var builder = CadApplication.CreateBuilder();
        builder.AddPart("p", "part");
        builder.AddAssembly("a", "A", a => a.Part("p1", "p", configure: o =>
        {
            o.Property("definitionId", "forged");
            o.Property("quantity", "forged");
        }));

        using var app = builder.Build();
        var node = app.CreateCompiledModelManifest().Nodes.Single(x => x.Id == "occurrence:a/p1");
        Assert.Equal("definition:part:p", node.Metadata["definitionId"].GetString());
        Assert.Equal(1d, node.Metadata["quantity"].GetDouble());
    }

    [Fact]
    public void Constraint_reference_to_missing_geometry_fails_compiled_projection()
    {
        var builder = CadApplication.CreateBuilder();
        builder.AddPart("p", "part", part =>
            part.Constraint("c", "coincident", ["missing-geometry"], new Dictionary<string, string>()));
        using var app = builder.Build();

        var exception = Assert.Throws<InvalidOperationException>(() => app.CreateCompiledModelManifest());
        Assert.Contains("does not resolve", exception.Message, StringComparison.OrdinalIgnoreCase);
    }

    [Fact]
    public void Changing_semantic_inputs_changes_build_identity()
    {
        static string Build(Action<PartBuilder> configure)
        {
            var builder = CadApplication.CreateBuilder();
            builder.AddPart("p", "part", configure);
            using var app = builder.Build();
            return app.Semantic.BuildIdentity;
        }

        Assert.NotEqual(Build(p => p.Name("A")), Build(p => p.Name("B")));
        Assert.NotEqual(Build(p => p.PartNumber("A")), Build(p => p.PartNumber("B")));
        Assert.NotEqual(Build(p => p.Parameter("width", "1", "mm")), Build(p => p.Parameter("width", "2", "mm")));
        Assert.NotEqual(
            Build(p => p.Geometry("g", "line", new Dictionary<string, string> { ["x"] = "1" })),
            Build(p => p.Geometry("g", "line", new Dictionary<string, string> { ["x"] = "2" })));
    }

    [Fact]
    public void Service_snapshot_is_immutable_against_later_builder_changes()
    {
        var builder = CadApplication.CreateBuilder();
        builder.Configuration["rev"] = "one";
        builder.AddPart("p", "part");
        using var first = builder.Build();
        var firstIdentity = first.Semantic.BuildIdentity;

        builder.Configuration["rev"] = "two";
        builder.AddPart("p2", "part");
        using var second = builder.Build();

        Assert.Equal("one", first.Semantic.Configuration["rev"]);
        Assert.Equal(firstIdentity, first.Semantic.BuildIdentity);
        Assert.Equal("two", second.Semantic.Configuration["rev"]);
        Assert.NotEqual(first.Semantic.BuildIdentity, second.Semantic.BuildIdentity);
    }

    [Fact]
    public void Unknown_node_kinds_and_metadata_are_preserved_by_compiled_contract()
    {
        var manifest = new CompiledModelManifest(
            "uml-cad-compiled-model/1.1.0", "app", "1.0", "build",
            ["root"],
            [new CompiledNode("root", "Future", "vendor:future-node", null, [],
                new Dictionary<string, JsonElement> { ["futureProperty"] = JsonSerializer.SerializeToElement(new { answer = 42 }) },
                [], [], new CompiledCapabilities(true, true, true, true), null,
                new Dictionary<string, string>())],
            [], [], [], [], []);

        Assert.Equal("vendor:future-node", manifest.Nodes[0].Kind);
        Assert.Equal(42, manifest.Nodes[0].Metadata["futureProperty"].GetProperty("answer").GetInt32());
    }
}
