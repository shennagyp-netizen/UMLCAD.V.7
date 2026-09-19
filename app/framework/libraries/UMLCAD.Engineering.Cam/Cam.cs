using UMLCAD.Cad.Contracts;
namespace UMLCAD.Engineering.Cam;
public sealed record CamOperation(CadId Id,CadResultId SourceBodyResult,string Strategy,string MachineId,string ToolId);
public sealed record ToolpathPoint(double X,double Y,double Z);
public sealed record Toolpath(CadId Id,CadResultId SourceBodyResult,IReadOnlyList<ToolpathPoint> Points);