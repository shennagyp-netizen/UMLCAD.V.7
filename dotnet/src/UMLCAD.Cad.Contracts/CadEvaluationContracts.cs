namespace UMLCAD.Cad.Contracts;

public static class CadContractVersions
{
    public const string KernelEvaluation = "uml-cad-kernel-evaluation/1.0.0";
    public const string Evaluation = "uml-cad-evaluation/1.0.0";
    public const string Representation = "uml-cad-representation/1.0.0";
}

public readonly record struct CadId(string Value)
{
    public bool IsValid => !string.IsNullOrWhiteSpace(Value);
}

public readonly record struct CadResultId(string Value)
{
    public bool IsValid => !string.IsNullOrWhiteSpace(Value);
}

public readonly record struct TopologyEntityId(string Value)
{
    public bool IsValid => !string.IsNullOrWhiteSpace(Value);
}

public enum CadFrameKind { World, Document, Part, Body, Sketch, Face, Occurrence, DrawingView, Simulation }
public enum ReferenceKind { Geometric, Topology, Support, Publication }
public enum ReferenceResolutionStatus { Resolved, Missing, Ambiguous, Indeterminate, Unsupported }
public enum TopologyEntityKind { Solid, Shell, Face, Edge, Vertex }
public enum FeatureBooleanOperation { Add, Remove }
public enum SketchConstraintKind { Fixed, FullyConstrained }
public enum CadEvaluationStatus { Succeeded, InvalidSpecification, MissingReference, AmbiguousReference, AmbiguousEvaluation, Indeterminate, Unsupported, KernelFailure }

public sealed record CadFrame(CadId Id, CadFrameKind Kind, double OriginX, double OriginY, double OriginZ)
{
    public CadVector3 XAxis { get; init; } = new(1d, 0d, 0d);
    public CadVector3 YAxis { get; init; } = new(0d, 1d, 0d);
    public CadVector3 ZAxis { get; init; } = new(0d, 0d, 1d);

    public void Validate()
    {
        Require(Id.IsValid, "Frame ID is required.");
        Require(double.IsFinite(OriginX) && double.IsFinite(OriginY) && double.IsFinite(OriginZ), "Frame origin must be finite.");
        ValidateBasis(XAxis, YAxis, ZAxis);
    }

    private static void ValidateBasis(
        CadVector3 xAxis,
        CadVector3 yAxis,
        CadVector3 zAxis)
    {
        Require(xAxis.IsFinite && yAxis.IsFinite && zAxis.IsFinite, "Frame basis must be finite.");

        const double tolerance = 1e-12;
        Require(Math.Abs(Dot(xAxis, xAxis) - 1d) <= tolerance, "Frame X axis must be unit length.");
        Require(Math.Abs(Dot(yAxis, yAxis) - 1d) <= tolerance, "Frame Y axis must be unit length.");
        Require(Math.Abs(Dot(zAxis, zAxis) - 1d) <= tolerance, "Frame Z axis must be unit length.");
        Require(Math.Abs(Dot(xAxis, yAxis)) <= tolerance, "Frame X/Y axes must be orthogonal.");
        Require(Math.Abs(Dot(xAxis, zAxis)) <= tolerance, "Frame X/Z axes must be orthogonal.");
        Require(Math.Abs(Dot(yAxis, zAxis)) <= tolerance, "Frame Y/Z axes must be orthogonal.");

        var cross = Cross(xAxis, yAxis);
        Require(
            NearlyEqual(cross.X, zAxis.X, tolerance) &&
            NearlyEqual(cross.Y, zAxis.Y, tolerance) &&
            NearlyEqual(cross.Z, zAxis.Z, tolerance),
            "Frame basis must be right-handed.");
    }

    public CadVector3 ToWorldPoint(CadVector3 localPoint)
    {
        if (!localPoint.IsFinite)
            throw new ArgumentException("Local point must be finite.", nameof(localPoint));

        var vector = ToWorldVector(localPoint);
        return new CadVector3(
            OriginX + vector.X,
            OriginY + vector.Y,
            OriginZ + vector.Z);
    }

