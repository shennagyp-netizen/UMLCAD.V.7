using UMLCAD.Framework;
using Xunit;

namespace UMLCAD.Framework.Tests;

public sealed class CompiledModelTests
{
    [Fact]
    public void Framework_supports_nested_assemblies_part_occurrences_and_cad_metadata()
    {
        var builder = CadApplication.CreateBuilder();

        builder.AddPart("motor-housing", "mechanical-part", part =>
        {
            part.Name("Motor Housing");
            part.PartNumber("MH-001");
            part.Description("Main motor housing");
            part.Material("Aluminium");
            part.Revision("B");
            part.LifecycleState("Released");
            part.Property("ProjectCode", "P-100");
            part.Parameter("width", "120", "mm");
            part.Geometry("outer-body", "Solid", new Dictionary<string, string> { ["representation"] = "body" });
        });

        builder.AddAssembly("gearbox", "Gearbox", assembly =>
        {
            assembly.PartNumber("GBX-10");
            assembly.Part("housing-instance", "motor-housing", "Housing:1", occurrence =>
            {
                occurrence.Configuration("Default");
                occurrence.Quantity(1);
                occurrence.Grounded();
            });
            assembly.Property("Subsystem", "Powertrain");
        });

        builder.AddAssembly("machine", "Machine", assembly =>
        {
            assembly.Assembly("gearbox-instance", "gearbox", "Gearbox:1", occurrence =>
            {
                occurrence.Flexible();
                occurrence.Property("PositionRole", "Drive");
            });

            assembly.Assembly("gearbox-instance-2", "gearbox", "Gearbox:2", occurrence =>
            {
                occurrence.Configuration("Service");
                occurrence.Visible(false);
                occurrence.Property("PositionRole", "Service");
            });
        });

        using var app = builder.Build();
        var manifest = app.CreateCompiledModelManifest();
        var gearboxSemantic = Assert.Single(app.Semantic.Assemblies, x => x.Id == "gearbox");

        Assert.Equal("Powertrain", gearboxSemantic.Metadata["custom:Subsystem"]);
        Assert.Contains(manifest.Nodes, x => x.Id == "definition:assembly:machine");
        Assert.Contains(manifest.Nodes, x => x.Id == "occurrence:machine/gearbox-instance");
        Assert.Contains(manifest.Nodes, x => x.Id == "occurrence:machine/gearbox-instance/occurrence:housing-instance");
        Assert.Contains(manifest.Nodes, x => x.Id == "occurrence:machine/gearbox-instance/occurrence:housing-instance/geometry:outer-body");

        var machine = manifest.Nodes.Single(x => x.Id == "definition:assembly:machine");
        Assert.Equal(2, machine.ChildIds.Count);

        var drive = manifest.Nodes.Single(x => x.Id == "occurrence:machine/gearbox-instance");
        Assert.Equal("Drive", drive.Metadata["PositionRole"].GetString());
        Assert.Equal("definition:assembly:gearbox", drive.Metadata["definitionId"].GetString());
    }

    [Fact]
    public void Occurrence_ids_may_repeat_in_different_assemblies_but_compiled_ids_are_unique()
    {
        var builder = CadApplication.CreateBuilder();
        builder.AddPart("part", "part");
        builder.AddAssembly("a", "A", assembly => assembly.Part("item-1", "part"));
        builder.AddAssembly("b", "B", assembly => assembly.Part("item-1", "part"));

        using var app = builder.Build();
        var manifest = app.CreateCompiledModelManifest();

        Assert.Contains(manifest.Nodes, x => x.Id == "occurrence:a/item-1");
        Assert.Contains(manifest.Nodes, x => x.Id == "occurrence:b/item-1");
        Assert.Equal(manifest.Nodes.Count, manifest.Nodes.Select(x => x.Id).Distinct(StringComparer.Ordinal).Count());
    }

    [Fact]
    public void Circular_assembly_references_are_rejected()
    {
        var builder = CadApplication.CreateBuilder();
        builder.AddAssembly("a", "A", assembly => assembly.Assembly("a-to-b", "b"));
        builder.AddAssembly("b", "B", assembly => assembly.Assembly("b-to-a", "a"));

        Assert.Throws<InvalidOperationException>(() => builder.Build());
    }

    [Fact]
    public void Assembly_metadata_changes_build_identity()
    {
        static string BuildIdentity(string value)
        {
            var builder = CadApplication.CreateBuilder();
            builder.AddAssembly("assembly", "Assembly", assembly => assembly.Property("purpose", value));
            using var app = builder.Build();
            return app.Semantic.BuildIdentity;
        }

        Assert.NotEqual(BuildIdentity("A"), BuildIdentity("B"));
    }

    [Fact]
    public void Equivalent_nested_models_have_stable_compiled_manifest_identity()
    {
        static string BuildIdentity()
        {
            var builder = CadApplication.CreateBuilder();
            builder.AddPart("part", "part", part =>
            {
                part.PartNumber("P-1");
                part.Property("x", "1");
            });
            builder.AddAssembly("sub", "Sub", assembly => assembly.Part("p1", "part"));
            builder.AddAssembly("top", "Top", assembly => assembly.Assembly("sub1", "sub"));
            using var app = builder.Build();
            return app.CreateCompiledModelManifest().BuildIdentity;
        }

        Assert.Equal(BuildIdentity(), BuildIdentity());
    }
}
