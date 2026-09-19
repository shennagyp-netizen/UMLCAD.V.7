namespace UMLCAD.Framework.Semantics;

public enum SemanticReferenceStatus
{
    Resolved,
    Missing,
    Ambiguous,
    Indeterminate,
    Unsupported
}

public sealed record SemanticReference(
    string ProducerId,
    string TargetId,
    string TargetKind,
    string? ExpectedResultIdentity = null);

public sealed record SemanticReferenceCandidate(
    string ProducerId,
    string TargetId,
    string TargetKind,
    string? ResultIdentity = null,
    string? ProvenanceId = null);

public sealed record SemanticReferenceResolution(
    SemanticReference Reference,
    SemanticReferenceStatus Status,
    IReadOnlyList<SemanticReferenceCandidate> Candidates,
    string DiagnosticCode,
    string Message)
{
    public bool IsResolved => Status == SemanticReferenceStatus.Resolved;

    public SemanticReferenceCandidate? Target =>
        IsResolved && Candidates.Count == 1 ? Candidates[0] : null;
}

public interface ISemanticReferenceService
{
    SemanticReferenceResolution Resolve(
        SemanticApplication application,
        SemanticReference reference);
}

internal sealed class SemanticReferenceService : ISemanticReferenceService
{
    public SemanticReferenceResolution Resolve(
        SemanticApplication application,
        SemanticReference reference)
    {
        ArgumentNullException.ThrowIfNull(application);
        ArgumentNullException.ThrowIfNull(reference);

        if (string.IsNullOrWhiteSpace(reference.ProducerId) ||
            string.IsNullOrWhiteSpace(reference.TargetId) ||
            string.IsNullOrWhiteSpace(reference.TargetKind) ||
            (reference.ExpectedResultIdentity is not null &&
             string.IsNullOrWhiteSpace(reference.ExpectedResultIdentity)))
        {
            return Fail(
                reference,
                SemanticReferenceStatus.Indeterminate,
                "REFERENCE_INVALID",
                "Reference producer, target, target kind, and any supplied result identity must be non-empty.");
        }

        if (string.Equals(reference.ProducerId, application.Id, StringComparison.Ordinal))
            return Classify(
                reference,
                ResolveApplicationTarget(application, reference),
                new[] { "part", "assembly", "drawing" });

        var part = application.Parts.FirstOrDefault(
            x => string.Equals(x.Id, reference.ProducerId, StringComparison.Ordinal));
        if (part is not null)
        {
            if (string.Equals(reference.TargetKind, "face", StringComparison.Ordinal))
                return ResolvePublishedFace(part, application, reference);

            return Classify(
                reference,
                ResolvePartTarget(part, reference),
                new[] { "geometry", "constraint", "component" });
        }

        if (TryFindOccurrence(application, reference.ProducerId, out var containingAssembly, out var occurrence))
        {
            if (!string.Equals(reference.TargetKind, "face", StringComparison.Ordinal))
                return Fail(
                    reference,
                    SemanticReferenceStatus.Unsupported,
                    "REFERENCE_UNSUPPORTED_TARGET_KIND",
                    $"Reference target kind '{reference.TargetKind}' is unsupported for occurrence producer '{reference.ProducerId}'.");

            if (occurrence.DefinitionKind != "part")
                return Fail(
                    reference,
                    SemanticReferenceStatus.Unsupported,
                    "REFERENCE_UNSUPPORTED_OCCURRENCE_DEFINITION",
                    $"Occurrence producer '{reference.ProducerId}' does not currently expose a Part face reference.");

            if (occurrence.ConfigurationName is not null)
                return Fail(
                    reference,
                    SemanticReferenceStatus.Indeterminate,
                    "REFERENCE_CONTEXT_UNSUPPORTED",
                    $"Occurrence producer '{reference.ProducerId}' has configuration context '{occurrence.ConfigurationName}', but configuration-aware result resolution is not yet implemented.");

            var sourcePart = application.Parts.FirstOrDefault(
                x => string.Equals(x.Id, occurrence.DefinitionId, StringComparison.Ordinal));
            if (sourcePart is null)
                return Fail(
                    reference,
                    SemanticReferenceStatus.Missing,
                    "REFERENCE_PRODUCER_MISSING",
                    $"Occurrence producer '{reference.ProducerId}' names missing Part definition '{occurrence.DefinitionId}'.");

            var sourceResolution = ResolvePublishedFace(sourcePart, application, reference);
            if (sourceResolution.Candidates.Count == 0)
                return new SemanticReferenceResolution(
                    reference,
                    sourceResolution.Status,
                    sourceResolution.Candidates,
                    sourceResolution.DiagnosticCode,
                    sourceResolution.Message);

            var reboundCandidates = sourceResolution.Candidates
                .Select(candidate => candidate with { ProducerId = reference.ProducerId })
                .ToArray();

            return new SemanticReferenceResolution(
                reference,
                sourceResolution.Status,
                reboundCandidates,
                sourceResolution.DiagnosticCode,
                sourceResolution.Message);
        }

        var assembly = application.Assemblies.FirstOrDefault(
            x => string.Equals(x.Id, reference.ProducerId, StringComparison.Ordinal));
        if (assembly is not null)
            return Classify(
                reference,
                ResolveAssemblyTarget(assembly, reference),
                new[] { "occurrence" });

        var drawing = application.Drawings.FirstOrDefault(
            x => string.Equals(x.Id, reference.ProducerId, StringComparison.Ordinal));
        if (drawing is not null)
            return Classify(
                reference,
                ResolveDrawingTarget(drawing, reference),
                new[] { "sheet" });

        return Fail(
            reference,
            SemanticReferenceStatus.Missing,
            "REFERENCE_PRODUCER_MISSING",
            $"Reference producer '{reference.ProducerId}' does not exist.");
    }

