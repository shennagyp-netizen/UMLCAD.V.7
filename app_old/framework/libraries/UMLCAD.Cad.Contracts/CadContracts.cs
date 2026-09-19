namespace UMLCAD.Cad.Contracts;

public readonly record struct CadId
{
    public CadId(string value)
    {
        if (string.IsNullOrWhiteSpace(value))
            throw new ArgumentException("CAD identifier cannot be empty.", nameof(value));

        Value = value;
    }

    public string Value { get; }

    public override string ToString() => Value;
}

public sealed record CadBuildIdentity(
    string ApplicationId,
    string ApplicationVersion,
    string SemanticIdentity)
{
    public CadBuildIdentity
    {
        if (string.IsNullOrWhiteSpace(ApplicationId))
            throw new ArgumentException("Application ID cannot be empty.", nameof(ApplicationId));
        if (string.IsNullOrWhiteSpace(ApplicationVersion))
            throw new ArgumentException("Application version cannot be empty.", nameof(ApplicationVersion));
        if (string.IsNullOrWhiteSpace(SemanticIdentity))
            throw new ArgumentException("Semantic identity cannot be empty.", nameof(SemanticIdentity));
    }
}

public sealed record CadBuildDefinition(
    CadBuildIdentity Identity,
    string SemanticJson)
{
    public CadBuildDefinition
    {
        if (string.IsNullOrWhiteSpace(SemanticJson))
            throw new ArgumentException("Semantic JSON cannot be empty.", nameof(SemanticJson));
    }
}
