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

public sealed record ViewAxis
{
    public double X { get; }
    public double Y { get; }
    public double Z { get; }

    public ViewAxis(double x, double y, double z)
    {
        var normSquared = (x * x) + (y * y) + (z * z);
        if (!double.IsFinite(normSquared) || normSquared <= 0d)
            throw new ArgumentException(
                "View axis must be a finite non-zero vector.");

        X = x;
        Y = y;
        Z = z;
    }
}

public sealed record DrawingViewSpecification
{
    public SemanticId ViewId { get; }
    public DrawingViewKind Kind { get; }
    public SemanticId SourceResultId { get; }
    public ViewAxis Axis { get; }
    public ViewDisplayMode DisplayMode { get; }
    public DrawingAssociativityState Associativity { get; }
    public IReadOnlySet<SemanticId> IncludedOccurrences { get; }

    public DrawingViewSpecification(
        SemanticId viewId,
        DrawingViewKind kind,
        SemanticId sourceResultId,
        ViewAxis axis,
        ViewDisplayMode displayMode,
        DrawingAssociativityState associativity,
        IReadOnlySet<SemanticId> includedOccurrences)
    {
        if (viewId.Value == Guid.Empty)
            throw new ArgumentException("ViewId is required.", nameof(viewId));
        if (sourceResultId.Value == Guid.Empty)
            throw new ArgumentException(
                "SourceResultId is required.",
                nameof(sourceResultId));

        ArgumentNullException.ThrowIfNull(includedOccurrences);

        ViewId = viewId;
        Kind = kind;
        SourceResultId = sourceResultId;
        Axis = axis;
        DisplayMode = displayMode;
        Associativity = associativity;
        IncludedOccurrences = new HashSet<SemanticId>(includedOccurrences);
    }
}

public sealed record DrawingBomItem(
    string ItemNumber,
    SemanticId ComponentId,
    string PartNumber,
    string Revision,
    int Quantity,
    string Description);

public sealed record DrawingBomTable
{
    public SemanticId TableId { get; }
    public IReadOnlyList<DrawingBomItem> Items { get; }

    public DrawingBomTable(
        SemanticId tableId,
        IReadOnlyList<DrawingBomItem> items)
    {
        if (tableId.Value == Guid.Empty)
            throw new ArgumentException(
                "TableId is required.",
                nameof(tableId));

        TableId = tableId;
        Items = items?.ToArray()
            ?? throw new ArgumentNullException(nameof(items));
    }
}

public sealed record DrawingSheetPresentation
{
    public string BorderName { get; }
    public string TitleBlockName { get; }
    public string RevisionBlockName { get; }
    public double Scale { get; }

    public DrawingSheetPresentation(
        string borderName,
        string titleBlockName,
        string revisionBlockName,
        double scale)
    {
        if (string.IsNullOrWhiteSpace(borderName))
            throw new ArgumentException(
                "BorderName is required.",
                nameof(borderName));
        if (string.IsNullOrWhiteSpace(titleBlockName))
            throw new ArgumentException(
                "TitleBlockName is required.",
                nameof(titleBlockName));
        if (string.IsNullOrWhiteSpace(revisionBlockName))
            throw new ArgumentException(
                "RevisionBlockName is required.",
                nameof(revisionBlockName));
        if (!double.IsFinite(scale) || scale <= 0d)
            throw new ArgumentOutOfRangeException(nameof(scale));

        BorderName = borderName;
        TitleBlockName = titleBlockName;
        RevisionBlockName = revisionBlockName;
        Scale = scale;
    }
}
