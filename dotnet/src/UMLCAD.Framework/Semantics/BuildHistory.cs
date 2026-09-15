namespace UMLCAD.Framework.Semantics;

public sealed record BuildSnapshot(
    long Sequence,
    SemanticApplication Application,
    DateTimeOffset CreatedAt,
    string PackageIdentity);

public interface IBuildHistory
{
    BuildSnapshot Current { get; }
    IReadOnlyList<BuildSnapshot> GetAll();
    bool TryGet(long sequence, out BuildSnapshot? snapshot);
    void Record(SemanticApplication application, string packageIdentity);
}

internal sealed class BuildHistory : IBuildHistory
{
    private readonly object _gate = new();
    private readonly List<BuildSnapshot> _snapshots = [];
    private long _nextSequence;

    public BuildSnapshot Current
    {
        get
        {
            lock (_gate)
            {
                return _snapshots.Count == 0
                    ? throw new InvalidOperationException("No CAD build has been recorded yet.")
                    : _snapshots[^1];
            }
        }
    }

    public IReadOnlyList<BuildSnapshot> GetAll()
    {
        lock (_gate)
            return _snapshots.ToArray();
    }

    public bool TryGet(long sequence, out BuildSnapshot? snapshot)
    {
        lock (_gate)
        {
            snapshot = _snapshots.FirstOrDefault(x => x.Sequence == sequence);
            return snapshot is not null;
        }
    }

    public void Record(SemanticApplication application, string packageIdentity)
    {
        ArgumentNullException.ThrowIfNull(application);
        ArgumentException.ThrowIfNullOrWhiteSpace(packageIdentity);

        lock (_gate)
        {
            _snapshots.Add(new BuildSnapshot(
                ++_nextSequence,
                application,
                DateTimeOffset.UtcNow,
                packageIdentity));
        }
    }
}