    private static bool TryFindOccurrence(
        SemanticApplication application,
        string producerId,
        out AssemblySemantic containingAssembly,
        out AssemblyOccurrenceSemantic occurrence)
    {
        containingAssembly = null!;
        occurrence = null!;

        const string prefix = "occurrence:";
        if (!producerId.StartsWith(prefix, StringComparison.Ordinal))
            return false;

        var encoded = producerId[prefix.Length..];
        var separator = encoded.IndexOf('/', StringComparison.Ordinal);
        if (separator <= 0 || separator == encoded.Length - 1)
            return false;

        string assemblyId;
        string occurrenceId;
        try
        {
            assemblyId = Uri.UnescapeDataString(encoded[..separator]);
            occurrenceId = Uri.UnescapeDataString(encoded[(separator + 1)..]);
        }
        catch (UriFormatException)
        {
            return false;
        }

        var assembly = application.Assemblies.FirstOrDefault(
            x => string.Equals(x.Id, assemblyId, StringComparison.Ordinal));
        if (assembly is null)
            return false;

        var match = assembly.Occurrences.FirstOrDefault(
            x => string.Equals(x.Id, occurrenceId, StringComparison.Ordinal));
        if (match is null)
            return false;

        containingAssembly = assembly;
        occurrence = match;
        return true;
    }

    private static IReadOnlyList<SemanticReferenceCandidate> ResolveApplicationTarget(
        SemanticApplication application,
        SemanticReference reference) =>
        reference.TargetKind switch
        {
            "part" => application.Parts
                .Where(x => string.Equals(x.Id, reference.TargetId, StringComparison.Ordinal))
                .Select(x => Candidate(application.Id, x.Id, "part"))
                .ToArray(),
            "assembly" => application.Assemblies
                .Where(x => string.Equals(x.Id, reference.TargetId, StringComparison.Ordinal))
                .Select(x => Candidate(application.Id, x.Id, "assembly"))
                .ToArray(),
            "drawing" => application.Drawings
                .Where(x => string.Equals(x.Id, reference.TargetId, StringComparison.Ordinal))
                .Select(x => Candidate(application.Id, x.Id, "drawing"))
                .ToArray(),
            _ => Array.Empty<SemanticReferenceCandidate>()
        };

