namespace UMLCAD.Framework.Semantics;

public interface ISemanticRegistry
{
    void RegisterPart(PartSemantic part);
    void RegisterAssembly(AssemblySemantic assembly);
    void RegisterDrawing(DrawingSemantic drawing);

    SemanticApplication Snapshot(
        string applicationId,
        string version,
        IReadOnlyDictionary<string, string> configuration,
        string buildIdentity);
}

internal sealed class SemanticRegistry : ISemanticRegistry
{
    private readonly Dictionary<string, PartSemantic> _parts = new(StringComparer.Ordinal);
    private readonly Dictionary<string, AssemblySemantic> _assemblies = new(StringComparer.Ordinal);
    private readonly Dictionary<string, DrawingSemantic> _drawings = new(StringComparer.Ordinal);

    public void RegisterPart(PartSemantic part) => AddUnique(_parts, part.Id, part);

    public void RegisterAssembly(AssemblySemantic assembly) => AddUnique(_assemblies, assembly.Id, assembly);

    public void RegisterDrawing(DrawingSemantic drawing) => AddUnique(_drawings, drawing.Id, drawing);

    public SemanticApplication Snapshot(
        string applicationId,
        string version,
        IReadOnlyDictionary<string, string> configuration,
        string buildIdentity)
    {
        return new SemanticApplication(
            applicationId,
            version,
            new SortedDictionary<string, string>(configuration.ToDictionary(x => x.Key, x => x.Value, StringComparer.Ordinal), StringComparer.Ordinal),
            _parts.Values.OrderBy(x => x.Id, StringComparer.Ordinal).ToArray(),
            _assemblies.Values.OrderBy(x => x.Id, StringComparer.Ordinal).ToArray(),
            _drawings.Values.OrderBy(x => x.Id, StringComparer.Ordinal).ToArray(),
            buildIdentity);
    }

    private static void AddUnique<T>(Dictionary<string, T> target, string id, T value)
    {
        if (string.IsNullOrWhiteSpace(id))
            throw new ArgumentException("Semantic entity IDs cannot be empty.", nameof(id));

        if (!target.TryAdd(id, value))
            throw new InvalidOperationException($"A semantic entity with ID '{id}' is already registered.");
    }
}
