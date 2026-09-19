using System.Net.Http.Json;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace UMLCAD.Kernel;

public sealed class UmlcadKernelOptions
{
    public Uri BaseAddress { get; init; } = new("http://127.0.0.1:8080/");
    public string BuildEvaluationPath { get; init; } = "v1/build/evaluate";
    public TimeSpan RequestTimeout { get; init; } = TimeSpan.FromMinutes(2);
    public long MaximumResponseBytes { get; init; } = 256L * 1024 * 1024;
}

public sealed record KernelBuildRequest(
    string ApplicationId,
    string ApplicationVersion,
    string BuildIdentity,
    JsonElement Semantic)
{
    public string Schema => "uml-cad-build-package/1.0.0";
}

public enum KernelDiagnosticSeverity
{
    Information,
    Warning,
    Error
}

public sealed record KernelDiagnostic(
    string Code,
    KernelDiagnosticSeverity Severity,
    string Message,
    string? TargetId = null);

public sealed record KernelEvaluationOutcome(
    bool Succeeded,
    JsonElement? CompiledModel,
    IReadOnlyList<KernelDiagnostic> Diagnostics);

public sealed class UmlcadKernel
{
    private static readonly JsonSerializerOptions JsonOptions = new(JsonSerializerDefaults.Web)
    {
        DefaultIgnoreCondition = JsonIgnoreCondition.Never
    };

    private readonly HttpClient _httpClient;
    private readonly UmlcadKernelOptions _options;

    public UmlcadKernel(HttpClient httpClient, UmlcadKernelOptions? options = null)
    {
        _httpClient = httpClient ?? throw new ArgumentNullException(nameof(httpClient));
        _options = options ?? new UmlcadKernelOptions();

        if (!_options.BaseAddress.IsAbsoluteUri)
            throw new ArgumentException("Kernel base address must be absolute.", nameof(options));

        if (_options.BaseAddress.Scheme is not ("http" or "https"))
            throw new ArgumentException("Kernel base address must use HTTP or HTTPS.", nameof(options));

        if (string.IsNullOrWhiteSpace(_options.BuildEvaluationPath))
            throw new ArgumentException("Kernel evaluation path cannot be empty.", nameof(options));

        if (Uri.TryCreate(_options.BuildEvaluationPath, UriKind.Absolute, out _))
            throw new ArgumentException("Kernel evaluation path must be relative.", nameof(options));

        if (_options.RequestTimeout <= TimeSpan.Zero)
            throw new ArgumentOutOfRangeException(nameof(options), "Kernel request timeout must be positive.");

        if (_options.MaximumResponseBytes <= 0)
            throw new ArgumentOutOfRangeException(nameof(options), "Kernel response limit must be positive.");
    }

