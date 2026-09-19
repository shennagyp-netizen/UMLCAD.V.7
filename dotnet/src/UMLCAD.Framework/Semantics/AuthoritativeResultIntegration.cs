namespace UMLCAD.Framework.Semantics;

/// <summary>
/// Integrates an evaluated result into an immutable semantic snapshot.
/// The result identity is derived-state identity and must not replace the
/// semantic build identity.
/// </summary>
public interface IAuthoritativeResultIntegrationService
{
    SemanticApplication Integrate(
        SemanticApplication application,
        AuthoritativeResultSemantic result);
}

internal sealed class AuthoritativeResultIntegrationService : IAuthoritativeResultIntegrationService
{
    public SemanticApplication Integrate(
        SemanticApplication application,
        AuthoritativeResultSemantic result)
    {
        ArgumentNullException.ThrowIfNull(application);
        ArgumentNullException.ThrowIfNull(result);

        ValidateResult(result);

        var producerExists =
            application.Parts.Any(x => string.Equals(x.Id, result.ProducerId, StringComparison.Ordinal)) ||
            application.Assemblies.Any(x => string.Equals(x.Id, result.ProducerId, StringComparison.Ordinal));

        if (!producerExists)
            throw new InvalidOperationException(
                $"RESULT_PRODUCER_MISSING: result '{result.Id}' names unknown producer '{result.ProducerId}'.");

        var existing = application.AuthoritativeResults
            .Where(x => string.Equals(x.Id, result.Id, StringComparison.Ordinal))
            .ToArray();

        if (existing.Length > 1)
            throw new InvalidOperationException(
                $"RESULT_ID_CONFLICT: semantic snapshot already contains multiple results with identity '{result.Id}'.");

        if (existing.Length == 1)
        {
            if (existing[0] == result)
                return application;

            throw new InvalidOperationException(
                $"RESULT_ID_CONFLICT: result identity '{result.Id}' is already bound to different result content.");
        }

        var results = application.AuthoritativeResults
            .Append(result)
            .OrderBy(x => x.Id, StringComparer.Ordinal)
            .ToArray();

        return application with { AuthoritativeResults = results };
    }

    private static void ValidateResult(AuthoritativeResultSemantic result)
    {
        if (string.IsNullOrWhiteSpace(result.Id))
            throw new ArgumentException("Authoritative result identity cannot be empty.", nameof(result));

        if (string.IsNullOrWhiteSpace(result.ProducerId))
            throw new ArgumentException("Authoritative result producer identity cannot be empty.", nameof(result));

        if (string.IsNullOrWhiteSpace(result.OperationIdentity))
            throw new ArgumentException("Authoritative result operation identity cannot be empty.", nameof(result));

        if (string.IsNullOrWhiteSpace(result.ContractIdentity))
            throw new ArgumentException("Authoritative result contract identity cannot be empty.", nameof(result));

        if (string.IsNullOrWhiteSpace(result.EvidenceIdentity))
            throw new ArgumentException("Authoritative result evidence identity cannot be empty.", nameof(result));
    }
}
