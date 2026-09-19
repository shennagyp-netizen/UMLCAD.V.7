using System.Collections.Concurrent;
using UMLCAD.Science;

namespace UMLCAD.Engineering.Runtime;

public interface IPhenomenaSimulationExecutor
{
    Task<SimulationResult> ExecuteAsync(
        SimulationRequest request,
        CancellationToken cancellationToken = default);
}

public interface IPhenomenaSimulationService
{
    Task<SimulationResult> GetOrRunAsync(
        SimulationRequest request,
        CancellationToken cancellationToken = default);
}

public sealed class SimulationCache
{
    private readonly ConcurrentDictionary<
        SimulationIdentity,
        Lazy<Task<SimulationResult>>> _entries = new();

    public async Task<SimulationResult> GetOrRunAsync(
        SimulationRequest request,
        IPhenomenaSimulationExecutor executor,
        CancellationToken cancellationToken = default)
    {
        ArgumentNullException.ThrowIfNull(request);
        ArgumentNullException.ThrowIfNull(executor);

        var lazy = new Lazy<Task<SimulationResult>>(
            () => executor.ExecuteAsync(request, cancellationToken),
            LazyThreadSafetyMode.ExecutionAndPublication);

        var entry = _entries.GetOrAdd(request.Identity, lazy);

        try
        {
            var result = await entry.Value;

            if (!result.IsReusable(request))
                _entries.TryRemove(
                    new KeyValuePair<SimulationIdentity, Lazy<Task<SimulationResult>>>(
                        request.Identity,
                        entry));

            return result;
        }
        catch
        {
            _entries.TryRemove(
                new KeyValuePair<SimulationIdentity, Lazy<Task<SimulationResult>>>(
                    request.Identity,
                    entry));
            throw;
        }
    }

    public bool Contains(SimulationIdentity identity) =>
        _entries.ContainsKey(identity);

    public void Clear() => _entries.Clear();
}

public sealed class CachedPhenomenaSimulationService : IPhenomenaSimulationService
{
    private readonly IPhenomenaSimulationExecutor _executor;
    private readonly SimulationCache _cache;

    public CachedPhenomenaSimulationService(
        IPhenomenaSimulationExecutor executor,
        SimulationCache? cache = null)
    {
        _executor = executor ?? throw new ArgumentNullException(nameof(executor));
        _cache = cache ?? new SimulationCache();
    }

    public Task<SimulationResult> GetOrRunAsync(
        SimulationRequest request,
        CancellationToken cancellationToken = default)
    {
        ArgumentNullException.ThrowIfNull(request);
        return _cache.GetOrRunAsync(request, _executor, cancellationToken);
    }
}
