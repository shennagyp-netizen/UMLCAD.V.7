using System.Globalization;
using System.Security.Cryptography;
using System.Text;
using UMLCAD.Cad.Contracts;

namespace UMLCAD.Engineering.Cam;

public sealed record CamOperation(
    CadId Id,
    CadResultId SourceBodyResult,
    string Strategy,
    string MachineId,
    string ToolId);

public sealed record ToolpathPoint(double X, double Y, double Z);

public sealed record Toolpath(
    string Identity,
    IReadOnlyList<ToolpathPoint> Points,
    string MachineId,
    string ToolId);

public static class DeterministicNcEmitter
{
    public static string Emit(Toolpath toolpath)
    {
        ArgumentNullException.ThrowIfNull(toolpath);

        var builder = new StringBuilder()
            .AppendLine($"; UMLCAD TOOLPATH {toolpath.Identity}")
            .AppendLine($"; MACHINE {toolpath.MachineId}")
            .AppendLine($"; TOOL {toolpath.ToolId}");

        foreach (var point in toolpath.Points)
        {
            builder.Append("G1 X")
                .Append(point.X.ToString("R", CultureInfo.InvariantCulture))
                .Append(" Y")
                .Append(point.Y.ToString("R", CultureInfo.InvariantCulture))
                .Append(" Z")
                .Append(point.Z.ToString("R", CultureInfo.InvariantCulture))
                .AppendLine();
        }

        return builder.ToString();
    }

    public static string Identity(
        CamOperation operation,
        IEnumerable<ToolpathPoint> points)
    {
        var canonical =
            $"{operation.Id}|{operation.SourceBodyResult}|{operation.Strategy}|{operation.MachineId}|{operation.ToolId}|" +
            string.Join(";", points.Select(p => $"{p.X:R},{p.Y:R},{p.Z:R}"));

        return "nc:" + Convert.ToHexString(
            SHA256.HashData(Encoding.UTF8.GetBytes(canonical)))
            .ToLowerInvariant();
    }
}
