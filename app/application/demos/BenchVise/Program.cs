using UMLCAD.Cad.Contracts;
using UMLCAD.Cad.Expressions;
using UMLCAD.Cad.Semantics;
using UMLCAD.Framework;

var part = CadPartProgram.Create(
        "bench-vise",
        "Bench Vise")
    .AddParameter(
        new CadParameter(
            "base_radius",
            CadExpression.Constant(20),
            "mm"))
    .AddSketch(
        new SketchOperation(
            new CadId("sketch"),
            new CadId("body"),
            new Sketch(
                new CadId("sketch"),
                "Base Profile",
                new SketchGeometry[]
                {
                    new CircleGeometry(
                        new CadId("outer"),
                        0,
                        0,
                        CadNumericValue.Expression(
                            CadExpression.Parameter("base_radius")))
                },
                Array.Empty<SketchConstraint>(),
                Array.Empty<CadReference>())))
    .Extrude(
        new ExtrusionOperation(
            new CadId("extrude"),
            new CadId("body"),
            new CadId("sketch"),
            CadExpression.Constant(70),
            "+Z"))
    .Hole(
        new HoleOperation(
            new CadId("hole"),
            new CadId("body"),
            new CadId("extrude"),
            CadExpression.Constant(8),
            CadExpression.Constant(35)));

using var application = UmlcadApplication.CreateDeterministicDemo();

var snapshot = await application.BuildAsync(part.Definition);

Console.WriteLine("UMLCAD Bench Vise");
Console.WriteLine($"Pipeline: {string.Join(" -> ", snapshot.Plan.OperationIds)}");
Console.WriteLine($"Succeeded: {snapshot.Succeeded}");
Console.WriteLine($"Current body result: {snapshot.CurrentBody(new CadId("body"))}");
