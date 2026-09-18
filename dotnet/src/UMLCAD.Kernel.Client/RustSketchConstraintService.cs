using System.Net;
using System.Net.Http.Json;
using System.Text.Json;
using Microsoft.Extensions.Options;
using UMLCAD.Cad.Contracts;

namespace UMLCAD.Kernel.Client;

public sealed class RustSketchConstraintService : ISketchConstraintService
{
    private const string Endpoint = "v1/geometry/solve-sketch";
    private const string Schema = SketchSolveRequest.ContractSchema;

    private static SketchSolveKernelResult Failed(
        string diagnostic,
        GeometryKernelStatus status = GeometryKernelStatus.Failed) =>
        new(
            status,
            false,
            status == GeometryKernelStatus.Unsupported ? "unsupported" : "failed",
            0,
            0d,
            0d,
            0d,
            0d,
            0d,
            0,
            0,
            0,
            0,
            0d,
            Array.Empty<SketchSolvedCircle>(),
            new[] { diagnostic });

    private readonly HttpClient _httpClient;
    private readonly IOptions<RustKernelOptions> _options;

    public RustSketchConstraintService(
        HttpClient httpClient,
        IOptions<RustKernelOptions> options)
    {
        _httpClient = httpClient ?? throw new ArgumentNullException(nameof(httpClient));
        _options = options ?? throw new ArgumentNullException(nameof(options));
    }

    public async Task<SketchSolveKernelResult> SolveAsync(
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
            circles = request.Circles.Select(circle => new
            {
                id = circle.Id,
                x = circle.X,
                y = circle.Y,
                radius = circle.Radius,
            }).ToArray(),
            fixedConstraints = request.FixedConstraints.Select(constraint => new
            {
                id = constraint.Id,
                geometryId = constraint.GeometryId,
            }).ToArray(),
            tolerance = new
            {
                absolute = request.Tolerance.Absolute,
                relative = request.Tolerance.Relative,
            },
            options = new
            {
                maxIterations = request.Options.MaxIterations,
                residualTolerance = request.Options.ResidualTolerance,
                stepTolerance = request.Options.StepTolerance,
                initialDamping = request.Options.InitialDamping,
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
            var dto = await JsonSerializer.DeserializeAsync<SketchSolveResponseDto>(
                stream,
                new JsonSerializerOptions(JsonSerializerDefaults.Web),
                linkedCts.Token);

            if (dto is null)
                return Failed("Kernel returned an empty sketch-solver result.");

            if (!string.Equals(dto.Schema, Schema, StringComparison.Ordinal))
                return Failed("Kernel returned an unexpected sketch-solver schema.");

            var status = dto.Status switch
            {
                "succeeded" => GeometryKernelStatus.Succeeded,
                "failed" => GeometryKernelStatus.Failed,
                "unsupported" => GeometryKernelStatus.Unsupported,
                "ambiguous" => GeometryKernelStatus.Ambiguous,
                "indeterminate" => GeometryKernelStatus.Indeterminate,
                _ => GeometryKernelStatus.Failed,
            };

            if (dto.Succeeded != (status == GeometryKernelStatus.Succeeded))
                return Failed(
                    "Kernel sketch-solver status is inconsistent with succeeded.");

            try
            {
                return new SketchSolveKernelResult(
                    status,
                    dto.Succeeded,
                    dto.Reason,
                    dto.Iterations,
                    dto.InitialResidualNorm,
                    dto.FinalResidualNorm,
                    dto.InitialScaledResidualNorm,
                    dto.FinalScaledResidualNorm,
                    dto.FinalStepNorm,
                    dto.VariableCount,
                    dto.EquationCount,
                    dto.Rank,
                    dto.DegreesOfFreedom,
                    dto.ConditionEstimate,
                    dto.Geometry?
                        .Select(circle => new SketchSolvedCircle(
                            circle.Id,
                            circle.X,
                            circle.Y,
                            circle.Radius))
                        .ToArray() ?? Array.Empty<SketchSolvedCircle>(),
                    dto.Diagnostics ?? Array.Empty<string>());
            }
            catch (ArgumentException)
            {
                return Failed("Kernel returned invalid sketch-solver diagnostics.");
            }
        }
        catch (JsonException)
        {
            return Failed("Kernel returned invalid JSON.");
        }
        catch (HttpRequestException)
        {
            return Failed("Kernel sketch-solver transport failed.");
        }
        catch (TaskCanceledException) when (cancellationToken.IsCancellationRequested)
        {
            throw;
        }
        catch (TaskCanceledException)
        {
            return Failed("Kernel sketch-solver request timed out.");
        }
    }
}

internal sealed record SketchSolveResponseDto(
    string Schema,
    string Status,
    bool Succeeded,
    string Reason,
    int Iterations,
    double InitialResidualNorm,
    double FinalResidualNorm,
    double InitialScaledResidualNorm,
    double FinalScaledResidualNorm,
    double FinalStepNorm,
    int VariableCount,
    int EquationCount,
    int Rank,
    int DegreesOfFreedom,
    double ConditionEstimate,
    IReadOnlyList<SketchSolveResponseCircleDto>? Geometry,
    IReadOnlyList<string>? Diagnostics);

internal sealed record SketchSolveResponseCircleDto(
    string Id,
    double X,
    double Y,
    double Radius);
