using System.Net;
using System.Net.Http.Json;
using System.Text.Json;
using Microsoft.Extensions.Options;
using UMLCAD.Cad.Contracts;

namespace UMLCAD.Kernel.Client;

public sealed class RustGeometryKernelService : IAuthoritativeGeometryService
{
    private const string Endpoint = "v1/geometry/box-solid";

    private static AxisAlignedBoxSolidKernelResult Failed(string diagnostic) =>
        new(
            AxisAlignedBoxSolidKernelStatus.Failed,
            null,
            null,
            Array.Empty<AxisAlignedBoxSolidKernelTopology>(),
            null,
            null,
            null,
            [diagnostic]);

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
            var dto = await JsonSerializer.DeserializeAsync<AxisAlignedBoxSolidResponseDto>(
                stream,
                new JsonSerializerOptions(JsonSerializerDefaults.Web),
                linkedCts.Token);

            if (dto is null)
                throw new InvalidOperationException("Kernel returned an empty box-solid result.");

            if (!string.Equals(dto.Schema, "uml-cad-axis-aligned-box-solid/1.0.0", StringComparison.Ordinal))
                return Failed("Kernel returned an unexpected box-solid schema.");

            var status = dto.Status switch
            {
                "succeeded" => AxisAlignedBoxSolidKernelStatus.Succeeded,
                "failed" => AxisAlignedBoxSolidKernelStatus.Failed,
                "unsupported" => AxisAlignedBoxSolidKernelStatus.Unsupported,
                "ambiguous" => AxisAlignedBoxSolidKernelStatus.Ambiguous,
                "indeterminate" => AxisAlignedBoxSolidKernelStatus.Indeterminate,
                _ => AxisAlignedBoxSolidKernelStatus.Failed,
            };

            var result = new AxisAlignedBoxSolidKernelResult(
                status,
                dto.ResultId is null ? null : new ContractResultId(dto.ResultId),
                dto.EvidenceHash,
                dto.Topology?
                    .Select(x => new AxisAlignedBoxSolidKernelTopology(x.Kind, x.Key))
                    .ToArray() ?? Array.Empty<AxisAlignedBoxSolidKernelTopology>(),
                dto.Volume,
                dto.SurfaceArea,
                dto.Centroid is null
                    ? null
                    : new KernelVector3(dto.Centroid.X, dto.Centroid.Y, dto.Centroid.Z),
                dto.Diagnostics ?? Array.Empty<string>());

            if (dto.Succeeded != (status == AxisAlignedBoxSolidKernelStatus.Succeeded))
                return Failed("Kernel box-solid status is inconsistent with succeeded.");

            return result;
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


internal sealed record AxisAlignedBoxSolidResponseDto(
    string Schema,
    string Status,
    bool Succeeded,
    string? ResultId,
    string? EvidenceHash,
    IReadOnlyList<AxisAlignedBoxSolidResponseTopologyDto>? Topology,
    double? Volume,
    double? SurfaceArea,
    AxisAlignedBoxSolidResponseVectorDto? Centroid,
    IReadOnlyList<string>? Diagnostics);

internal sealed record AxisAlignedBoxSolidResponseTopologyDto(
    string Kind,
    string Key);

internal sealed record AxisAlignedBoxSolidResponseVectorDto(
    double X,
    double Y,
    double Z);
