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

public sealed record DrawingSheet(
    SemanticId SheetId,
    string Name,
    string Format,
    double Scale)
{
    public DrawingSheet
    {
        if (SheetId.Value == Guid.Empty)
            throw new ArgumentException("SheetId is required.", nameof(SheetId));
        if (string.IsNullOrWhiteSpace(Name))
            throw new ArgumentException("Name is required.", nameof(Name));
        if (string.IsNullOrWhiteSpace(Format))
            throw new ArgumentException("Format is required.", nameof(Format));
        if (!double.IsFinite(Scale) || Scale <= 0d)
            throw new ArgumentOutOfRangeException(nameof(Scale));
    }
}

public sealed record DrawingView(
    SemanticId ViewId,
    DrawingViewKind Kind,
    SemanticId SourceResultId,
    Quantity Scale)
{
    public DrawingView
    {
        if (ViewId.Value == Guid.Empty)
            throw new ArgumentException("ViewId is required.", nameof(ViewId));
        if (SourceResultId.Value == Guid.Empty)
            throw new ArgumentException("SourceResultId is required.", nameof(SourceResultId));

        if (Scale.Dimension != QuantityDimension.Dimensionless)
            throw new ArgumentException("Drawing view scale must be dimensionless.", nameof(Scale));
        if (Scale.SiValue <= 0d)
            throw new ArgumentOutOfRangeException(nameof(Scale));
    }
}

public sealed record DrawingDimension(
    SemanticId DimensionId,
    DimensionKind Kind,
    SemanticId ReferenceA,
    SemanticId? ReferenceB,
    Quantity Value,
    string ToleranceText)
{
    public DrawingDimension
    {
        if (DimensionId.Value == Guid.Empty)
            throw new ArgumentException("DimensionId is required.", nameof(DimensionId));
        if (ReferenceA.Value == Guid.Empty)
            throw new ArgumentException("ReferenceA is required.", nameof(ReferenceA));
        if (Kind is DimensionKind.Linear or DimensionKind.Angular or DimensionKind.Radius or DimensionKind.Diameter)
        {
            if (Value.Dimension == QuantityDimension.Dimensionless)
                throw new ArgumentException("Engineering dimensions require a physical quantity.", nameof(Value));
        }

        ToleranceText ??= string.Empty;
    }
}

public sealed record DrawingAnnotation(
    SemanticId AnnotationId,
    AnnotationKind Kind,
    string Text)
{
    public DrawingAnnotation
    {
        if (AnnotationId.Value == Guid.Empty)
            throw new ArgumentException("AnnotationId is required.", nameof(AnnotationId));
        if (string.IsNullOrWhiteSpace(Text))
            throw new ArgumentException("Text is required.", nameof(Text));
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