    public CadVector3 ToWorldVector(CadVector3 localVector)
    {
        if (!localVector.IsFinite)
            throw new ArgumentException("Local vector must be finite.", nameof(localVector));

        return new CadVector3(
            (XAxis.X * localVector.X) + (YAxis.X * localVector.Y) + (ZAxis.X * localVector.Z),
            (XAxis.Y * localVector.X) + (YAxis.Y * localVector.Y) + (ZAxis.Y * localVector.Z),
            (XAxis.Z * localVector.X) + (YAxis.Z * localVector.Y) + (ZAxis.Z * localVector.Z));
    }

    public CadVector3 ToLocalVector(CadVector3 worldVector)
    {
        if (!worldVector.IsFinite)
            throw new ArgumentException("World vector must be finite.", nameof(worldVector));

        return new CadVector3(
            Dot(worldVector, XAxis),
            Dot(worldVector, YAxis),
            Dot(worldVector, ZAxis));
    }

    public CadVector3 ToLocalPoint(CadVector3 worldPoint)
    {
        if (!worldPoint.IsFinite)
            throw new ArgumentException("World point must be finite.", nameof(worldPoint));

        return ToLocalVector(new CadVector3(
            worldPoint.X - OriginX,
            worldPoint.Y - OriginY,
            worldPoint.Z - OriginZ));
    }

    private static double Dot(CadVector3 left, CadVector3 right) =>
        (left.X * right.X) + (left.Y * right.Y) + (left.Z * right.Z);

    private static CadVector3 Cross(CadVector3 left, CadVector3 right) =>
        new(
            (left.Y * right.Z) - (left.Z * right.Y),
            (left.Z * right.X) - (left.X * right.Z),
            (left.X * right.Y) - (left.Y * right.X));

    private static bool NearlyEqual(double left, double right, double tolerance) =>
        Math.Abs(left - right) <= tolerance;

    internal static void Require(bool condition, string message)
    {
        if (!condition) throw new ArgumentException(message);
    }
}

public readonly record struct CadVector3(double X, double Y, double Z)
{
    public bool IsFinite => double.IsFinite(X) && double.IsFinite(Y) && double.IsFinite(Z);
    public double Length => Math.Sqrt((X * X) + (Y * Y) + (Z * Z));

    public CadVector3 Normalize(string name)
    {
        if (!IsFinite || !double.IsFinite(Length) || Length <= 0d)
            throw new ArgumentException($"{name} must be finite and non-zero.");
        return new CadVector3(X / Length, Y / Length, Z / Length);
    }
}

public readonly record struct CadBoundingBox3(double MinX, double MinY, double MinZ, double MaxX, double MaxY, double MaxZ)
{
    public void Validate()
    {
        if (!double.IsFinite(MinX) || !double.IsFinite(MinY) || !double.IsFinite(MinZ) ||
            !double.IsFinite(MaxX) || !double.IsFinite(MaxY) || !double.IsFinite(MaxZ) ||
            MinX > MaxX || MinY > MaxY || MinZ > MaxZ)
            throw new ArgumentException("Invalid bounding box.");
    }
}

public sealed record ReferenceContext(
    CadFrameKind FrameKind,
    string Configuration,
    CadResultId? ExpectedResultId = null,
    string? OccurrencePath = null)
{
    public void Validate()
    {
        if (string.IsNullOrWhiteSpace(Configuration))
            throw new ArgumentException("Reference configuration is required.");
        if (OccurrencePath is not null && string.IsNullOrWhiteSpace(OccurrencePath))
            throw new ArgumentException("Occurrence path cannot be empty.");
    }
}

