namespace UMLCAD.Framework.Semantics;

public sealed record ParameterSemantic(string Name, string Value, string? Unit = null);

public sealed record CadMetadata(
    string? PartNumber = null,
    string? Description = null,
    string? Material = null,
    string? Manufacturer = null,
    string? Vendor = null,
    string? Revision = null,
    string? LifecycleState = null,
    string? Author = null,
    string? DocumentCode = null,
    IReadOnlyDictionary<string, string>? Custom = null) : IReadOnlyDictionary<string, string>
{
    public static CadMetadata Empty { get; } = new();

    private IReadOnlyDictionary<string, string> Values => BuildValues();

    public string this[string key] => Values[key];
    public IEnumerable<string> Keys => Values.Keys;
    IEnumerable<string> IReadOnlyDictionary<string, string>.Values => Values.Values;
    public int Count => Values.Count;
    public bool ContainsKey(string key) => Values.ContainsKey(key);
    public bool TryGetValue(string key, out string value) => Values.TryGetValue(key, out value!);
    public IEnumerator<KeyValuePair<string, string>> GetEnumerator() => Values.GetEnumerator();
    System.Collections.IEnumerator System.Collections.IEnumerable.GetEnumerator() => GetEnumerator();
    public IReadOnlyDictionary<string, string> CustomProperties => Custom ?? new Dictionary<string, string>(StringComparer.Ordinal);

    private IReadOnlyDictionary<string, string> BuildValues()
    {
        var result = new Dictionary<string, string>(StringComparer.Ordinal);
        Add(result, "partNumber", PartNumber);
        Add(result, "description", Description);
        Add(result, "material", Material);
        Add(result, "manufacturer", Manufacturer);
        Add(result, "vendor", Vendor);
        Add(result, "revision", Revision);
        Add(result, "lifecycleState", LifecycleState);
        Add(result, "author", Author);
        Add(result, "documentCode", DocumentCode);
        foreach (var pair in CustomProperties)
            result[$"custom:{pair.Key}"] = pair.Value;
        return new SortedDictionary<string, string>(result, StringComparer.Ordinal);
    }

    private static void Add(IDictionary<string, string> target, string key, string? value)
    {
        if (value is not null)
            target[key] = value;
    }
}

public sealed record GeometrySemantic(
    string Id,
    string Kind,
    IReadOnlyDictionary<string, string> Properties) : SemanticEntity(Id, Kind);

public sealed record ConstraintSemantic(
    string Id,
    string Kind,
    IReadOnlyList<string> References,
    IReadOnlyDictionary<string, string> Properties) : SemanticEntity(Id, Kind);

public sealed record ComponentSemantic(
    string Id,
    string ComponentType,
    IReadOnlyList<string> Children,
    IReadOnlyDictionary<string, string> Parameters) : SemanticEntity(Id, "component");

public sealed record TransformSemantic(IReadOnlyList<double> Matrix)
{
    public static TransformSemantic Identity { get; } = new([
        1, 0, 0, 0,
        0, 1, 0, 0,
        0, 0, 1, 0,
        0, 0, 0, 1]);

    public static TransformSemantic FromArray(IReadOnlyList<double> matrix)
    {
        ArgumentNullException.ThrowIfNull(matrix);
        if (matrix.Count != 16)
            throw new ArgumentException("A transform matrix must contain 16 values.", nameof(matrix));
        if (matrix.Any(double.IsNaN) || matrix.Any(double.IsInfinity))
            throw new ArgumentException("A transform matrix cannot contain NaN or infinity.", nameof(matrix));
        if (matrix[15] == 0)
            throw new ArgumentException("A transform matrix must have a non-zero homogeneous component.", nameof(matrix));
        return new TransformSemantic(matrix.ToArray());
    }
}

public sealed record AssemblyOccurrenceSemantic(
    string Id,
    string Name,
    string DefinitionId,
    string DefinitionKind,
    TransformSemantic Transform,
    IReadOnlyDictionary<string, string> Metadata,
    string? ConfigurationName,
    double Quantity,
    string? BomStructure,
    bool Visible,
    bool Suppressed,
    bool Grounded,
    bool Flexible);

public sealed record PartSemantic(
    string Id,
    string PartType,
    IReadOnlyList<ParameterSemantic> Parameters,
    IReadOnlyList<GeometrySemantic> Geometry,
    IReadOnlyList<ConstraintSemantic> Constraints,
    IReadOnlyList<string> References,
    IReadOnlyList<string> Components) : SemanticEntity(Id, "part")
{
    public string Name { get; init; } = Id;
    public CadMetadata StructuredMetadata { get; init; } = CadMetadata.Empty;
}

public sealed record SheetSemantic(
    string Id,
    string Name,
    IReadOnlyList<string> DrawingReferences,
    IReadOnlyDictionary<string, string> Settings) : SemanticEntity(Id, "sheet");

public sealed record DrawingSemantic(
    string Id,
    string Name,
    IReadOnlyList<SheetSemantic> Sheets,
    IReadOnlyList<string> PartReferences,
    IReadOnlyDictionary<string, string> Settings) : SemanticEntity(Id, "drawing");

public sealed record AssemblySemantic(
    string Id,
    string Name,
    IReadOnlyList<string> ComponentReferences,
    IReadOnlyDictionary<string, string> Settings) : SemanticEntity(Id, "assembly")
{
    public CadMetadata StructuredMetadata { get; init; } = CadMetadata.Empty;
    public IReadOnlyList<AssemblyOccurrenceSemantic> Occurrences { get; init; } = Array.Empty<AssemblyOccurrenceSemantic>();
}

public sealed record SemanticApplication(
    string Id,
    string Version,
    IReadOnlyDictionary<string, string> Configuration,
    IReadOnlyList<PartSemantic> Parts,
    IReadOnlyList<AssemblySemantic> Assemblies,
    IReadOnlyList<DrawingSemantic> Drawings,
    string BuildIdentity);
