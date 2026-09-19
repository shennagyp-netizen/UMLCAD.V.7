using Microsoft.Extensions.DependencyInjection;
using UMLCAD.Framework;
using UMLCAD.Framework.Semantics;
using Xunit;

namespace UMLCAD.Framework.Tests;

public sealed class SemanticFrameTests
{
    [Fact]
    public void Resolves_exact_frame_identity()
    {
        using var app = Build(builder =>
        {
            builder.Frame("world", SemanticFrameKind.World);
            builder.Frame("part", SemanticFrameKind.Part, "world");
        });

        var result = app.GetRequiredService<ISemanticFrameService>()
            .Resolve(app.Semantic, "part");

        Assert.Equal(SemanticReferenceStatus.Resolved, result.Status);
        Assert.Equal("part", result.Frame!.Id);
        Assert.Equal(SemanticFrameKind.Part, result.Frame.Kind);
        Assert.Equal("world", result.Frame.ParentId);
    }

    [Fact]
    public void Missing_frame_is_distinct_from_invalid_reference()
    {
        using var app = Build(builder =>
            builder.Frame("world", SemanticFrameKind.World));

        var result = app.GetRequiredService<ISemanticFrameService>()
            .Resolve(app.Semantic, "missing");

        Assert.Equal(SemanticReferenceStatus.Missing, result.Status);
        Assert.Equal("FRAME_MISSING", result.DiagnosticCode);
    }

    [Fact]
    public void Duplicate_frame_identity_is_ambiguous()
    {
        var builder = CadApplication.CreateBuilder();
        builder.Frame("world", SemanticFrameKind.World);
        builder.Frame("part", SemanticFrameKind.Part, "world");
        builder.Frame("part", SemanticFrameKind.Face, "world");

        Assert.Throws<InvalidOperationException>(() => builder.Build());
    }

    [Fact]
    public void Non_world_frame_requires_a_parent()
    {
        var builder = CadApplication.CreateBuilder();
        builder.Frame("part", SemanticFrameKind.Part);

        var exception = Assert.Throws<InvalidOperationException>(() => builder.Build());
        Assert.Contains("FRAME_PARENT_MISSING", exception.Message, StringComparison.Ordinal);
    }

    [Fact]
    public void World_frame_cannot_have_a_parent()
    {
        var builder = CadApplication.CreateBuilder();
        builder.Frame("parent", SemanticFrameKind.Document);
        builder.Frame("world", SemanticFrameKind.World, "parent");

        var exception = Assert.Throws<InvalidOperationException>(() => builder.Build());
        Assert.Contains("FRAME_WORLD_PARENT", exception.Message, StringComparison.Ordinal);
    }

    [Fact]
    public void World_frame_requires_identity_transform()
    {
        var builder = CadApplication.CreateBuilder();
        builder.Frame("world", SemanticFrameKind.World, transform: [
            1, 0, 0, 10,
            0, 1, 0, 0,
            0, 0, 1, 0,
            0, 0, 0, 1
        ]);

        var exception = Assert.Throws<InvalidOperationException>(() => builder.Build());
        Assert.Contains("FRAME_WORLD_TRANSFORM", exception.Message, StringComparison.Ordinal);
    }

    [Fact]
    public void Missing_frame_parent_fails_closed()
    {
        var builder = CadApplication.CreateBuilder();
        builder.Frame("world", SemanticFrameKind.World);
        builder.Frame("part", SemanticFrameKind.Part, "missing");

        var exception = Assert.Throws<InvalidOperationException>(() => builder.Build());
        Assert.Contains("FRAME_PARENT_MISSING", exception.Message, StringComparison.Ordinal);
    }

    [Fact]
    public void Frame_cycle_is_rejected()
    {
        var builder = CadApplication.CreateBuilder();
        builder.Frame("a", SemanticFrameKind.Document, "b");
        builder.Frame("b", SemanticFrameKind.Part, "a");

        var exception = Assert.Throws<InvalidOperationException>(() => builder.Build());
        Assert.Contains("FRAME_CYCLE", exception.Message, StringComparison.Ordinal);
    }

    [Fact]
    public void Equivalent_frame_definitions_have_stable_build_identity()
    {
        static string Build(bool reverse)
        {
            var builder = CadApplication.CreateBuilder();
            builder.ApplicationId = "frames";
            builder.Frame("world", SemanticFrameKind.World);
            if (reverse)
            {
                builder.Frame("body", SemanticFrameKind.Body, "part");
                builder.Frame("part", SemanticFrameKind.Part, "world");
            }
            else
            {
                builder.Frame("part", SemanticFrameKind.Part, "world");
                builder.Frame("body", SemanticFrameKind.Body, "part");
            }

            using var app = builder.Build();
            return app.Semantic.BuildIdentity;
        }

        Assert.Equal(Build(false), Build(true));
    }

    [Fact]
    public void Changing_frame_orientation_changes_build_identity()
    {
        static string Build(double translation)
        {
            var builder = CadApplication.CreateBuilder();
            builder.ApplicationId = "frame-identity";
            builder.Frame("world", SemanticFrameKind.World);
            builder.Frame("part", SemanticFrameKind.Part, "world", [
                1, 0, 0, translation,
                0, 1, 0, 0,
                0, 0, 1, 0,
                0, 0, 0, 1
            ]);

            using var app = builder.Build();
            return app.Semantic.BuildIdentity;
        }

        Assert.NotEqual(Build(0), Build(10));
    }
    
    private static CadApplication Build(Action<CadApplicationBuilder> configure)
    {
        var builder = CadApplication.CreateBuilder();
        configure(builder);
        return builder.Build();
    }
}
