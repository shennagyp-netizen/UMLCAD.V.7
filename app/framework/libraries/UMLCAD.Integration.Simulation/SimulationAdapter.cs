using UMLCAD.Science;
namespace UMLCAD.Integration.Simulation;
public interface IExternalSimulationProvider{string ProviderId{get;}Task<SimulationResult> RunAsync(SimulationRequest request,CancellationToken cancellationToken=default);}
public sealed class SimulationProviderAdapter(IExternalSimulationProvider provider):IPhenomenaSimulationService
{
    public Task<SimulationResult> GetOrRunAsync(SimulationRequest request,CancellationToken cancellationToken=default)=>provider.RunAsync(request,cancellationToken);
}
