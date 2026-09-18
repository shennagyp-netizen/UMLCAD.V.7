namespace UMLCAD.Cad.Contracts;

public sealed record SketchKernelCircle(
    string Id,
    double X,
    double Y,
    double Radius)
{
    public SketchKernelCircle
    {
        if (string.IsNullOrWhiteSpace(Id))
            throw new ArgumentException("Sketch circle ID is required.", nameof(Id));
        if (!double.IsFinite(X) || !double.IsFinite(Y) ||
            !double.IsFinite(Radius) || Radius <= 0d)
            throw new ArgumentException(
                "Sketch circle coordinates and radius must be finite; radius must be positive.",
                nameof(Radius));
    }
}

public sealed record SketchKernelFixedConstraint(
    string Id,
    string GeometryId)
{
    public SketchKernelFixedConstraint
    {
        if (string.IsNullOrWhiteSpace(Id))
            throw new ArgumentException("Sketch constraint ID is required.", nameof(Id));
        if (string.IsNullOrWhiteSpace(GeometryId))
            throw new ArgumentException("Sketch constraint geometry ID is required.", nameof(GeometryId));
    }
}

public sealed record KernelSolveOptions(
    int MaxIterations,
    double ResidualTolerance,
    double StepTolerance,
    double InitialDamping)
{
    public KernelSolveOptions
    {
        if (MaxIterations <= 0)
            throw new ArgumentOutOfRangeException(nameof(MaxIterations));
        if (!double.IsFinite(ResidualTolerance) || ResidualTolerance < 0d)
            throw new ArgumentOutOfRangeException(nameof(ResidualTolerance));
        if (!double.IsFinite(StepTolerance) || StepTolerance < 0d)
            throw new ArgumentOutOfRangeException(nameof(StepTolerance));
        if (!double.IsFinite(InitialDamping) || InitialDamping < 0d)
            throw new ArgumentOutOfRangeException(nameof(InitialDamping));
    }

    public static KernelSolveOptions Default { get; } =
        new(100, 1e-8, 1e-10, 1e-3);
}

public sealed record SketchSolveRequest(
    string OperationIdentity,
    IReadOnlyList<SketchKernelCircle> Circles,
    IReadOnlyList<SketchKernelFixedConstraint> FixedConstraints,
    KernelTolerance Tolerance,
    KernelSolveOptions Options)
{
    public const string ContractId = "UMLCAD.Geometry.SketchSolve";
    public const string ContractSchema = "uml-cad-sketch-solve/1.0.0";

    public SketchSolveRequest
    {
        if (string.IsNullOrWhiteSpace(OperationIdentity))
            throw new ArgumentException("OperationIdentity is required.", nameof(OperationIdentity));

        Circles = Circles?.OrderBy(x => x.Id, StringComparer.Ordinal).ToArray()
            ?? throw new ArgumentNullException(nameof(Circles));
        FixedConstraints = FixedConstraints?.OrderBy(x => x.Id, StringComparer.Ordinal).ToArray()
            ?? throw new ArgumentNullException(nameof(FixedConstraints));

        var ids = new HashSet<string>(StringComparer.Ordinal);
        foreach (var circle in Circles)
            if (!ids.Add(circle.Id))
                throw new ArgumentException($"Duplicate sketch circle '{circle.Id}'.", nameof(Circles));

        var constraintIds = new HashSet<string>(StringComparer.Ordinal);
        foreach (var constraint in FixedConstraints)
        {
            if (!constraintIds.Add(constraint.Id))
                throw new ArgumentException(
                    $"Duplicate sketch constraint '{constraint.Id}'.",
                    nameof(FixedConstraints));

            if (!ids.Contains(constraint.GeometryId))
                throw new ArgumentException(
                    $"Sketch constraint '{constraint.Id}' references missing circle '{constraint.GeometryId}'.",
                    nameof(FixedConstraints));
        }

        if (Circles.Count == 0)
            throw new ArgumentException("At least one sketch circle is required.", nameof(Circles));
    }
}

public sealed record SketchSolvedCircle(
    string Id,
    double X,
    double Y,
    double Radius);

