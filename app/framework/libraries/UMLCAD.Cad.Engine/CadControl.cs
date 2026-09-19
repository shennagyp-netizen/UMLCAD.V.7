using System.Collections.Immutable;
using UMLCAD.Cad.Contracts;
using UMLCAD.Cad.Semantics;

namespace UMLCAD.Cad.Engine;

public enum CadCommandKind
{
    AddFeature,
    RemoveFeature,
    RenameFeature
}

public abstract record CadCommand
{
    public abstract CadCommandKind Kind { get; }

    public abstract CadId TargetId { get; }
}

public sealed record AddFeatureCommand(CadFeatureDefinition Feature) : CadCommand
{
    public override CadCommandKind Kind => CadCommandKind.AddFeature;
    public override CadId TargetId => Feature.Id;
}

public sealed record RemoveFeatureCommand(CadId TargetId) : CadCommand
{
    public override CadCommandKind Kind => CadCommandKind.RemoveFeature;
}

public sealed record RenameFeatureCommand(CadId TargetId, string Name) : CadCommand
{
    public RenameFeatureCommand
    {
        if (string.IsNullOrWhiteSpace(Name))
            throw new ArgumentException("Feature name cannot be empty.", nameof(Name));
    }

    public override CadCommandKind Kind => CadCommandKind.RenameFeature;
}

public sealed record CadChangeSet(IReadOnlyList<CadCommand> Commands)
{
    public CadChangeSet
    {
        ArgumentNullException.ThrowIfNull(Commands);
    }
}

public interface ICadControlService
{
    CadCommandResult AddFeature(CadFeatureDefinition feature);
    CadCommandResult RemoveFeature(CadId featureId);
    CadCommandResult RenameFeature(CadId featureId, string name);
    CadCommandResult Commit();
    void Rollback();
}

public enum CadCommandStatus
{
    Accepted,
    Rejected,
    Failed
}

public sealed record CadCommandResult(
    CadCommandStatus Status,
    CadId? TargetId,
    string? Code,
    string? Message)
{
    public static CadCommandResult Accepted(CadId targetId) =>
        new(CadCommandStatus.Accepted, targetId, null, null);

    public static CadCommandResult Rejected(CadId? targetId, string code, string message) =>
        new(CadCommandStatus.Rejected, targetId, code, message);

    public static CadCommandResult Failed(CadId? targetId, string code, string message) =>
        new(CadCommandStatus.Failed, targetId, code, message);
}

public sealed class InMemoryCadControlService : ICadControlService, IEngineeringTransaction
{
    private readonly CadDocumentStore _store;
    private readonly List<CadCommand> _pending = [];
    private bool _completed;

    public InMemoryCadControlService(CadDocumentStore store)
    {
        _store = store ?? throw new ArgumentNullException(nameof(store));
    }

    public CadCommandResult AddFeature(CadFeatureDefinition feature)
    {
        EnsureOpen();
        ArgumentNullException.ThrowIfNull(feature);

        if (_store.Contains(feature.Id) || _pending.Any(x => x.TargetId == feature.Id))
            return CadCommandResult.Rejected(
                feature.Id,
                "CAD_FEATURE_ID_EXISTS",
                $"Feature '{feature.Id}' already exists.");

        _pending.Add(new AddFeatureCommand(feature));
        return CadCommandResult.Accepted(feature.Id);
    }

    public CadCommandResult RemoveFeature(CadId featureId)
    {
        EnsureOpen();

        if (!_store.Contains(featureId) && !_pending.Any(x => x is AddFeatureCommand add && add.Feature.Id == featureId))
            return CadCommandResult.Rejected(
                featureId,
                "CAD_FEATURE_NOT_FOUND",
                $"Feature '{featureId}' does not exist.");

        _pending.Add(new RemoveFeatureCommand(featureId));
        return CadCommandResult.Accepted(featureId);
    }

