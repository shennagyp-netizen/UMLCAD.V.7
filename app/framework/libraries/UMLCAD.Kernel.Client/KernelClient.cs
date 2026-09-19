using System.Net.Http.Json;
using System.Text;
using System.Text.Json;
using UMLCAD.Cad.Contracts;

namespace UMLCAD.Kernel.Client;

public sealed record UmlcadKernelClientOptions(
    Uri BaseAddress,
    string OperationPath = "v2/cad/operation",
    TimeSpan? RequestTimeout = null,
    long MaxResponseBytes = 256L * 1024 * 1024)
{
    public UmlcadKernelClientOptions()
        : this(new Uri("http://127.0.0.1:8080/"))
    {
    }
}

public sealed class UmlcadKernelClient : IKernelGateway, IDisposable
{
    private readonly HttpClient _client;
    private readonly bool _ownsClient;
    private readonly TimeSpan _timeout;
    private readonly long _maxResponseBytes;

    public UmlcadKernelClient(
        UmlcadKernelClientOptions? options = null,
        HttpClient? client = null)
    {
        options ??= new UmlcadKernelClientOptions();

        if (!options.BaseAddress.IsAbsoluteUri ||
            options.BaseAddress.Scheme is not ("http" or "https"))
        {
            throw new ArgumentException(
                "Kernel BaseAddress must be an absolute HTTP(S) URI.",
                nameof(options));
        }

        if (string.IsNullOrWhiteSpace(options.OperationPath) ||
            Uri.TryCreate(options.OperationPath, UriKind.Absolute, out _))
        {
            throw new ArgumentException(
                "Kernel operation path must be non-empty and relative.",
                nameof(options));
        }

        _timeout = options.RequestTimeout ?? TimeSpan.FromMinutes(2);
        if (_timeout <= TimeSpan.Zero || _timeout > TimeSpan.FromHours(1))
            throw new ArgumentOutOfRangeException(
                nameof(options),
                "Kernel request timeout is outside the allowed range.");

        if (options.MaxResponseBytes <= 0 ||
            options.MaxResponseBytes > 2L * 1024 * 1024 * 1024)
        {
            throw new ArgumentOutOfRangeException(
                nameof(options),
                "Kernel response-size limit is outside the allowed range.");
        }

        _maxResponseBytes = options.MaxResponseBytes;
        OperationPath = options.OperationPath;

        _client = client ?? new HttpClient();
        _ownsClient = client is null;
        _client.BaseAddress = options.BaseAddress;
        _client.Timeout = _timeout;
    }

    public string OperationPath { get; }

    public async Task<KernelOperationResponse> EvaluateAsync(
        KernelOperationRequest request,
        CancellationToken cancellationToken = default)
    {
        ArgumentNullException.ThrowIfNull(request);
        ValidateRequest(request);

        using var timeoutCts =
            CancellationTokenSource.CreateLinkedTokenSource(cancellationToken);
        timeoutCts.CancelAfter(_timeout);

        try
        {
            using var response = await _client.PostAsJsonAsync(
                OperationPath,
                request,
                timeoutCts.Token);

            if (response.Content.Headers.ContentLength is > 0 and var contentLength &&
                contentLength > _maxResponseBytes)
            {
                return Failure(
                    request,
                    "KERNEL_RESPONSE_TOO_LARGE",
                    "Kernel response exceeds the configured response limit.");
            }

            var body = await ReadBodyAsync(
                response,
                _maxResponseBytes,
                timeoutCts.Token);

            if (body is null)
            {
                return Failure(
                    request,
                    "KERNEL_RESPONSE_TOO_LARGE",
                    "Kernel response exceeds the configured response limit.");
            }

            if (!response.IsSuccessStatusCode)
            {
                var detail = body.Length == 0
                    ? string.Empty
                    : $" Body: {Encoding.UTF8.GetString(body)}";

                return Failure(
                    request,
                    "KERNEL_HTTP",
                    $"Kernel returned HTTP {(int)response.StatusCode} ({response.ReasonPhrase}).{detail}");
            }

            if (body.Length == 0)
            {
                return Failure(
                    request,
                    "KERNEL_EMPTY_RESPONSE",
                    "Kernel returned an empty response.");
            }

            var result = JsonSerializer.Deserialize<KernelOperationResponse>(
                body,
                new JsonSerializerOptions(JsonSerializerDefaults.Web));

            if (result is null)
            {
                return Failure(
                    request,
                    "KERNEL_INVALID_RESPONSE",
                    "Kernel returned no usable operation response.");
            }

            try
            {
                KernelOperationResponseValidator.Validate(request, result);
            }
            catch (InvalidOperationException exception)
            {
                return Failure(
                    request,
                    "KERNEL_INVALID_RESULT",
                    exception.Message);
            }

            return result;
        }
        catch (HttpRequestException)
        {
            return Failure(
                request,
                "KERNEL_TRANSPORT",
                "Kernel transport failed or the kernel could not be reached.");
        }
        catch (OperationCanceledException) when (cancellationToken.IsCancellationRequested)
        {
            throw;
        }
        catch (OperationCanceledException) when (timeoutCts.IsCancellationRequested)
        {
            return Failure(
                request,
                "KERNEL_TIMEOUT",
                "Kernel operation exceeded the configured timeout.");
        }
        catch (JsonException)
        {
            return Failure(
                request,
                "KERNEL_INVALID_JSON",
                "Kernel returned invalid JSON.");
        }
        catch (Exception)
        {
            return Failure(
                request,
                "KERNEL_CLIENT",
                "Kernel client failed while processing the operation.");
        }
    }

