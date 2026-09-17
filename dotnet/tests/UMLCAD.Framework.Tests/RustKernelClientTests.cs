using System.Net;
using System.Text;
using System.Text.Json;
using Microsoft.Extensions.Options;
using UMLCAD.Framework;
using UMLCAD.Framework.Semantics;
using UMLCAD.Kernel.Client;
using Xunit;

namespace UMLCAD.Framework.Tests;

public sealed class RustKernelClientTests
{
    [Fact]
    public async Task Successful_kernel_response_is_deserialized()
    {
        using var app = CadApplication.CreateBuilder().Build();
        var payload = JsonSerializer.Serialize(new KernelEvaluationResult(true, null, []), JsonOptions);
        var service = new RustKernelService(Client(new StaticHandler(JsonResponse(payload))), Options.Create(new RustKernelOptions()));
        var result = await service.EvaluateAsync(app.CreateBuildPackage());
        Assert.True(result.Succeeded);
        Assert.Empty(result.Diagnostics);
    }

    [Theory]
    [InlineData(HttpStatusCode.BadRequest)]
    [InlineData(HttpStatusCode.InternalServerError)]
    [InlineData(HttpStatusCode.ServiceUnavailable)]
    public async Task Http_failures_have_stable_codes(HttpStatusCode status)
    {
        using var app = CadApplication.CreateBuilder().Build();
        var service = new RustKernelService(Client(new StaticHandler(new HttpResponseMessage(status))), Options.Create(new RustKernelOptions()));
        var result = await service.EvaluateAsync(app.CreateBuildPackage());
        Assert.Equal("KERNEL_HTTP", Assert.Single(result.Diagnostics).Code);
    }

    [Fact]
    public async Task Empty_and_invalid_json_responses_are_rejected()
    {
        using var app = CadApplication.CreateBuilder().Build();
        var empty = new RustKernelService(Client(new StaticHandler(new HttpResponseMessage(HttpStatusCode.OK) { Content = new StringContent("") })), Options.Create(new RustKernelOptions()));
        Assert.Equal("KERNEL_EMPTY_RESPONSE", Assert.Single((await empty.EvaluateAsync(app.CreateBuildPackage())).Diagnostics).Code);

        var invalid = new RustKernelService(Client(new StaticHandler(new HttpResponseMessage(HttpStatusCode.OK) { Content = new StringContent("not-json", Encoding.UTF8, "application/json") })), Options.Create(new RustKernelOptions()));
        Assert.Equal("KERNEL_INVALID_JSON", Assert.Single((await invalid.EvaluateAsync(app.CreateBuildPackage())).Diagnostics).Code);
    }

    [Fact]
    public async Task Response_size_limit_is_enforced_without_ContentLength()
    {
        using var app = CadApplication.CreateBuilder().Build();
        var content = new StringContent(new string('x', 20_000), Encoding.UTF8, "application/json");
        content.Headers.ContentLength = null;
        var service = new RustKernelService(Client(new StaticHandler(new HttpResponseMessage(HttpStatusCode.OK) { Content = content })), Options.Create(new RustKernelOptions { MaxResponseBytes = 1024 }));
        var result = await service.EvaluateAsync(app.CreateBuildPackage());
        Assert.Equal("KERNEL_RESPONSE_TOO_LARGE", Assert.Single(result.Diagnostics).Code);
    }

    [Fact]
    public async Task Kernel_rejects_compiled_result_with_wrong_build_identity()
    {
        using var app = CadApplication.CreateBuilder().Build();
        var package = app.CreateBuildPackage();
        var manifest = EmptyManifest("wrong");
        var compiled = new CompiledModelPackage("uml-cad-compiled-model/1.1.0", package.ApplicationId, package.ApplicationVersion, "wrong", manifest, null, []);
        var payload = JsonSerializer.Serialize(new KernelEvaluationResult(true, compiled, []), JsonOptions);
        var service = new RustKernelService(Client(new StaticHandler(JsonResponse(payload))), Options.Create(new RustKernelOptions()));
        Assert.Equal("KERNEL_INVALID_RESULT", Assert.Single((await service.EvaluateAsync(package)).Diagnostics).Code);
    }

