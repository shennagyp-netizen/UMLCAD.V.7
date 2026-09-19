using UMLCAD.Cad.Semantics;
using UMLCAD.Science;

namespace UMLCAD.Engineering.Drawing;

public enum DrawingStandard
{
    Iso,
    Ansi,
    Jis,
}

public enum DrawingViewKind
{
    Front,
    Rear,
    Top,
    Bottom,
    Left,
    Right,
    Isometric,
    Auxiliary,
    Section,
    AlignedSection,
    OffsetSection,
    Detail,
    CircularDetail,
    ProfiledDetail,
    Clipping,
    Broken,
    Unfolded,
}

public enum DimensionKind
{
    Linear,
    Angular,
    Radius,
    Diameter,
    Coordinate,
    Baseline,
    Chain,
}

public enum AnnotationKind
{
    Text,
    Note,
    Leader,
    Balloon,
    Datum,
    DatumTarget,
    GeometricTolerance,
    SurfaceRoughness,
    WeldingSymbol,
    FlagNote,
}

public enum DressUpKind
{
    Centerline,
    Axis,
    SymmetryLine,
    ThreadLine,
    AreaFill,
    Hatch,
    BreakLine,
    ConstructionGeometry,
    MarkupArrow,
}

public sealed record DrawingSheet
{
    public SemanticId SheetId { get; }
    public string Name { get; }
    public string Format { get; }
    public double Scale { get; }

    public DrawingSheet(
        SemanticId sheetId,
        string name,
        string format,
        double scale)
    {
        if (sheetId.Value == Guid.Empty)
            throw new ArgumentException("SheetId is required.", nameof(sheetId));
        if (string.IsNullOrWhiteSpace(name))
            throw new ArgumentException("Name is required.", nameof(name));
        if (string.IsNullOrWhiteSpace(format))
            throw new ArgumentException("Format is required.", nameof(format));
        if (!double.IsFinite(scale) || scale <= 0d)
            throw new ArgumentOutOfRangeException(nameof(scale));

        SheetId = sheetId;
        Name = name;
        Format = format;
        Scale = scale;
    }
}

public sealed record DrawingView
{
    public SemanticId ViewId { get; }
    public DrawingViewKind Kind { get; }
    public SemanticId SourceResultId { get; }
    public Quantity Scale { get; }

    public DrawingView(
        SemanticId viewId,
        DrawingViewKind kind,
        SemanticId sourceResultId,
        Quantity scale)
    {
        if (viewId.Value == Guid.Empty)
            throw new ArgumentException("ViewId is required.", nameof(viewId));
        if (sourceResultId.Value == Guid.Empty)
            throw new ArgumentException(
                "SourceResultId is required.",
                nameof(sourceResultId));

        if (scale.Dimension != QuantityDimension.Dimensionless)
            throw new ArgumentException(
                "Drawing view scale must be dimensionless.",
                nameof(scale));
        if (scale.SiValue <= 0d)
            throw new ArgumentOutOfRangeException(nameof(scale));

        ViewId = viewId;
        Kind = kind;
        SourceResultId = sourceResultId;
        Scale = scale;
    }
}

public sealed record DrawingDimension
{
    public SemanticId DimensionId { get; }
    public DimensionKind Kind { get; }
    public SemanticId ReferenceA { get; }
    public SemanticId? ReferenceB { get; }
    public Quantity Value { get; }
    public string ToleranceText { get; }

    public DrawingDimension(
        SemanticId dimensionId,
        DimensionKind kind,
        SemanticId referenceA,
        SemanticId? referenceB,
        Quantity value,
        string toleranceText)
    {
        if (dimensionId.Value == Guid.Empty)
            throw new ArgumentException(
                "DimensionId is required.",
                nameof(dimensionId));
        if (referenceA.Value == Guid.Empty)
            throw new ArgumentException(
                "ReferenceA is required.",
                nameof(referenceA));

        if (kind is
            DimensionKind.Linear or
            DimensionKind.Angular or
            DimensionKind.Radius or
            DimensionKind.Diameter)
        {
            if (value.Dimension == QuantityDimension.Dimensionless)
                throw new ArgumentException(
                    "Engineering dimensions require a physical quantity.",
                    nameof(value));
        }

        DimensionId = dimensionId;
        Kind = kind;
        ReferenceA = referenceA;
        ReferenceB = referenceB;
        Value = value;
        ToleranceText = toleranceText ?? string.Empty;
    }
}

public sealed record DrawingAnnotation
{
    public SemanticId AnnotationId { get; }
    public AnnotationKind Kind { get; }
    public string Text { get; }

    public DrawingAnnotation(
        SemanticId annotationId,
        AnnotationKind kind,
        string text)
    {
        if (annotationId.Value == Guid.Empty)
            throw new ArgumentException(
                "AnnotationId is required.",
                nameof(annotationId));
        if (string.IsNullOrWhiteSpace(text))
            throw new ArgumentException(
                "Text is required.",
                nameof(text));

        AnnotationId = annotationId;
        Kind = kind;
        Text = text;
    }
}

public sealed record DrawingCapabilityMatrix(
    DrawingStandard Standard,
    IReadOnlySet<DrawingViewKind> ViewKinds,
    IReadOnlySet<DimensionKind> DimensionKinds,
    IReadOnlySet<AnnotationKind> AnnotationKinds,
    IReadOnlySet<DressUpKind> DressUpKinds);

public static class MechanicalDraftingCapabilityProfile
{
    public static DrawingCapabilityMatrix Baseline(DrawingStandard standard)
    {
        return new DrawingCapabilityMatrix(
            standard,
            new HashSet<DrawingViewKind>
            {
                DrawingViewKind.Front,
                DrawingViewKind.Rear,
                DrawingViewKind.Top,
                DrawingViewKind.Bottom,
                DrawingViewKind.Left,
                DrawingViewKind.Right,
                DrawingViewKind.Isometric,
                DrawingViewKind.Auxiliary,
                DrawingViewKind.Section,
                DrawingViewKind.AlignedSection,
                DrawingViewKind.OffsetSection,
                DrawingViewKind.Detail,
                DrawingViewKind.CircularDetail,
                DrawingViewKind.ProfiledDetail,
                DrawingViewKind.Clipping,
                DrawingViewKind.Broken,
                DrawingViewKind.Unfolded,
            },
            new HashSet<DimensionKind>
            {
                DimensionKind.Linear,
                DimensionKind.Angular,
                DimensionKind.Radius,
                DimensionKind.Diameter,
                DimensionKind.Coordinate,
                DimensionKind.Baseline,
                DimensionKind.Chain,
            },
            new HashSet<AnnotationKind>
            {
                AnnotationKind.Text,
                AnnotationKind.Note,
                AnnotationKind.Leader,
                AnnotationKind.Balloon,
                AnnotationKind.Datum,
                AnnotationKind.DatumTarget,
                AnnotationKind.GeometricTolerance,
                AnnotationKind.SurfaceRoughness,
                AnnotationKind.WeldingSymbol,
                AnnotationKind.FlagNote,
            },
            new HashSet<DressUpKind>
            {
                DressUpKind.Centerline,
                DressUpKind.Axis,
                DressUpKind.SymmetryLine,
                DressUpKind.ThreadLine,
                DressUpKind.AreaFill,
                DressUpKind.Hatch,
                DressUpKind.BreakLine,
                DressUpKind.ConstructionGeometry,
                DressUpKind.MarkupArrow,
            });
    }
}
