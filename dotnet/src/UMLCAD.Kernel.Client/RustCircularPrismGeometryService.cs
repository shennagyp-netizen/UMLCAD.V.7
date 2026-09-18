using System.Net;
using System.Net.Http.Json;
using System.Text.Json;
using Microsoft.Extensions.Options;
using UMLCAD.Cad.Contracts;

namespace UMLCAD.Kernel.Client;

public sealed class RustCircularPrismGeometryService : ICircularPrismGeometryService
{
    private const string Endpoint = "v1/geometry/circular-prism-solid";
    private const string Schema = CircularPrismSolidRequest.ContractSchema;

    private static CircularPrismSolidKernelResult Failed(string diagnostic) =>
        new(
            GeometryKernelStatus.Failed,
            false,
            null,
            null,
            new KernelVector3(0d, 0d, 0d),
            new KernelVector3(0d, 0d, 1d),
            1d,
            1d,
            null,
            null,
            null,
            null,
            Array.Empty<CircularPrismKernelTopology>(),
            new[] { diagnostic });

    private readonly HttpClient _httpClient;
    private readonly IOptions<RustKernelOptions> _options;

    public RustCircularPrismGeometryService(
        HttpClient httpClient,
        IOptions<RustKernelOptions> options)
    {
        _httpClient = httpClient ?? throw new ArgumentNullException(nameof(httpClient));
        _options = options ?? throw new ArgumentNullException(nameof(options));
    }

    public async Task<CircularPrismSolidKernelResult> BuildCircularPrismSolidAsync(
        CircularPrismSolidRequest request,
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
                z = request.Origin.Z
            },
            axis = new
            {
                x = request.Axis.X,
                y = request.Axis.Y,
                z = request.Axis.Z
            },
            radius = request.Radius,
            depth = request.Depth,
            tolerance = new
            {
                absolute = request.Tolerance.Absolute,
                relative = request.Tolerance.Relative
            }
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
            var dto = await JsonSerializer.DeserializeAsync<CircularPrismResponseDto>(
                stream,
                new JsonSerializerOptions(JsonSerializerDefaults.Web),
                linkedCts.Token);

            if (dto is null)
                return Failed("Kernel returned an empty circular-prism result.");

            if (!string.Equals(dto.Schema, Schema, StringComparison.Ordinal))
                return Failed("Kernel returned an unexpected circular-prism schema.");

            var status = dto.Status switch
            {
                "succeeded" => GeometryKernelStatus.Succeeded,
                "failed" => GeometryKernelStatus.Failed,
                "unsupported" => GeometryKernelStatus.Unsupported,
                "ambiguous" => GeometryKernelStatus.Ambiguous,
                "indeterminate" => GeometryKernelStatus.Indeterminate,
                _ => GeometryKernelStatus.Failed
            };

            if (dto.Succeeded != (status == GeometryKernelStatus.Succeeded))
                return Failed("Kernel circular-prism status is inconsistent with succeeded.");

            try
            {
                return new CircularPrismSolidKernelResult(
                    status,
                    dto.Succeeded,
                    dto.ResultId is null ? null : new ContractResultId(dto.ResultId),
                    dto.EvidenceHash,
                    new KernelVector3(dto.Origin.X, dto.Origin.Y, dto.Origin.Z),
                    new KernelVector3(dto.Axis.X, dto.Axis.Y, dto.Axis.Z),
                    dto.Radius,
                    dto.Depth,
                    dto.Volume,
                    dto.SurfaceArea,
                    dto.Centroid is null
                        ? null
                        : new KernelVector3(
                            dto.Centroid.X,
                            dto.Centroid.Y,
                            dto.Centroid.Z),
                    dto.Bounds is null
                        ? null
                        : new CadBoundingBox3(
                            dto.Bounds.Min.X,
                            dto.Bounds.Min.Y,
                            dto.Bounds.Min.Z,
                            dto.Bounds.Max.X,
                            dto.Bounds.Max.Y,
                            dto.Bounds.Max.Z),
                    dto.Topology?
                        .Select(x => new CircularPrismKernelTopology(x.Kind, x.Key))
                        .ToArray() ?? Array.Empty<CircularPrismKernelTopology>(),
                    dto.Diagnostics ?? Array.Empty<string>());
            }
            catch (ArgumentException)
            {
                return Failed("Kernel returned an invalid circular-prism result.");
            }
        }
        catch (JsonException)
        {
            return Failed("Kernel returned invalid JSON.");
        }
        catch (HttpRequestException)
        {
            return Failed("Kernel circular-prism transport failed.");
        }
        catch (TaskCanceledException) when (cancellationToken.IsCancellationRequested)
        {
            throw;
        }
        catch (TaskCanceledException)
        {
            return Failed("Kernel circular-prism request timed out.");
        }
    }
}

internal sealed record CircularPrismResponseDto(
    string Schema,
    string Status,
    bool Succeeded,
    string? ResultId,
    string? EvidenceHash,
    CircularPrismResponseVectorDto Origin,
    CircularPrismResponseVectorDto Axis,
    double Radius,
    double Depth,
    double? Volume,
    double? SurfaceArea,
    CircularPrismResponseVectorDto? Centroid,
    CircularPrismResponseBoundsDto? Bounds,
    IReadOnlyList<CircularPrismResponseTopologyDto>? Topology,
    IReadOnlyList<string>? Diagnostics);

internal sealed record CircularPrismResponseVectorDto(
    double X,
    double Y,
    double Z);

internal sealed record CircularPrismResponseBoundsDto(
    CircularPrismResponseVectorDto Min,
    CircularPrismResponseVectorDto Max);

internal sealed record CircularPrismResponseTopologyDto(
    string Kind,
    string Key);
