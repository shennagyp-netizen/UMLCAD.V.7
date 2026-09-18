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
    double MinimumDiameterMm,
    double MaximumDiameterMm)
{
    public ToolDefinition
    {
        if (string.IsNullOrWhiteSpace(ToolId))
            throw new ArgumentException("ToolId is required.", nameof(ToolId));
        if (string.IsNullOrWhiteSpace(InterfaceId))
            throw new ArgumentException("InterfaceId is required.", nameof(InterfaceId));
        if (!double.IsFinite(MinimumDiameterMm) ||
            !double.IsFinite(MaximumDiameterMm) ||
            MinimumDiameterMm < 0d ||
            MaximumDiameterMm < MinimumDiameterMm)
        {
            throw new ArgumentOutOfRangeException(nameof(MinimumDiameterMm));
        }
    }

    public bool SupportsDiameter(double diameterMm) =>
        double.IsFinite(diameterMm) &&
        diameterMm >= MinimumDiameterMm &&
        diameterMm <= MaximumDiameterMm;
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
