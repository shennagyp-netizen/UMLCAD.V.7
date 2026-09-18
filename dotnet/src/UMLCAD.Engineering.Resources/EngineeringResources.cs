using UMLCAD.Science;

namespace UMLCAD.Engineering.Resources;

public enum MachineKind
{
    MachiningCenter,
    Lathe,
    WireEdm,
    PressBrake,
    LaserCutter,
    Waterjet,
    GrindingMachine,
}

public enum ToolKind
{
    EndMill,
    Drill,
    Reamer,
    WireElectrode,
    PressBrakePunch,
    PressBrakeDie,
    LaserNozzle,
    WaterjetNozzle,
    GrindingWheel,
}

public enum ManufacturingProcessKind
{
    Milling,
    Turning,
    WireEdmCutting,
    SheetMetalBending,
    LaserCutting,
    WaterjetCutting,
    Grinding,
}

public sealed record MachineCapability(
    ManufacturingProcessKind Process,
    double MinimumStockThicknessMm,
    double MaximumStockThicknessMm)
{
    public MachineCapability
    {
        if (!double.IsFinite(MinimumStockThicknessMm) ||
            !double.IsFinite(MaximumStockThicknessMm) ||
            MinimumStockThicknessMm < 0d ||
            MaximumStockThicknessMm < MinimumStockThicknessMm)
        {
            throw new ArgumentOutOfRangeException(nameof(MinimumStockThicknessMm));
        }
    }

    public bool SupportsThickness(double thicknessMm) =>
        double.IsFinite(thicknessMm) &&
        thicknessMm >= MinimumStockThicknessMm &&
        thicknessMm <= MaximumStockThicknessMm;
}

public sealed record ToolDefinition(
    string ToolId,
    ToolKind Kind,
    string InterfaceId,
    double NominalDiameterMm,
    double MinimumDiameterMm,
    double MaximumDiameterMm)
{
    public ToolDefinition
    {
        if (string.IsNullOrWhiteSpace(ToolId))
            throw new ArgumentException("ToolId is required.", nameof(ToolId));
        if (string.IsNullOrWhiteSpace(InterfaceId))
            throw new ArgumentException("InterfaceId is required.", nameof(InterfaceId));
        if (!double.IsFinite(NominalDiameterMm) ||
            NominalDiameterMm < 0d)
        {
            throw new ArgumentOutOfRangeException(nameof(NominalDiameterMm));
        }

        if (!double.IsFinite(MinimumDiameterMm) ||
            !double.IsFinite(MaximumDiameterMm) ||
            MinimumDiameterMm < 0d ||
            MaximumDiameterMm < MinimumDiameterMm ||
            NominalDiameterMm < MinimumDiameterMm ||
            NominalDiameterMm > MaximumDiameterMm)
        {
            throw new ArgumentOutOfRangeException(nameof(MinimumDiameterMm));
        }
    }

    public bool SupportsDiameter(double diameterMm) =>
        double.IsFinite(diameterMm) &&
        diameterMm >= MinimumDiameterMm &&
        diameterMm <= MaximumDiameterMm;
}

public static class MachineProcessCompatibility
{
    public static bool IsCompatible(MachineKind machine, ManufacturingProcessKind process) =>
        machine switch
        {
            MachineKind.MachiningCenter => process == ManufacturingProcessKind.Milling,
            MachineKind.Lathe => process == ManufacturingProcessKind.Turning,
            MachineKind.WireEdm => process == ManufacturingProcessKind.WireEdmCutting,
            MachineKind.PressBrake => process == ManufacturingProcessKind.SheetMetalBending,
            MachineKind.LaserCutter => process == ManufacturingProcessKind.LaserCutting,
            MachineKind.Waterjet => process == ManufacturingProcessKind.WaterjetCutting,
            MachineKind.GrindingMachine => process == ManufacturingProcessKind.Grinding,
            _ => false,
        };
}

public static class ToolProcessCompatibility
{
    public static bool IsCompatible(ManufacturingProcessKind process, ToolKind tool) =>
        process switch
        {
            ManufacturingProcessKind.Milling =>
                tool is ToolKind.EndMill or ToolKind.Drill or ToolKind.Reamer,
            ManufacturingProcessKind.Turning => false,
            ManufacturingProcessKind.WireEdmCutting => tool == ToolKind.WireElectrode,
            ManufacturingProcessKind.SheetMetalBending =>
                tool is ToolKind.PressBrakePunch or ToolKind.PressBrakeDie,
            ManufacturingProcessKind.LaserCutting => tool == ToolKind.LaserNozzle,
            ManufacturingProcessKind.WaterjetCutting => tool == ToolKind.WaterjetNozzle,
            ManufacturingProcessKind.Grinding => tool == ToolKind.GrindingWheel,
            _ => false,
        };
}

public sealed record MachineDefinition(
    string MachineId,
    MachineKind Kind,
    string ToolInterfaceId,
    IReadOnlyList<MachineCapability> Capabilities)
{
    public MachineDefinition
    {
        if (string.IsNullOrWhiteSpace(MachineId))
            throw new ArgumentException("MachineId is required.", nameof(MachineId));
        if (string.IsNullOrWhiteSpace(ToolInterfaceId))
            throw new ArgumentException("ToolInterfaceId is required.", nameof(ToolInterfaceId));

        Capabilities = Capabilities?.ToArray() ??
            throw new ArgumentNullException(nameof(Capabilities));
    }

    public bool SupportsProcess(ManufacturingProcessKind process, double thicknessMm) =>
        Capabilities.Any(x => x.Process == process && x.SupportsThickness(thicknessMm));
}

public static class MachineToolCompatibility
{
    public static bool IsCompatible(MachineDefinition machine, ToolDefinition tool) =>
        string.Equals(machine.ToolInterfaceId, tool.InterfaceId, StringComparison.Ordinal);
}
