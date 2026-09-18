namespace UMLCAD.Cad.Contracts;

public sealed record SketchSolveRequest(
    string OperationIdentity,
    CadId SketchId,
    CadFrame Frame,
    IReadOnlyList<SketchCircle> Circles,
    IReadOnlyList<SketchConstraintSpecification> Constraints,
    KernelTolerance Tolerance,
    ContractVersion ContractVersion)
{
    public const string ContractId = "UMLCAD.Geometry.SketchSolve";
    public const string ContractSchema = "uml-cad-sketch-solve/1.0.0";

    public SketchSolveRequest
    {
        if (string.IsNullOrWhiteSpace(OperationIdentity))
            throw new ArgumentException("Operation identity is required.", nameof(OperationIdentity));
        if (!SketchId.IsValid)
            throw new ArgumentException("Sketch identity is required.", nameof(SketchId));
        ArgumentNullException.ThrowIfNull(Frame);
        Frame.Validate();

        Circles = Circles?.ToArray() ??
            throw new ArgumentNullException(nameof(Circles));
        Constraints = Constraints?.ToArray() ??
            throw new ArgumentNullException(nameof(Constraints));

        if (Circles.Count == 0)
            throw new ArgumentException("At least one sketch circle is required.", nameof(Circles));

        var circleIds = new HashSet<CadId>();
        foreach (var circle in Circles)
        {
            circle.Validate();
            if (!circleIds.Add(circle.Id))
                throw new ArgumentException($"Duplicate sketch circle '{circle.Id}'.", nameof(Circles));
        }

        var constraintIds = new HashSet<CadId>();
        foreach (var constraint in Constraints)
        {
            constraint.Validate();
            if (!constraintIds.Add(constraint.Id))
                throw new ArgumentException($"Duplicate sketch constraint '{constraint.Id}'.", nameof(Constraints));
            if (!circleIds.Contains(constraint.GeometryId))
                throw new ArgumentException(
                    $"Constraint '{constraint.Id}' references missing circle '{constraint.GeometryId}'.",
                    nameof(Constraints));

            if (constraint.Kind != SketchConstraintKind.Fixed)
                throw new ArgumentException(
                    $"Sketch constraint kind '{constraint.Kind}' is not yet certified by the explicit sketch solver contract.",
                    nameof(Constraints));
        }

        if (ContractVersion.Value != "1.0")
            throw new ArgumentException("Unsupported sketch solver contract version.", nameof(ContractVersion));
    }
}

public sealed record SketchCircleResult(
    string Id,
    double X,
    double Y,
    double Radius)
{
    public SketchCircleResult
    {
        if (string.IsNullOrWhiteSpace(Id))
            throw new ArgumentException("Circle result identity is required.", nameof(Id));
        if (!double.IsFinite(X) || !double.IsFinite(Y) || !double.IsFinite(Radius) || Radius <= 0d)
            throw new ArgumentException("Sketch circle result is invalid.", nameof(Radius));
    }
}

public sealed record SketchKernelResult(
    GeometryKernelStatus Status,
    ContractResultId? ResultId,
    string? EvidenceHash,
    IReadOnlyList<SketchCircleResult> Circles,
    bool? Converged,
    string? Reason,
    int? Iterations,
    double? FinalResidualNorm,
    double? FinalScaledResidualNorm,
    double? FinalStepNorm,
    int? DegreesOfFreedom,
    int? VariableCount,
    int? EquationCount,
    IReadOnlyList<string> Diagnostics)
{
    public SketchKernelResult
    {
        Circles = Circles?.ToArray() ??
            throw new ArgumentNullException(nameof(Circles));
        Diagnostics = Diagnostics?.ToArray() ??
            throw new ArgumentNullException(nameof(Diagnostics));

        if (Status == GeometryKernelStatus.Succeeded)
        {
            if (ResultId is null || string.IsNullOrWhiteSpace(EvidenceHash))
                throw new ArgumentException("Successful sketch solve requires result and evidence identities.");
            if (Converged != true)
                throw new ArgumentException("Successful sketch solve must be converged.");
            if (Circles.Count == 0)
                throw new ArgumentException("Successful sketch solve requires solved circles.");
            if (Iterations is null || Iterations < 0 ||
                FinalResidualNorm is null || !double.IsFinite(FinalResidualNorm.Value) ||
                FinalScaledResidualNorm is null || !double.IsFinite(FinalScaledResidualNorm.Value) ||
                FinalStepNorm is null || !double.IsFinite(FinalStepNorm.Value) ||
                DegreesOfFreedom is null || DegreesOfFreedom < 0 ||
                VariableCount is null || VariableCount < 0 ||
                EquationCount is null || EquationCount < 0)
            {
                throw new ArgumentException("Successful sketch solve is missing authoritative solver evidence.");
            }
        }
    }
}

public sealed record CadSketchEvaluationResult(
    CadId SketchId,
    CadResultId ResultId,
    string KernelContractVersion,
    CadFrame Frame,
    IReadOnlyList<SketchCircleResult> Circles,
    bool Converged,
    int Iterations,
    double FinalResidualNorm,
    double FinalScaledResidualNorm,
    double FinalStepNorm,
    int DegreesOfFreedom,
    int VariableCount,
    int EquationCount,
    string EvidenceHash)
{
    public CadSketchEvaluationResult
    {
        Circles = Circles?.ToArray() ??
            throw new ArgumentNullException(nameof(Circles));
    }

    public void Validate()
    {
        if (!SketchId.IsValid || !ResultId.IsValid)
            throw new ArgumentException("Sketch evaluation identities are required.");
        if (KernelContractVersion != CadContractVersions.KernelEvaluation)
            throw new ArgumentException("Unsupported sketch kernel contract version.");
        Frame.Validate();

        if (Circles.Count == 0 || Circles.Any(x => x is null))
            throw new ArgumentException("Sketch evaluation requires solved circles.", nameof(Circles));
        if (!Circles.Select(x => x.Id).Distinct(StringComparer.Ordinal).SequenceEqual(
                Circles.Select(x => x.Id),
                StringComparer.Ordinal))
            throw new ArgumentException("Solved circle identities must be unique.");

        if (Iterations < 0 || DegreesOfFreedom < 0 || VariableCount < 0 || EquationCount < 0 ||
            !double.IsFinite(FinalResidualNorm) ||
            !double.IsFinite(FinalScaledResidualNorm) ||
            !double.IsFinite(FinalStepNorm) ||
            string.IsNullOrWhiteSpace(EvidenceHash))
        {
            throw new ArgumentException("Sketch evaluation evidence is invalid.");
        }

        if (!Converged)
            throw new ArgumentException("Authoritative sketch result must be converged.");
    }
}

public interface ISketchGeometryService
{
    Task<SketchKernelResult> SolveAsync(
        SketchSolveRequest request,
        CancellationToken cancellationToken = default);
}