public sealed record TopologySelector(
    TopologyEntityKind EntityKind,
    string SelectorKind,
    IReadOnlyDictionary<string, string> Parameters)
{
    public void Validate()
    {
        if (string.IsNullOrWhiteSpace(SelectorKind))
            throw new ArgumentException("Topology selector kind is required.");
        if (Parameters.Any(x => string.IsNullOrWhiteSpace(x.Key) || x.Value is null))
            throw new ArgumentException("Topology selector parameters are invalid.");
    }

    public static TopologySelector PlanarFaceByNormalAndPoint(CadVector3 normal, CadVector3 point)
    {
        normal = normal.Normalize("Face normal");
        if (!point.IsFinite) throw new ArgumentException("Face selection point must be finite.");
        return new TopologySelector(
            TopologyEntityKind.Face,
            "planar-face",
            new Dictionary<string, string>(StringComparer.Ordinal)
            {
                ["normalX"] = normal.X.ToString("R", System.Globalization.CultureInfo.InvariantCulture),
                ["normalY"] = normal.Y.ToString("R", System.Globalization.CultureInfo.InvariantCulture),
                ["normalZ"] = normal.Z.ToString("R", System.Globalization.CultureInfo.InvariantCulture),
                ["pointX"] = point.X.ToString("R", System.Globalization.CultureInfo.InvariantCulture),
                ["pointY"] = point.Y.ToString("R", System.Globalization.CultureInfo.InvariantCulture),
                ["pointZ"] = point.Z.ToString("R", System.Globalization.CultureInfo.InvariantCulture)
            });
    }
}

public sealed record CadReference(
    CadId ReferenceId,
    ReferenceKind Kind,
    CadId TargetSpecificationId,
    TopologySelector Selector,
    ReferenceContext Context)
{
    public void Validate()
    {
        if (!ReferenceId.IsValid || !TargetSpecificationId.IsValid)
            throw new ArgumentException("Reference identities are required.");
        Selector.Validate();
        Context.Validate();
    }
}

public sealed record ReferenceResolution(
    CadReference Reference,
    ReferenceResolutionStatus Status,
    IReadOnlyList<TopologyEntityId> Candidates,
    string? DiagnosticCode,
    string? Evidence)
{
    public bool IsResolved => Status == ReferenceResolutionStatus.Resolved && Candidates.Count == 1;
}

public abstract record CadFeatureSpecification(CadId Id)
{
    public virtual void Validate()
    {
        if (!Id.IsValid) throw new ArgumentException("Feature ID is required.");
    }
}

public sealed record BoxFeatureSpecification(CadId Id, CadFrame Frame, double Width, double Depth, double Height)
    : CadFeatureSpecification(Id)
{
    public override void Validate()
    {
        base.Validate();
        Frame.Validate();
        if (!double.IsFinite(Width) || !double.IsFinite(Depth) || !double.IsFinite(Height) ||
            Width <= 0d || Depth <= 0d || Height <= 0d)
            throw new ArgumentException("Box dimensions must be finite and positive.");
    }
}

public sealed record SketchCircle(CadId Id, double X, double Y, double Radius)
{
    public void Validate()
    {
        if (!Id.IsValid || !double.IsFinite(X) || !double.IsFinite(Y) ||
            !double.IsFinite(Radius) || Radius <= 0d)
            throw new ArgumentException("Sketch circle is invalid.");
    }
}

public sealed record SketchConstraintSpecification(CadId Id, SketchConstraintKind Kind, CadId GeometryId)
{
    public void Validate()
    {
        if (!Id.IsValid || !GeometryId.IsValid)
            throw new ArgumentException("Sketch constraint identities are required.");
    }
}

