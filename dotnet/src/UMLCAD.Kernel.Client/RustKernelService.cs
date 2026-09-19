using System.Text;
using System.Text.Json;
using System.Net.Http.Json;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Options;
using UMLCAD.Framework.Semantics;

namespace UMLCAD.Kernel.Client;

public sealed class RustKernelOptions
{
    public Uri BaseAddress { get; set; } = new("http://localhost:8080/");
    public string EvaluatePath { get; set; } = "v1/build/evaluate";
    public TimeSpan RequestTimeout { get; set; } = TimeSpan.FromMinutes(2);
    public long MaxResponseBytes { get; set; } = 256L * 1024 * 1024;
}

public interface IRustKernelService
{
    Task<KernelEvaluationResult> EvaluateAsync(
        BuildPackage package,
        CancellationToken cancellationToken = default);
}

public sealed record KernelEvaluationResult(
    bool Succeeded,
    CompiledModelPackage? CompiledModel,
    IReadOnlyList<KernelDiagnostic> Diagnostics);

public sealed record KernelDiagnostic(
    string Code,
    string Severity,
    string Message,
    string? TargetId = null);

public sealed class RustKernelService(
    HttpClient httpClient,
    IOptions<RustKernelOptions> options) : IRustKernelService
{
    public async Task<KernelEvaluationResult> EvaluateAsync(
        BuildPackage package,
        CancellationToken cancellationToken = default)
    {
        ArgumentNullException.ThrowIfNull(package);

        try
        {
            using var linkedCts = CancellationTokenSource.CreateLinkedTokenSource(cancellationToken);
            linkedCts.CancelAfter(options.Value.RequestTimeout);

            using var response = await httpClient.PostAsJsonAsync(
                options.Value.EvaluatePath,
                package,
                linkedCts.Token);

            if (response.Content.Headers.ContentLength is > 0 and var contentLength && contentLength > options.Value.MaxResponseBytes)
                return Failure("KERNEL_RESPONSE_TOO_LARGE", "Kernel response exceeds the configured response limit.");

            var body = await ReadResponseBodyAsync(response, options.Value.MaxResponseBytes, linkedCts.Token);
            if (body is null)
                return Failure("KERNEL_RESPONSE_TOO_LARGE", "Kernel response exceeds the configured response limit.");

            if (!response.IsSuccessStatusCode)
            {
                var detail = body.Length == 0 ? string.Empty : $" Body: {Encoding.UTF8.GetString(body)}";
                return Failure(
                    "KERNEL_HTTP",
                    $"Kernel returned HTTP {(int)response.StatusCode} ({response.ReasonPhrase}).{detail}");
            }

            if (body.Length == 0)
                return Failure("KERNEL_EMPTY_RESPONSE", "The kernel returned an empty response.");

            var result = JsonSerializer.Deserialize<KernelEvaluationResult>(body, new JsonSerializerOptions(JsonSerializerDefaults.Web));
            if (result is null)
                return Failure("KERNEL_EMPTY_RESPONSE", "The kernel returned an empty response.");

            if (result.CompiledModel is not null)
            {
                try
                {
                    CompiledModelValidator.Validate(result.CompiledModel, package.BuildIdentity);
                }
                catch (InvalidOperationException exception)
                {
                    return Failure("KERNEL_INVALID_RESULT", exception.Message);
                }
            }

            return result;
        }
        catch (HttpRequestException)
        {
            return Failure("KERNEL_TRANSPORT", "The kernel could not be reached or the transport failed.");
        }
        catch (TaskCanceledException) when (cancellationToken.IsCancellationRequested)
        {
            throw;
        }
        catch (TaskCanceledException)
        {
            return Failure("KERNEL_TIMEOUT", "The kernel request exceeded the configured timeout.");
        }
        catch (JsonException)
        {
            return Failure("KERNEL_INVALID_JSON", "The kernel returned invalid JSON.");
        }
        catch (Exception)
        {
            return Failure("KERNEL_CLIENT", "The kernel client failed while processing the request.");
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

    private static KernelEvaluationResult Failure(string code, string message) =>
        new(false, null, [new KernelDiagnostic(code, "error", message)]);
}

internal static class CompiledModelValidator
{
    public static void Validate(CompiledModelPackage package, string expectedBuildIdentity)
    {
        if (!string.Equals(package.BuildIdentity, expectedBuildIdentity, StringComparison.Ordinal))
            throw new InvalidOperationException("Compiled model build identity does not match the submitted build.");
        if (!string.Equals(package.Manifest.BuildIdentity, expectedBuildIdentity, StringComparison.Ordinal))
            throw new InvalidOperationException("Compiled manifest build identity does not match the submitted build.");
        if (!string.Equals(package.Manifest.ApplicationId, package.ApplicationId, StringComparison.Ordinal) ||
            !string.Equals(package.Manifest.ApplicationVersion, package.ApplicationVersion, StringComparison.Ordinal))
            throw new InvalidOperationException("Compiled model application identity is inconsistent.");

        var nodeIds = new HashSet<string>(package.Manifest.Nodes.Select(x => x.Id), StringComparer.Ordinal);
        if (nodeIds.Count != package.Manifest.Nodes.Count)
            throw new InvalidOperationException("Compiled model contains duplicate node IDs.");

        foreach (var root in package.Manifest.RootNodeIds)
            if (!nodeIds.Contains(root))
                throw new InvalidOperationException($"Compiled model root '{root}' does not resolve.");

        foreach (var relationship in package.Manifest.Relationships)
        {
            if (!nodeIds.Contains(relationship.SourceId))
                throw new InvalidOperationException($"Relationship '{relationship.Id}' source does not resolve.");
            foreach (var target in relationship.TargetIds)
                if (!nodeIds.Contains(target))
                    throw new InvalidOperationException($"Relationship '{relationship.Id}' target '{target}' does not resolve.");
        }

        if (package.RenderArtifact is not null &&
            !string.Equals(package.RenderArtifact.BuildIdentity, expectedBuildIdentity, StringComparison.Ordinal))
            throw new InvalidOperationException("Render artifact build identity does not match the submitted build.");
    }
}

public static class RustKernelServiceCollectionExtensions
{
    public static IServiceCollection AddRustKernel(
        this IServiceCollection services,
        Action<RustKernelOptions>? configure = null)
    {
        ArgumentNullException.ThrowIfNull(services);

        var options = new RustKernelOptions();
        configure?.Invoke(options);

        if (!options.BaseAddress.IsAbsoluteUri || options.BaseAddress.Scheme is not ("http" or "https"))
            throw new ArgumentException("Rust kernel BaseAddress must be an absolute HTTP(S) URI.", nameof(configure));
        if (string.IsNullOrWhiteSpace(options.EvaluatePath))
            throw new ArgumentException("Rust kernel evaluation path cannot be empty.", nameof(configure));
        if (Uri.TryCreate(options.EvaluatePath, UriKind.Absolute, out _))
            throw new ArgumentException("Rust kernel evaluation path must be relative to the configured BaseAddress.", nameof(configure));
        if (options.RequestTimeout <= TimeSpan.Zero || options.RequestTimeout > TimeSpan.FromHours(1))
            throw new ArgumentOutOfRangeException(nameof(configure), "Rust kernel request timeout is outside the allowed range.");
        if (options.MaxResponseBytes <= 0 || options.MaxResponseBytes > 2L * 1024 * 1024 * 1024)
            throw new ArgumentOutOfRangeException(nameof(configure), "Rust kernel response limit is outside the allowed range.");

        services.AddOptions<RustKernelOptions>()
            .Configure(current =>
            {
                current.BaseAddress = options.BaseAddress;
                current.EvaluatePath = options.EvaluatePath;
                current.RequestTimeout = options.RequestTimeout;
                current.MaxResponseBytes = options.MaxResponseBytes;
            });

        services.AddHttpClient<IRustKernelService, RustKernelService>(client =>
        {
            client.BaseAddress = options.BaseAddress;
            client.Timeout = options.RequestTimeout;
        });

        services.AddHttpClient<IAxisAlignedBoxKernelService, AxisAlignedBoxKernelService>(client =>
        {
            client.BaseAddress = options.BaseAddress;
            client.Timeout = options.RequestTimeout;
        });

        return services;
    }
}