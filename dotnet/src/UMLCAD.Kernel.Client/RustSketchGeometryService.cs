using System.Net;
using System.Net.Http.Json;
using System.Text.Json;
using Microsoft.Extensions.Options;
using UMLCAD.Cad.Contracts;

namespace UMLCAD.Kernel.Client;

public sealed class RustSketchGeometryService : ISketchGeometryService
{
    private const string Endpoint = "v1/sketch/solve";
    private const string Schema = SketchSolveRequest.ContractSchema;

    private static SketchKernelResult Failed(string diagnostic) =>
        new(
            GeometryKernelStatus.Failed,
            null,
            null,
            Array.Empty<SketchCircleResult>(),
            false,
            null,
            null,
            null,
            null,
            null,
            null,
            null,
            null,
            [diagnostic]);

    private readonly HttpClient _httpClient;
    private readonly IOptions<RustKernelOptions> _options;

    public RustSketchGeometryService(
        HttpClient httpClient,
        IOptions<RustKernelOptions> options)
    {
        _httpClient = httpClient ?? throw new ArgumentNullException(nameof(httpClient));
        _options = options ?? throw new ArgumentNullException(nameof(options));
    }

    public async Task<SketchKernelResult> SolveAsync(
        SketchSolveRequest request,
        CancellationToken cancellationToken = default)
    {
        ArgumentNullException.ThrowIfNull(request);

        using var linkedCts = CancellationTokenSource.CreateLinkedTokenSource(cancellationToken);
        linkedCts.CancelAfter(_options.Value.RequestTimeout);

        var payload = new
        {
            schema = Schema,
            operationIdentity = request.OperationIdentity,
            sketchId = request.SketchId.Value,
            frame = new
            {
                origin = new { x = request.Frame.OriginX, y = request.Frame.OriginY, z = request.Frame.OriginZ },
                xAxis = new { x = request.Frame.XAxis.X, y = request.Frame.XAxis.Y, z = request.Frame.XAxis.Z },
                yAxis = new { x = request.Frame.YAxis.X, y = request.Frame.YAxis.Y, z = request.Frame.YAxis.Z },
                zAxis = new { x = request.Frame.ZAxis.X, y = request.Frame.ZAxis.Y, z = request.Frame.ZAxis.Z }
            },
            circles = request.Circles
                .OrderBy(x => x.Id.Value, StringComparer.Ordinal)
                .Select(x => new { id = x.Id.Value, x = x.X, y = x.Y, radius = x.Radius })
                .ToArray(),
            constraints = request.Constraints
                .OrderBy(x => x.Id.Value, StringComparer.Ordinal)
                .Select(x => new
                {
                    id = x.Id.Value,
                    kind = x.Kind switch
                    {
                        SketchConstraintKind.Fixed => "fixed",
                        _ => throw new ArgumentOutOfRangeException()
                    },
                    geometryId = x.GeometryId.Value
                })
                .ToArray(),
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
            var dto = await JsonSerializer.DeserializeAsync<SketchResponseDto>(
                stream,
                new JsonSerializerOptions(JsonSerializerDefaults.Web),
                linkedCts.Token);

            if (dto is null)
                return Failed("Kernel returned an empty sketch result.");

            if (!string.Equals(dto.Schema, Schema, StringComparison.Ordinal))
                return Failed("Kernel returned an unexpected sketch schema.");

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
                return Failed("Kernel sketch status is inconsistent with succeeded.");

            return new SketchKernelResult(
                status,
                dto.ResultId is null ? null : new ContractResultId(dto.ResultId),
                dto.EvidenceHash,
                dto.Circles?
                    .Select(x => new SketchCircleResult(x.Id, x.X, x.Y, x.Radius))
                    .OrderBy(x => x.Id, StringComparer.Ordinal)
                    .ToArray() ?? Array.Empty<SketchCircleResult>(),
                dto.Converged,
                dto.Reason,
                dto.Iterations,
                dto.FinalResidualNorm,
                dto.FinalScaledResidualNorm,
                dto.FinalStepNorm,
                dto.DegreesOfFreedom,
                dto.VariableCount,
                dto.EquationCount,
                dto.Diagnostics ?? Array.Empty<string>());
        }
        catch (JsonException)
        {
            return Failed("Kernel returned invalid JSON.");
        }
        catch (HttpRequestException)
        {
            return Failed("Kernel sketch transport failed.");
        }
        catch (TaskCanceledException) when (cancellationToken.IsCancellationRequested)
        {
            throw;
        }
        catch (TaskCanceledException)
        {
            return Failed("Kernel sketch request timed out.");
        }
    }
}

internal sealed record SketchResponseDto(
    string Schema,
    string Status,
    bool Succeeded,
    string? ResultId,
    string? EvidenceHash,
    IReadOnlyList<SketchResponseCircleDto>? Circles,
    bool? Converged,
    string? Reason,
    int? Iterations,
    double? FinalResidualNorm,
    double? FinalScaledResidualNorm,
    double? FinalStepNorm,
    int? DegreesOfFreedom,
    int? VariableCount,
    int? EquationCount,
    IReadOnlyList<string>? Diagnostics);

internal sealed record SketchResponseCircleDto(
    string Id,
    double X,
    double Y,
    double Radius);
