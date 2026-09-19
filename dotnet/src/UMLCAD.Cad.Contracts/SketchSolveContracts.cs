namespace UMLCAD.Cad.Contracts;

public sealed record SketchKernelCircle
{
    public string Id { get; }
    public double X { get; }
    public double Y { get; }
    public double Radius { get; }

    public SketchKernelCircle(string id, double x, double y, double radius)
    {
        if (string.IsNullOrWhiteSpace(id))
            throw new ArgumentException(
                "Sketch circle ID is required.",
                nameof(id));

        if (!double.IsFinite(x) ||
            !double.IsFinite(y) ||
            !double.IsFinite(radius) ||
            radius <= 0d)
            throw new ArgumentException(
                "Sketch circle coordinates and radius must be finite; radius must be positive.",
                nameof(radius));

        Id = id;
        X = x;
        Y = y;
        Radius = radius;
    }
}

public sealed record SketchKernelFixedConstraint
{
    public string Id { get; }
    public string GeometryId { get; }

    public SketchKernelFixedConstraint(string id, string geometryId)
    {
        if (string.IsNullOrWhiteSpace(id))
            throw new ArgumentException(
                "Sketch constraint ID is required.",
                nameof(id));
        if (string.IsNullOrWhiteSpace(geometryId))
            throw new ArgumentException(
                "Sketch constraint geometry ID is required.",
                nameof(geometryId));

        Id = id;
        GeometryId = geometryId;
    }
}

public sealed record KernelSolveOptions
{
    public int MaxIterations { get; }
    public double ResidualTolerance { get; }
    public double StepTolerance { get; }
    public double InitialDamping { get; }

    public KernelSolveOptions(
        int maxIterations,
        double residualTolerance,
        double stepTolerance,
        double initialDamping)
    {
        if (maxIterations <= 0)
            throw new ArgumentOutOfRangeException(nameof(maxIterations));

        if (!double.IsFinite(residualTolerance) || residualTolerance < 0d)
            throw new ArgumentOutOfRangeException(nameof(residualTolerance));

        if (!double.IsFinite(stepTolerance) || stepTolerance < 0d)
            throw new ArgumentOutOfRangeException(nameof(stepTolerance));

        if (!double.IsFinite(initialDamping) || initialDamping < 0d)
            throw new ArgumentOutOfRangeException(nameof(initialDamping));

        MaxIterations = maxIterations;
        ResidualTolerance = residualTolerance;
        StepTolerance = stepTolerance;
        InitialDamping = initialDamping;
    }

    public static KernelSolveOptions Default { get; } =
        new(100, 1e-8, 1e-10, 1e-3);
}

public sealed record SketchSolveRequest
{
    public string OperationIdentity { get; }
    public IReadOnlyList<SketchKernelCircle> Circles { get; }
    public IReadOnlyList<SketchKernelFixedConstraint> FixedConstraints { get; }
    public KernelTolerance Tolerance { get; }
    public KernelSolveOptions Options { get; }

    public const string ContractId = "UMLCAD.Geometry.SketchSolve";
    public const string ContractSchema = "uml-cad-sketch-solve/1.0.0";

    public SketchSolveRequest(
        string operationIdentity,
        IReadOnlyList<SketchKernelCircle> circles,
        IReadOnlyList<SketchKernelFixedConstraint> fixedConstraints,
        KernelTolerance tolerance,
        KernelSolveOptions options)
    {
        if (string.IsNullOrWhiteSpace(operationIdentity))
            throw new ArgumentException(
                "OperationIdentity is required.",
                nameof(operationIdentity));

        Circles = circles?.OrderBy(x => x.Id, StringComparer.Ordinal).ToArray()
            ?? throw new ArgumentNullException(nameof(circles));
        FixedConstraints = fixedConstraints?.OrderBy(x => x.Id, StringComparer.Ordinal).ToArray()
            ?? throw new ArgumentNullException(nameof(fixedConstraints));

        var ids = new HashSet<string>(StringComparer.Ordinal);
        foreach (var circle in Circles)
        {
            if (!ids.Add(circle.Id))
                throw new ArgumentException(
                    $"Duplicate sketch circle '{circle.Id}'.",
                    nameof(circles));
        }

        var constraintIds = new HashSet<string>(StringComparer.Ordinal);
        foreach (var constraint in FixedConstraints)
        {
            if (!constraintIds.Add(constraint.Id))
                throw new ArgumentException(
                    $"Duplicate sketch constraint '{constraint.Id}'.",
                    nameof(fixedConstraints));

            if (!ids.Contains(constraint.GeometryId))
                throw new ArgumentException(
                    $"Sketch constraint '{constraint.Id}' references missing circle '{constraint.GeometryId}'.",
                    nameof(fixedConstraints));
        }

        if (Circles.Count == 0)
            throw new ArgumentException(
                "At least one sketch circle is required.",
                nameof(circles));

        OperationIdentity = operationIdentity;
        Tolerance = tolerance;
        Options = options;
    }
}

