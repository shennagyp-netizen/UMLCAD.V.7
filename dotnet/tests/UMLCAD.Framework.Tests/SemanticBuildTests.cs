using Microsoft.Extensions.DependencyInjection;
using UMLCAD.Framework;
using UMLCAD.Framework.Semantics;
using Xunit;

namespace UMLCAD.Framework.Tests;

public sealed class SemanticBuildTests
{
    [Fact]
    public void Build_collects_parts_drawings_sheets_configuration_and_services()
    {
        var builder = CadApplication.CreateBuilder();
        builder.ApplicationId = "test-app";
        builder.Version = "1.0.0";
        builder.Configuration["cad:units"] = "mm";
        builder.Services.AddSingleton<TestMarker>();

        builder.AddPart("bracket", "mechanical-part", part =>
        {
            part.Parameter("length", "100", "mm")
                .Geometry("line-1", "line", new Dictionary<string, string>
                {
                    ["start"] = "0,0",
                    ["end"] = "100,0"
                })
                .Constraint("constraint-1", "horizontal", ["line-1"],
                    new Dictionary<string, string>());
        });

        builder.AddDrawing("drawing-1", "Bracket Drawing", drawing =>
        {
            drawing.PartReference("bracket")
                .Setting("units", "mm")
                .Sheet("sheet-1", "A3");
        });

        using var app = builder.Build();

        Assert.Equal("test-app", app.Semantic.Id);
        Assert.Equal("mm", app.Configuration["cad:units"]);
        Assert.Equal("mm", app.Semantic.Configuration["cad:units"]);
        Assert.Single(app.Semantic.Parts);
        Assert.Single(app.Semantic.Drawings);
        Assert.Single(app.Semantic.Drawings[0].Sheets);
        Assert.Equal("bracket", app.Semantic.Drawings[0].PartReferences[0]);

        using var scope = app.Services.CreateScope();
        var parts = scope.ServiceProvider.GetRequiredService<IPartSemanticService>();
        var marker = scope.ServiceProvider.GetRequiredService<TestMarker>();

        Assert.Equal("bracket", parts.Get("bracket")!.Id);
        Assert.NotNull(marker);
    }

    [Fact]
    public void Equivalent_builds_have_the_same_identity_and_package_bytes_even_when_dictionary_order_differs()
    {
        static CadApplication Build(bool reverseProperties)
        {
            var builder = CadApplication.CreateBuilder();
            builder.ApplicationId = "identity-test";
            builder.Version = "1.0.0";
            builder.Configuration["cad:units"] = "mm";

            builder.AddPart("part-a", "part", part =>
            {
                var properties = reverseProperties
                    ? new Dictionary<string, string> { ["z"] = "2", ["a"] = "1" }
                    : new Dictionary<string, string> { ["a"] = "1", ["z"] = "2" };
                part.Geometry("geometry-a", "line", properties);
            });

            builder.AddDrawing("drawing-a", "Drawing", drawing =>
                drawing.PartReference("part-a").Sheet("sheet-a", "A4"));
            return builder.Build();
        }

        using var first = Build(false);
        using var second = Build(true);

        Assert.Equal(first.Semantic.BuildIdentity, second.Semantic.BuildIdentity);
        Assert.Equal(first.CreateBuildPackage().BuildIdentity, second.CreateBuildPackage().BuildIdentity);

        var firstBytes = first.GetRequiredService<IBuildPackageService>().Serialize(first.CreateBuildPackage());
        var secondBytes = second.GetRequiredService<IBuildPackageService>().Serialize(second.CreateBuildPackage());
        Assert.Equal(firstBytes, secondBytes);
    }

    [Fact]
    public void Builder_collects_every_build_and_keeps_snapshot_packages_local()
    {
        var builder = CadApplication.CreateBuilder();
        builder.ApplicationId = "history-test";
        builder.Configuration["revision"] = "one";
        builder.AddPart("part-a", "part");

        using var first = builder.Build();
        var firstPackage = first.CreateBuildPackage();

        builder.Configuration["revision"] = "two";
        using var second = builder.Build();
        var secondPackage = second.CreateBuildPackage();

        var history = second.GetRequiredService<IBuildHistory>();
        Assert.Equal(2, history.GetAll().Count);
        Assert.Equal(1, history.GetAll()[0].Sequence);
        Assert.Equal(2, history.GetAll()[1].Sequence);
        Assert.Equal("one", history.GetAll()[0].Application.Configuration["revision"]);
        Assert.Equal("two", history.GetAll()[1].Application.Configuration["revision"]);
        Assert.NotEqual(first.Semantic.BuildIdentity, second.Semantic.BuildIdentity);
        Assert.Equal(first.Semantic.BuildIdentity, firstPackage.BuildIdentity);
        Assert.Equal(second.Semantic.BuildIdentity, secondPackage.BuildIdentity);
        Assert.Equal("one", firstPackage.Semantic.Configuration["revision"]);
        Assert.Equal("two", secondPackage.Semantic.Configuration["revision"]);
    }

    [Fact]
    public void Package_serialization_is_deterministic_for_the_same_snapshot()
    {
        var builder = CadApplication.CreateBuilder();
        builder.ApplicationId = "package-test";
        builder.AddPart("part-a", "part");
        using var app = builder.Build();

        var service = app.GetRequiredService<IBuildPackageService>();
        var package = app.CreateBuildPackage();

        Assert.Equal(service.SerializeToString(package), service.SerializeToString(package));
        Assert.Equal(service.Serialize(package), service.Serialize(package));
        Assert.Equal("uml-cad-build-package/1.0.0", package.Schema);
    }

    [Fact]
    public void Duplicate_semantic_ids_fail_during_build()
    {
        var builder = CadApplication.CreateBuilder();
        builder.AddPart("part-a", "part");
        builder.AddPart("part-a", "other-part");

        Assert.Throws<InvalidOperationException>(() => builder.Build());
    }

    private sealed class TestMarker;
}
