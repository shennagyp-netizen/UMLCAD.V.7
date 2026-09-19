using System.Collections.Immutable;
using System.Collections.ObjectModel;
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

public sealed record CadFeatureDefinition
{
    public CadFeatureDefinition(
        CadId id,
        CadFeatureKind kind,
        string name,
        IReadOnlyList<CadId>? dependencies = null)
    {
        if (string.IsNullOrWhiteSpace(name))
            throw new ArgumentException("Feature name cannot be empty.", nameof(name));

        ArgumentNullException.ThrowIfNull(dependencies);

        if (dependencies.Contains(id))
            throw new ArgumentException(
                $"Feature '{id}' cannot depend on itself.",
                nameof(dependencies));

        var duplicateDependencies = dependencies
            .GroupBy(x => x)
            .Where(group => group.Count() > 1)
            .Select(group => group.Key.Value)
            .ToArray();

        if (duplicateDependencies.Length > 0)
        {
            throw new ArgumentException(
                $"Duplicate feature dependencies: {string.Join(", ", duplicateDependencies)}",
                nameof(dependencies));
        }

        Id = id;
        Kind = kind;
        Name = name;
        Dependencies = new ReadOnlyCollection<CadId>(dependencies.ToArray());
    }

    public CadId Id { get; }

    public CadFeatureKind Kind { get; }

    public string Name { get; }

    public IReadOnlyList<CadId> Dependencies { get; }
}

public sealed record CadDocumentDefinition
{
    public CadDocumentDefinition(
        CadId id,
        string name,
        IReadOnlyList<CadFeatureDefinition> features)
    {
        if (string.IsNullOrWhiteSpace(name))
            throw new ArgumentException("Document name cannot be empty.", nameof(name));

        ArgumentNullException.ThrowIfNull(features);

        var copiedFeatures =
            new ReadOnlyCollection<CadFeatureDefinition>(features.ToArray());

        var duplicateIds = copiedFeatures
            .GroupBy(x => x.Id)
            .Where(group => group.Count() > 1)
            .Select(group => group.Key.Value)
            .ToArray();

        if (duplicateIds.Length > 0)
        {
            throw new ArgumentException(
                $"Duplicate feature IDs: {string.Join(", ", duplicateIds)}",
                nameof(features));
        }

        var knownIds = copiedFeatures.Select(feature => feature.Id).ToImmutableHashSet();
        var unknownDependencies = copiedFeatures
            .SelectMany(feature => feature.Dependencies
                .Where(dependency => !knownIds.Contains(dependency))
                .Select(dependency => $"{feature.Id.Value}->{dependency.Value}"))
            .ToArray();

        if (unknownDependencies.Length > 0)
        {
            throw new ArgumentException(
                $"Unknown feature dependencies: {string.Join(", ", unknownDependencies)}",
                nameof(features));
        }

        Id = id;
        Name = name;
        Features = copiedFeatures;
    }

    public CadId Id { get; }

    public string Name { get; }

    public IReadOnlyList<CadFeatureDefinition> Features { get; }
}

public sealed record CadDocumentSnapshot
{
    public CadDocumentSnapshot(
        CadId id,
        string name,
        IReadOnlyList<CadFeatureDefinition> features)
    {
        if (string.IsNullOrWhiteSpace(name))
            throw new ArgumentException("Document name cannot be empty.", nameof(name));

        ArgumentNullException.ThrowIfNull(features);

        Id = id;
        Name = name;
        Features = new ReadOnlyCollection<CadFeatureDefinition>(features.ToArray());
    }

    public CadId Id { get; }

    public string Name { get; }

    public IReadOnlyList<CadFeatureDefinition> Features { get; }
}
