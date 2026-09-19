using UMLCAD.Cad.Contracts;

namespace UMLCAD.Engineering.Drawing;

public sealed record DrawingView(
    CadId Id,
    CadResultId SourceResult,
    string Projection);

public sealed record DrawingSheet(
    CadId Id,
    string Name,
    IReadOnlyList<DrawingView> Views);

public sealed record DrawingDefinition(
    CadId Id,
    string Name,
    IReadOnlyList<DrawingSheet> Sheets);