public sealed record SketchFeatureSpecification(
    CadId Id,
    CadReference Support,
    CadFrame Frame,
    IReadOnlyList<SketchCircle> Circles,
    IReadOnlyList<SketchConstraintSpecification> Constraints)
    : CadFeatureSpecification(Id)
{
    public override void Validate()
    {
        base.Validate();
        Support.Validate();
        Frame.Validate();
        if (Circles.Count == 0) throw new ArgumentException("The S1 sketch must contain a circular profile.");

        var circleIds = new HashSet<CadId>();
        foreach (var circle in Circles)
        {
            circle.Validate();
            if (!circleIds.Add(circle.Id)) throw new ArgumentException($"Duplicate circle '{circle.Id}'.");
        }

        var constraintIds = new HashSet<CadId>();
        foreach (var constraint in Constraints)
        {
            constraint.Validate();
            if (!constraintIds.Add(constraint.Id)) throw new ArgumentException($"Duplicate constraint '{constraint.Id}'.");
            if (!circleIds.Contains(constraint.GeometryId))
                throw new ArgumentException($"Constraint '{constraint.Id}' references missing geometry '{constraint.GeometryId}'.");
        }
    }
}

public sealed record ExtrusionFeatureSpecification(
    CadId Id,
    CadId ProfileSketchId,
    CadId ProfileGeometryId,
    CadReference Support,
    CadVector3 Direction,
    double Distance,
    FeatureBooleanOperation Operation)
    : CadFeatureSpecification(Id)
{
    public override void Validate()
    {
        base.Validate();
        if (!ProfileSketchId.IsValid) throw new ArgumentException("Extrusion profile sketch ID is required.");
        if (!ProfileGeometryId.IsValid) throw new ArgumentException("Extrusion profile geometry ID is required.");
        Support.Validate();
        Direction.Normalize("Extrusion direction");
        if (!double.IsFinite(Distance) || Distance <= 0d)
            throw new ArgumentException("Extrusion distance must be finite and positive.");
    }
}

public sealed record CadDocumentSpecification(
    CadId DocumentId,
    string Revision,
    string Configuration,
    IReadOnlyList<CadFeatureSpecification> Features)
{
    public KernelTolerance EvaluationTolerance { get; init; } = new(1e-9, 1e-9);

    public void Validate()
    {
        if (!DocumentId.IsValid) throw new ArgumentException("Document ID is required.");
        if (string.IsNullOrWhiteSpace(Revision) || string.IsNullOrWhiteSpace(Configuration))
            throw new ArgumentException("Document revision and configuration are required.");

        var ids = new HashSet<CadId>();
        foreach (var feature in Features)
        {
            feature.Validate();
            if (!ids.Add(feature.Id)) throw new ArgumentException($"Duplicate feature '{feature.Id}'.");
        }
    }
}

public sealed record CadDiagnostic(string Code, CadEvaluationStatus Status, string Message);

public sealed record TopologyProvenance(
    CadId ProducingFeatureId,
    CadResultId SourceResultId,
    string EvolutionKind,
    IReadOnlyList<TopologyEntityId> SourceEntities)
{
    public void Validate()
    {
        if (!ProducingFeatureId.IsValid || !SourceResultId.IsValid || string.IsNullOrWhiteSpace(EvolutionKind))
            throw new ArgumentException("Topology provenance is incomplete.");
    }
}

public sealed record TopologyEntityResult(
    TopologyEntityId Id,
    TopologyEntityKind Kind,
    CadVector3? Normal,
    CadVector3? Point,
    double? Measure,
    int UseCount,
    TopologyProvenance Provenance)
{
    public void Validate()
    {
        if (!Id.IsValid || UseCount < 0) throw new ArgumentException("Topology entity is invalid.");
        if (Normal is { } n && !n.IsFinite) throw new ArgumentException("Topology normal must be finite.");
        if (Point is { } p && !p.IsFinite) throw new ArgumentException("Topology point must be finite.");
        if (Measure is { } m && (!double.IsFinite(m) || m < 0d)) throw new ArgumentException("Topology measure is invalid.");
        Provenance.Validate();
    }
}