    [Fact]
    public async Task Kernel_rejects_render_artifact_from_wrong_build()
    {
        using var app = CadApplication.CreateBuilder().Build();
        var package = app.CreateBuildPackage();
        var artifact = new CompiledRenderArtifact("artifact", "wrong", "glb", "model/gltf-binary", "asset", null, null, null, null, new Dictionary<string, string>());
        var compiled = new CompiledModelPackage("uml-cad-compiled-model/1.1.0", package.ApplicationId, package.ApplicationVersion, package.BuildIdentity, EmptyManifest(package.BuildIdentity), artifact, []);
        var payload = JsonSerializer.Serialize(new KernelEvaluationResult(true, compiled, []), JsonOptions);
        var service = new RustKernelService(Client(new StaticHandler(JsonResponse(payload))), Options.Create(new RustKernelOptions()));
        Assert.Equal("KERNEL_INVALID_RESULT", Assert.Single((await service.EvaluateAsync(package)).Diagnostics).Code);
    }

    [Fact]
    public async Task Timeout_and_caller_cancellation_are_distinguished()
    {
        using var app = CadApplication.CreateBuilder().Build();
        var timeoutService = new RustKernelService(Client(new DelayHandler(TimeSpan.FromMilliseconds(150))), Options.Create(new RustKernelOptions { RequestTimeout = TimeSpan.FromMilliseconds(20) }));
        Assert.Equal("KERNEL_TIMEOUT", Assert.Single((await timeoutService.EvaluateAsync(app.CreateBuildPackage())).Diagnostics).Code);

        var cancelService = new RustKernelService(Client(new DelayHandler(TimeSpan.FromSeconds(1))), Options.Create(new RustKernelOptions { RequestTimeout = TimeSpan.FromSeconds(5) }));
        using var cts = new CancellationTokenSource(TimeSpan.FromMilliseconds(20));
        await Assert.ThrowsAnyAsync<OperationCanceledException>(() => cancelService.EvaluateAsync(app.CreateBuildPackage(), cts.Token));
    }

    [Fact]
    public void Endpoint_validation_rejects_absolute_evaluate_path_and_invalid_limits()
    {
        var services = new Microsoft.Extensions.DependencyInjection.ServiceCollection();
        Assert.Throws<ArgumentException>(() => services.AddRustKernel(o => o.EvaluatePath = "https://evil.example/evaluate"));
        Assert.Throws<ArgumentOutOfRangeException>(() => services.AddRustKernel(o => o.RequestTimeout = TimeSpan.Zero));
        Assert.Throws<ArgumentOutOfRangeException>(() => services.AddRustKernel(o => o.MaxResponseBytes = 0));
    }

    private static HttpClient Client(HttpMessageHandler handler) => new(handler) { BaseAddress = new Uri("http://localhost:8080/") };
    private static HttpResponseMessage JsonResponse(string json) => new(HttpStatusCode.OK) { Content = new StringContent(json, Encoding.UTF8, "application/json") };
    private static CompiledModelManifest EmptyManifest(string buildIdentity) => new("uml-cad-compiled-model/1.1.0", "uml-cad-application", "1.0.0", buildIdentity, [], [], [], [], [], [], []);
    private static readonly JsonSerializerOptions JsonOptions = new(JsonSerializerDefaults.Web);

    private sealed class StaticHandler(HttpResponseMessage response) : HttpMessageHandler
    {
        protected override Task<HttpResponseMessage> SendAsync(HttpRequestMessage request, CancellationToken cancellationToken) => Task.FromResult(response);
    }

    private sealed class DelayHandler(TimeSpan delay) : HttpMessageHandler
    {
        protected override async Task<HttpResponseMessage> SendAsync(HttpRequestMessage request, CancellationToken cancellationToken)
        {
            await Task.Delay(delay, cancellationToken);
            return JsonResponse(JsonSerializer.Serialize(new KernelEvaluationResult(true, null, []), JsonOptions));
        }
    }
}