    private static SemanticReferenceResolution ResolvePublishedFace(
        PartSemantic part,
        SemanticApplication application,
        SemanticReference reference)
    {
        var publications = part.Publications
            .Where(x =>
                string.Equals(x.TargetKind, "face", StringComparison.Ordinal) &&
                string.Equals(x.TargetId, reference.TargetId, StringComparison.Ordinal))
            .OrderBy(x => x.Id, StringComparer.Ordinal)
            .ToArray();

        if (publications.Length == 0)
            return Fail(
                reference,
                SemanticReferenceStatus.Indeterminate,
                "REFERENCE_PUBLICATION_MISSING",
                $"No face publication for target '{reference.TargetId}' exists under producer '{reference.ProducerId}'.");

        var selectedPublications = reference.ExpectedResultIdentity is null
            ? publications
            : publications
                .Where(x => string.Equals(x.ResultIdentity, reference.ExpectedResultIdentity, StringComparison.Ordinal))
                .ToArray();

        if (selectedPublications.Length == 0 && reference.ExpectedResultIdentity is not null)
            return Fail(
                reference,
                SemanticReferenceStatus.Indeterminate,
                "REFERENCE_STALE_RESULT",
                $"No face publication for target '{reference.TargetId}' is associated with expected result identity '{reference.ExpectedResultIdentity}'.");

        var candidates = new List<SemanticReferenceCandidate>();
        var resultMissing = false;
        var resultOwnerMismatch = false;
        var resultAmbiguous = false;
        var resultNotAuthoritative = false;
        var resultInvalid = false;
        var provenanceMissing = false;
        var provenanceMismatch = false;

        foreach (var publication in selectedPublications)
        {
            var results = application.AuthoritativeResults
                .Where(x => string.Equals(x.Id, publication.ResultIdentity, StringComparison.Ordinal))
                .ToArray();

            if (results.Length == 0)
            {
                resultMissing = true;
                continue;
            }

            if (results.Length > 1)
            {
                resultAmbiguous = true;
                continue;
            }

            var result = results[0];
            if (string.IsNullOrWhiteSpace(result.ProducerId) ||
                string.IsNullOrWhiteSpace(result.OperationIdentity) ||
                string.IsNullOrWhiteSpace(result.ContractIdentity) ||
                string.IsNullOrWhiteSpace(result.EvidenceIdentity))
            {
                resultInvalid = true;
                continue;
            }

            if (!string.Equals(result.ProducerId, part.Id, StringComparison.Ordinal))
            {
                resultOwnerMismatch = true;
                continue;
            }

            if (result.Status != AuthoritativeResultStatus.Authoritative)
            {
                resultNotAuthoritative = true;
                continue;
            }

            var binding = part.TopologyBindings.FirstOrDefault(
                x => string.Equals(x.Id, publication.TopologyBindingId, StringComparison.Ordinal));

            if (binding is null)
            {
                provenanceMissing = true;
                continue;
            }

            var valid =
                string.Equals(binding.TopologyKind, publication.TargetKind, StringComparison.Ordinal) &&
                string.Equals(binding.SemanticTargetId, publication.TargetId, StringComparison.Ordinal) &&
                string.Equals(binding.ResultIdentity, publication.ResultIdentity, StringComparison.Ordinal);

            if (!valid)
            {
                provenanceMismatch = true;
                continue;
            }

            candidates.Add(new SemanticReferenceCandidate(
                part.Id,
                publication.TargetId,
                publication.TargetKind,
                publication.ResultIdentity,
                publication.TopologyBindingId));
        }

        if (reference.ExpectedResultIdentity is null &&
            (resultInvalid || resultAmbiguous || resultOwnerMismatch || resultNotAuthoritative ||
             resultMissing || provenanceMissing || provenanceMismatch))
        {
            return Fail(
                reference,
                SemanticReferenceStatus.Indeterminate,
                resultInvalid
                    ? "REFERENCE_RESULT_INVALID"
                    : resultAmbiguous
                        ? "REFERENCE_RESULT_AMBIGUOUS"
                        : resultOwnerMismatch
                            ? "REFERENCE_RESULT_OWNER_MISMATCH"
                            : resultNotAuthoritative
                                ? "REFERENCE_RESULT_NOT_AUTHORITATIVE"
                                : resultMissing
                                    ? "REFERENCE_RESULT_MISSING"
                                    : provenanceMissing
                                        ? "REFERENCE_PROVENANCE_MISSING"
                                        : "REFERENCE_PROVENANCE_MISMATCH",
                $"Face publication '{reference.TargetId}' cannot be resolved safely because not all matching publications have valid authoritative result provenance.");
        }

        if (candidates.Count == 0)
        {
            var code = resultInvalid
                ? "REFERENCE_RESULT_INVALID"
                : resultAmbiguous
                    ? "REFERENCE_RESULT_AMBIGUOUS"
                    : resultOwnerMismatch
                        ? "REFERENCE_RESULT_OWNER_MISMATCH"
                        : resultNotAuthoritative
                            ? "REFERENCE_RESULT_NOT_AUTHORITATIVE"
                            : resultMissing
                                ? "REFERENCE_RESULT_MISSING"
                                : provenanceMissing
                                    ? "REFERENCE_PROVENANCE_MISSING"
                                    : provenanceMismatch
                                        ? "REFERENCE_PROVENANCE_MISMATCH"
                                        : "REFERENCE_STALE_RESULT";

            return Fail(
                reference,
                SemanticReferenceStatus.Indeterminate,
                code,
                $"Face publication '{reference.TargetId}' has no single authoritative result with matching topology provenance.");
        }

        return Classify(
            reference,
            candidates,
            new[] { "face" });
    }

