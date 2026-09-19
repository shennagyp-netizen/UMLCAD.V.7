using UMLCAD.Cad.Contracts;
using UMLCAD.Cad.Semantics;
using UMLCAD.Framework;

if (args.Contains("--e2e", StringComparer.Ordinal))
{
    using var application = UmlcadApplication.ConnectDefault();

    var part = new CadPartDefinition(
        new CadId("bench-vise-part"),
        "Bench Vise Demonstration Part",
        [
            new CadLineSegment(
                new CadId("base-edge"),
                new CadPoint2D(0, 0),
                new CadPoint2D(80, 0)),
            new CadCircle(
                new CadId("mount-hole"),
                new CadPoint2D(20, 20),
                5)
        ],
        [
            new CadPlanarConstraint(
                new CadId("base-horizontal"),
                CadPlanarConstraintKind.Horizontal,
                new CadId("base-edge"))
        ]);

    var result = await application.BuildAsync(
        part,
        new CadBuildIdentity(
            "uml-cad-bench-vise-demo",
            "1.0.0",
            "bench-vise-build-001"));

    if (!result.Succeeded)
    {
        foreach (var diagnostic in result.Diagnostics)
            Console.Error.WriteLine($"{diagnostic.Code}: {diagnostic.Message}");

        return 1;
    }

    Console.WriteLine("Bench Vise E2E build succeeded.");
    return 0;
}

Console.WriteLine("UMLCAD Bench Vise demo");
Console.WriteLine("Run with --e2e while the UMLCAD kernel host is available.");
return 0;
