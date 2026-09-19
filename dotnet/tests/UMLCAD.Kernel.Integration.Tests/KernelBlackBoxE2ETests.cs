using System.Net;
using System.Net.Http.Json;
using System.Text.Json;
using UMLCAD.Framework;
using UMLCAD.Framework.Semantics;
using UMLCAD.Kernel.Client;
using Xunit;

namespace UMLCAD.Kernel.Integration.Tests;

public sealed class KernelBlackBoxE2ETests
{
    private static readonly Uri KernelBaseAddress = new(
        Environment.GetEnvironmentVariable("UMLCAD_KERNEL_URL") ?? "http://127.0.0.1:8080/");

    [Fact]
    [Trait("Category", "KernelIntegration")]
    public async Task Production_client_round_trips_a_drawn_rectangle_and_preserves_graph_identity()
    {
        var app = BuildRectangleApplication("blackbox-rectangle");
        var package = app.CreateBuildPackage();
        var result = await Service().EvaluateAsync(package);

        Assert.True(result.Succeeded, Diagnostics(result));
        Assert.Empty(result.Diagnostics);
        var compiled = Assert.IsType<CompiledModelPackage>(result.CompiledModel);

        Assert.Equal(package.BuildIdentity, compiled.BuildIdentity);
        Assert.Equal(package.ApplicationId, compiled.ApplicationId);
        Assert.Equal("uml-cad-compiled-model/1.1.0", compiled.Schema);
        Assert.Contains("definition:part:plate", compiled.Manifest.RootNodeIds);
        Assert.Contains(compiled.Manifest.Nodes, x => x.Id == "definition:part:plate/geometry:bottom");
        Assert.Contains(compiled.Manifest.Nodes, x => x.Id == "definition:part:plate/geometry:right");
        Assert.Contains(compiled.Manifest.Nodes, x => x.Id == "definition:part:plate/geometry:top");
        Assert.Contains(compiled.Manifest.Nodes, x => x.Id == "definition:part:plate/geometry:left");
        Assert.Contains(compiled.Manifest.Nodes, x => x.Id == "definition:part:plate/constraint:horizontal-bottom");
    }

    [Fact]
    [Trait("Category", "KernelIntegration")]
    public async Task Axis_aligned_box_contract_returns_authoritative_numeric_evidence()
    {
        using var client = new HttpClient { BaseAddress = KernelBaseAddress, Timeout = TimeSpan.FromSeconds(10) };
        var payload = new
        {
            schema = "uml-cad-axis-aligned-box/1.0.0",
            evaluationIdentity = "eval-box-1",
            resultIdentity = "result-box-1",
            min = new[] { 0.0, 0.0, 0.0 },
            max = new[] { 1.0, 2.0, 3.0 },
            absoluteTolerance = 0.0,
            relativeTolerance = 1.0e-12
        };

        using var response = await client.PostAsJsonAsync(
            "v1/solid/axis-aligned-box/evaluate",
            payload);

        Assert.Equal(HttpStatusCode.OK, response.StatusCode);
        var body = await response.Content.ReadFromJsonAsync<JsonDocument>();
        Assert.NotNull(body);
        var root = body!.RootElement;

        Assert.True(root.GetProperty("succeeded").GetBoolean());
        var result = root.GetProperty("result");
        Assert.Equal("eval-box-1", result.GetProperty("evaluationIdentity").GetString());
        Assert.Equal("result-box-1", result.GetProperty("resultIdentity").GetString());
        Assert.Equal(6.0, result.GetProperty("volume").GetDouble());
        Assert.Equal(22.0, result.GetProperty("surfaceArea").GetDouble());
        Assert.Equal(0.5, result.GetProperty("centroid")[0].GetDouble());
        Assert.Equal(1.0, result.GetProperty("centroid")[1].GetDouble());
        Assert.Equal(1.5, result.GetProperty("centroid")[2].GetDouble());
    }

    [Fact]
    [Trait("Category", "KernelIntegration")]
    public async Task Axis_aligned_box_contract_rejects_degenerate_geometry()
    {
        using var client = new HttpClient { BaseAddress = KernelBaseAddress, Timeout = TimeSpan.FromSeconds(10) };
        var payload = new
        {
            schema = "uml-cad-axis-aligned-box/1.0.0",
            evaluationIdentity = "eval-box-degenerate",
            resultIdentity = "result-box-degenerate",
            min = new[] { 0.0, 0.0, 0.0 },
            max = new[] { 0.0, 2.0, 3.0 },
            absoluteTolerance = 0.0,
            relativeTolerance = 1.0e-12
        };

        using var response = await client.PostAsJsonAsync(
            "v1/solid/axis-aligned-box/evaluate",
            payload);

        Assert.Equal(HttpStatusCode.UnprocessableEntity, response.StatusCode);
        Assert.Contains("KERNEL_BOX_EVALUATION", await response.Content.ReadAsStringAsync());
    }

