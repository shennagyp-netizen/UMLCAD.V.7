using UMLCAD.Cad.Contracts;
using UMLCAD.Cad.Semantics;

namespace UMLCAD.ApplicationLayer.Tests;

public sealed class CadSemanticsTests
{
    [Fact]
    public void CadId_Rejects_Empty_Identity()
    {
        Assert.Throws<ArgumentException>(() => new CadId(" "));
    }

    [Fact]
    public void Document_Preserves_Explicit_Feature_Identity()
    {
        var feature = new CadFeatureDefinition(
            new CadId("feature-001"),
            CadFeatureKind.Sketch,
            "Base Sketch");

        var document = new CadDocumentDefinition(
            new CadId("document-001"),
            "Demo Part",
            [feature]);

        var actual = Assert.Single(document.Features);

        Assert.Equal("feature-001", actual.Id.Value);
        Assert.Equal("Base Sketch", actual.Name);
        Assert.Equal(CadFeatureKind.Sketch, actual.Kind);
    }

    [Fact]
    public void NonFinite_ExpressionValue_Is_Rejected()
    {
        Assert.Throws<ArgumentOutOfRangeException>(
            () => new UMLCAD.Cad.Expressions.ExpressionValue(double.NaN, "mm"));
    }
}
