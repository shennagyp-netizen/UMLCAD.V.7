using UMLCAD.Cad.Semantics;
using UMLCAD.Engineering.Resources;
using UMLCAD.Science;

namespace UMLCAD.Engineering.SheetMetal;

public sealed record SheetMetalPartDefinition
{
    public SemanticId PartId { get; }
    public Material Material { get; }
    public double ThicknessMm { get; }

    public SheetMetalPartDefinition(
        SemanticId partId,
        Material material,
        double thicknessMm)
    {
        if (partId.Value == Guid.Empty)
            throw new ArgumentException(
                "PartId is required.",
                nameof(partId));

        ArgumentNullException.ThrowIfNull(material);

        if (!double.IsFinite(thicknessMm) || thicknessMm <= 0d)
            throw new ArgumentOutOfRangeException(nameof(thicknessMm));

        PartId = partId;
        Material = material;
        ThicknessMm = thicknessMm;
    }
}

public sealed record BendDefinition
{
    public SemanticId BendId { get; }
    public double AngleDegrees { get; }
    public double RadiusMm { get; }
    public double KFactor { get; }

    public BendDefinition(
        SemanticId bendId,
        double angleDegrees,
        double radiusMm,
        double kFactor)
    {
        if (bendId.Value == Guid.Empty)
            throw new ArgumentException(
                "BendId is required.",
                nameof(bendId));

        if (!double.IsFinite(angleDegrees) ||
            angleDegrees <= 0d ||
            angleDegrees >= 180d)
            throw new ArgumentOutOfRangeException(nameof(angleDegrees));

        if (!double.IsFinite(radiusMm) || radiusMm <= 0d)
            throw new ArgumentOutOfRangeException(nameof(radiusMm));

        if (!double.IsFinite(kFactor) ||
            kFactor < 0d ||
            kFactor > 1d)
            throw new ArgumentOutOfRangeException(nameof(kFactor));

        BendId = bendId;
        AngleDegrees = angleDegrees;
        RadiusMm = radiusMm;
        KFactor = kFactor;
    }
}

public sealed record SheetMetalValidationResult(
    bool IsValid,
    IReadOnlyList<string> Diagnostics);

public static class SheetMetalValidator
{
    public static SheetMetalValidationResult Validate(
        SheetMetalPartDefinition part,
        BendDefinition bend,
        MachineDefinition? machine = null,
        ToolDefinition? tool = null)
    {
        ArgumentNullException.ThrowIfNull(part);
        ArgumentNullException.ThrowIfNull(bend);

        var diagnostics = new List<string>();

        if (part.Material.Family != MaterialFamily.Metal)
            diagnostics.Add(
                "Sheet Metal requires a material classified as Metal.");

        if (part.Material.Properties.DuctilityPercent <= 0d)
            diagnostics.Add(
                "Sheet Metal requires positive material ductility.");

        if (bend.RadiusMm < part.ThicknessMm * 0.5d)
            diagnostics.Add(
                "Bend radius is below the current minimum engineering policy.");

        if (machine is not null &&
            !machine.SupportsProcess(
                ManufacturingProcessKind.SheetMetalBending,
                part.ThicknessMm))
        {
            diagnostics.Add(
                "Machine does not support the requested sheet-metal thickness/process.");
        }

        if (machine is not null &&
            tool is not null &&
            !MachineToolCompatibility.IsCompatible(
                machine,
                tool,
                ManufacturingProcessKind.SheetMetalBending))
        {
            diagnostics.Add(
                "Tool is incompatible with the selected sheet-metal bending machine/process.");
        }

        return new SheetMetalValidationResult(
            diagnostics.Count == 0,
            diagnostics);
    }
}