    [Fact]
    [Trait("Category", "KernelIntegration")]
    public async Task Axis_aligned_box_contract_rejects_non_finite_geometry()
    {
        using var client = new HttpClient { BaseAddress = KernelBaseAddress, Timeout = TimeSpan.FromSeconds(10) };
        var payload = """
            {
              "schema":"uml-cad-axis-aligned-box/1.0.0",
              "evaluationIdentity":"eval-box-nonfinite",
              "resultIdentity":"result-box-nonfinite",
              "min":[0,0,0],
              "max":[1,2,"Infinity"],
              "absoluteTolerance":0,
              "relativeTolerance":1e-12
            }
            """;

        using var response = await client.PostAsync(
            "v1/solid/axis-aligned-box/evaluate",
            JsonContent.Create(JsonSerializer.Deserialize<JsonDocument>(payload)!.RootElement));

        Assert.Equal(HttpStatusCode.UnprocessableEntity, response.StatusCode);
        Assert.Contains("KERNEL_BOX_EVALUATION", await response.Content.ReadAsStringAsync());
    }

    [Fact]
    [Trait("Category", "KernelIntegration")]
    public async Task Axis_aligned_box_contract_is_repeat_deterministic()
    {
        using var client = new HttpClient { BaseAddress = KernelBaseAddress, Timeout = TimeSpan.FromSeconds(10) };
        var payload = new
        {
            schema = "uml-cad-axis-aligned-box/1.0.0",
            evaluationIdentity = "eval-box-repeat",
            resultIdentity = "result-box-repeat",
            min = new[] { -2.0, 4.0, 8.0 },
            max = new[] { 3.0, 10.0, 11.0 },
            absoluteTolerance = 0.0,
            relativeTolerance = 1.0e-12
        };

        using var first = await client.PostAsJsonAsync("v1/solid/axis-aligned-box/evaluate", payload);
        using var second = await client.PostAsJsonAsync("v1/solid/axis-aligned-box/evaluate", payload);

        Assert.Equal(HttpStatusCode.OK, first.StatusCode);
        Assert.Equal(HttpStatusCode.OK, second.StatusCode);
        Assert.Equal(
            await first.Content.ReadAsStringAsync(),
            await second.Content.ReadAsStringAsync());
    }

    [Fact]
    [Trait("Category", "KernelIntegration")]
    public async Task Invalid_geometry_is_rejected_by_the_real_kernel_process()
    {
        var app = BuildRectangleApplication("blackbox-invalid");
        var invalid = ReplacePackageGeometry(app.CreateBuildPackage(), "bottom", "0,0", "0,0");

        var result = await Service().EvaluateAsync(invalid);

        Assert.False(result.Succeeded);
        Assert.Null(result.CompiledModel);
        Assert.Contains(result.Diagnostics, x => x.Code == "INVALID_GEOMETRY");
    }

    [Fact]
    [Trait("Category", "KernelIntegration")]
    public async Task Missing_reference_is_reported_across_the_real_http_boundary()
    {
        var app = BuildRectangleApplication("blackbox-stale-reference");
        var package = ReplaceConstraintReference(app.CreateBuildPackage(), "missing-edge");

        var result = await Service().EvaluateAsync(package);

        Assert.False(result.Succeeded);
        Assert.Null(result.CompiledModel);
        Assert.Contains(result.Diagnostics, x => x.Code == "STALE_REFERENCE");
    }

    [Fact]
    [Trait("Category", "KernelIntegration")]
    public async Task Concurrent_requests_do_not_cross_contaminate_identity_or_results()
    {
        var apps = Enumerable.Range(0, 12)
            .Select(i => BuildRectangleApplication($"blackbox-concurrent-{i}"))
            .ToArray();

        var results = await Task.WhenAll(apps.Select(app => Service().EvaluateAsync(app.CreateBuildPackage())));

        Assert.All(results, result => Assert.True(result.Succeeded, Diagnostics(result)));
        for (var i = 0; i < results.Length; i++)
        {
            var expected = apps[i].Semantic;
            var compiled = Assert.IsType<CompiledModelPackage>(results[i].CompiledModel);
            Assert.Equal(expected.BuildIdentity, compiled.BuildIdentity);
            Assert.Equal(expected.Id, compiled.ApplicationId);
        }
    }

    [Fact]
    [Trait("Category", "KernelIntegration")]
    public async Task Kernel_rejects_malformed_schema_and_missing_semantic_payloads_fail_closed()
    {
        using var client = new HttpClient { BaseAddress = KernelBaseAddress, Timeout = TimeSpan.FromSeconds(10) };

        using var wrongSchema = await client.PostAsync(
            "v1/build/evaluate",
            JsonContent.Create(new { schema = "attack/0", semantic = new { parts = Array.Empty<object>() } }));
        Assert.Equal(HttpStatusCode.UnprocessableEntity, wrongSchema.StatusCode);
        Assert.Contains("KERNEL_BUILD_SCHEMA", await wrongSchema.Content.ReadAsStringAsync());

        using var missingSemantic = await client.PostAsync(
            "v1/build/evaluate",
            JsonContent.Create(new { schema = "uml-cad-build-package/1.0.0" }));
        Assert.Equal(HttpStatusCode.UnprocessableEntity, missingSemantic.StatusCode);
        Assert.Contains("KERNEL_BUILD_SCHEMA", await missingSemantic.Content.ReadAsStringAsync());
    }

