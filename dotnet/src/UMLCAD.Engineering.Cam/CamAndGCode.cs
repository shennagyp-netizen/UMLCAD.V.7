using System.Globalization;
using System.Text;
using UMLCAD.Cad.Semantics;
using UMLCAD.Engineering.Resources;

namespace UMLCAD.Engineering.Cam;

public sealed record ToolpathPoint(double X, double Y, double Z)
{
    public ToolpathPoint
    {
        if (!double.IsFinite(X) || !double.IsFinite(Y) || !double.IsFinite(Z))
            throw new ArgumentOutOfRangeException(nameof(X), "Toolpath coordinates must be finite.");
    }
}

public sealed record ManufacturingOperation(
    SemanticId OperationId,
    ManufacturingProcessKind Process,
    string ToolId,
    IReadOnlyList<ToolpathPoint> Path)
{
    public ManufacturingOperation
    {
        if (OperationId.Value == Guid.Empty)
            throw new ArgumentException("OperationId is required.", nameof(OperationId));
        if (string.IsNullOrWhiteSpace(ToolId))
            throw new ArgumentException("ToolId is required.", nameof(ToolId));

        Path = Path?.ToArray() ??
            throw new ArgumentNullException(nameof(Path));

        if (Path.Count == 0)
            throw new ArgumentException("A manufacturing operation requires at least one toolpath point.", nameof(Path));
    }
}

public sealed record NcProgram(
    string ProgramId,
    string MachineId,
    IReadOnlyList<string> Lines)
{
    public NcProgram
    {
        if (string.IsNullOrWhiteSpace(ProgramId))
            throw new ArgumentException("ProgramId is required.", nameof(ProgramId));
        if (string.IsNullOrWhiteSpace(MachineId))
            throw new ArgumentException("MachineId is required.", nameof(MachineId));

        Lines = Lines?.ToArray() ??
            throw new ArgumentNullException(nameof(Lines));
    }

    public string Serialize() => string.Join("\n", Lines);
}

public interface INcPostprocessor
{
    string Id { get; }

    NcProgram Generate(
        ManufacturingOperation operation,
        MachineDefinition machine,
        ToolDefinition tool);
}

public sealed class DeterministicGCodePostprocessor : INcPostprocessor
{
    public string Id => "UMLCAD.GCODE.BASIC.1";

    public NcProgram Generate(
        ManufacturingOperation operation,
        MachineDefinition machine,
        ToolDefinition tool)
    {
        ArgumentNullException.ThrowIfNull(operation);
        ArgumentNullException.ThrowIfNull(machine);
        ArgumentNullException.ThrowIfNull(tool);

        if (!MachineToolCompatibility.IsCompatible(machine, tool))
            throw new InvalidOperationException("Machine/tool interfaces are incompatible.");

        if (operation.Process is not (ManufacturingProcessKind.Milling or ManufacturingProcessKind.Turning))
            throw new NotSupportedException($"The deterministic G-code postprocessor does not support {operation.Process}.");

        var lines = new List<string>
        {
            "%",
            $"(UMLCAD OP {operation.OperationId})",
            "G21",
            "G90",
            $"(TOOL {tool.ToolId})",
        };

        var first = true;
        foreach (var point in operation.Path)
        {
            var prefix = first ? "G00" : "G01";
            first = false;
            lines.Add(string.Create(
                CultureInfo.InvariantCulture,
                $"{prefix} X{point.X:0.##########} Y{point.Y:0.##########} Z{point.Z:0.##########}"));
        }

        lines.Add("M30");
        lines.Add("%");

        return new NcProgram(
            ProgramId: $"NC-{operation.OperationId}",
            MachineId: machine.MachineId,
            Lines: lines);
    }
}
