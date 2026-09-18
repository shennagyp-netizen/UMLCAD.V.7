using UMLCAD.Cad.Semantics;

namespace UMLCAD.Cad.Engine;

public enum RecomputeMode
{
    Full,
    Incremental,
}

public sealed record CadChangeSet(
    RecomputeMode Mode,
    IReadOnlySet<SemanticId> ChangedFeatureIds)
{
    public CadChangeSet
    {
        ArgumentNullException.ThrowIfNull(ChangedFeatureIds);
        ChangedFeatureIds = new HashSet<SemanticId>(ChangedFeatureIds);

        if (Mode == RecomputeMode.Incremental && ChangedFeatureIds.Count == 0)
            throw new ArgumentException(
                "Incremental recompute requires at least one changed feature.",
                nameof(ChangedFeatureIds));
    }

    public static CadChangeSet Full() =>
        new(RecomputeMode.Full, new HashSet<SemanticId>());
}
