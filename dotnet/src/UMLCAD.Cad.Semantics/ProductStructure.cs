namespace UMLCAD.Cad.Semantics;

public readonly record struct SemanticId(Guid Value)
{
    public static SemanticId New() => new(Guid.NewGuid());

    public override string ToString() => Value.ToString("D");
}

public sealed record ProductComponent(
    SemanticId ComponentId,
    string PartNumber,
    string Revision,
    string Description)
{
    public ProductComponent
    {
        if (ComponentId.Value == Guid.Empty)
            throw new ArgumentException("ComponentId is required.", nameof(ComponentComponentId));

        if (string.IsNullOrWhiteSpace(PartNumber))
            throw new ArgumentException("PartNumber is required.", nameof(PartNumber));

        if (string.IsNullOrWhiteSpace(Revision))
            throw new ArgumentException("Revision is required.", nameof(Revision));

        if (string.IsNullOrWhiteSpace(Description))
            throw new ArgumentException("Description is required.", nameof(Description));
    }
}

public sealed record ProductOccurrence(
    SemanticId OccurrenceId,
    SemanticId ComponentId,
    int Quantity,
    string Context)
{
    public ProductOccurrence
    {
        if (OccurrenceId.Value == Guid.Empty)
            throw new ArgumentException("OccurrenceId is required.", nameof(OccurrenceId));

        if (ComponentId.Value == Guid.Empty)
            throw new ArgumentException("ComponentId is required.", nameof(ComponentId));

        if (Quantity <= 0)
            throw new ArgumentOutOfRangeException(nameof(Quantity));

        if (string.IsNullOrWhiteSpace(Context))
            throw new ArgumentException("Context is required.", nameof(Context));
    }
}

public sealed record ProductDefinition(
    SemanticId ProductId,
    string PartNumber,
    string Revision,
    IReadOnlyList<ProductComponent> Components,
    IReadOnlyList<ProductOccurrence> Occurrences)
{
    public ProductDefinition
    {
        if (ProductId.Value == Guid.Empty)
            throw new ArgumentException("ProductId is required.", nameof(ProductId));

        if (string.IsNullOrWhiteSpace(PartNumber))
            throw new ArgumentException("PartNumber is required.", nameof(PartNumber));

        if (string.IsNullOrWhiteSpace(Revision))
            throw new ArgumentException("Revision is required.", nameof(Revision));

        Components = Components?.ToArray() ??
            throw new ArgumentNullException(nameof(Components));

        Occurrences = Occurrences?.ToArray() ??
            throw new ArgumentNullException(nameof(Occurrences));
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
