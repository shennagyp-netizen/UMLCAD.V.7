namespace UMLCAD.Framework.Semantics;

public interface ISemanticFrameService
{
    SemanticFrameResolution Resolve(
        SemanticApplication application,
        string frameId);
}

internal sealed class SemanticFrameService : ISemanticFrameService
{
    public SemanticFrameResolution Resolve(
        SemanticApplication application,
        string frameId)
    {
        ArgumentNullException.ThrowIfNull(application);

        if (string.IsNullOrWhiteSpace(frameId))
            return Failure(
                frameId,
                SemanticReferenceStatus.Indeterminate,
                "FRAME_INVALID",
                "Frame identity cannot be empty.");

        var validation = ValidateGraph(application.Frames);
        if (validation is not null)
            return Failure(frameId, SemanticReferenceStatus.Indeterminate, validation.Value.Code, validation.Value.Message);

        var candidates = application.Frames
            .Where(x => string.Equals(x.Id, frameId, StringComparison.Ordinal))
            .OrderBy(x => x.Id, StringComparer.Ordinal)
            .ToArray();

        if (candidates.Length == 1)
            return new SemanticFrameResolution(
                frameId,
                SemanticReferenceStatus.Resolved,
                candidates,
                "FRAME_RESOLVED",
                $"Frame '{frameId}' resolved.");

        if (candidates.Length > 1)
            return new SemanticFrameResolution(
                frameId,
                SemanticReferenceStatus.Ambiguous,
                candidates,
                "FRAME_AMBIGUOUS",
                $"Frame '{frameId}' is ambiguous.");

        return Failure(
            frameId,
            SemanticReferenceStatus.Missing,
            "FRAME_MISSING",
            $"Frame '{frameId}' does not exist.");
    }

    private static (string Code, string Message)? ValidateGraph(
        IReadOnlyList<SemanticFrame> frames)
    {
        var duplicate = frames
            .GroupBy(x => x.Id, StringComparer.Ordinal)
            .FirstOrDefault(x => x.Count() > 1);
        if (duplicate is not null)
            return (
                "FRAME_AMBIGUOUS",
                $"Frame identity '{duplicate.Key}' is duplicated.");

        var worldFrames = frames
            .Where(x => x.Kind == SemanticFrameKind.World)
            .ToArray();

        if (worldFrames.Length > 1)
            return (
                "FRAME_WORLD_AMBIGUOUS",
                "A semantic application may contain at most one World frame.");

        var byId = frames.ToDictionary(x => x.Id, StringComparer.Ordinal);

        foreach (var frame in frames)
        {
            if (frame.Kind == SemanticFrameKind.World)
            {
                if (frame.ParentId is not null)
                    return (
                        "FRAME_WORLD_PARENT",
                        $"World frame '{frame.Id}' cannot have a parent frame.");

                if (!IsIdentity(frame.TransformToParent))
                    return (
                        "FRAME_WORLD_TRANSFORM",
                        $"World frame '{frame.Id}' must use the identity transform.");

                continue;
            }

            if (string.IsNullOrWhiteSpace(frame.ParentId) ||
                !byId.ContainsKey(frame.ParentId))
            {
                return (
                    "FRAME_PARENT_MISSING",
                    $"Frame '{frame.Id}' requires an existing parent frame.");
            }
        }

        var state = new Dictionary<string, VisitState>(StringComparer.Ordinal);
        foreach (var frame in frames.OrderBy(x => x.Id, StringComparer.Ordinal))
        {
            var cycle = Visit(frame.Id, byId, state);
            if (cycle is not null)
                return ("FRAME_CYCLE", $"Frame hierarchy contains a cycle at '{cycle}'.");
        }

        return null;
    }

    private static string? Visit(
        string frameId,
        IReadOnlyDictionary<string, SemanticFrame> frames,
        IDictionary<string, VisitState> state)
    {
        if (state.TryGetValue(frameId, out var current))
        {
            if (current == VisitState.Active)
                return frameId;
            if (current == VisitState.Done)
                return null;
        }

        state[frameId] = VisitState.Active;
        var frame = frames[frameId];
        if (frame.ParentId is not null)
        {
            var cycle = Visit(frame.ParentId, frames, state);
            if (cycle is not null)
                return cycle;
        }

        state[frameId] = VisitState.Done;
        return null;
    }

    private static bool IsIdentity(TransformSemantic transform) =>
        transform.Matrix.SequenceEqual(TransformSemantic.Identity.Matrix);

    private static SemanticFrameResolution Failure(
        string frameId,
        SemanticReferenceStatus status,
        string code,
        string message) =>
        new(
            frameId,
            status,
            Array.Empty<SemanticFrame>(),
            code,
            message);

    private enum VisitState
    {
        Active,
        Done
    }
}
