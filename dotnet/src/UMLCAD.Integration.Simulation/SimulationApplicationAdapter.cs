using UMLCAD.Science;

namespace UMLCAD.Integration.Simulation;

public interface ISimulationApplicationAdapter
{
    string ApplicationId { get; }

    bool Supports(PhenomenonKind phenomenon);

    PhenomenaSimulationResult Simulate(PhenomenaSimulationRequest request);
}

public sealed class SimulationApplicationProvider : IPhenomenaSimulationProvider
{
    private readonly ISimulationApplicationAdapter _adapter;

    public SimulationApplicationProvider(ISimulationApplicationAdapter adapter)
    {
        _adapter = adapter ?? throw new ArgumentNullException(nameof(adapter));
        if (string.IsNullOrWhiteSpace(_adapter.ApplicationId))
            throw new ArgumentException("Simulation application adapter must have an identifier.", nameof(adapter));
    }

    public string ProviderId => _adapter.ApplicationId;

    public bool Supports(PhenomenonKind phenomenon) => _adapter.Supports(phenomenon);

    public PhenomenaSimulationResult Simulate(PhenomenaSimulationRequest request)
    {
        ArgumentNullException.ThrowIfNull(request);
        var result = _adapter.Simulate(request);

        if (!string.Equals(result.ProviderId, ProviderId, StringComparison.Ordinal))
            throw new InvalidOperationException(
                $"Simulation application adapter '{ProviderId}' returned a mismatched ProviderId.");

        return result;
    }
}
