using System.Net.Http.Json;
using System.Text.Json;
using UMLCAD.Cad.Contracts;

namespace UMLCAD.Kernel;

public sealed record UmlcadKernelOptions(
    Uri BaseAddress,
    string EvaluationPath = "v2/cad/evaluate",
    TimeSpan? Timeout = null)
{
    public UmlcadKernelOptions() : this(
        new Uri("http://localhost:8080/"),
        "v2/cad/evaluate",
        TimeSpan.FromMinutes(2))
    {
    }
}

public sealed class UmlcadKernelGateway : IKernelGateway, IDisposable
{
    private readonly HttpClient _client;
    private readonly bool _ownsClient;
    private readonly UmlcadKernelOptions _options;

    public UmlcadKernelGateway(
        UmlcadKernelOptions? options = null,
        HttpClient? client = null)
    {
        _options = options ?? new UmlcadKernelOptions();

        if (!_options.BaseAddress.IsAbsoluteUri ||
            _options.BaseAddress.Scheme is not ("http" or "https"))
            throw new ArgumentException(
                "Kernel base address must be HTTP(S).",
                nameof(options));

        if (string.IsNullOrWhiteSpace(_options.EvaluationPath) ||
            Uri.TryCreate(
                _options.EvaluationPath,
                UriKind.Absolute,
                out _))
            throw new ArgumentException(
                "Kernel evaluation path must be relative.",
                nameof(options));

        _client = client ?? new HttpClient();
        _ownsClient = client is null;
        _client.BaseAddress = _options.BaseAddress;
        _client.Timeout = _options.Timeout ?? TimeSpan.FromMinutes(2);
    }

    public async Task<KernelOperationResponse> EvaluateAsync(
        KernelOperationRequest request,
        CancellationToken cancellationToken = default)
    {
        ArgumentNullException.ThrowIfNull(request);

        using var response = await _client.PostAsJsonAsync(
            _options.EvaluationPath,
            request,
            new JsonSerializerOptions(JsonSerializerDefaults.Web),
            cancellationToken);

        if (!response.IsSuccessStatusCode)
        {
            return new KernelOperationResponse(
                CadEvaluationStatus.Failed,
                null,
                null,
                Array.Empty<KernelTopologyBinding>(),
                new[]
                {
                    new CadDiagnostic(
                        "KERNEL_HTTP",
                        $"Kernel returned HTTP {(int)response.StatusCode}.")
                });
        }

        var value =
            await response.Content.ReadFromJsonAsync<KernelOperationResponse>(
                new JsonSerializerOptions(JsonSerializerDefaults.Web),
                cancellationToken);

        if (value is null)
        {
            return new KernelOperationResponse(
                CadEvaluationStatus.Failed,
                null,
                null,
                Array.Empty<KernelTopologyBinding>(),
                new[]
                {
                    new CadDiagnostic(
                        "KERNEL_EMPTY_RESPONSE",
                        "Kernel returned no operation response.")
                });
        }

        if (value.Status == CadEvaluationStatus.Succeeded &&
            (value.AuthoritativeResultId is null ||
             string.IsNullOrWhiteSpace(value.EvidenceHash)))
        {
            return new KernelOperationResponse(
                CadEvaluationStatus.Failed,
                null,
                null,
                Array.Empty<KernelTopologyBinding>(),
                new[]
                {
                    new CadDiagnostic(
                        "KERNEL_INVALID_RESULT",
                        "Successful response lacks result identity or evidence.")
                });
        }

        return value;
    }

    public void Dispose()
    {
        if (_ownsClient)
            _client.Dispose();
    }
}
