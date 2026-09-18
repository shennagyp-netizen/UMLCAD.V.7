using UMLCAD.Cad.Contracts;
using UMLCAD.Cad.Engine;
using UMLCAD.Cad.Semantics;

namespace UMLCAD.Engineering.Tests;

public sealed class CadCoreContractTests
{
    [Fact]
    public void ReferenceResolverResolvesExactTopologyBinding()
    {
        var producer = new SemanticId(Guid.Parse("11111111-1111-1111-1111-111111111111"));
        var result = new AuthoritativeCadResult(
            new AuthoritativeResultIdentity("result-001"),
            AuthoritativeResultKind.Solid,
            AuthoritativeResultStatus.Succeeded,
            producer,
            new[]
            {
                new TopologyBinding("Face", "face:top", producer),
                new TopologyBinding("Face", "face:bottom", producer),
            },
            new ResultEvidence(
                "UMLCAD.Geometry.BuildSolid",
                "1.0",
                "evidence-001",
                Array.Empty<TopologyEvolution>(),
                Array.Empty<string>()));

        var catalog = new AuthoritativeResultCatalog();
        catalog.Register(result);

        var reference = new TopologyReference
        {
            ReferenceId = SemanticId.New(),
            ContextId = ReferenceContextId.New(),
            AuthoritativeResultId = "result-001",
            TopologyKind = "Face",
            TopologyKey = "face:top",
        };

        var resolution = catalog.Resolve(reference);

        Assert.Equal(ReferenceResolutionStatus.Resolved, resolution.Status);
        Assert.Equal("result-001", resolution.ResolvedResultId);
    }

    [Fact]
    public void ReferenceResolverFailsClosedForWrongTopologyKind()
    {
        var producer = SemanticId.New();
        var result = SuccessfulResult("result-002", producer, ("Face", "face:top"));

        var catalog = new AuthoritativeResultCatalog();
        catalog.Register(result);

        var reference = new TopologyReference
        {
            ReferenceId = SemanticId.New(),
            ContextId = ReferenceContextId.New(),
            AuthoritativeResultId = "result-002",
            TopologyKind = "Edge",
            TopologyKey = "face:top",
        };

        var resolution = catalog.Resolve(reference);

        Assert.Equal(ReferenceResolutionStatus.Missing, resolution.Status);
        Assert.Null(resolution.ResolvedResultId);
    }

    [Fact]
    public void ReferenceResolverDetectsAmbiguousSemanticProducer()
    {
        var producer = SemanticId.New();
        var catalog = new AuthoritativeResultCatalog();

        catalog.Register(SuccessfulResult("result-a", producer, ("Face", "a")));
        catalog.Register(SuccessfulResult("result-b", producer, ("Face", "b")));

        var reference = new SemanticReference
        {
            ReferenceId = SemanticId.New(),
            ContextId = ReferenceContextId.New(),
            TargetId = producer,
        };

        var resolution = catalog.Resolve(reference);

        Assert.Equal(ReferenceResolutionStatus.Ambiguous, resolution.Status);
        Assert.Null(resolution.ResolvedResultId);
    }

    [Fact]
    public void EvaluationPlannerProducesStableDependencyOrder()
    {
        var a = Id("10000000-0000-0000-0000-000000000001");
        var b = Id("10000000-0000-0000-0000-000000000002");
        var c = Id("10000000-0000-0000-0000-000000000003");

        var steps = new[]
        {
            Step(c, "Feature.C", new[] { b }),
            Step(a, "Feature.A", Array.Empty<SemanticId>()),
            Step(b, "Feature.B", new[] { a }),
        };

        var plan = EvaluationPlanner.Plan(steps);

        Assert.Equal(new[] { a, b, c }, plan.OrderedStepIds);
        Assert.Equal(
            new HashSet<SemanticId> { b, c },
            plan.AffectedBy(new[] { b }));
    }

    [Fact]
    public void EvaluationPlannerRejectsCycles()
    {
        var a = Id("20000000-0000-0000-0000-000000000001");
        var b = Id("20000000-0000-0000-0000-000000000002");

        var steps = new[]
        {
            Step(a, "Feature.A", new[] { b }),
            Step(b, "Feature.B", new[] { a }),
        };

        Assert.Throws<InvalidOperationException>(() => EvaluationPlanner.Plan(steps));
    }

