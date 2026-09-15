using UMLCAD.Framework;
using Xunit;

namespace UMLCAD.Framework.Tests;

public sealed class IdentityCollisionTests
{
    [Fact]
    public void Compiled_occurrence_ids_remain_unique_when_user_ids_contain_path_delimiters()
    {
        var builder = CadApplication.CreateBuilder();
        builder.AddPart("part/a", "part");
        builder.AddPart("part", "part");
        builder.AddAssembly("root/a", "Root A", a => a.Part("b", "part/a"));
        builder.AddAssembly("root", "Root", a => a.Part("a/b", "part"));

        using var app = builder.Build();
        var manifest = app.CreateCompiledModelManifest();

        Assert.Equal(manifest.Nodes.Count, manifest.Nodes.Select(x => x.Id).Distinct(StringComparer.Ordinal).Count());
        Assert.Equal(2, manifest.Nodes.Count(x => x.Kind == "ComponentInstance"));
    }

    [Fact]
    public void Compiled_definition_ids_remain_unique_when_user_ids_contain_colons_and_slashes()
    {
        var builder = CadApplication.CreateBuilder();
        builder.AddPart("a:b", "part");
        builder.AddPart("a", "part");
        builder.AddAssembly("root", "Root", a =>
        {
            a.Part("first", "a:b");
            a.Part("second", "a");
        });

        using var app = builder.Build();
        var manifest = app.CreateCompiledModelManifest();
        var definitionIds = manifest.Nodes.Where(x => x.Kind == "Part").Select(x => x.Id).ToArray();

        Assert.Equal(2, definitionIds.Distinct(StringComparer.Ordinal).Count());
    }

    [Fact]
    public void Duplicate_geometry_ids_are_rejected_during_compilation()
    {
        var builder = CadApplication.CreateBuilder();
        builder.AddPart("p", "part", part =>
        {
            part.Geometry("g", "line", new Dictionary<string, string>());
            part.Geometry("g", "arc", new Dictionary<string, string>());
        });

        using var app = builder.Build();
        Assert.Throws<InvalidOperationException>(() => app.CreateCompiledModelManifest());
    }

    [Fact]
    public void Duplicate_constraint_ids_are_rejected_during_compilation()
    {
        var builder = CadApplication.CreateBuilder();
        builder.AddPart("p", "part", part =>
        {
            part.Constraint("c", "horizontal", [], new Dictionary<string, string>());
            part.Constraint("c", "vertical", [], new Dictionary<string, string>());
        });

        using var app = builder.Build();
        Assert.Throws<InvalidOperationException>(() => app.CreateCompiledModelManifest());
    }
}
