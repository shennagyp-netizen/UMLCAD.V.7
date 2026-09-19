using System.Collections.Concurrent;
using System.Security.Cryptography;
using System.Text;

namespace UMLCAD.Science;

public enum PhenomenonKind
{
    Thermal,
    Structural,
    Fluid,
    Electromagnetic,
    CoupledMultiphysics
}

public enum SimulationStatus
{
    Completed,
    Failed,
    Invalid,
    Incomplete,
    Indeterminate
}

public readonly record struct SimulationIdentity(string Value)
{
    public static SimulationIdentity Create(
        PhenomenonKind phenomenon,
        string modelContract,
        IReadOnlyDictionary<string, string> inputs)
    {
        var canonical =
            phenomenon + "|" +
            modelContract + "|" +
            string.Join(
                "|",
                inputs.OrderBy(x => x.Key, StringComparer.Ordinal)
                    .Select(x => $"{x.Key}={x.Value}"));

        return new SimulationIdentity(
            "sha256:" + Convert.ToHexString(
                SHA256.HashData(
                    Encoding.UTF8.GetBytes(canonical)))
                .ToLowerInvariant());
    }
}

public sealed record SimulationRequest(
    PhenomenonKind Phenomenon,
    string ModelContract,
    IReadOnlyDictionary<string, string> Inputs)
{
    public SimulationIdentity Identity =>
        SimulationIdentity.Create(
            Phenomenon,
            ModelContract,
            Inputs);
}

public sealed record SpatialRegion(
    string Id,
    string Kind,
    string FrameId,
    SimulationIdentity Source);

public sealed record ScalarField(
    string Id,
    string QuantityId,
    string FrameId,
    IReadOnlyList<(double X, double Y, double Z, double Value)> Samples,
    SimulationIdentity Source);

public sealed record SimulationResult(
    SimulationIdentity Identity,
    SimulationStatus Status,
    IReadOnlyList<SpatialRegion> Regions,
    IReadOnlyList<ScalarField> Fields,
    IReadOnlyList<string> Diagnostics);

public interface IPhenomenaSimulationService
{
    Task<SimulationResult> GetOrRunAsync(
        SimulationRequest request,
        CancellationToken cancellationToken = default);
}

public sealed class ConcurrentSimulationCache :
    IPhenomenaSimulationService
{
    private readonly IPhenomenaSimulationService _inner;
    private readonly ConcurrentDictionary<
        string,
        Lazy<Task<SimulationResult>>> _inflight = new(
            StringComparer.Ordinal);

    public ConcurrentSimulationCache(
        IPhenomenaSimulationService inner)
    {
        _inner = inner ?? throw new ArgumentNullException(nameof(inner));
    }

    public async Task<SimulationResult> GetOrRunAsync(
        SimulationRequest request,
        CancellationToken cancellationToken = default)
    {
        ArgumentNullException.ThrowIfNull(request);

        var lazy = _inflight.GetOrAdd(
            request.Identity.Value,
            _ => new Lazy<Task<SimulationResult>>(
                () => _inner.GetOrRunAsync(
                    request,
                    CancellationToken.None),
                LazyThreadSafetyMode.ExecutionAndPublication));

        var result = await lazy.Value.WaitAsync(cancellationToken);

        if (result.Status != SimulationStatus.Completed)
            _inflight.TryRemove(
                request.Identity.Value,
                out _);

        return result;
    }
}
