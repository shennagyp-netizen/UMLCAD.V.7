using System.Net.Http.Json;
using System.Text.Json;
using Microsoft.Extensions.Options;

namespace UMLCAD.Kernel.Client;

public sealed record AxisAlignedBoxEvaluationRequest(
    string EvaluationIdentity,
    string ResultIdentity,
    IReadOnlyList<double> Minimum,
    IReadOnlyList<double> Maximum,
    double AbsoluteTolerance,
    double RelativeTolerance);

public sealed record AxisAlignedBoxEvaluationResponse(
    bool Succeeded,
    string? EvaluationIdentity,
    string? ResultIdentity,
    IReadOnlyList<double> Minimum,
    IReadOnlyList<double> Maximum,
    double? Volume,
    double? SurfaceArea,
    IReadOnlyList<double> Centroid,
    IReadOnlyList<KernelDiagnostic> Diagnostics);

public interface IAxisAlignedBoxKernelService
{
    Task<AxisAlignedBoxEvaluationResponse> EvaluateAsync(
        AxisAlignedBoxEvaluationRequest request,
        CancellationToken cancellationToken = default);
}

public sealed class AxisAlignedBoxKernelService(
    HttpClient httpClient,
    IOptions<RustKernelOptions> options) : IAxisAlignedBoxKernelService
{
    private const string EvaluatePath = "v1/solid/axis-aligned-box/evaluate";
    private static readonly JsonSerializerOptions JsonOptions = new(JsonSerializerDefaults.Web);

    public async Task<AxisAlignedBoxEvaluationResponse> EvaluateAsync(
        AxisAlignedBoxEvaluationRequest request,
        CancellationToken cancellationToken = default)
    {
        ArgumentNullException.ThrowIfNull(request);
        ValidateRequest(request);

        try
        {
            using var linkedCts = CancellationTokenSource.CreateLinkedTokenSource(cancellationToken);
            linkedCts.CancelAfter(options.Value.RequestTimeout);

            using var response = await httpClient.PostAsJsonAsync(
                EvaluatePath,
                new
                {
                    schema = "uml-cad-axis-aligned-box/1.0.0",
                    evaluationIdentity = request.EvaluationIdentity,
                    resultIdentity = request.ResultIdentity,
                    min = request.Minimum,
                    max = request.Maximum,
                    absoluteTolerance = request.AbsoluteTolerance,
                    relativeTolerance = request.RelativeTolerance
                },
                JsonOptions,
                linkedCts.Token);

            if (response.Content.Headers.ContentLength is > 0 and var contentLength &&
                contentLength > options.Value.MaxResponseBytes)
            {
                return Failure(
                    "KERNEL_RESPONSE_TOO_LARGE",
                    "AxisAlignedBox kernel response exceeds the configured response limit.");
            }

            var body = await ReadResponseBodyAsync(
                response,
                options.Value.MaxResponseBytes,
                linkedCts.Token);

            if (body is null)
                return Failure(
                    "KERNEL_RESPONSE_TOO_LARGE",
                    "AxisAlignedBox kernel response exceeds the configured response limit.");

            if (body.Length == 0)
                return Failure("KERNEL_EMPTY_RESPONSE", "AxisAlignedBox kernel returned an empty response.");

            RawResponse? raw;
            try
            {
                raw = JsonSerializer.Deserialize<RawResponse>(body, JsonOptions);
            }
            catch (JsonException)
            {
                return Failure(
                    "KERNEL_INVALID_JSON",
                    "AxisAlignedBox kernel returned invalid JSON.");
            }

            if (raw is null)
                return Failure("KERNEL_EMPTY_RESPONSE", "AxisAlignedBox kernel returned an empty response.");

            if (!response.IsSuccessStatusCode)
            {
                if (raw.Diagnostics.Count > 0)
                    return Failure(raw.Diagnostics);

                return Failure(
                    "KERNEL_HTTP",
                    $"AxisAlignedBox kernel returned HTTP {(int)response.StatusCode} ({response.ReasonPhrase}).");
            }

            if (raw.Result is null && raw.Succeeded)
                return Failure(
                    "KERNEL_INVALID_RESULT",
                    "AxisAlignedBox kernel reported success without a result.");

            if (raw.Result is null)
                return Failure(
                    raw.Diagnostics);

            var result = new AxisAlignedBoxEvaluationResponse(
                raw.Succeeded,
                raw.Result.EvaluationIdentity,
                raw.Result.ResultIdentity,
                raw.Result.Min,
                raw.Result.Max,
                raw.Result.Volume,
                raw.Result.SurfaceArea,
                raw.Result.Centroid,
                raw.Diagnostics);

            ValidateResponse(result, request);
            return result;
        }
        catch (HttpRequestException)
        {
            return Failure(
                "KERNEL_TRANSPORT",
                "The AxisAlignedBox kernel could not be reached or the transport failed.");
        }
        catch (TaskCanceledException) when (cancellationToken.IsCancellationRequested)
        {
            throw;
        }
        catch (TaskCanceledException)
        {
            return Failure(
                "KERNEL_TIMEOUT",
                "The AxisAlignedBox kernel request exceeded the configured timeout.");
        }
    }

    private static void ValidateRequest(AxisAlignedBoxEvaluationRequest request)
    {
        if (string.IsNullOrWhiteSpace(request.EvaluationIdentity))
            throw new ArgumentException(
                "Evaluation identity cannot be empty.",
                nameof(request));

        if (string.IsNullOrWhiteSpace(request.ResultIdentity))
            throw new ArgumentException(
                "Result identity cannot be empty.",
                nameof(request));

        ArgumentNullException.ThrowIfNull(request.Minimum);
        ArgumentNullException.ThrowIfNull(request.Maximum);

        if (request.Minimum.Count != 3)
            throw new ArgumentException(
                "Minimum must contain exactly three coordinates.",
                nameof(request));

        if (request.Maximum.Count != 3)
            throw new ArgumentException(
                "Maximum must contain exactly three coordinates.",
                nameof(request));

        if (request.Minimum.Any(value => !double.IsFinite(value)) ||
            request.Maximum.Any(value => !double.IsFinite(value)))
        {
            throw new ArgumentException(
                "AxisAlignedBox coordinates must be finite.",
                nameof(request));
        }

        if (!double.IsFinite(request.AbsoluteTolerance) ||
            !double.IsFinite(request.RelativeTolerance))
        {
            throw new ArgumentException(
                "AxisAlignedBox tolerances must be finite.",
                nameof(request));
        }

        if (request.AbsoluteTolerance < 0)
            throw new ArgumentOutOfRangeException(
                nameof(request),
                request.AbsoluteTolerance,
                "Absolute tolerance cannot be negative.");

        if (request.RelativeTolerance < 0)
            throw new ArgumentOutOfRangeException(
                nameof(request),
                request.RelativeTolerance,
                "Relative tolerance cannot be negative.");
    }

    private static void ValidateResponse(
        AxisAlignedBoxEvaluationResponse response,
        AxisAlignedBoxEvaluationRequest request)
    {
        if (!response.Succeeded)
            return;

        if (!string.Equals(response.EvaluationIdentity, request.EvaluationIdentity, StringComparison.Ordinal) ||
            !string.Equals(response.ResultIdentity, request.ResultIdentity, StringComparison.Ordinal))
        {
            throw new InvalidOperationException(
                "AxisAlignedBox kernel response identities do not match the submitted evaluation.");
        }

        if (response.Minimum.Count != 3 ||
            response.Maximum.Count != 3 ||
            response.Centroid.Count != 3 ||
            response.Volume is null ||
            response.SurfaceArea is null)
        {
            throw new InvalidOperationException(
                "AxisAlignedBox kernel returned incomplete authoritative evidence.");
        }

        if (response.Minimum.Any(value => !double.IsFinite(value)) ||
            response.Maximum.Any(value => !double.IsFinite(value)) ||
            response.Centroid.Any(value => !double.IsFinite(value)) ||
            !double.IsFinite(response.Volume.Value) ||
            !double.IsFinite(response.SurfaceArea.Value))
        {
            throw new InvalidOperationException(
                "AxisAlignedBox kernel returned non-finite authoritative evidence.");
        }
    }

    private static async Task<byte[]?> ReadResponseBodyAsync(
        HttpResponseMessage response,
        long maxBytes,
        CancellationToken cancellationToken)
    {
        await using var stream = await response.Content.ReadAsStreamAsync(cancellationToken);
        await using var buffer = new MemoryStream();
        var chunk = new byte[81_920];

        while (true)
        {
            var read = await stream.ReadAsync(chunk.AsMemory(), cancellationToken);
            if (read == 0)
                break;

            if (buffer.Length + read > maxBytes)
                return null;

            await buffer.WriteAsync(chunk.AsMemory(0, read), cancellationToken);
        }

        return buffer.ToArray();
    }

    private static AxisAlignedBoxEvaluationResponse Failure(
        string code,
        string message) =>
        Failure([new KernelDiagnostic(code, "error", message)]);

    private static AxisAlignedBoxEvaluationResponse Failure(
        IReadOnlyList<KernelDiagnostic> diagnostics) =>
        new(
            false,
            null,
            null,
            Array.Empty<double>(),
            Array.Empty<double>(),
            null,
            null,
            Array.Empty<double>(),
            diagnostics);

    private sealed record RawResponse(
        bool Succeeded,
        RawResult? Result,
        IReadOnlyList<KernelDiagnostic> Diagnostics);

    private sealed record RawResult(
        string EvaluationIdentity,
        string ResultIdentity,
        IReadOnlyList<double> Min,
        IReadOnlyList<double> Max,
        double Volume,
        double SurfaceArea,
        IReadOnlyList<double> Centroid);
}
