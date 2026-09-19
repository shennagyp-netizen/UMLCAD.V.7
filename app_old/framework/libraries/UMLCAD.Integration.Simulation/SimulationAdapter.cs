using UMLCAD.Science;

namespace UMLCAD.Integration.Simulation;

public interface IExternalSimulationProvider
{
    string ProviderId { get; }

    Task<SimulationResult> RunAsync(
        SimulationRequest request,
        CancellationToken cancellationToken = default);
}

public sealed class SimulationProviderAdapter(
    IExternalSimulationProvider provider) : IPhenomenaSimulationService
{
    private readonly IExternalSimulationProvider _provider =
        provider ?? throw new ArgumentNullException(nameof(provider));

    public Task<SimulationResult> GetOrRunAsync(
        SimulationRequest request,
        CancellationToken cancellationToken = default) =>
        _provider.RunAsync(request, cancellationToken);
}
