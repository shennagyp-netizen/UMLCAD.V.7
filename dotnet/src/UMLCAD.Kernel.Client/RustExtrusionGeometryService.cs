using System.Net;
using System.Net.Http.Json;
using System.Text.Json;
using Microsoft.Extensions.Options;
using UMLCAD.Cad.Contracts;

namespace UMLCAD.Kernel.Client;

public sealed class RustExtrusionGeometryService : IExtrusionGeometryService
{
    private const string Endpoint = "v1/geometry/extrude-convex-planar-profile";
    private const string Schema = "uml-cad-extrude-convex-planar-profile/1.0.0";

    private static ExtrusionKernelResult Failed(string diagnostic) =>
        new(
            AxisAlignedBoxSolidKernelStatus.Failed,
            null,
            null,
            Array.Empty<ExtrusionTopology>(),
            null,
            null,
            null,
            [diagnostic]);

    private readonly HttpClient _httpClient;
    private readonly IOptions<RustKernelOptions> _options;

    public RustExtrusionGeometryService(
        HttpClient httpClient,
        IOptions<RustKernelOptions> options)
    {
        _httpClient = httpClient ?? throw new ArgumentNullException(nameof(httpClient));
        _options = options ?? throw new ArgumentNullException(nameof(options));
    }

    public async Task<ExtrusionKernelResult> ExtrudeConvexPlanarProfileAsync(
        ExtrusionRequest request,
        CancellationToken cancellationToken = default)
    {
        ArgumentNullException.ThrowIfNull(request);

        using var linkedCts = CancellationTokenSource.CreateLinkedTokenSource(cancellationToken);
        linkedCts.CancelAfter(_options.Value.RequestTimeout);

        var payload = new
        {
            schema = Schema,
            operationIdentity = request.OperationIdentity,
            origin = new
            {
                x = request.Origin.X,
                y = request.Origin.Y,
                z = request.Origin.Z,
            },
            uDirection = new
            {
                x = request.UDirection.X,
                y = request.UDirection.Y,
                z = request.UDirection.Z,
            },
            vDirection = new
            {
                x = request.VDirection.X,
                y = request.VDirection.Y,
                z = request.VDirection.Z,
            },
            profile = request.Profile.Select(point => new { u = point.U, v = point.V }).ToArray(),
            depth = request.Depth,
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
                return Failed($"Kernel returned HTTP {(int)response.StatusCode}.");

            await using var stream = await response.Content.ReadAsStreamAsync(linkedCts.Token);
            var dto = await JsonSerializer.DeserializeAsync<ExtrusionResponseDto>(
                stream,
                new JsonSerializerOptions(JsonSerializerDefaults.Web),
                linkedCts.Token);

            if (dto is null)
                return Failed("Kernel returned an empty extrusion result.");

            if (!string.Equals(dto.Schema, Schema, StringComparison.Ordinal))
                return Failed("Kernel returned an unexpected extrusion schema.");

            var status = dto.Status switch
            {
                "succeeded" => AxisAlignedBoxSolidKernelStatus.Succeeded,
                "failed" => AxisAlignedBoxSolidKernelStatus.Failed,
                "unsupported" => AxisAlignedBoxSolidKernelStatus.Unsupported,
                "ambiguous" => AxisAlignedBoxSolidKernelStatus.Ambiguous,
                "indeterminate" => AxisAlignedBoxSolidKernelStatus.Indeterminate,
                _ => AxisAlignedBoxSolidKernelStatus.Failed,
            };

            if (dto.Succeeded != (status == AxisAlignedBoxSolidKernelStatus.Succeeded))
                return Failed("Kernel extrusion status is inconsistent with succeeded.");

            return new ExtrusionKernelResult(
                status,
                dto.ResultId is null ? null : new ContractResultId(dto.ResultId),
                dto.EvidenceHash,
                dto.Topology?
                    .Select(x => new ExtrusionTopology(x.Kind, x.Key))
                    .ToArray() ?? Array.Empty<ExtrusionTopology>(),
                dto.Volume,
                dto.SurfaceArea,
                dto.Centroid is null
                    ? null
                    : new KernelVector3(dto.Centroid.X, dto.Centroid.Y, dto.Centroid.Z),
                dto.Diagnostics ?? Array.Empty<string>());
        }
        catch (JsonException)
        {
            return Failed("Kernel returned invalid JSON.");
        }
        catch (HttpRequestException)
        {
            return Failed("Kernel extrusion transport failed.");
        }
        catch (TaskCanceledException) when (cancellationToken.IsCancellationRequested)
        {
            throw;
        }
        catch (TaskCanceledException)
        {
            return Failed("Kernel extrusion request timed out.");
        }
    }
}

internal sealed record ExtrusionResponseDto(
    string Schema,
    string Status,
    bool Succeeded,
    string? ResultId,
    string? EvidenceHash,
    IReadOnlyList<ExtrusionResponseTopologyDto>? Topology,
    double? Volume,
    double? SurfaceArea,
    ExtrusionResponseVectorDto? Centroid,
    IReadOnlyList<string>? Diagnostics);

internal sealed record ExtrusionResponseTopologyDto(
    string Kind,
    string Key);

internal sealed record ExtrusionResponseVectorDto(
    double X,
    double Y,
    double Z);
