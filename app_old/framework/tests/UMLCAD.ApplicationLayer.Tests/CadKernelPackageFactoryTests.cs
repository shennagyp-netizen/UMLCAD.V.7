using System.Text.Json;
using UMLCAD.Cad.Contracts;
using UMLCAD.Cad.Engine;
using UMLCAD.Cad.Semantics;

namespace UMLCAD.ApplicationLayer.Tests;

public sealed class CadKernelPackageFactoryTests
{
    [Fact]
    public void Factory_Produces_Kernel_Build_Schema_With_Deterministic_Order()
    {
        var part = new CadPartDefinition(
            new CadId("part-1"),
            "Test Part",
            [
                new CadCircle(
                    new CadId("circle-2"),
                    new CadPoint2D(10, 20),
                    2.5),
                new CadLineSegment(
                    new CadId("line-1"),
                    new CadPoint2D(0, 0),
                    new CadPoint2D(20, 0)),
                new CadArc(
                    new CadId("arc-3"),
                    new CadPoint2D(5, 5),
                    4,
                    0,
                    1.5707963267948966)
            ],
            [
                new CadPlanarConstraint(
                    new CadId("constraint-1"),
                    CadPlanarConstraintKind.Horizontal,
                    new CadId("line-1"))
            ]);

        var identity = new CadBuildIdentity(
            "application-test",
            "1.0.0",
            "semantic-build-001");

        var factory = new CadKernelBuildPackageFactory();
        var first = factory.Create(part, identity);
        var second = factory.Create(part, identity);

        Assert.Equal(first.SemanticJson, second.SemanticJson);

        using var json = JsonDocument.Parse(first.SemanticJson);
        var root = json.RootElement;

        Assert.Equal(
            "uml-cad-build-package/1.0.0",
            root.GetProperty("schema").GetString());
        Assert.Equal(
            "semantic-build-001",
            root.GetProperty("buildIdentity").GetString());

        var semantic = root.GetProperty("semantic");
        var geometry = semantic
            .GetProperty("parts")[0]
            .GetProperty("geometry");

        Assert.Equal(3, geometry.GetArrayLength());
        Assert.Equal("arc-3", geometry[0].GetProperty("id").GetString());
        Assert.Equal("circle-2", geometry[1].GetProperty("id").GetString());
        Assert.Equal("line-1", geometry[2].GetProperty("id").GetString());

        var constraints = semantic
            .GetProperty("parts")[0]
            .GetProperty("constraints");

        Assert.Single(constraints);
        Assert.Equal("horizontal", constraints[0].GetProperty("kind").GetString());
        Assert.Equal(
            "line-1",
            constraints[0].GetProperty("references")[0].GetString());
    }

    [Fact]
    public void Part_Rejects_Constraint_Referencing_Unknown_Geometry()
    {
        Assert.Throws<ArgumentException>(
            () => new CadPartDefinition(
                new CadId("part-1"),
                "Invalid Part",
                [
                    new CadCircle(
                        new CadId("circle-1"),
                        new CadPoint2D(0, 0),
                        1)
                ],
                [
                    new CadPlanarConstraint(
                        new CadId("constraint-1"),
                        CadPlanarConstraintKind.Fixed,
                        new CadId("missing"))
                ]));
    }

    [Fact]
    public void Planar_Geometry_Rejects_NonFinite_Or_Invalid_Values()
    {
        Assert.Throws<ArgumentOutOfRangeException>(
            () => new CadPoint2D(double.NaN, 0));

        Assert.Throws<ArgumentException>(
            () => new CadLineSegment(
                new CadId("line"),
                new CadPoint2D(1, 1),
                new CadPoint2D(1, 1)));

        Assert.Throws<ArgumentOutOfRangeException>(
            () => new CadCircle(
                new CadId("circle"),
                new CadPoint2D(0, 0),
                0));
    }
}
