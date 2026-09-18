using UMLCAD.Cad.Semantics;
using UMLCAD.Engineering.Resources;
using UMLCAD.Science;

namespace UMLCAD.Engineering.SheetMetal;

public sealed record SheetMetalPartDefinition(
    SemanticId PartId,
    Material Material,
    double ThicknessMm)
{
    public SheetMetalPartDefinition
    {
        if (PartId.Value == Guid.Empty)
            throw new ArgumentException("PartId is required.", nameof(PartId));
        ArgumentNullException.ThrowIfNull(Material);
        if (!double.IsFinite(ThicknessMm) || ThicknessMm <= 0d)
            throw new ArgumentOutOfRangeException(nameof(ThicknessMm));
    }
}

public sealed record BendDefinition(
    SemanticId BendId,
    double AngleDegrees,
    double RadiusMm,
    double KFactor)
{
    public BendDefinition
    {
        if (BendId.Value == Guid.Empty)
            throw new ArgumentException("BendId is required.", nameof(BendId));
        if (!double.IsFinite(AngleDegrees) || AngleDegrees <= 0d || AngleDegrees >= 180d)
            throw new ArgumentOutOfRangeException(nameof(AngleDegrees));
        if (!double.IsFinite(RadiusMm) || RadiusMm <= 0d)
            throw new ArgumentOutOfRangeException(nameof(RadiusMm));
        if (!double.IsFinite(KFactor) || KFactor < 0d || KFactor > 1d)
            throw new ArgumentOutOfRangeException(nameof(KFactor));
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
            diagnostics.Add("Sheet Metal requires a material classified as Metal.");

        if (part.Material.Properties.DuctilityPercent <= 0d)
            diagnostics.Add("Sheet Metal requires positive material ductility.");

        if (bend.RadiusMm < part.ThicknessMm * 0.5d)
            diagnostics.Add("Bend radius is below the current minimum engineering policy.");

        if (machine is not null &&
            !machine.SupportsProcess(ManufacturingProcessKind.SheetMetalBending, part.ThicknessMm))
        {
            diagnostics.Add("Machine does not support the requested sheet-metal thickness/process.");
        }

        if (machine is not null && tool is not null &&
            !MachineToolCompatibility.IsCompatible(
                machine,
                tool,
                ManufacturingProcessKind.SheetMetalBending))
        {
            diagnostics.Add("Tool is incompatible with the selected sheet-metal bending machine/process.");
        }

        return new SheetMetalValidationResult(diagnostics.Count == 0, diagnostics);
    }
}