    [Fact]
    [Trait("Category", "KernelIntegration")]
    public async Task Kernel_rejects_unsupported_geometry_without_process_failure()
    {
        using var client = new HttpClient { BaseAddress = KernelBaseAddress, Timeout = TimeSpan.FromSeconds(10) };
        var payload = new
        {
            schema = "uml-cad-build-package/1.0.0",
            applicationId = "blackbox-unsupported",
            applicationVersion = "1.0.0",
            buildIdentity = "unused",
            semantic = new
            {
                id = "blackbox-unsupported",
                version = "1.0.0",
                buildIdentity = "unused",
                parts = new[]
                {
                    new
                    {
                        id = "p",
                        geometry = new[]
                        {
                            new { id = "g", kind = "ellipse", properties = new { center = "0,0", radius = "5" } }
                        }
                    }
                }
            }
        };

        using var response = await client.PostAsJsonAsync("v1/build/evaluate", payload);
        Assert.Equal(HttpStatusCode.UnprocessableEntity, response.StatusCode);
        Assert.Contains("KERNEL_UNSUPPORTED_GEOMETRY", await response.Content.ReadAsStringAsync());
    }

    private static IRustKernelService Service()
    {
        var http = new HttpClient
        {
            BaseAddress = KernelBaseAddress,
            Timeout = TimeSpan.FromSeconds(20)
        };
        return new RustKernelService(
            http,
            Microsoft.Extensions.Options.Options.Create(new RustKernelOptions
            {
                BaseAddress = KernelBaseAddress,
                RequestTimeout = TimeSpan.FromSeconds(20),
                MaxResponseBytes = 32 * 1024 * 1024
            }));
    }

    private static CadApplication BuildRectangleApplication(string id)
    {
        var builder = CadApplication.CreateBuilder();
        builder.ApplicationId = id;
        builder.Version = "1.0.0";
        builder.Configuration["cad:units"] = "mm";

        builder.AddPart("plate", "machined-part", part =>
        {
            part.Name("Test Plate").PartNumber("E2E-PLATE").Material("Steel")
                .Geometry("bottom", "line", Line("0,0", "100,0"))
                .Geometry("right", "line", Line("100,0", "100,50"))
                .Geometry("top", "line", Line("100,50", "0,50"))
                .Geometry("left", "line", Line("0,50", "0,0"))
                .Geometry("hole", "circle", new Dictionary<string, string>
                {
                    ["center"] = "50,25",
                    ["radius"] = "5"
                })
                .Constraint("horizontal-bottom", "horizontal", ["bottom"], new Dictionary<string, string>())
                .Constraint("vertical-right", "vertical", ["right"], new Dictionary<string, string>());
        });

        builder.AddDrawing("drawing", "E2E Plate Drawing", drawing =>
            drawing.PartReference("plate")
                .Setting("units", "mm")
                .Sheet("sheet-1", "A3"));

        return builder.Build();
    }

    private static Dictionary<string, string> Line(string start, string end) =>
        new() { ["start"] = start, ["end"] = end };

    private static BuildPackage ReplacePackageGeometry(
        BuildPackage source,
        string geometryId,
        string start,
        string end)
    {
        var semantic = source.Semantic;
        var parts = semantic.Parts.ToArray();
        var partIndex = Array.FindIndex(parts, part => part.Geometry.Any(g => g.Id == geometryId));
        if (partIndex < 0)
            throw new InvalidOperationException($"Geometry '{geometryId}' was not found.");

        var part = parts[partIndex];
        var geometry = part.Geometry.ToArray();
        var geometryIndex = Array.FindIndex(geometry, g => g.Id == geometryId);
        var properties = new Dictionary<string, string>
        {
            ["start"] = start,
            ["end"] = end
        };
        geometry[geometryIndex] = geometry[geometryIndex] with { Properties = properties };
        parts[partIndex] = part with { Geometry = geometry };

        var mutatedSemantic = semantic with { Parts = parts };
        return source with { Semantic = mutatedSemantic };
    }

    private static BuildPackage ReplaceConstraintReference(BuildPackage source, string reference)
    {
        var semantic = source.Semantic;
        var parts = semantic.Parts.ToArray();
        if (parts.Length == 0)
            throw new InvalidOperationException("The package contains no parts.");

        var part = parts[0];
        var constraints = part.Constraints.ToList();
        constraints.Add(new ConstraintSemantic(
            "stale",
            "horizontal",
            [reference],
            new Dictionary<string, string>()));
        parts[0] = part with { Constraints = constraints };

        var mutatedSemantic = semantic with { Parts = parts };

        return source with { Semantic = mutatedSemantic };
    }

    private static string Diagnostics(KernelEvaluationResult result) =>
        string.Join("; ", result.Diagnostics.Select(x => $"{x.Code}: {x.Message}"));
}