    public CadCommandResult RenameFeature(CadId featureId, string name)
    {
        EnsureOpen();

        if (string.IsNullOrWhiteSpace(name))
            return CadCommandResult.Rejected(
                featureId,
                "CAD_FEATURE_NAME_INVALID",
                "Feature name cannot be empty.");

        if (!_store.Contains(featureId) && !PendingAdds(featureId))
            return CadCommandResult.Rejected(
                featureId,
                "CAD_FEATURE_NOT_FOUND",
                $"Feature '{featureId}' does not exist.");

        _pending.Add(new RenameFeatureCommand(featureId, name));
        return CadCommandResult.Accepted(featureId);
    }

    public CadCommandResult Commit()
    {
        EnsureOpen();

        foreach (var command in _pending)
        {
            var result = _store.Apply(command);
            if (result.Status is not CadCommandStatus.Accepted)
            {
                Rollback();
                return result;
            }
        }

        _pending.Clear();
        _completed = true;
        return new(CadCommandStatus.Accepted, null, null, null);
    }

    public void Rollback()
    {
        if (_completed)
            return;

        _pending.Clear();
        _completed = true;
    }

    private bool PendingAdds(CadId id) =>
        _pending.Any(x => x is AddFeatureCommand add && add.Feature.Id == id);

    private void EnsureOpen()
    {
        if (_completed)
            throw new InvalidOperationException("The CAD transaction is already completed.");
    }
}

public interface IEngineeringTransaction
{
    CadCommandResult Commit();
    void Rollback();
}

public sealed class CadDocumentStore
{
    private ImmutableDictionary<CadId, CadFeatureDefinition> _features;

    public CadDocumentStore(CadDocumentDefinition initialDocument)
    {
        ArgumentNullException.ThrowIfNull(initialDocument);
        _features = initialDocument.Features.ToImmutableDictionary(x => x.Id);
        DocumentId = initialDocument.Id;
        Name = initialDocument.Name;
    }

    public CadId DocumentId { get; }

    public string Name { get; }

    public CadDocumentSnapshot Snapshot() =>
        new(DocumentId, Name, _features.Values.OrderBy(x => x.Id.Value, StringComparer.Ordinal).ToArray());

    public bool Contains(CadId id) => _features.ContainsKey(id);

    internal CadCommandResult Apply(CadCommand command) =>
        command switch
        {
            AddFeatureCommand add when _features.ContainsKey(add.Feature.Id) =>
                CadCommandResult.Rejected(
                    add.Feature.Id,
                    "CAD_FEATURE_ID_EXISTS",
                    $"Feature '{add.Feature.Id}' already exists."),

            AddFeatureCommand add =>
                ApplyAdd(add),

            RemoveFeatureCommand remove when !_features.ContainsKey(remove.TargetId) =>
                CadCommandResult.Rejected(
                    remove.TargetId,
                    "CAD_FEATURE_NOT_FOUND",
                    $"Feature '{remove.TargetId}' does not exist."),

            RemoveFeatureCommand remove =>
                ApplyRemove(remove),

            RenameFeatureCommand rename when !_features.ContainsKey(rename.TargetId) =>
                CadCommandResult.Rejected(
                    rename.TargetId,
                    "CAD_FEATURE_NOT_FOUND",
                    $"Feature '{rename.TargetId}' does not exist."),

            RenameFeatureCommand rename =>
                ApplyRename(rename),

            _ =>
                CadCommandResult.Failed(
                    command.TargetId,
                    "CAD_COMMAND_UNSUPPORTED",
                    $"Command '{command.Kind}' is unsupported.")
        };

    private CadCommandResult ApplyAdd(AddFeatureCommand command)
    {
        _features = _features.Add(command.Feature.Id, command.Feature);
        return CadCommandResult.Accepted(command.Feature.Id);
    }

    private CadCommandResult ApplyRemove(RemoveFeatureCommand command)
    {
        _features = _features.Remove(command.TargetId);
        return CadCommandResult.Accepted(command.TargetId);
    }

    private CadCommandResult ApplyRename(RenameFeatureCommand command)
    {
        var current = _features[command.TargetId];
        _features = _features.SetItem(
            command.TargetId,
            current with { Name = command.Name });
        return CadCommandResult.Accepted(command.TargetId);
    }
}