    private static IReadOnlyList<SemanticReferenceCandidate> ResolvePartTarget(
        PartSemantic part,
        SemanticReference reference) =>
        reference.TargetKind switch
        {
            "geometry" => part.Geometry
                .Where(x => string.Equals(x.Id, reference.TargetId, StringComparison.Ordinal))
                .Select(x => Candidate(part.Id, x.Id, "geometry"))
                .ToArray(),
            "constraint" => part.Constraints
                .Where(x => string.Equals(x.Id, reference.TargetId, StringComparison.Ordinal))
                .Select(x => Candidate(part.Id, x.Id, "constraint"))
                .ToArray(),
            "component" => part.Components
                .Where(x => string.Equals(x, reference.TargetId, StringComparison.Ordinal))
                .Select(x => Candidate(part.Id, x, "component"))
                .ToArray(),
            _ => Array.Empty<SemanticReferenceCandidate>()
        };

    private static IReadOnlyList<SemanticReferenceCandidate> ResolveAssemblyTarget(
        AssemblySemantic assembly,
        SemanticReference reference) =>
        reference.TargetKind == "occurrence"
            ? assembly.Occurrences
                .Where(x => string.Equals(x.Id, reference.TargetId, StringComparison.Ordinal))
                .Select(x => Candidate(assembly.Id, x.Id, "occurrence"))
                .ToArray()
            : Array.Empty<SemanticReferenceCandidate>();

    private static IReadOnlyList<SemanticReferenceCandidate> ResolveDrawingTarget(
        DrawingSemantic drawing,
        SemanticReference reference) =>
        reference.TargetKind == "sheet"
            ? drawing.Sheets
                .Where(x => string.Equals(x.Id, reference.TargetId, StringComparison.Ordinal))
                .Select(x => Candidate(drawing.Id, x.Id, "sheet"))
                .ToArray()
            : Array.Empty<SemanticReferenceCandidate>();

    private static SemanticReferenceCandidate Candidate(
        string producerId,
        string targetId,
        string targetKind) =>
        new(producerId, targetId, targetKind);

    private static SemanticReferenceResolution Classify(
        SemanticReference reference,
        IReadOnlyList<SemanticReferenceCandidate> candidates,
        IReadOnlyList<string> supportedKinds)
    {
        if (candidates.Count == 0)
        {
            if (!supportedKinds.Contains(reference.TargetKind, StringComparer.Ordinal))
                return Fail(
                    reference,
                    SemanticReferenceStatus.Unsupported,
                    "REFERENCE_UNSUPPORTED_TARGET_KIND",
                    $"Reference target kind '{reference.TargetKind}' is unsupported for producer '{reference.ProducerId}'.");

            return Fail(
                reference,
                SemanticReferenceStatus.Missing,
                "REFERENCE_TARGET_MISSING",
                $"Reference target '{reference.TargetId}' of kind '{reference.TargetKind}' does not exist under producer '{reference.ProducerId}'.");
        }

        if (reference.ExpectedResultIdentity is not null)
        {
            var matching = candidates
                .Where(x => string.Equals(x.ResultIdentity, reference.ExpectedResultIdentity, StringComparison.Ordinal))
                .ToArray();

            if (matching.Length == 0)
                return Fail(
                    reference,
                    SemanticReferenceStatus.Indeterminate,
                    "REFERENCE_STALE_RESULT",
                    $"Reference expected result identity '{reference.ExpectedResultIdentity}', but no resolved candidate is backed by that result identity.");

            candidates = matching;
        }

        if (candidates.Count == 1)
            return new(
                reference,
                SemanticReferenceStatus.Resolved,
                candidates,
                "REFERENCE_RESOLVED",
                $"Reference resolved to {reference.TargetKind} '{reference.TargetId}' from producer '{reference.ProducerId}'.");

        if (candidates.Count > 1)
            return new(
                reference,
                SemanticReferenceStatus.Ambiguous,
                candidates,
                "REFERENCE_AMBIGUOUS",
                $"Reference target '{reference.TargetId}' is ambiguous within producer '{reference.ProducerId}'.");

    }

    private static SemanticReferenceResolution Fail(
        SemanticReference reference,
        SemanticReferenceStatus status,
        string code,
        string message) =>
        new(reference, status, Array.Empty<SemanticReferenceCandidate>(), code, message);
}
