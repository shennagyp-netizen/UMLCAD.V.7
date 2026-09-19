using UMLCAD.Cad.Semantics;

namespace UMLCAD.Cad.Engine;

public interface IAuthoritativeResultCatalog
{
    void Register(AuthoritativeCadResult result);

    ReferenceResolution Resolve(CadReference reference);
}

public sealed class AuthoritativeResultCatalog : IAuthoritativeResultCatalog
{
    private readonly Dictionary<AuthoritativeResultIdentity, AuthoritativeCadResult> _results = new();

    public void Register(AuthoritativeCadResult result)
    {
        ArgumentNullException.ThrowIfNull(result);

        if (_results.ContainsKey(result.Identity))
            throw new InvalidOperationException(
                $"Authoritative result '{result.Identity}' is already registered.");

        _results.Add(result.Identity, result);
    }

    public ReferenceResolution Resolve(CadReference reference)
    {
        ArgumentNullException.ThrowIfNull(reference);

        return reference switch
        {
            SemanticReference semantic => ResolveSemantic(semantic),
            TopologyReference topology => ResolveTopology(topology),
            GeometricReference geometric => ResolveGeometric(geometric),
            _ => new ReferenceResolution(
                reference,
                ReferenceResolutionStatus.Unsupported,
                null,
                $"Reference type '{reference.GetType().Name}' is not supported by this resolver."),
        };
    }

    private ReferenceResolution ResolveSemantic(SemanticReference reference)
    {
        var matches = _results.Values
            .Where(x => x.ProducingSemanticId == reference.TargetId)
            .OrderBy(x => x.Identity.Value, StringComparer.Ordinal)
            .ToArray();

        return matches.Length switch
        {
            0 => new ReferenceResolution(
                reference,
                ReferenceResolutionStatus.Missing,
                null,
                $"No authoritative result is produced by semantic id '{reference.TargetId}'."),
            1 => new ReferenceResolution(
                reference,
                ReferenceResolutionStatus.Resolved,
                matches[0].Identity.Value,
                null),
            _ => new ReferenceResolution(
                reference,
                ReferenceResolutionStatus.Ambiguous,
                null,
                $"Multiple authoritative results are produced by semantic id '{reference.TargetId}'."),
        };
    }

    private ReferenceResolution ResolveTopology(TopologyReference reference)
    {
        if (!_results.TryGetValue(
                new AuthoritativeResultIdentity(reference.AuthoritativeResultId),
                out var result))
        {
            return new ReferenceResolution(
                reference,
                ReferenceResolutionStatus.Missing,
                null,
                $"Authoritative result '{reference.AuthoritativeResultId}' does not exist.");
        }

        var matches = result.TopologyBindings
            .Where(x =>
                string.Equals(x.TopologyKey, reference.TopologyKey, StringComparison.Ordinal) &&
                string.Equals(x.TopologyKind, reference.TopologyKind, StringComparison.Ordinal))
            .ToArray();

        return matches.Length switch
        {
            0 => new ReferenceResolution(
                reference,
                ReferenceResolutionStatus.Missing,
                null,
                $"Topology '{reference.TopologyKind}:{reference.TopologyKey}' is not present in result '{result.Identity}'."),
            1 when result.Status == AuthoritativeResultStatus.Succeeded => new ReferenceResolution(
                reference,
                ReferenceResolutionStatus.Resolved,
                result.Identity.Value,
                null),
            1 => new ReferenceResolution(
                reference,
                ReferenceResolutionStatus.Indeterminate,
                null,
                $"Result '{result.Identity}' is not successful and cannot resolve topology authoritatively."),
            _ => new ReferenceResolution(
                reference,
                ReferenceResolutionStatus.Ambiguous,
                null,
                $"Topology '{reference.TopologyKind}:{reference.TopologyKey}' is multiply bound in result '{result.Identity}'."),
        };
    }

    private ReferenceResolution ResolveGeometric(GeometricReference reference)
    {
        var matches = _results.Values
            .Where(x => x.Status == AuthoritativeResultStatus.Succeeded)
            .Where(x => x.TopologyBindings.Any(binding =>
                binding.SourceSemanticId == reference.GeometryOwnerId &&
                string.Equals(binding.TopologyKey, reference.GeometryKey, StringComparison.Ordinal)))
            .OrderBy(x => x.Identity.Value, StringComparer.Ordinal)
            .ToArray();

        return matches.Length switch
        {
            0 => new ReferenceResolution(
                reference,
                ReferenceResolutionStatus.Missing,
                null,
                $"No authoritative result exposes geometry key '{reference.GeometryKey}' for owner '{reference.GeometryOwnerId}'."),
            1 => new ReferenceResolution(
                reference,
                ReferenceResolutionStatus.Resolved,
                matches[0].Identity.Value,
                null),
            _ => new ReferenceResolution(
                reference,
                ReferenceResolutionStatus.Ambiguous,
                null,
                $"Multiple authoritative results expose geometry key '{reference.GeometryKey}' for owner '{reference.GeometryOwnerId}'."),
        };
    }
}
