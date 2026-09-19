namespace UMLCAD.Science;

public interface IPhenomenaSimulationService
{
    PhenomenaSimulationResult Simulate(PhenomenaSimulationRequest request);
}

public sealed class PhenomenaSimulationService : IPhenomenaSimulationService
{
    private readonly IPhenomenaSimulationProvider[] _providers;

    public PhenomenaSimulationService(IEnumerable<IPhenomenaSimulationProvider> providers)
    {
        ArgumentNullException.ThrowIfNull(providers);

        _providers = providers
            .Where(provider => provider is not null)
            .OrderBy(provider => provider.ProviderId, StringComparer.Ordinal)
            .ToArray();

        if (_providers.Length == 0)
            throw new ArgumentException("At least one simulation provider is required.", nameof(providers));

        if (_providers.GroupBy(x => x.ProviderId, StringComparer.Ordinal).Any(g => g.Count() != 1))
            throw new ArgumentException("Simulation provider identifiers must be unique.", nameof(providers));
    }

    public PhenomenaSimulationResult Simulate(PhenomenaSimulationRequest request)
    {
        ArgumentNullException.ThrowIfNull(request);

        var provider = _providers.FirstOrDefault(x => x.Supports(request.Phenomenon));
        if (provider is null)
            throw new NotSupportedException(
                $"No simulation provider supports phenomenon '{request.Phenomenon}'.");

        var result = provider.Simulate(request);

        if (!string.Equals(result.RequestId, request.RequestId, StringComparison.Ordinal) ||
            result.Phenomenon != request.Phenomenon)
        {
            throw new InvalidOperationException(
                $"Simulation provider '{provider.ProviderId}' returned a result for the wrong request.");
        }

        if (!string.Equals(result.ProviderId, provider.ProviderId, StringComparison.Ordinal))
        {
            throw new InvalidOperationException(
                $"Simulation provider '{provider.ProviderId}' returned a mismatched ProviderId.");
        }

        return result;
    }
}
