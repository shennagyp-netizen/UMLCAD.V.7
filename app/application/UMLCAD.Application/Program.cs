using UMLCAD.Cad.Engine;
using UMLCAD.Cad.Expressions;
using UMLCAD.Cad.Semantics;

namespace UMLCAD.Application;

public static class BenchViseScenario
{
    public static CadPart Create()
    {
        return CadPartProgram.Create("bench-vise", "Bench Vise")
            .Sketch(new SketchOperation(
                new CadId("sketch"),
                new CadId("body"),
                new Sketch(
                    new CadId("sketch"),
                    "Profile",
                    new SketchGeometry[]
                    {
                        new CircleGeometry(new CadId("circle"), 0, 0, 25)
                    },
                    Array.Empty<SketchConstraint>(),
                    Array.Empty<CadReference>())))
            .Extrude(new ExtrusionOperation(
                new CadId("extrude"),
                new CadId("body"),
                new CadId("sketch"),
                CadExpression.Constant(80),
                "+Z"))
            .Hole(new HoleOperation(
                new CadId("hole"),
                new CadId("body"),
                new CadId("extrude"),
                CadExpression.Constant(10),
                CadExpression.Constant(25)))
            .Part;
    }

    public static string Describe()
    {
        var part = Create();
        var plan = new CadDependencyGraph(part).Plan();
        return string.Join(" -> ", plan.OperationIds);
    }
}

internal static class Program
{
    private static void Main()
    {
        Console.WriteLine($"Semantic pipeline: {BenchViseScenario.Describe()}");
    }
}
