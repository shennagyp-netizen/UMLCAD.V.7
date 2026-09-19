using UMLCAD.Cad.Contracts;
using UMLCAD.Engineering.Resources;

namespace UMLCAD.Engineering.SheetMetal;

public sealed record BendOperation(
    CadId Id,
    CadId BodyId,
    CadResultId InputBodyResult,
    double AngleDegrees,
    double RadiusMm);

public sealed record SheetMetalProcess(
    string Material,
    double ThicknessMm,
    IReadOnlyList<MachineResource> Machines,
    IReadOnlyList<BendOperation> Bends);
