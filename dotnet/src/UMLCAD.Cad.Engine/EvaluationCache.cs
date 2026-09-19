using System.Collections.Concurrent;

namespace UMLCAD.Cad.Engine;

public interface IEvaluationCache
{
    bool TryGet(
        EvaluationIdentity identity,
        out EvaluationOutcome outcome);

    void Put(
        EvaluationIdentity identity,
        EvaluationOutcome outcome);
}

public sealed class DeterministicEvaluationCache : IEvaluationCache
{
    private readonly ConcurrentDictionary<string, EvaluationOutcome> _entries =
        new(StringComparer.Ordinal);

    public bool TryGet(
        EvaluationIdentity identity,
        out EvaluationOutcome outcome) =>
        _entries.TryGetValue(identity.Value, out outcome!);

    public void Put(
        EvaluationIdentity identity,
        EvaluationOutcome outcome)
    {
        ArgumentNullException.ThrowIfNull(outcome);

        if (outcome.Identity != identity)
            throw new ArgumentException(
                "Cache entry identity does not match the supplied identity.",
                nameof(outcome));

        _entries[identity.Value] = outcome;
    }

    public int Count => _entries.Count;
}
