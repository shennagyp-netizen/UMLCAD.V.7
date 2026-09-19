using UMLCAD.Cad.Contracts;
using UMLCAD.Cad.Engine;
using UMLCAD.Cad.Semantics;

namespace UMLCAD.ApplicationLayer.Tests;

public sealed class DependencyPlanningTests
{
    [Fact]
    public void Plan_Produces_Deterministic_Dependency_First_Order()
    {
        var document = new CadDocumentDefinition(
            new CadId("document"),
            "Dependency Test",
            [
                new CadFeatureDefinition(
                    new CadId("feature-c"),
                    CadFeatureKind.Feature,
                    "C",
                    [new CadId("feature-b")]),
                new CadFeatureDefinition(
                    new CadId("feature-a"),
                    CadFeatureKind.Feature,
                    "A"),
                new CadFeatureDefinition(
                    new CadId("feature-b"),
                    CadFeatureKind.Feature,
                    "B",
                    [new CadId("feature-a")])
            ]);

        var plan = new CadDependencyGraph(document).CreatePlan();

        Assert.True(plan.IsValid);
        Assert.Equal(
            ["feature-a", "feature-b", "feature-c"],
            plan.EvaluationOrder.Select(id => id.Value).ToArray());
        Assert.Empty(plan.CyclePath);
    }

    [Fact]
    public void Plan_Reports_Complete_Cycle_Path()
    {
        var document = new CadDocumentDefinition(
            new CadId("document"),
            "Cycle Test",
            [
                new CadFeatureDefinition(
                    new CadId("a"),
                    CadFeatureKind.Feature,
                    "A",
                    [new CadId("b")]),
                new CadFeatureDefinition(
                    new CadId("b"),
                    CadFeatureKind.Feature,
                    "B",
                    [new CadId("c")]),
                new CadFeatureDefinition(
                    new CadId("c"),
                    CadFeatureKind.Feature,
                    "C",
                    [new CadId("a")])
            ]);

        var plan = new CadDependencyGraph(document).CreatePlan();

        Assert.False(plan.IsValid);
        Assert.Equal(CadEvaluationPlanStatus.CycleDetected, plan.Status);
        Assert.Equal(["a", "b", "c", "a"], plan.CyclePath.Select(id => id.Value).ToArray());
        Assert.Empty(plan.EvaluationOrder);
    }

    [Fact]
    public void Document_Rejects_Unknown_Feature_Dependency()
    {
        Assert.Throws<ArgumentException>(
            () => new CadDocumentDefinition(
                new CadId("document"),
                "Invalid Dependency",
                [
                    new CadFeatureDefinition(
                        new CadId("feature"),
                        CadFeatureKind.Feature,
                        "Feature",
                        [new CadId("missing")])
                ]));
    }

    [Fact]
    public void Feature_Copies_Dependency_List_To_Protect_Semantic_State()
    {
        var dependencies = new List<CadId> { new("base") };

        var feature = new CadFeatureDefinition(
            new CadId("derived"),
            CadFeatureKind.Feature,
            "Derived",
            dependencies);

        dependencies.Add(new CadId("later"));

        Assert.Single(feature.Dependencies);
        Assert.Equal(new CadId("base"), feature.Dependencies[0]);
    }
}
