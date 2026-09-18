using UMLCAD.Cad.Semantics;

namespace UMLCAD.Engineering.Drawing;

public enum DrawingAssociativityState
{
    Associative,
    NeedsUpdate,
    MissingReference,
    AmbiguousReference,
    Unsupported,
}

public enum ViewDisplayMode
{
    Exact,
    Shaded,
    HiddenLine,
    Sectioned,
    Wireframe,
}

public sealed record ViewAxis(
    double X,
    double Y,
    double Z)
{
    public ViewAxis
    {
        var normSquared = (X * X) + (Y * Y) + (Z * Z);
        if (!double.IsFinite(normSquared) || normSquared <= 0d)
            throw new ArgumentException("View axis must be a finite non-zero vector.");
    }
}

public sealed record DrawingViewSpecification(
    SemanticId ViewId,
    DrawingViewKind Kind,
    SemanticId SourceResultId,
    ViewAxis Axis,
    ViewDisplayMode DisplayMode,
    DrawingAssociativityState Associativity,
    IReadOnlySet<SemanticId> IncludedOccurrences)
{
    public DrawingViewSpecification
    {
        if (ViewId.Value == Guid.Empty)
            throw new ArgumentException("ViewId is required.", nameof(ViewId));
        if (SourceResultId.Value == Guid.Empty)
            throw new ArgumentException("SourceResultId is required.", nameof(SourceResultId));
        ArgumentNullException.ThrowIfNull(IncludedOccurrences);
        IncludedOccurrences = new HashSet<SemanticId>(IncludedOccurrences);
    }
}

public sealed record DrawingBomItem(
    string ItemNumber,
    SemanticId ComponentId,
    string PartNumber,
    string Revision,
    int Quantity,
    string Description);

public sealed record DrawingBomTable(
    SemanticId TableId,
    IReadOnlyList<DrawingBomItem> Items)
{
    public DrawingBomTable
    {
        if (TableId.Value == Guid.Empty)
            throw new ArgumentException("TableId is required.", nameof(TableId));
        Items = Items?.ToArray() ?? throw new ArgumentNullException(nameof(Items));
    }
}

public sealed record DrawingSheetPresentation(
    string BorderName,
    string TitleBlockName,
    string RevisionBlockName,
    double Scale)
{
    public DrawingSheetPresentation
    {
        if (string.IsNullOrWhiteSpace(BorderName))
            throw new ArgumentException("BorderName is required.", nameof(BorderName));
        if (string.IsNullOrWhiteSpace(TitleBlockName))
            throw new ArgumentException("TitleBlockName is required.", nameof(TitleBlockName));
        if (string.IsNullOrWhiteSpace(RevisionBlockName))
            throw new ArgumentException("RevisionBlockName is required.", nameof(RevisionBlockName));
        if (!double.IsFinite(Scale) || Scale <= 0d)
            throw new ArgumentOutOfRangeException(nameof(Scale));
    }
}
