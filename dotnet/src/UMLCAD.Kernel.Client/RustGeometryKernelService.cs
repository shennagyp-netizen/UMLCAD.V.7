using System.Net;
using System.Net.Http.Json;
using System.Text.Json;
using Microsoft.Extensions.Options;
using UMLCAD.Cad.Contracts;

namespace UMLCAD.Kernel.Client;

public sealed class RustGeometryKernelService : IAuthoritativeGeometryService
{
    private const string Endpoint = "v1/geometry/box-solid";

    private readonly HttpClient _httpClient;
    private readonly IOptions<RustKernelOptions> _options;

    public RustGeometryKernelService(
        HttpClient httpClient,
        IOptions<RustKernelOptions> options)
    {
        _httpClient = httpClient ?? throw new ArgumentNullException(nameof(httpClient));
        _options = options ?? throw new ArgumentNullException(nameof(options));
    }

    public async Task<AxisAlignedBoxSolidKernelResult> BuildAxisAlignedBoxSolidAsync(
        AxisAlignedBoxSolidRequest request,
        CancellationToken cancellationToken = default)
    {
        ArgumentNullException.ThrowIfNull(request);

        using var linkedCts = CancellationTokenSource.CreateLinkedTokenSource(cancellationToken);
        linkedCts.CancelAfter(_options.Value.RequestTimeout);

        var payload = new
        {
            schema = "uml-cad-axis-aligned-box-solid/1.0.0",
            operationIdentity = request.OperationIdentity,
            min = new { x = request.Min.X, y = request.Min.Y, z = request.Min.Z },
            max = new { x = request.Max.X, y = request.Max.Y, z = request.Max.Z },
            tolerance = new
            {
                absolute = request.Tolerance.Absolute,
                relative = request.Tolerance.Relative,
            },
        };

        try
        {
            using var response = await _httpClient.PostAsJsonAsync(
                Endpoint,
                payload,
                linkedCts.Token);

            if (response.StatusCode != HttpStatusCode.OK)
                return new AxisAlignedBoxSolidKernelResult(
                    AxisAlignedBoxSolidKernelStatus.Failed,
                    null,
                    null,
                    Array.Empty<AxisAlignedBoxSolidKernelTopology>(),
                    null,
                    null,
                    null,
                    [$"Kernel returned HTTP {(int)response.StatusCode}."]);

            await using var stream = await response.Content.ReadAsStreamAsync(linkedCts.Token);
            var result = await JsonSerializer.DeserializeAsync<AxisAlignedBoxSolidKernelResult>(
                stream,
                new JsonSerializerOptions(JsonSerializerDefaults.Web),
                linkedCts.Token);

            return result ??
                throw new InvalidOperationException("Kernel returned an empty box-solid result.");
        }
        catch (JsonException)
        {
            return new AxisAlignedBoxSolidKernelResult(
                AxisAlignedBoxSolidKernelStatus.Failed,
                null,
                null,
                Array.Empty<AxisAlignedBoxSolidKernelTopology>(),
                null,
                null,
                null,
                ["Kernel returned invalid JSON."]);
        }
        catch (HttpRequestException)
        {
            return new AxisAlignedBoxSolidKernelResult(
                AxisAlignedBoxSolidKernelStatus.Failed,
                null,
                null,
                Array.Empty<AxisAlignedBoxSolidKernelTopology>(),
                null,
                null,
                null,
                ["Kernel transport failed."]);
        }
        catch (TaskCanceledException) when (cancellationToken.IsCancellationRequested)
        {
            throw;
        }
        catch (TaskCanceledException)
        {
            return new AxisAlignedBoxSolidKernelResult(
                AxisAlignedBoxSolidKernelStatus.Failed,
                null,
                null,
                Array.Empty<AxisAlignedBoxSolidKernelTopology>(),
                null,
                null,
                null,
                ["Kernel geometry request timed out."]);
        }
    }
}