    public async Task<KernelEvaluationOutcome> EvaluateBuildAsync(
        KernelBuildRequest request,
        CancellationToken cancellationToken = default)
    {
        ArgumentNullException.ThrowIfNull(request);

        var payload = new
        {
            schema = request.Schema,
            applicationId = request.ApplicationId,
            applicationVersion = request.ApplicationVersion,
            buildIdentity = request.BuildIdentity,
            semantic = request.Semantic
        };

        using var linkedCancellation = CancellationTokenSource.CreateLinkedTokenSource(cancellationToken);
        linkedCancellation.CancelAfter(_options.RequestTimeout);

        try
        {
            using var response = await _httpClient.PostAsJsonAsync(
                _options.BuildEvaluationPath,
                payload,
                JsonOptions,
                linkedCancellation.Token);

            var body = await ReadBodyAsync(response, linkedCancellation.Token);
            if (body is null)
                return Failure("KERNEL_RESPONSE_TOO_LARGE", "Kernel response exceeded the configured limit.");

            if (!response.IsSuccessStatusCode)
                return Failure(
                    "KERNEL_TRANSPORT",
                    $"Kernel returned HTTP {(int)response.StatusCode} ({response.ReasonPhrase}).");

            if (body.Length == 0)
                return Failure("KERNEL_EMPTY_RESPONSE", "Kernel returned an empty response.");

            using var document = JsonDocument.Parse(body);
            var root = document.RootElement;

            if (!root.TryGetProperty("succeeded", out var succeededElement) ||
                (succeededElement.ValueKind is not JsonValueKind.True and not JsonValueKind.False))
            {
                return Failure("KERNEL_RESPONSE_SCHEMA", "Kernel response does not contain a valid succeeded value.");
            }

            var succeeded = succeededElement.GetBoolean();

            JsonElement? compiledModel = null;
            if (root.TryGetProperty("compiledModel", out var modelElement) &&
                modelElement.ValueKind is not JsonValueKind.Null and not JsonValueKind.Undefined)
            {
                compiledModel = modelElement.Clone();
            }

            var diagnostics = ReadDiagnostics(root);
            if (succeeded && diagnostics.Any(x => x.Severity == KernelDiagnosticSeverity.Error))
                return Failure("KERNEL_CONTRADICTORY_RESULT", "Kernel reported success while returning an error diagnostic.");

            return new KernelEvaluationOutcome(succeeded, compiledModel, diagnostics);
        }
        catch (HttpRequestException exception)
        {
            return Failure("KERNEL_TRANSPORT", $"Kernel transport failed: {exception.Message}");
        }
        catch (TaskCanceledException) when (cancellationToken.IsCancellationRequested)
        {
            throw;
        }
        catch (TaskCanceledException)
        {
            return Failure("KERNEL_TIMEOUT", "Kernel request exceeded the configured timeout.");
        }
        catch (JsonException exception)
        {
            return Failure("KERNEL_INVALID_RESPONSE", $"Kernel response was invalid JSON: {exception.Message}");
        }
        catch (Exception exception)
        {
            return Failure("KERNEL_CLIENT", $"Kernel gateway failed: {exception.Message}");
        }
    }

    private async Task<byte[]?> ReadBodyAsync(
        HttpResponseMessage response,
        CancellationToken cancellationToken)
    {
        await using var source = await response.Content.ReadAsStreamAsync(cancellationToken);
        await using var target = new MemoryStream();
        var buffer = new byte[81_920];

        while (true)
        {
            var read = await source.ReadAsync(buffer.AsMemory(), cancellationToken);
            if (read == 0)
                break;

            if (target.Length + read > _options.MaximumResponseBytes)
                return null;

            await target.WriteAsync(buffer.AsMemory(0, read), cancellationToken);
        }

        return target.ToArray();
    }

    private static IReadOnlyList<KernelDiagnostic> ReadDiagnostics(JsonElement root)
    {
        if (!root.TryGetProperty("diagnostics", out var diagnosticsElement) ||
            diagnosticsElement.ValueKind != JsonValueKind.Array)
        {
            return [];
        }

        var diagnostics = new List<KernelDiagnostic>();
        foreach (var element in diagnosticsElement.EnumerateArray())
        {
            if (element.ValueKind != JsonValueKind.Object)
                continue;

            var code = element.TryGetProperty("code", out var codeElement)
                ? codeElement.GetString()
                : null;
            var severityText = element.TryGetProperty("severity", out var severityElement)
                ? severityElement.GetString()
                : null;
            var message = element.TryGetProperty("message", out var messageElement)
                ? messageElement.GetString()
                : null;
            var targetId = element.TryGetProperty("targetId", out var targetElement)
                ? targetElement.GetString()
                : null;

            if (string.IsNullOrWhiteSpace(code) || string.IsNullOrWhiteSpace(message))
                continue;

            diagnostics.Add(new KernelDiagnostic(
                code,
                ParseSeverity(severityText),
                message,
                targetId));
        }

        return diagnostics;
    }

    private static KernelDiagnosticSeverity ParseSeverity(string? value) =>
        value?.Trim().ToLowerInvariant() switch
        {
            "warning" => KernelDiagnosticSeverity.Warning,
            "error" => KernelDiagnosticSeverity.Error,
            _ => KernelDiagnosticSeverity.Information
        };

    private static KernelEvaluationOutcome Failure(string code, string message) =>
        new(false, null, [new KernelDiagnostic(code, KernelDiagnosticSeverity.Error, message)]);
}