public sealed record TopologySnapshot(
    string KernelContractVersion,
    IReadOnlyList<TopologyEntityResult> Entities)
{
    public void Validate()
    {
        if (KernelContractVersion != CadContractVersions.KernelEvaluation)
            throw new ArgumentException("Unsupported kernel contract version.");
        var ids = new HashSet<TopologyEntityId>();
        foreach (var entity in Entities)
        {
            entity.Validate();
            if (!ids.Add(entity.Id)) throw new ArgumentException($"Duplicate topology entity '{entity.Id}'.");
        }
    }

    public IReadOnlyList<TopologyEntityResult> Faces =>
        Entities.Where(x => x.Kind == TopologyEntityKind.Face).ToArray();
}

public sealed record AuthoritativeCadResult(
    CadResultId ResultId,
    string KernelContractVersion,
    CadBoundingBox3 Bounds,
    double Volume,
    double SurfaceArea,
    TopologySnapshot Topology)
{
    public void Validate()
    {
        if (!ResultId.IsValid || KernelContractVersion != CadContractVersions.KernelEvaluation)
            throw new ArgumentException("Invalid authoritative result identity.");
        Bounds.Validate();
        if (!double.IsFinite(Volume) || Volume < 0d ||
            !double.IsFinite(SurfaceArea) || SurfaceArea < 0d)
            throw new ArgumentException("Authoritative result measures must be finite and non-negative.");
        Topology.Validate();
    }
}

public sealed record CadFeatureEvaluationResult(
    CadId FeatureId,
    string EvaluationIdentity,
    CadEvaluationStatus Status,
    CadResultId? ResultId,
    IReadOnlyList<ReferenceResolution> References,
    IReadOnlyList<CadDiagnostic> Diagnostics,
    AuthoritativeCadResult? AuthoritativeResult)
{
    public SketchSolveKernelResult? SketchSolve { get; init; }
    public CadSketchEvaluationResult? SketchResult { get; init; }

    public bool Succeeded => Status == CadEvaluationStatus.Succeeded;
}

public sealed record KernelEvaluationRequest(
    CadId EvaluationId,
    CadFeatureSpecification Feature,
    AuthoritativeCadResult? UpstreamResult,
    IReadOnlyList<ReferenceResolution> ResolvedReferences,
    IReadOnlyList<CadFeatureEvaluationResult> UpstreamEvaluations)
{
    public KernelTolerance Tolerance { get; init; } = new(1e-9, 1e-9);

    public void Validate()
    {
        if (!EvaluationId.IsValid) throw new ArgumentException("Kernel evaluation ID is required.");
        Feature.Validate();
        foreach (var reference in ResolvedReferences) reference.Reference.Validate();
        UpstreamResult?.Validate();
    }
}

public sealed record KernelEvaluationResponse(
    CadEvaluationStatus Status,
    AuthoritativeCadResult? AuthoritativeResult,
    IReadOnlyList<CadDiagnostic> Diagnostics)
{
    public SketchSolveKernelResult? SketchSolve { get; init; }
    public CadSketchEvaluationResult? SketchResult { get; init; }
}

public interface ICadKernelEvaluator
{
    Task<KernelEvaluationResponse> EvaluateAsync(
        KernelEvaluationRequest request,
        CancellationToken cancellationToken = default);
}

public enum RecomputeMode { Full, Incremental }

public sealed record CadRepresentation(
    string ContractVersion,
    CadId RepresentationId,
    CadResultId SourceResultId,
    string RepresentationKind,
    string DisplayPolicy,
    string ContentIdentity);

public sealed record CadEvaluationResult(
    string ContractVersion,
    CadId DocumentId,
    CadResultId EvaluationIdentity,
    RecomputeMode Mode,
    IReadOnlyList<CadId> EvaluationPlan,
    IReadOnlySet<CadId> InvalidatedFeatures,
    IReadOnlyList<CadFeatureEvaluationResult> Features,
    AuthoritativeCadResult? FinalAuthoritativeResult,
    CadRepresentation? Representation,
    CadDiagnostic? Failure)
{
    public bool Succeeded =>
        Failure is null && Features.All(x => x.Succeeded) && FinalAuthoritativeResult is not null;
}
