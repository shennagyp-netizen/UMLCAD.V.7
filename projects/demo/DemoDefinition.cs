using UMLCAD.Framework;

internal static class DemoDefinition
{
    public static CadApplicationBuilder ConfigureDemo(this CadApplicationBuilder builder)
    {
        builder.ApplicationId = "demo";
        builder.Version = "1.0.0";

        builder.AddPart("vise_base", "machined_part", p =>
        {
            p.Name("Vise Base")
                .PartNumber("V-100")
                .Material("Cast Iron");

            Rectangle(p, "base_profile", 0, 0, 120, 80);
            p.Geometry("mount_hole_front", "circle", new Dictionary<string, string>
            {
                ["center"] = "20,20", ["radius"] = "6"
            });
            p.Geometry("mount_hole_rear", "circle", new Dictionary<string, string>
            {
                ["center"] = "100,20", ["radius"] = "6"
            });
        });

        builder.AddPart("fixed_jaw", "machined_part", p =>
        {
            p.Name("Fixed Jaw")
                .PartNumber("V-110")
                .Material("Steel")
                .Property("jawFace", "fixed jaw contact face");

            Rectangle(p, "jaw_profile", 10, 80, 100, 55);
            p.Geometry("jaw_face", "line", LineProperties(95, 88, 95, 127));
        });

        builder.AddPart("sliding_jaw", "machined_part", p =>
        {
            p.Name("Sliding Jaw")
                .PartNumber("V-120")
                .Material("Steel")
                .Property("jawFace", "sliding jaw contact face");

            Rectangle(p, "jaw_profile", 98, 80, 35, 55);
            p.Geometry("jaw_face", "line", LineProperties(105, 88, 105, 127));
        });

        builder.AddPart("lead_screw", "turned_part", p =>
        {
            p.Name("Lead Screw")
                .PartNumber("V-200")
                .Material("Steel")
                .Parameter("diameter", "20", "mm")
                .Parameter("length", "170", "mm");

            p.Geometry("shaft", "circle", new Dictionary<string, string>
            {
                ["center"] = "0,0", ["radius"] = "10"
            });
        });

        builder.AddPart("handle", "turned_part", p =>
        {
            p.Name("Operating Handle")
                .PartNumber("V-210")
                .Material("Steel")
                .Parameter("diameter", "10", "mm")
                .Parameter("length", "90", "mm");

            Rectangle(p, "bar", -45, -5, 90, 10);
        });

        builder.AddAssembly("body_assembly", "Vise Body", a => a
            .PartNumber("VA-100")
            .Description("Fixed base and jaw structure of the bench vise")
            .Part("base", "vise_base", "Vise Base", o => o.Grounded().BomStructure("normal"))
            .Part("fixed_jaw", "fixed_jaw", "Fixed Jaw", o => o
                .Transform(Translation(0, 80, 0)).BomStructure("normal")));

        builder.AddAssembly("screw_assembly", "Vise Screw", a => a
            .PartNumber("VA-200")
            .Description("Lead screw mechanism")
            .Part("screw", "lead_screw", "Lead Screw", o => o
                .Transform(Translation(60, 105, 0)).BomStructure("normal")));

        builder.AddAssembly("handle_assembly", "Vise Handle", a => a
            .PartNumber("VA-210")
            .Description("Sliding operating handle")
            .Part("handle", "handle", "Operating Handle", o => o
                .Transform(Translation(60, 105, 20)).BomStructure("normal")));

        builder.AddAssembly("demo", "Bench Vise Demo", a => a
            .PartNumber("DEMO-VISE-001")
            .Description("Top-level demonstration assembly containing multiple nested assemblies")
            .Property("device", "bench vise")
            .Property("purpose", "UMLCAD V5 nested assembly demonstration")
            .Assembly("body", "body_assembly", "Body Assembly", o => o.Grounded().Property("role", "base structure"))
            .Assembly("screw", "screw_assembly", "Screw Assembly", o => o.Property("role", "clamping mechanism"))
            .Assembly("handle", "handle_assembly", "Handle Assembly", o => o.Property("role", "manual drive"))
            .Part("moving_jaw", "sliding_jaw", "Sliding Jaw", o => o
                .Transform(Translation(25, 0, 0)).Property("role", "moving jaw")));

        return builder;
    }

    private static void Rectangle(PartBuilder part, string id, double x, double y, double width, double height)
    {
        part.Geometry($"{id}_bottom", "line", LineProperties(x, y, x + width, y));
        part.Geometry($"{id}_right", "line", LineProperties(x + width, y, x + width, y + height));
        part.Geometry($"{id}_top", "line", LineProperties(x + width, y + height, x, y + height));
        part.Geometry($"{id}_left", "line", LineProperties(x, y + height, x, y));
    }

    private static Dictionary<string, string> LineProperties(double startX, double startY, double endX, double endY) =>
        new()
        {
            ["start"] = $"{startX},{startY}",
            ["end"] = $"{endX},{endY}"
        };

    private static double[] Translation(double x, double y, double z) =>
    [
        1, 0, 0, x,
        0, 1, 0, y,
        0, 0, 1, z,
        0, 0, 0, 1
    ];
}