public sealed record SketchSolvedCircle(
    string Id,
    double X,
    double Y,
    double Radius);

public sealed record SketchSolveKernelResult
{
    public GeometryKernelStatus Status { get; }
    public bool Succeeded { get; }
    public string Reason { get; }
    public int Iterations { get; }
    public double InitialResidualNorm { get; }
    public double FinalResidualNorm { get; }
    public double InitialScaledResidualNorm { get; }
    public double FinalScaledResidualNorm { get; }
    public double FinalStepNorm { get; }
    public int VariableCount { get; }
    public int EquationCount { get; }
    public int Rank { get; }
    public int DegreesOfFreedom { get; }
    public double ConditionEstimate { get; }
    public IReadOnlyList<SketchSolvedCircle> Geometry { get; }
    public IReadOnlyList<string> Diagnostics { get; }

    public SketchSolveKernelResult(
        GeometryKernelStatus status,
        bool succeeded,
        string reason,
        int iterations,
        double initialResidualNorm,
        double finalResidualNorm,
        double initialScaledResidualNorm,
        double finalScaledResidualNorm,
        double finalStepNorm,
        int variableCount,
        int equationCount,
        int rank,
        int degreesOfFreedom,
        double conditionEstimate,
        IReadOnlyList<SketchSolvedCircle> geometry,
        IReadOnlyList<string> diagnostics)
    {
        Geometry = geometry?.ToArray()
            ?? throw new ArgumentNullException(nameof(geometry));
        Diagnostics = diagnostics?.ToArray()
            ?? throw new ArgumentNullException(nameof(diagnostics));

        if (!Enum.IsDefined(status) ||
            string.IsNullOrWhiteSpace(reason))
            throw new ArgumentException(
                "Sketch solver result status/reason is invalid.");

        var expectedSuccess = status == GeometryKernelStatus.Succeeded;
        if (succeeded != expectedSuccess)
            throw new ArgumentException(
                "Sketch solver status is inconsistent with succeeded.");

        if (iterations < 0 ||
            variableCount < 0 ||
            equationCount < 0 ||
            rank < 0 ||
            degreesOfFreedom < 0 ||
            !double.IsFinite(initialResidualNorm) ||
            !double.IsFinite(finalResidualNorm) ||
            !double.IsFinite(initialScaledResidualNorm) ||
            !double.IsFinite(finalScaledResidualNorm) ||
            !double.IsFinite(finalStepNorm) ||
            !double.IsFinite(conditionEstimate))
            throw new ArgumentException(
                "Sketch solver diagnostics are invalid.");

        Status = status;
        Succeeded = succeeded;
        Reason = reason;
        Iterations = iterations;
        InitialResidualNorm = initialResidualNorm;
        FinalResidualNorm = finalResidualNorm;
        InitialScaledResidualNorm = initialScaledResidualNorm;
        FinalScaledResidualNorm = finalScaledResidualNorm;
        FinalStepNorm = finalStepNorm;
        VariableCount = variableCount;
        EquationCount = equationCount;
        Rank = rank;
        DegreesOfFreedom = degreesOfFreedom;
        ConditionEstimate = conditionEstimate;
    }
}

public interface ISketchConstraintService
{
    Task<SketchSolveKernelResult> SolveAsync(
        SketchSolveRequest request,
        CancellationToken cancellationToken = default);
}
