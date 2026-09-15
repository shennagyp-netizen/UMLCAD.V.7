namespace UMLCAD.Framework.Semantics;

public abstract record SemanticEntity(string Id, string Kind)
{
    public string? Source { get; init; }
    public IReadOnlyDictionary<string, string> Metadata { get; init; } =
        new Dictionary<string, string>(StringComparer.Ordinal);
}