    [Fact]
    public void EvaluationIdentityChangesWhenToleranceOrUpstreamInputChanges()
    {
        var step = Step(
            Id("30000000-0000-0000-0000-000000000001"),
            "Feature.Extrude",
            Array.Empty<SemanticId>(),
            inputs: new[]
            {
                new EvaluationInputIdentity("profile", "profile-1"),
                new EvaluationInputIdentity("length", "10mm"),
            },
            tolerance: "model-standard-v1");

        var same = EvaluationIdentityBuilder.Build(step);

        var differentTolerance = EvaluationIdentityBuilder.Build(
            step with { TolerancePolicy = "model-standard-v2" });

        var differentInput = EvaluationIdentityBuilder.Build(
            step with
            {
                Inputs = new[]
                {
                    new EvaluationInputIdentity("profile", "profile-2"),
                    new EvaluationInputIdentity("length", "10mm"),
                },
            });

        Assert.NotEqual(same, differentTolerance);
        Assert.NotEqual(same, differentInput);
    }

    [Fact]
    public void KernelResultIntegrationPreservesContractProvenanceAndTopologyKind()
    {
        var producer = SemanticId.New();
        var request = new KernelRequest(
            "UMLCAD.Geometry.BuildSolid",
            new ContractVersion("2.1"),
            KernelOperationKind.BuildSolid,
            "operation-001",
            new[]
            {
                new KernelInputBinding(
                    "profile",
                    new ContractResultId("profile-001"),
                    null,
                    "planar-profile"),
            },
            "(depth=10mm)");

        var kernelResult = new KernelResult(
            new ContractResultId("solid-001"),
            KernelResultStatus.Succeeded,
            new[]
            {
                new ContractTopologyId(
                    new ContractResultId("solid-001"),
                    "Face",
                    "face:1"),
            },
            "evidence-002",
            Array.Empty<string>());

        var result = ResultIntegrator.Integrate(
            producer,
            AuthoritativeResultKind.Solid,
            request,
            kernelResult);

        Assert.Equal("UMLCAD.Geometry.BuildSolid", result.Evidence.ContractId);
        Assert.Equal("2.1", result.Evidence.ContractVersion);
        Assert.Equal("Face", result.TopologyBindings.Single().TopologyKind);
        Assert.Equal("face:1", result.TopologyBindings.Single().TopologyKey);
        Assert.Equal(AuthoritativeResultStatus.Succeeded, result.Status);
    }

    [Fact]
    public void SuccessfulKernelGeometryWithoutTopologyFailsClosed()
    {
        var producer = SemanticId.New();
        var request = new KernelRequest(
            "UMLCAD.Geometry.BuildSolid",
            new ContractVersion("1.0"),
            KernelOperationKind.BuildSolid,
            "operation-002",
            Array.Empty<KernelInputBinding>(),
            "(depth=5mm)");

        var kernelResult = new KernelResult(
            new ContractResultId("solid-002"),
            KernelResultStatus.Succeeded,
            Array.Empty<ContractTopologyId>(),
            "evidence-003",
            Array.Empty<string>());

        Assert.Throws<InvalidOperationException>(() =>
            ResultIntegrator.Integrate(
                producer,
                AuthoritativeResultKind.Solid,
                request,
                kernelResult));
    }

    private static SemanticId Id(string value) => new(Guid.Parse(value));

    private static EvaluationStep Step(
        SemanticId id,
        string operationKind,
        IReadOnlyList<SemanticId> dependencies,
        IReadOnlyList<EvaluationInputIdentity>? inputs = null,
        string tolerance = "model-standard-v1")
    {
        return new EvaluationStep(
            id,
            operationKind,
            $"({operationKind} definition)",
            dependencies,
            inputs ?? Array.Empty<EvaluationInputIdentity>(),
            "configuration:default",
            tolerance,
            "kernel-contract-v1",
            "representation:none");
    }

    private static AuthoritativeCadResult SuccessfulResult(
        string resultId,
        SemanticId producer,
        params (string Kind, string Key)[] bindings)
    {
        return new AuthoritativeCadResult(
            new AuthoritativeResultIdentity(resultId),
            AuthoritativeResultKind.Solid,
            AuthoritativeResultStatus.Succeeded,
            producer,
            bindings.Select(x => new TopologyBinding(x.Kind, x.Key, producer)).ToArray(),
            new ResultEvidence(
                "test",
                "1",
                "evidence",
                Array.Empty<TopologyEvolution>(),
                Array.Empty<string>()));
    }
}
