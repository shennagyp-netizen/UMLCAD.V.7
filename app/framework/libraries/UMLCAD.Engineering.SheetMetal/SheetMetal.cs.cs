using UMLCAD.Cad.Contracts;
namespace UMLCAD.Engineering.SheetMetal;
public sealed record BendOperation(CadId Id,CadId BodyId,CadResultId InputBodyResult,double AngleDegrees,double RadiusMm);
public sealed record SheetMetalDefinition(string Material,double ThicknessMm,IReadOnlyList<BendOperation> Bends);