    public void Dispose()
    {
        if (_ownsClient)
            _client.Dispose();
    }

    private static void ValidateRequest(KernelOperationRequest request)
    {
        if (!string.Equals(
                request.ContractVersion,
                CadContractVersions.KernelOperation,
                StringComparison.Ordinal))
        {
            throw new ArgumentException(
                "Kernel operation contract version is not supported.",
                nameof(request));
        }

        if (string.IsNullOrWhiteSpace(request.PartId))
            throw new ArgumentException(
                "Kernel operation part identity is required.",
                nameof(request));

        if (string.IsNullOrWhiteSpace(request.OperationId.Value))
            throw new ArgumentException(
                "Kernel operation identity is required.",
                nameof(request));

        if (string.IsNullOrWhiteSpace(request.OperationKind))
            throw new ArgumentException(
                "Kernel operation kind is required.",
                nameof(request));

        if (string.IsNullOrWhiteSpace(request.EvaluationIdentity.Value))
            throw new ArgumentException(
                "Kernel evaluation identity is required.",
                nameof(request));
    }

    private static async Task<byte[]?> ReadBodyAsync(
        HttpResponseMessage response,
        long maxBytes,
        CancellationToken cancellationToken)
    {
        await using var stream =
            await response.Content.ReadAsStreamAsync(cancellationToken);
        await using var buffer = new MemoryStream();
        var chunk = new byte[81_920];

        while (true)
        {
            var read = await stream.ReadAsync(
                chunk.AsMemory(),
                cancellationToken);

            if (read == 0)
                break;

            if (buffer.Length + read > maxBytes)
                return null;

            await buffer.WriteAsync(
                chunk.AsMemory(0, read),
                cancellationToken);
        }

        return buffer.ToArray();
    }

    private static KernelOperationResponse Failure(
        KernelOperationRequest request,
        string code,
        string message) =>
        KernelOperationResponse.Failure(request, code, message);
}

internal static class KernelOperationResponseValidator
{
    public static void Validate(
        KernelOperationRequest request,
        KernelOperationResponse response)
    {
        if (!string.Equals(
                response.ContractVersion,
                request.ContractVersion,
                StringComparison.Ordinal))
        {
            throw new InvalidOperationException(
                "Kernel response contract version does not match the request.");
        }

        if (response.EvaluationIdentity != request.EvaluationIdentity)
        {
            throw new InvalidOperationException(
                "Kernel response evaluation identity does not match the request.");
        }

        if (response.OperationId != request.OperationId)
        {
            throw new InvalidOperationException(
                "Kernel response operation identity does not match the request.");
        }

        if (response.Status == CadEvaluationStatus.Succeeded)
        {
            if (response.AuthoritativeResultId is null)
                throw new InvalidOperationException(
                    "Successful kernel operation has no authoritative result identity.");

            if (string.IsNullOrWhiteSpace(response.EvidenceHash))
                throw new InvalidOperationException(
                    "Successful kernel operation has no authoritative evidence hash.");
}

        if (response.Status != CadEvaluationStatus.Succeeded &&
            response.AuthoritativeResultId is not null)
        {
            throw new InvalidOperationException(
                "Unsuccessful kernel operation must not claim an authoritative result.");
        }

        if (response.Status != CadEvaluationStatus.Succeeded &&
            !string.IsNullOrWhiteSpace(response.EvidenceHash))
        {
            throw new InvalidOperationException(
                "Unsuccessful kernel operation must not claim authoritative evidence.");
        }

        foreach (var binding in response.Topology)
        {
            if (string.IsNullOrWhiteSpace(binding.Kind) ||
                string.IsNullOrWhiteSpace(binding.Key))
            {
                throw new InvalidOperationException(
                    "Kernel topology bindings must contain non-empty kind and key.");
            }
        }
    }
}
