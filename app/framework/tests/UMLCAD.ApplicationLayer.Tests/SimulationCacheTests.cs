using System.Collections.Concurrent;
using UMLCAD.Engineering.Runtime;
using UMLCAD.Science;

namespace UMLCAD.ApplicationLayer.Tests;

public sealed class SimulationCacheTests
{
    [Fact]
    public async Task Concurrent_Identical_Requests_Execute_Exactly_Once()
    {
        var executionCount = 0;
        var executor = new CountingExecutor(
            () => Interlocked.Increment(ref executionCount));

        var service = new CachedPhenomenaSimulationService(executor);
        var request = Request("geometry-1");

        var results = await Task.WhenAll(
            service.GetOrRunAsync(request),
            service.GetOrRunAsync(request),
            service.GetOrRunAsync(request));

        Assert.Equal(1, executionCount);
        Assert.All(results, result => Assert.True(result.IsReusable(request)));
        Assert.All(
            results,
            result => Assert.Equal(request.Identity, result.RequestIdentity));
    }

    [Fact]
    public async Task Changing_A_Semantic_Input_Changes_Simulation_Identity()
    {
        var first = Request("geometry-1");
        var changed = Request("geometry-2");

        Assert.NotEqual(first.Identity, changed.Identity);
    }

    [Fact]
    public async Task NonReusable_Result_Is_Not_Kept_In_Cache()
    {
        var executions = 0;
        var executor = new CountingExecutor(
            () => Interlocked.Increment(ref executions),
            () => executions == 1
                ? SimulationResultStatus.Incomplete
                : SimulationResultStatus.Completed);

        var service = new CachedPhenomenaSimulationService(executor);
        var request = Request("geometry-cache");

        var first = await service.GetOrRunAsync(request);
        var second = await service.GetOrRunAsync(request);

        Assert.Equal(SimulationResultStatus.Incomplete, first.Status);
        Assert.Equal(SimulationResultStatus.Completed, second.Status);
        Assert.Equal(2, executions);
    }

    [Fact]
    public void Request_Identity_Changes_When_Any_Declared_Input_Changes()
    {
        var baseRequest = Request("geometry-identity");

        var variants = new[]
        {
            baseRequest with { },
            Request("geometry-changed"),
            CreateRequest(
                phenomenon: PhenomenonKind.Structural,
                model: "model-1",
                geometry: "geometry-identity",
                material: "material-1",
                process: "process-1",
                boundary: "bc-1",
                environment: "env-1",
                configuration: "config-1",
                numerical: "numerical-1",
                uncertainty: "uncertainty-1"),
            CreateRequest(
                phenomenon: PhenomenonKind.Thermal,
                model: "model-2",
                geometry: "geometry-identity",
                material: "material-1",
                process: "process-1",
                boundary: "bc-1",
                environment: "env-1",
                configuration: "config-1",
                numerical: "numerical-1",
                uncertainty: "uncertainty-1")
        };

        Assert.Equal(baseRequest.Identity, variants[0].Identity);
        Assert.NotEqual(baseRequest.Identity, variants[1].Identity);
        Assert.NotEqual(baseRequest.Identity, variants[2].Identity);
        Assert.NotEqual(baseRequest.Identity, variants[3].Identity);
    }

    private static SimulationRequest Request(string geometry) =>
        CreateRequest(
            PhenomenonKind.Thermal,
            "thermal-model-1",
            geometry,
            "material-1",
            "process-1",
            "bc-1",
            "environment-1",
            "config-1",
            "numerical-1",
            "uncertainty-1");

    private static SimulationRequest CreateRequest(
        PhenomenonKind phenomenon,
        string model,
        string geometry,
        string material,
        string process,
        string boundary,
        string environment,
        string configuration,
        string numerical,
        string uncertainty) =>
        new(
            phenomenon,
            model,
            geometry,
            material,
            process,
            boundary,
            environment,
            configuration,
            numerical,
            uncertainty);

    private sealed class CountingExecutor(
        Action? onExecute = null,
        Func<SimulationResultStatus>? statusFactory = null) : IPhenomenaSimulationExecutor
    {
        private readonly Action _onExecute = onExecute ?? (() => { });
        private readonly Func<SimulationResultStatus> _statusFactory =
            statusFactory ?? (() => SimulationResultStatus.Completed);

        public async Task<SimulationResult> ExecuteAsync(
            SimulationRequest request,
            CancellationToken cancellationToken = default)
        {
            _onExecute();
            await Task.Yield();

            return new SimulationResult(
                request,
                _statusFactory(),
                [],
                []);
        }
    }
}
