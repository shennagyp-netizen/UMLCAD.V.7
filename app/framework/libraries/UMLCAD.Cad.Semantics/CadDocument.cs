using System.Collections.Immutable;
using UMLCAD.Cad.Contracts;

namespace UMLCAD.Cad.Semantics;

public enum CadFeatureKind
{
    Sketch,
    Feature,
    Reference,
    Constraint,
    Product,
    Occurrence,
    Drawing
}

public sealed record CadFeatureDefinition(
    CadId Id,
    CadFeatureKind Kind,
    string Name)
{
    public CadFeatureDefinition
    {
        if (string.IsNullOrWhiteSpace(Name))
            throw new ArgumentException("Feature name cannot be empty.", nameof(Name));
    }
}

public sealed record CadDocumentDefinition(
    CadId Id,
    string Name,
    IReadOnlyList<CadFeatureDefinition> Features)
{
    public CadDocumentDefinition
    {
        if (string.IsNullOrWhiteSpace(Name))
            throw new ArgumentException("Document name cannot be empty.", nameof(Name));
        ArgumentNullException.ThrowIfNull(Features);
    }
}
