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

    [Fact]
    public void Document_Copies_Feature_Collection()
    {
        var features = new List<CadFeatureDefinition>
        {
            new(new CadId("f1"), CadFeatureKind.Feature, "Original")
        };

        var document = new CadDocumentDefinition(
            new CadId("document-1"),
            "Test",
            features);

        features.Clear();

        Assert.Single(document.Features);
        Assert.Equal(new CadId("f1"), document.Features[0].Id);
    }

    [Fact]
    public void Snapshot_Cannot_Be_Modified_Through_IReadOnlyList_Cast()
    {
        var document = new CadDocumentDefinition(
            new CadId("document-1"),
            "Test",
            [
                new CadFeatureDefinition(
                    new CadId("f1"),
                    CadFeatureKind.Feature,
                    "Original")
            ]);

        var snapshot = new UMLCAD.Cad.Semantics.CadDocumentSnapshot(
            document.Id,
            document.Name,
            document.Features);

        Assert.False(snapshot.Features is IList<CadFeatureDefinition>);
    }

}