public sealed record CadSketchEvaluationResult(
    CadId SketchId,
    CadResultId ResultId,
    string KernelContractVersion,
    CadFrame Frame,
    IReadOnlyList<SketchSolvedCircle> Circles,
    bool Converged,
    string Reason,
    int Iterations,
    double FinalResidualNorm,
    double FinalScaledResidualNorm,
    double FinalStepNorm,
    int DegreesOfFreedom,
    int VariableCount,
    int EquationCount,
    double ConditionEstimate,
    string EvidenceHash)
{
    public void Validate()
    {
        if (!SketchId.IsValid || !ResultId.IsValid)
            throw new ArgumentException("Sketch evaluation identities are required.");
        if (KernelContractVersion != CadContractVersions.KernelEvaluation)
            throw new ArgumentException("Unsupported sketch kernel contract version.");

        Frame.Validate();

        if (Circles is null || Circles.Count == 0)
            throw new ArgumentException("Sketch evaluation requires solved circle geometry.", nameof(Circles));

        var ids = new HashSet<string>(StringComparer.Ordinal);
        foreach (var circle in Circles)
        {
            if (!ids.Add(circle.Id))
                throw new ArgumentException($"Duplicate solved sketch circle '{circle.Id}'.");
            if (!double.IsFinite(circle.X) ||
                !double.IsFinite(circle.Y) ||
                !double.IsFinite(circle.Radius) ||
                circle.Radius <= 0d)
                throw new ArgumentException($"Solved sketch circle '{circle.Id}' is invalid.");
        }

        if (!Converged ||
            string.IsNullOrWhiteSpace(Reason) ||
            Iterations < 0 ||
            DegreesOfFreedom < 0 ||
            VariableCount < 0 ||
            EquationCount < 0 ||
            !double.IsFinite(FinalResidualNorm) ||
            !double.IsFinite(FinalScaledResidualNorm) ||
            !double.IsFinite(FinalStepNorm) ||
            !double.IsFinite(ConditionEstimate) ||
            string.IsNullOrWhiteSpace(EvidenceHash))
        {
            throw new ArgumentException("Sketch evaluation evidence is incomplete or invalid.");
        }
    }
}

public sealed record SketchSolveKernelResult(
    GeometryKernelStatus Status,
    bool Succeeded,
    string Reason,
    int Iterations,
    double InitialResidualNorm,
    double FinalResidualNorm,
    double InitialScaledResidualNorm,
    double FinalScaledResidualNorm,
    double FinalStepNorm,
    int VariableCount,
    int EquationCount,
    int Rank,
    int DegreesOfFreedom,
    double ConditionEstimate,
    IReadOnlyList<SketchSolvedCircle> Geometry,
    IReadOnlyList<string> Diagnostics)
{
    public SketchSolveKernelResult
    {
        Geometry = Geometry?.ToArray()
            ?? throw new ArgumentNullException(nameof(Geometry));
        Diagnostics = Diagnostics?.ToArray()
            ?? throw new ArgumentNullException(nameof(Diagnostics));

        if (!Enum.IsDefined(Status) || string.IsNullOrWhiteSpace(Reason))
            throw new ArgumentException("Sketch solver result status/reason is invalid.");

        var expectedSuccess = Status == GeometryKernelStatus.Succeeded;
        if (Succeeded != expectedSuccess)
            throw new ArgumentException(
                "Sketch solver status is inconsistent with succeeded.");

        if (Iterations < 0 || VariableCount < 0 || EquationCount < 0 ||
            Rank < 0 || DegreesOfFreedom < 0 ||
            !double.IsFinite(InitialResidualNorm) ||
            !double.IsFinite(FinalResidualNorm) ||
            !double.IsFinite(InitialScaledResidualNorm) ||
            !double.IsFinite(FinalScaledResidualNorm) ||
            !double.IsFinite(FinalStepNorm) ||
            !double.IsFinite(ConditionEstimate))
            throw new ArgumentException("Sketch solver diagnostics are invalid.");
    }
}

public interface ISketchConstraintService
{
    Task<SketchSolveKernelResult> SolveAsync(
        SketchSolveRequest request,
        CancellationToken cancellationToken = default);
}
