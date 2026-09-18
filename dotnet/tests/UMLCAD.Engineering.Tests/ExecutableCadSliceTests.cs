using UMLCAD.Cad.Contracts;
using UMLCAD.Cad.Engine;
using UMLCAD.Cad.Semantics;

namespace UMLCAD.Engineering.Tests;

public sealed class ExecutableCadSliceTests
{
    [Fact]
    public async Task TypedBoxSpecificationEvaluatesToAuthoritativeResult()
    {
        var featureId = new SemanticId(Guid.Parse("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa"));
        var partId = new SemanticId(Guid.Parse("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb"));

        var specification = new AxisAlignedBoxSolidSpecification(
            featureId,
            partId,
            "Seed Cube",
            Array.Empty<SemanticId>(),
            0d,
            0d,
            0d,
            2d,
            3d,
            4d);

        var fake = new FakeGeometryService(
            new AxisAlignedBoxSolidKernelResult(
                AxisAlignedBoxSolidKernelStatus.Succeeded,
                new ContractResultId("solid:seed"),
                "evidence:seed",
                new[]
                {
                    new AxisAlignedBoxSolidKernelTopology("Face", "f_bottom"),
                    new AxisAlignedBoxSolidKernelTopology("Face", "f_top"),
                    new AxisAlignedBoxSolidKernelTopology("Face", "f_back"),
                    new AxisAlignedBoxSolidKernelTopology("Face", "f_front"),
                    new AxisAlignedBoxSolidKernelTopology("Face", "f_left"),
                    new AxisAlignedBoxSolidKernelTopology("Face", "f_right"),
                },
                24d,
                52d,
                new KernelVector3(1d, 1.5d, 2d),
                Array.Empty<string>()));

        var evaluator = new AxisAlignedBoxSolidEvaluator(fake);

        var result = await evaluator.EvaluateAsync(
            specification,
            "configuration:default",
            new KernelTolerance(1e-9, 1e-9));

        Assert.Equal(featureId, result.ProducingSemanticId);
        Assert.Equal(AuthoritativeResultKind.Solid, result.Kind);
        Assert.Equal(AuthoritativeResultStatus.Succeeded, result.Status);
        Assert.Equal(6, result.TopologyBindings.Count);
        Assert.True(fake.LastRequest is not null);
        Assert.Equal(specification.CanonicalDefinition, fake.LastRequest!.OperationIdentity == featureId.Value.ToString("D")
            ? specification.CanonicalDefinition
            : string.Empty);
    }

    [Fact]
    public void SpecificationProducesEvaluationStepWithSemanticInputs()
    {
        var featureId = new SemanticId(Guid.Parse("cccccccc-cccc-cccc-cccc-cccccccccccc"));
        var partId = new SemanticId(Guid.Parse("dddddddd-dddd-dddd-dddd-dddddddddddd"));

        var specification = new AxisAlignedBoxSolidSpecification(
            featureId,
            partId,
            "Cube",
            Array.Empty<SemanticId>(),
            0d,
            0d,
            0d,
            10d,
            20d,
            30d);

        var step = EvaluationStepFactory.Create(
            specification,
            "configuration:default",
            "model-standard-v1",
            "render-mesh-v1");

        Assert.Equal(featureId, step.StepId);
        Assert.Equal(specification.CanonicalDefinition, step.NormalizedDefinition);
        Assert.Contains(step.Inputs, x =>
            x.Role == "feature" &&
            x.Identity == featureId.Value.ToString("D"));
        Assert.Contains(step.Inputs, x =>
            x.Role == "part" &&
            x.Identity == partId.Value.ToString("D"));

        var identityA = EvaluationIdentityBuilder.Build(step);
        var identityB = EvaluationIdentityBuilder.Build(step);

        Assert.Equal(identityA, identityB);
    }

    [Fact]
    public void IncrementalAffectedSetMatchesDependencyClosure()
    {
        var a = new SemanticId(Guid.Parse("eeeeeeee-eeee-eeee-eeee-eeeeeeeeeeee"));
        var b = new SemanticId(Guid.Parse("ffffffff-ffff-ffff-ffff-ffffffffffff"));
        var c = new SemanticId(Guid.Parse("12121212-1212-1212-1212-121212121212"));

        var steps = new[]
        {
            MakeStep(a, Array.Empty<SemanticId>()),
            MakeStep(b, new[] { a }),
            MakeStep(c, new[] { b }),
        };

        var plan = EvaluationPlanner.Plan(steps);

        Assert.Equal(
            new HashSet<SemanticId> { a, b, c },
            plan.AffectedBy(new[] { a }));
        Assert.Equal(
            new HashSet<SemanticId> { c },
            plan.AffectedBy(new[] { b }));
    }

    private static EvaluationStep MakeStep(
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

    private sealed class FakeGeometryService(
        AxisAlignedBoxSolidKernelResult result) : IAuthoritativeGeometryService
    {
        public AxisAlignedBoxSolidRequest? LastRequest { get; private set; }

        public Task<AxisAlignedBoxSolidKernelResult> BuildAxisAlignedBoxSolidAsync(
            AxisAlignedBoxSolidRequest request,
            CancellationToken cancellationToken = default)
        {
            LastRequest = request;
            return Task.FromResult(result);
        }
    }
}
