using UMLCAD.Cad.Contracts;
using UMLCAD.Cad.Engine;
using UMLCAD.Cad.Semantics;

namespace UMLCAD.ApplicationLayer.Tests;

public sealed class CadControlTests
{
    [Fact]
    public void Commit_Persists_All_Accepted_Commands()
    {
        var document = NewDocument();
        var store = new CadDocumentStore(document);
        var control = new InMemoryCadControlService(store);

        Assert.Equal(
            CadCommandStatus.Accepted,
            control.AddFeature(new CadFeatureDefinition(
                new CadId("f2"), CadFeatureKind.Feature, "Feature 2")).Status);

        Assert.Equal(
            CadCommandStatus.Accepted,
            control.RenameFeature(new CadId("f1"), "Renamed").Status);

        Assert.Equal(CadCommandStatus.Accepted, control.Commit().Status);

        var snapshot = store.Snapshot();

        Assert.Equal("Renamed", Assert.Single(snapshot.Features, f => f.Id.Value == "f1").Name);
        Assert.Contains(snapshot.Features, f => f.Id.Value == "f2");
    }

    [Fact]
    public void Explicit_Rollback_Restores_Original_Semantic_State()
    {
        var document = NewDocument();
        var store = new CadDocumentStore(document);
        var control = new InMemoryCadControlService(store);

        control.RenameFeature(new CadId("f1"), "Temporary");
        control.AddFeature(new CadFeatureDefinition(
            new CadId("f2"), CadFeatureKind.Feature, "Temporary Feature"));

        control.Rollback();

        var snapshot = store.Snapshot();

        Assert.Single(snapshot.Features);
        Assert.Equal("Original", snapshot.Features[0].Name);
    }

    [Fact]
    public void Commit_Failure_Restores_Every_Previous_Mutation()
    {
        var document = NewDocument();
        var store = new CadDocumentStore(document);
        var control = new InMemoryCadControlService(store);

        control.RenameFeature(new CadId("f1"), "Changed");
        control.RemoveFeature(new CadId("missing"));

        var result = control.Commit();

        Assert.Equal(CadCommandStatus.Rejected, result.Status);
        var snapshot = store.Snapshot();
        var feature = Assert.Single(snapshot.Features);

        Assert.Equal(new CadId("f1"), feature.Id);
        Assert.Equal("Original", feature.Name);
    }

    [Fact]
    public void Transaction_Is_Closed_After_Commit()
    {
        var store = new CadDocumentStore(NewDocument());
        var control = new InMemoryCadControlService(store);

        control.Commit();

        Assert.Throws<InvalidOperationException>(
            () => control.AddFeature(
                new CadFeatureDefinition(new CadId("f2"), CadFeatureKind.Feature, "Feature 2")));
    }

    private static CadDocumentDefinition NewDocument() =>
        new(
            new CadId("document-1"),
            "Test",
            [
                new CadFeatureDefinition(
                    new CadId("f1"),
                    CadFeatureKind.Feature,
                    "Original")
            ]);
}
