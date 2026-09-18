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

public sealed record MachineCapability
{
    public ManufacturingProcessKind Process { get; }
    public double MinimumStockThicknessMm { get; }
    public double MaximumStockThicknessMm { get; }

    public MachineCapability(
        ManufacturingProcessKind process,
        double minimumStockThicknessMm,
        double maximumStockThicknessMm)
    {
        if (!double.IsFinite(minimumStockThicknessMm) ||
            !double.IsFinite(maximumStockThicknessMm) ||
            minimumStockThicknessMm < 0d ||
            maximumStockThicknessMm < minimumStockThicknessMm)
            throw new ArgumentOutOfRangeException(nameof(minimumStockThicknessMm));

        Process = process;
        MinimumStockThicknessMm = minimumStockThicknessMm;
        MaximumStockThicknessMm = maximumStockThicknessMm;
    }

    public bool SupportsThickness(double thicknessMm) =>
        double.IsFinite(thicknessMm) &&
        thicknessMm >= MinimumStockThicknessMm &&
        thicknessMm <= MaximumStockThicknessMm;
}

public sealed record ToolDefinition
{
    public string ToolId { get; }
    public ToolKind Kind { get; }
    public string InterfaceId { get; }
    public double NominalDiameterMm { get; }
    public double MinimumDiameterMm { get; }
    public double MaximumDiameterMm { get; }

    public ToolDefinition(
        string toolId,
        ToolKind kind,
        string interfaceId,
        double nominalDiameterMm,
        double minimumDiameterMm,
        double maximumDiameterMm)
    {
        if (string.IsNullOrWhiteSpace(toolId))
            throw new ArgumentException("ToolId is required.", nameof(toolId));
        if (string.IsNullOrWhiteSpace(interfaceId))
            throw new ArgumentException("InterfaceId is required.", nameof(interfaceId));

        if (!double.IsFinite(nominalDiameterMm) || nominalDiameterMm < 0d)
            throw new ArgumentOutOfRangeException(nameof(nominalDiameterMm));

        if (!double.IsFinite(minimumDiameterMm) ||
            !double.IsFinite(maximumDiameterMm) ||
            minimumDiameterMm < 0d ||
            maximumDiameterMm < minimumDiameterMm ||
            nominalDiameterMm < minimumDiameterMm ||
            nominalDiameterMm > maximumDiameterMm)
            throw new ArgumentOutOfRangeException(nameof(minimumDiameterMm));

        ToolId = toolId;
        Kind = kind;
        InterfaceId = interfaceId;
        NominalDiameterMm = nominalDiameterMm;
        MinimumDiameterMm = minimumDiameterMm;
        MaximumDiameterMm = maximumDiameterMm;
    }

    public bool SupportsDiameter(double diameterMm) =>
        double.IsFinite(diameterMm) &&
        diameterMm >= MinimumDiameterMm &&
        diameterMm <= MaximumDiameterMm;
}

public static class MachineProcessCompatibility
{
    public static bool IsCompatible(
        MachineKind machine,
        ManufacturingProcessKind process) =>
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
    public static bool IsCompatible(
        ManufacturingProcessKind process,
        ToolKind tool) =>
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

public sealed record MachineDefinition
{
    public string MachineId { get; }
    public MachineKind Kind { get; }
    public string ToolInterfaceId { get; }
    public IReadOnlyList<MachineCapability> Capabilities { get; }

    public MachineDefinition(
        string machineId,
        MachineKind kind,
        string toolInterfaceId,
        IReadOnlyList<MachineCapability> capabilities)
    {
        if (string.IsNullOrWhiteSpace(machineId))
            throw new ArgumentException("MachineId is required.", nameof(machineId));
        if (string.IsNullOrWhiteSpace(toolInterfaceId))
            throw new ArgumentException("ToolInterfaceId is required.", nameof(toolInterfaceId));

        Capabilities = capabilities?.ToArray()
            ?? throw new ArgumentNullException(nameof(capabilities));

        MachineId = machineId;
        Kind = kind;
        ToolInterfaceId = toolInterfaceId;
    }

    public bool SupportsProcess(
        ManufacturingProcessKind process,
        double thicknessMm) =>
        Capabilities.Any(
            x => x.Process == process && x.SupportsThickness(thicknessMm));
}

public static class MachineToolCompatibility
{
    public static bool IsCompatible(
        MachineDefinition machine,
        ToolDefinition tool,
        ManufacturingProcessKind process)
    {
        ArgumentNullException.ThrowIfNull(machine);
        ArgumentNullException.ThrowIfNull(tool);

        return string.Equals(
                machine.ToolInterfaceId,
                tool.InterfaceId,
                StringComparison.Ordinal) &&
            ToolProcessCompatibility.IsCompatible(process, tool.Kind);
    }
}
