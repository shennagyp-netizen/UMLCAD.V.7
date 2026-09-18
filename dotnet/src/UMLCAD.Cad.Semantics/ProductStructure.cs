namespace UMLCAD.Cad.Semantics;

public readonly record struct SemanticId(Guid Value)
{
    public static SemanticId New() => new(Guid.NewGuid());

    public override string ToString() => Value.ToString("D");
}

public sealed record ProductComponent
{
    public SemanticId ComponentId { get; }
    public string PartNumber { get; }
    public string Revision { get; }
    public string Description { get; }

    public ProductComponent(
        SemanticId componentId,
        string partNumber,
        string revision,
        string description)
    {
        if (componentId.Value == Guid.Empty)
            throw new ArgumentException("ComponentId is required.", nameof(componentId));
        if (string.IsNullOrWhiteSpace(partNumber))
            throw new ArgumentException("PartNumber is required.", nameof(partNumber));
        if (string.IsNullOrWhiteSpace(revision))
            throw new ArgumentException("Revision is required.", nameof(revision));
        if (string.IsNullOrWhiteSpace(description))
            throw new ArgumentException("Description is required.", nameof(description));

        ComponentId = componentId;
        PartNumber = partNumber;
        Revision = revision;
        Description = description;
    }
}

public sealed record ProductOccurrence
{
    public SemanticId OccurrenceId { get; }
    public SemanticId ComponentId { get; }
    public int Quantity { get; }
    public string Context { get; }

    public ProductOccurrence(
        SemanticId occurrenceId,
        SemanticId componentId,
        int quantity,
        string context)
    {
        if (occurrenceId.Value == Guid.Empty)
            throw new ArgumentException("OccurrenceId is required.", nameof(occurrenceId));
        if (componentId.Value == Guid.Empty)
            throw new ArgumentException("ComponentId is required.", nameof(componentId));
        if (quantity <= 0)
            throw new ArgumentOutOfRangeException(nameof(quantity));
        if (string.IsNullOrWhiteSpace(context))
            throw new ArgumentException("Context is required.", nameof(context));

        OccurrenceId = occurrenceId;
        ComponentId = componentId;
        Quantity = quantity;
        Context = context;
    }
}

public sealed record ProductDefinition
{
    public SemanticId ProductId { get; }
    public string PartNumber { get; }
    public string Revision { get; }
    public IReadOnlyList<ProductComponent> Components { get; }
    public IReadOnlyList<ProductOccurrence> Occurrences { get; }

    public ProductDefinition(
        SemanticId productId,
        string partNumber,
        string revision,
        IReadOnlyList<ProductComponent> components,
        IReadOnlyList<ProductOccurrence> occurrences)
    {
        if (productId.Value == Guid.Empty)
            throw new ArgumentException("ProductId is required.", nameof(productId));
        if (string.IsNullOrWhiteSpace(partNumber))
            throw new ArgumentException("PartNumber is required.", nameof(partNumber));
        if (string.IsNullOrWhiteSpace(revision))
            throw new ArgumentException("Revision is required.", nameof(revision));

        Components = components?.ToArray()
            ?? throw new ArgumentNullException(nameof(components));
        Occurrences = occurrences?.ToArray()
            ?? throw new ArgumentNullException(nameof(occurrences));

        if (Components.GroupBy(x => x.ComponentId).Any(g => g.Count() != 1))
            throw new ArgumentException(
                "Product component identifiers must be unique.",
                nameof(components));

        var componentIds = Components.Select(x => x.ComponentId).ToHashSet();
        if (Occurrences.Any(x => !componentIds.Contains(x.ComponentId)))
            throw new ArgumentException(
                "Every occurrence must reference an existing component.",
                nameof(occurrences));

        ProductId = productId;
        PartNumber = partNumber;
        Revision = revision;
    }
}

public sealed record BomLine(
    string ItemNumber,
    SemanticId ComponentId,
    string PartNumber,
    string Revision,
    string Description,
    int Quantity);

public static class BomService
{
    public static IReadOnlyList<BomLine> Generate(ProductDefinition product)
    {
        ArgumentNullException.ThrowIfNull(product);

        var componentsById = product.Components.ToDictionary(x => x.ComponentId);

        var groups = product.Occurrences
            .GroupBy(x => x.ComponentId)
            .OrderBy(x => componentsById[x.Key].PartNumber, StringComparer.Ordinal)
            .ThenBy(x => componentsById[x.Key].Revision, StringComparer.Ordinal)
            .ThenBy(x => x.Key.Value)
            .ToArray();

        var lines = new List<BomLine>(groups.Length);
        for (var index = 0; index < groups.Length; index++)
        {
            var componentId = groups[index].Key;
            var component = componentsById[componentId];
            var quantity = groups[index].Sum(x => x.Quantity);

            lines.Add(new BomLine(
                ItemNumber: (index + 1).ToString(System.Globalization.CultureInfo.InvariantCulture),
                ComponentId: component.ComponentId,
                PartNumber: component.PartNumber,
                Revision: component.Revision,
                Description: component.Description,
                Quantity: quantity));
        }

        return lines;
    }
}
