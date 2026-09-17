using System.Net;
using System.Net.Http;
using Microsoft.Extensions.Options;
using UMLCAD.Framework;
using UMLCAD.Framework.Semantics;
using UMLCAD.Kernel.Client;
using Xunit;

namespace UMLCAD.Framework.Tests;

public sealed class RustKernelEndToEndTests
{
    private static readonly Uri KernelBaseAddress = new(
        Environment.GetEnvironmentVariable("UMLCAD_KERNEL_URL") ?? "http://127.0.0.1:8080/");

    [Fact]
    [Trait("Category", "KernelEndToEnd")]
    public async Task Real_framework_package_reaches_rust_and_returns_valid_compiled_model()
    {
        using var app = BuildApplication("e2e-valid", includeInvalidGeometry: false, includeStaleConstraint: false);
        var result = await Service().EvaluateAsync(app.CreateBuildPackage());

        Assert.True(result.Succeeded, string.Join("; ", result.Diagnostics.Select(x => $"{x.Code}: {x.Message}")));
        Assert.Empty(result.Diagnostics);
        var compiled = Assert.IsType<CompiledModelPackage>(result.CompiledModel);
        Assert.Equal("uml-cad-compiled-model/1.1.0", compiled.Schema);
        Assert.Equal(app.Semantic.BuildIdentity, compiled.BuildIdentity);
        Assert.Equal(app.Semantic.Id, compiled.ApplicationId);
        Assert.Contains(compiled.Manifest.RootNodeIds, x => x == "definition:part:bracket");
        Assert.Contains(compiled.Manifest.Nodes, x => x.Id == "definition:part:bracket/geometry:line-1");
        Assert.Contains(compiled.Manifest.Nodes, x => x.Id == "definition:part:bracket/constraint:constraint-1");
    }

    [Fact]
    [Trait("Category", "KernelEndToEnd")]
    public async Task Rust_validation_diagnostics_cross_the_real_http_boundary()
    {
        using var app = BuildApplication("e2e-invalid-geometry", includeInvalidGeometry: true, includeStaleConstraint: false);
        var result = await Service().EvaluateAsync(app.CreateBuildPackage());

        Assert.False(result.Succeeded);
        var diagnostic = Assert.Single(result.Diagnostics);
        Assert.Equal("INVALID_GEOMETRY", diagnostic.Code);
        Assert.Null(result.CompiledModel);
    }

    [Fact]
    [Trait("Category", "KernelEndToEnd")]
    public async Task Rust_stale_reference_diagnostic_crosses_the_real_boundary()
    {
        using var app = BuildApplication("e2e-stale-reference", includeInvalidGeometry: false, includeStaleConstraint: true);
        var result = await Service().EvaluateAsync(app.CreateBuildPackage());

        Assert.False(result.Succeeded);
        Assert.Contains(result.Diagnostics, x => x.Code == "STALE_REFERENCE");
        Assert.Null(result.CompiledModel);
    }

    [Fact]
    [Trait("Category", "KernelEndToEnd")]
    public async Task Concurrent_real_requests_are_independent_and_deterministic()
    {
        using var app = BuildApplication("e2e-concurrency", includeInvalidGeometry: false, includeStaleConstraint: false);
        var package = app.CreateBuildPackage();

        var results = await Task.WhenAll(
            Enumerable.Range(0, 8).Select(_ => Service().EvaluateAsync(package)));

        Assert.All(results, x => Assert.True(x.Succeeded));
        foreach (var result in results)
        {
            var compiled = Assert.IsType<CompiledModelPackage>(result.CompiledModel);
            Assert.Equal(package.BuildIdentity, compiled.BuildIdentity);
            Assert.Contains("definition:part:bracket", compiled.Manifest.RootNodeIds);
            Assert.Empty(result.Diagnostics);
        }
    }

    [Fact]
    [Trait("Category", "KernelEndToEnd")]
    public async Task Health_endpoint_is_reachable_from_the_same_runner_environment()
    {
        using var client = new HttpClient { BaseAddress = KernelBaseAddress };
        using var response = await client.GetAsync("health");
        Assert.Equal(HttpStatusCode.OK, response.StatusCode);
        Assert.Equal("{\"status\":\"ok\"}", await response.Content.ReadAsStringAsync());
    }

    private static IRustKernelService Service()
    {
        var client = new HttpClient { BaseAddress = KernelBaseAddress, Timeout = TimeSpan.FromSeconds(10) };
        return new RustKernelService(client, Options.Create(new RustKernelOptions
        {
            BaseAddress = KernelBaseAddress,
            RequestTimeout = TimeSpan.FromSeconds(10),
            MaxResponseBytes = 16 * 1024 * 1024
        }));
    }

    private static CadApplication BuildApplication(
        string id,
        bool includeInvalidGeometry,
        bool includeStaleConstraint)
    {
        var builder = CadApplication.CreateBuilder();
        builder.ApplicationId = id;
        builder.Version = "1.0.0";
        builder.Configuration["cad:units"] = "mm";

        builder.AddPart("bracket", "mechanical-part", part =>
        {
            part.Parameter("length", "100", "mm")
                .Geometry("line-1", "line", includeInvalidGeometry
                    ? new Dictionary<string, string>
                    {
                        ["start"] = "0,0",
                        ["end"] = "0,0"
                    }
                    : new Dictionary<string, string>
                    {
                        ["start"] = "0,0",
                        ["end"] = "100,0"
                    })
                .Constraint(
                    "constraint-1",
                    "horizontal",
                    ["line-1"],
                    new Dictionary<string, string>())
                .Constraint(
                    "missing-constraint",
                    "horizontal",
                    includeStaleConstraint ? ["missing-line"] : ["line-1"],
                    new Dictionary<string, string>());
        });

        builder.AddDrawing("drawing-1", "Bracket Drawing", drawing =>
            drawing.PartReference("bracket")
                .Setting("units", "mm")
                .Sheet("sheet-1", "A3"));

        return builder.Build();
    }
}
