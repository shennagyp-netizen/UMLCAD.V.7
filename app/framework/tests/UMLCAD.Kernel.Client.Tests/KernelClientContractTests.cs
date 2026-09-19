using System.Net;
using System.Text;
using System.Text.Json;
using UMLCAD.Cad.Contracts;
using UMLCAD.Kernel.Client;

namespace UMLCAD.Kernel.Client.Tests;

public sealed class KernelClientContractTests
{
    [Fact]
    public async Task SuccessfulResponseRoundTripsAndPreservesRequestIdentity()
    {
        var request = CreateRequest();
        var handler = new RecordingHandler(_ =>
        {
            var response = KernelOperationResponse.Success(
                request,
                new CadResultId("result:1"),
                "evidence:1",
                new[] { new KernelTopologyBinding("body", "face:1") });

            return JsonResponse(response);
        });

        using var client = new UmlcadKernelClient(
            new UmlcadKernelClientOptions(
                new Uri("http://kernel.test/"),
                "v2/cad/operation",
                TimeSpan.FromSeconds(5),
                1024 * 1024),
            new HttpClient(handler));

        var result = await client.EvaluateAsync(request);

        Assert.Equal(CadEvaluationStatus.Succeeded, result.Status);
        Assert.Equal(request.EvaluationIdentity, result.EvaluationIdentity);
        Assert.Equal(request.OperationId, result.OperationId);
        Assert.Equal(new CadResultId("result:1"), result.AuthoritativeResultId);
        Assert.Equal("POST", handler.Method);
        Assert.Equal("/v2/cad/operation", handler.Path);
    }

    [Fact]
    public async Task MismatchedResponseIdentityFailsClosed()
    {
        var request = CreateRequest();
        var handler = new RecordingHandler(_ =>
        {
            var response = KernelOperationResponse.Success(
                    request,
                    new CadResultId("result:1"),
                    "evidence:1")
                with
                {
                    OperationId = new CadId("other-operation")
                };

            return JsonResponse(response);
        });

        using var client = CreateClient(handler);
        var result = await client.EvaluateAsync(request);

        Assert.Equal(CadEvaluationStatus.Failed, result.Status);
        Assert.Null(result.AuthoritativeResultId);
        Assert.Contains(
            result.Diagnostics,
            x => x.Code == "KERNEL_INVALID_RESULT");
    }

    [Fact]
    public async Task FailedResponseCannotClaimAuthoritativeResult()
    {
        var request = CreateRequest();
        var handler = new RecordingHandler(_ =>
        {
            var response = new KernelOperationResponse(
                request.ContractVersion,
                request.EvaluationIdentity,
                request.OperationId,
                CadEvaluationStatus.Failed,
                new CadResultId("forged-result"),
                "forged-evidence",
                Array.Empty<KernelTopologyBinding>(),
                Array.Empty<CadDiagnostic>());

            return JsonResponse(response);
        });

        using var client = CreateClient(handler);
        var result = await client.EvaluateAsync(request);

        Assert.Equal(CadEvaluationStatus.Failed, result.Status);
        Assert.Null(result.AuthoritativeResultId);
        Assert.Contains(
            result.Diagnostics,
            x => x.Code == "KERNEL_INVALID_RESULT");
    }

    [Fact]
    public async Task ResponseSizeLimitIsEnforcedBeforeParsing()
    {
        var request = CreateRequest();
        var handler = new RecordingHandler(_ =>
        {
            var payload = new string('x', 4096);
            return new HttpResponseMessage(HttpStatusCode.OK)
            {
                Content = new StringContent(payload, Encoding.UTF8, "application/json")
            };
        });

        using var client = new UmlcadKernelClient(
            new UmlcadKernelClientOptions(
                new Uri("http://kernel.test/"),
                "v2/cad/operation",
                TimeSpan.FromSeconds(5),
                256),
            new HttpClient(handler));

        var result = await client.EvaluateAsync(request);

        Assert.Equal(CadEvaluationStatus.Failed, result.Status);
        Assert.Contains(
            result.Diagnostics,
            x => x.Code == "KERNEL_RESPONSE_TOO_LARGE");
    }

    [Fact]
    public async Task CallerCancellationIsNotConvertedIntoKernelTimeout()
    {
        var request = CreateRequest();
        var handler = new RecordingHandler(async cancellationToken =>
        {
            await Task.Delay(Timeout.InfiniteTimeSpan, cancellationToken);
            return new HttpResponseMessage(HttpStatusCode.OK);
        });

        using var client = CreateClient(
            handler,
            TimeSpan.FromMinutes(1));

        using var cancellation = new CancellationTokenSource();
        var operation = client.EvaluateAsync(request, cancellation.Token);
        cancellation.Cancel();

        await Assert.ThrowsAnyAsync<OperationCanceledException>(
            async () => await operation);
    }

    private static UmlcadKernelClient CreateClient(
        HttpMessageHandler handler,
        TimeSpan? timeout = null) =>
        new(
            new UmlcadKernelClientOptions(
                new Uri("http://kernel.test/"),
                "v2/cad/operation",
                timeout ?? TimeSpan.FromSeconds(5),
                1024 * 1024),
            new HttpClient(handler));

    private static KernelOperationRequest CreateRequest() =>
        new(
            CadContractVersions.KernelOperation,
            new CadEvaluationIdentity("sha256:eval"),
            "part:1",
            new CadId("operation:1"),
            "Cad.Extrusion",
            KernelEvaluationMode.Full,
            null,
            Array.Empty<CadResultId>(),
            new Dictionary<string, string>
            {
                ["distance"] = "const:20",
                ["direction"] = "+Z"
            });

    private static HttpResponseMessage JsonResponse(
        KernelOperationResponse response) =>
        new(HttpStatusCode.OK)
        {
            Content = new StringContent(
                JsonSerializer.Serialize(response),
                Encoding.UTF8,
                "application/json")
        };

    private sealed class RecordingHandler(
        Func<CancellationToken, Task<HttpResponseMessage>> responder)
        : HttpMessageHandler
    {
        public string? Method { get; private set; }
        public string? Path { get; private set; }

        public RecordingHandler(
            Func<CancellationToken, HttpResponseMessage> responder)
            : this(token => Task.FromResult(responder(token)))
        {
        }

        protected override async Task<HttpResponseMessage> SendAsync(
            HttpRequestMessage request,
            CancellationToken cancellationToken)
        {
            Method = request.Method.Method;
            Path = request.RequestUri?.AbsolutePath;
            return await responder(cancellationToken);
        }
    }
}
