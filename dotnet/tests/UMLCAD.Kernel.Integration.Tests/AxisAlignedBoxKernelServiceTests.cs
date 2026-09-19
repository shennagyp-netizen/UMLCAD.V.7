using System.Net;
using System.Text.Json;
using Microsoft.Extensions.Options;
using UMLCAD.Kernel.Client;
using Xunit;

namespace UMLCAD.Kernel.Integration.Tests;

public sealed class AxisAlignedBoxKernelServiceTests
{
    [Fact]
    public async Task Client_evaluates_box_and_preserves_authoritative_evidence()
    {
        var service = CreateService();

        var result = await service.EvaluateAsync(new AxisAlignedBoxEvaluationRequest(
            "eval-box-client",
            "result-box-client",
            [0, 0, 0],
            [1, 2, 3],
            0,
            1.0e-12));

        Assert.True(result.Succeeded, Diagnostics(result));
        Assert.Equal("eval-box-client", result.EvaluationIdentity);
        Assert.Equal("result-box-client", result.ResultIdentity);
        Assert.Equal([0d, 0d, 0d], result.Minimum);
        Assert.Equal([1d, 2d, 3d], result.Maximum);
        Assert.Equal(6d, result.Volume);
        Assert.Equal(22d, result.SurfaceArea);
        Assert.Equal([0.5d, 1d, 1.5d], result.Centroid);
    }

    [Fact]
    public async Task Client_rejects_invalid_request_before_transport()
    {
        var service = CreateService();

        await Assert.ThrowsAsync<ArgumentException>(() =>
            service.EvaluateAsync(new AxisAlignedBoxEvaluationRequest(
                "eval",
                "result",
                [0, 0],
                [1, 2, 3],
                0,
                0)));

        await Assert.ThrowsAsync<ArgumentOutOfRangeException>(() =>
            service.EvaluateAsync(new AxisAlignedBoxEvaluationRequest(
                "eval",
                "result",
                [0, 0, 0],
                [1, 2, 3],
                -1,
                0)));

        await Assert.ThrowsAsync<ArgumentException>(() =>
            service.EvaluateAsync(new AxisAlignedBoxEvaluationRequest(
                " ",
                "result",
                [0, 0, 0],
                [1, 2, 3],
                0,
                0)));
    }

    [Fact]
    public async Task Client_rejects_non_finite_numeric_input_before_transport()
    {
        var service = CreateService();

        await Assert.ThrowsAsync<ArgumentException>(() =>
            service.EvaluateAsync(new AxisAlignedBoxEvaluationRequest(
                "eval",
                "result",
                [0, 0, double.NaN],
                [1, 2, 3],
                0,
                0)));

        await Assert.ThrowsAsync<ArgumentException>(() =>
            service.EvaluateAsync(new AxisAlignedBoxEvaluationRequest(
                "eval",
                "result",
                [0, 0, 0],
                [1, double.PositiveInfinity, 3],
                0,
                0)));
    }

    [Fact]
    public async Task Client_rejects_degenerate_box_from_kernel()
    {
        var service = CreateService();

        var result = await service.EvaluateAsync(new AxisAlignedBoxEvaluationRequest(
            "eval-degenerate",
            "result-degenerate",
            [0, 0, 0],
            [0, 2, 3],
            0,
            1.0e-12));

        Assert.False(result.Succeeded);
        Assert.Contains(result.Diagnostics, x => x.Code == "KERNEL_BOX_EVALUATION");
    }

    [Fact]
    public async Task Client_preserves_kernel_http_failure_as_diagnostic()
    {
        var service = new AxisAlignedBoxKernelService(
            new HttpClient(new StaticFailureHandler())
            {
                BaseAddress = new Uri("http://127.0.0.1/")
            },
            Options.Create(new RustKernelOptions
            {
                BaseAddress = new Uri("http://127.0.0.1/"),
                RequestTimeout = TimeSpan.FromSeconds(5),
                MaxResponseBytes = 1024
            }));

        var result = await service.EvaluateAsync(new AxisAlignedBoxEvaluationRequest(
            "eval",
            "result",
            [0, 0, 0],
            [1, 1, 1],
            0,
            0));

        Assert.False(result.Succeeded);
        Assert.Contains(result.Diagnostics, x => x.Code == "KERNEL_HTTP");
    }

    private static IAxisAlignedBoxKernelService CreateService() =>
        new AxisAlignedBoxKernelService(
            new HttpClient
            {
                BaseAddress = new Uri(
                    Environment.GetEnvironmentVariable("UMLCAD_KERNEL_URL")
                    ?? "http://127.0.0.1:8080/")
            },
            Options.Create(new RustKernelOptions
            {
                BaseAddress = new Uri(
                    Environment.GetEnvironmentVariable("UMLCAD_KERNEL_URL")
                    ?? "http://127.0.0.1:8080/"),
                RequestTimeout = TimeSpan.FromSeconds(20),
                MaxResponseBytes = 32 * 1024 * 1024
            }));

    private static string Diagnostics(AxisAlignedBoxEvaluationResponse result) =>
        string.Join("; ", result.Diagnostics.Select(x => $"{x.Code}: {x.Message}"));

    private sealed class StaticFailureHandler : HttpMessageHandler
    {
        protected override Task<HttpResponseMessage> SendAsync(
            HttpRequestMessage request,
            CancellationToken cancellationToken) =>
            Task.FromResult(new HttpResponseMessage(HttpStatusCode.BadGateway)
            {
                Content = new StringContent("{"error":"upstream-failure"}")
            });
    }
}
