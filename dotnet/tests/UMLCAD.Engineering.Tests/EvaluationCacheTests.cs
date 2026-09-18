using UMLCAD.Cad.Contracts;
using UMLCAD.Cad.Engine;
using UMLCAD.Cad.Semantics;

namespace UMLCAD.Engineering.Tests;

public sealed class EvaluationCacheTests
{
    [Fact]
    public async Task IncrementalRecomputeReusesUnaffectedCachedResults()
    {
        var a = new SemanticId(Guid.Parse("81000000-0000-0000-0000-000000000001"));
        var b = new SemanticId(Guid.Parse("81000000-0000-0000-0000-000000000002"));
        var c = new SemanticId(Guid.Parse("81000000-0000-0000-0000-000000000003"));

        var steps = new[]
        {
            Step(a, Array.Empty<SemanticId>()),
            Step(b, new[] { a }),
            Step(c, new[] { b }),
        };

        var counts = new Dictionary<SemanticId, int>();
        var executors = new IAsyncEvaluationStepExecutor[]
        {
            new CountingExecutor(a, counts),
            new CountingExecutor(b, counts),
            new CountingExecutor(c, counts),
        };

        var cache = new DeterministicEvaluationCache();
        var engine = new AsyncEvaluationEngine(executors);
        var plan = EvaluationPlanner.Plan(steps);

        var first = await engine.EvaluateAsync(
            plan,
            cache,
            plan.Steps.Select(x => x.StepId).ToHashSet());

        Assert.Equal(3, first.Count);
        Assert.Equal(1, counts[a]);
        Assert.Equal(1, counts[b]);
        Assert.Equal(1, counts[c]);

        var affected = plan.AffectedBy(new[] { b });
        var second = await engine.EvaluateAsync(
            plan,
            cache,
            affected);

        Assert.Equal(3, second.Count);
        Assert.Equal(1, counts[a]);
        Assert.Equal(2, counts[b]);
        Assert.Equal(2, counts[c]);
        Assert.Equal(first.Select(x => x.ResultIdentity), second.Select(x => x.ResultIdentity));
    }

    [Fact]
    public async Task CorruptSuccessfulCacheEntryFailsClosed()
    {
        var id = new SemanticId(Guid.Parse("82000000-0000-0000-0000-000000000001"));
        var step = Step(id, Array.Empty<SemanticId>());
        var plan = EvaluationPlanner.Plan(new[] { step });
        var identity = EvaluationIdentityBuilder.Build(step);

        var cache = new DeterministicEvaluationCache();
        cache.Put(
            identity,
            new EvaluationOutcome(
                id,
                identity,
                EvaluationOutcomeStatus.Succeeded,
                new AuthoritativeResultIdentity("solid:corrupt"),
                Array.Empty<string>()));

        var engine = new AsyncEvaluationEngine(
            new[] { new ThrowIfExecutedExecutor("Test.Operation") });

        await Assert.ThrowsAsync<InvalidOperationException>(() =>
            engine.EvaluateAsync(
                plan,
                cache,
                Array.Empty<SemanticId>().ToHashSet()));
    }

    private static EvaluationStep Step(
        SemanticId id,
        IReadOnlyList<SemanticId> dependencies) =>
        new(
            id,
            "Test.Operation",
            $"(definition:{id.Value:D})",
            dependencies,
            new[] { new EvaluationInputIdentity("feature", id.Value.ToString("D")) },
            "configuration:default",
            "model-standard-v1",
            "kernel-contract-v1",
            "representation:none");

    private sealed class CountingExecutor(
        SemanticId id,
        IDictionary<SemanticId, int> counts) : IAsyncEvaluationStepExecutor
    {
        public string OperationKind => "Test.Operation";

        public Task<EvaluationOutcome> ExecuteAsync(
            EvaluationStep step,
            EvaluationIdentity identity,
            IReadOnlyDictionary<SemanticId, EvaluationOutcome> completed,
            CancellationToken cancellationToken)
        {
            if (step.StepId != id)
                throw new InvalidOperationException("Unexpected step.");

            counts[id] = counts.TryGetValue(id, out var value) ? value + 1 : 1;

            var result = new AuthoritativeCadResult(
                new AuthoritativeResultIdentity($"solid:{id.Value:D}"),
                AuthoritativeResultKind.Solid,
                AuthoritativeResultStatus.Succeeded,
                id,
                new[]
                {
                    new TopologyBinding("Face", $"f:{id.Value:D}", id),
                },
                new ResultEvidence(
                    "test",
                    "1",
                    "evidence",
                    Array.Empty<TopologyEvolution>(),
                    Array.Empty<string>()));

            return Task.FromResult(
                new EvaluationOutcome(
                    id,
                    identity,
                    EvaluationOutcomeStatus.Succeeded,
                    result.Identity,
                    Array.Empty<string>())
                {
                    Result = result,
                });
        }
    }

    private sealed class ThrowIfExecutedExecutor(string operationKind)
        : IAsyncEvaluationStepExecutor
    {
        public string OperationKind => operationKind;

        public Task<EvaluationOutcome> ExecuteAsync(
            EvaluationStep step,
            EvaluationIdentity identity,
            IReadOnlyDictionary<SemanticId, EvaluationOutcome> completed,
            CancellationToken cancellationToken) =>
            throw new InvalidOperationException("Corrupt cache should have prevented execution.");
    }
}
