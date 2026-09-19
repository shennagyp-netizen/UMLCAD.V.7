using Microsoft.Extensions.DependencyInjection;
using UMLCAD.Framework;
using UMLCAD.Framework.Semantics;
using UMLCAD.Kernel.Client;
using UMLCAD.SystemCad;
using Xunit;

namespace UMLCAD.Framework.Tests;

public sealed class SystemCadEvaluationTests
{
    [Fact]
    [Trait("Gate", "RED")]
    public async Task Evaluates_cube_from_semantic_part_through_kernel_and_returns_authoritative_result()
    {
        var builder = CadApplication.CreateBuilder();
        builder.AddPart("cube", "solid", part =>
            part.Geometry("body", "solid", new Dictionary<string, string>
            {
                ["primitive"] = "cube",
                ["size"] = "10"
            }));

        using var app = builder.Build();

        var evaluator = CreateEvaluator(app);
        var result = await evaluator.EvaluateCubeAsync(
            app.Semantic,
            "cube",
            "body",
            "world");

        Assert.True(result.Succeeded, Diagnostics(result));
        Assert.Equal("cube", result.ProducerId);
        Assert.Equal("body", result.SourceGeometryId);
        Assert.Equal("world", result.FrameId);
        Assert.Equal(1000d, result.Volume);
        Assert.Equal(600d, result.SurfaceArea);
        Assert.Equal([5d, 5d, 5d], result.Centroid);
        Assert.NotNull(result.AuthoritativeResult);
    }

    [Fact]
    [Trait("Gate", "RED")]
    public async Task Cube_evaluation_produces_deterministic_face_publications()
    {
        var builder = CadApplication.CreateBuilder();
        builder.Frame("world", SemanticFrameKind.World);
        builder.AddPart("cube", "solid", part =>
            part.Geometry("body", "solid", new Dictionary<string, string>
            {
                ["primitive"] = "cube",
                ["size"] = "10"
            }));

        using var app = builder.Build();

        var evaluator = CreateEvaluator(app);
        var first = await evaluator.EvaluateCubeAsync(app.Semantic, "cube", "body", "world");
        var second = await evaluator.EvaluateCubeAsync(app.Semantic, "cube", "body", "world");

        Assert.True(first.Succeeded, Diagnostics(first));
        Assert.True(second.Succeeded, Diagnostics(second));
        Assert.Equal(first.ResultIdentity, second.ResultIdentity);
        Assert.Equal(first.Publications, second.Publications);
        Assert.Equal(6, first.Publications.Count);
    }

    [Fact]
    public async Task Cube_evaluation_rejects_missing_geometry()
    {
        var builder = CadApplication.CreateBuilder();
        builder.Frame("world", SemanticFrameKind.World);
        builder.AddPart("cube", "solid");

        using var app = builder.Build();

        var evaluator = CreateEvaluator(app);
        var result = await evaluator.EvaluateCubeAsync(app.Semantic, "cube", "missing", "world");

        Assert.False(result.Succeeded);
        Assert.Equal("SOLID_GEOMETRY_MISSING", result.DiagnosticCode);
    }

    [Fact]
    public async Task Cube_evaluation_rejects_non_cube_solid_semantics()
    {
        var builder = CadApplication.CreateBuilder();
        builder.Frame("world", SemanticFrameKind.World);
        builder.AddPart("part", "solid", part =>
            part.Geometry("body", "solid", new Dictionary<string, string>
            {
                ["primitive"] = "sphere",
                ["size"] = "10"
            }));

        using var app = builder.Build();

        var evaluator = CreateEvaluator(app);
        var result = await evaluator.EvaluateCubeAsync(app.Semantic, "part", "body", "world");

        Assert.False(result.Succeeded);
        Assert.Equal("SOLID_UNSUPPORTED_PRIMITIVE", result.DiagnosticCode);
    }

    [Fact]
    public async Task Cube_evaluation_rejects_missing_or_invalid_frame()
    {
        var builder = CadApplication.CreateBuilder();
        builder.AddPart("cube", "solid", part =>
            part.Geometry("body", "solid", new Dictionary<string, string>
            {
                ["primitive"] = "cube",
                ["size"] = "10"
            }));

        using var app = builder.Build();

        var evaluator = CreateEvaluator(app);
        var missing = await evaluator.EvaluateCubeAsync(app.Semantic, "cube", "body", "missing");

        Assert.False(missing.Succeeded);
        Assert.Equal("FRAME_MISSING", missing.DiagnosticCode);
    }

    private static ICubeEvaluationService CreateEvaluator(CadApplication app)
    {
        var kernel = new AxisAlignedBoxKernelService(
            new HttpClient
            {
                BaseAddress = new Uri(
                    Environment.GetEnvironmentVariable("UMLCAD_KERNEL_URL")
                    ?? "http://127.0.0.1:8080/")
            },
            Microsoft.Extensions.Options.Options.Create(new RustKernelOptions
            {
                BaseAddress = new Uri(
                    Environment.GetEnvironmentVariable("UMLCAD_KERNEL_URL")
                    ?? "http://127.0.0.1:8080/"),
                RequestTimeout = TimeSpan.FromSeconds(20),
                MaxResponseBytes = 32 * 1024 * 1024
            }));

        return new CubeEvaluationService(
            kernel,
            app.GetRequiredService<ISemanticFrameService>(),
            app.GetRequiredService<IAuthoritativeResultIntegrationService>());
    }

    private static string Diagnostics(CubeEvaluationResult result) =>
        string.Join("; ", result.Diagnostics.Select(x => $"{x.Code}: {x.Message}"));
}
