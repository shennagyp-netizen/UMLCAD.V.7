using UMLCAD.Cad.Contracts;
using UMLCAD.Cad.Engine;
using UMLCAD.Cad.Semantics;
using Xunit;

namespace UMLCAD.ApplicationLayer.Tests;

public sealed class EvaluationHistoryTests
{
    [Fact]
    public void Equivalent_history_snapshots_have_identical_deterministic_identity()
    {
        var plan = new CadEvaluationPlan([new CadId("op")]);
        var first = new Dictionary<CadId, CadEvaluationOutcome>
        {
            [new CadId("op")] = Succeeded("result:1", "evidence:1")
        };
        var second = new Dictionary<CadId, CadEvaluationOutcome>
        {
            [new CadId("op")] = Succeeded("result:1", "evidence:1")
        };

        var a = CadEvaluationHistory.Create(plan, first);
        var b = CadEvaluationHistory.Create(plan, second);

        Assert.Equal(a.Identity, b.Identity);
        Assert.Equal(a.Entries, b.Entries);
    }

    [Fact]
    public void Changing_an_authoritative_result_changes_history_identity()
    {
        var plan = new CadEvaluationPlan([new CadId("op")]);
        var first = CadEvaluationHistory.Create(
            plan,
            new Dictionary<CadId, CadEvaluationOutcome>
            {
                [new CadId("op")] = Succeeded("result:1", "evidence:1")
            });

        var changed = CadEvaluationHistory.Create(
            plan,
            new Dictionary<CadId, CadEvaluationOutcome>
            {
                [new CadId("op")] = Succeeded("result:2", "evidence:2")
            });

        Assert.NotEqual(first.Identity, changed.Identity);
    }

    [Fact]
    public async Task Rebuild_replaces_history_snapshot_without_mutating_the_previous_snapshot()
    {
        var part =
            CadPartProgram.Create("p", "P")
                .Sketch(
                    new SketchOperation(
                        new CadId("sketch"),
                        new CadId("body"),
                        new Sketch(
                            new CadId("sketch"),
                            "Sketch",
                            [
                                new CircleGeometry(
                                    new CadId("circle"),
                                    0,
                                    0,
                                    10)
                            ],
                            Array.Empty<SketchConstraint>(),
                            Array.Empty<CadReference>())))
                .Part;

        var gateway = new SequencedGateway();
        var engine = new CadEvaluationEngine(gateway);

        var first = await engine.EvaluateAsync(
            part,
            cancellationToken: TestContext.Current.CancellationToken);

        var firstHistory = first.History;
        var firstEntry = firstHistory.Entries.Single();

        var second = await engine.RebuildAsync(
            part,
            new HashSet<CadId> { new("sketch") },
            cancellationToken: TestContext.Current.CancellationToken);

        var secondEntry = second.History.Entries.Single();

        Assert.NotSame(firstHistory, second.History);
        Assert.NotEqual(firstHistory.Identity, second.History.Identity);
        Assert.Equal("evidence:1", firstEntry.EvidenceHash);
        Assert.Equal("evidence:2", secondEntry.EvidenceHash);
        Assert.Equal(second.History, engine.PreviousHistory);
        Assert.Equal("evidence:1", firstHistory.Entries.Single().EvidenceHash);
    }

    private static CadEvaluationOutcome Succeeded(
        string resultId,
        string evidenceHash) =>
        new(
            new CadId("op"),
            "Cad.Test",
            new CadEvaluationIdentity("eval:" + evidenceHash),
            CadEvaluationStatus.Succeeded,
            new CadResult(
                new CadResultId(resultId),
                CadResultKind.Body,
                new CadId("op"),
                Array.Empty<CadResultId>(),
                evidenceHash,
                Array.Empty<KernelTopologyBinding>()),
            Array.Empty<CadDiagnostic>());

    private sealed class SequencedGateway : IKernelGateway
    {
        private int _count;

        public Task<KernelOperationResponse> EvaluateAsync(
            KernelOperationRequest request,
            CancellationToken cancellationToken = default)
        {
            var sequence = Interlocked.Increment(ref _count);

            return Task.FromResult(
                KernelOperationResponse.Success(
                    request,
                    new CadResultId("result:" + sequence),
                    "evidence:" + sequence,
                    [new KernelTopologyBinding("operation", request.OperationId.Value)]));
        }
    }
}
