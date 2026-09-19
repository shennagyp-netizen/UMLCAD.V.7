using System.Collections.ObjectModel;
using System.Security.Cryptography;
using System.Text;
using UMLCAD.Cad.Contracts;

namespace UMLCAD.Science;

public enum PhenomenonKind
{
    Thermal,
    Structural,
    Fluid,
    Electromagnetic,
    CoupledMultiphysics
}

public readonly record struct EngineeringFrameId
{
    public EngineeringFrameId(string value)
    {
        if (string.IsNullOrWhiteSpace(value))
            throw new ArgumentException("Engineering frame ID cannot be empty.", nameof(value));

        Value = value;
    }

    public string Value { get; }

    public override string ToString() => Value;
}

public readonly record struct SimulationIdentity
{
    public SimulationIdentity(string value)
    {
        if (string.IsNullOrWhiteSpace(value))
            throw new ArgumentException("Simulation identity cannot be empty.", nameof(value));

        Value = value;
    }

    public string Value { get; }

    public static SimulationIdentity Create(SimulationRequest request)
    {
        ArgumentNullException.ThrowIfNull(request);

        var canonical = string.Join(
            "\n",
            request.Phenomenon,
            request.ModelContractId,
            request.GeometryIdentity,
            request.MaterialStateIdentity,
            request.ProcessStateIdentity,
            request.BoundaryConditionIdentity,
            request.EnvironmentIdentity,
            request.ConfigurationIdentity,
            request.NumericalPolicyIdentity,
            request.UncertaintyPolicyIdentity);

        var hash = Convert.ToHexString(
            SHA256.HashData(Encoding.UTF8.GetBytes(canonical)));

        return new SimulationIdentity($"sha256:{hash}");
    }

    public override string ToString() => Value;
}

public sealed record SimulationRequest
{
    public SimulationRequest(
        PhenomenonKind phenomenon,
        string modelContractId,
        string geometryIdentity,
        string materialStateIdentity,
        string processStateIdentity,
        string boundaryConditionIdentity,
        string environmentIdentity,
        string configurationIdentity,
        string numericalPolicyIdentity,
        string uncertaintyPolicyIdentity)
    {
        ModelContractId = Require(modelContractId, nameof(modelContractId));
        GeometryIdentity = Require(geometryIdentity, nameof(geometryIdentity));
        MaterialStateIdentity = Require(materialStateIdentity, nameof(materialStateIdentity));
        ProcessStateIdentity = Require(processStateIdentity, nameof(processStateIdentity));
        BoundaryConditionIdentity = Require(
            boundaryConditionIdentity,
            nameof(boundaryConditionIdentity));
        EnvironmentIdentity = Require(environmentIdentity, nameof(environmentIdentity));
        ConfigurationIdentity = Require(configurationIdentity, nameof(configurationIdentity));
        NumericalPolicyIdentity = Require(
            numericalPolicyIdentity,
            nameof(numericalPolicyIdentity));
        UncertaintyPolicyIdentity = Require(
            uncertaintyPolicyIdentity,
            nameof(uncertaintyPolicyIdentity));

        Phenomenon = phenomenon;
        Identity = SimulationIdentity.Create(this);
    }

    public PhenomenonKind Phenomenon { get; }

    public string ModelContractId { get; }

    public string GeometryIdentity { get; }

    public string MaterialStateIdentity { get; }

    public string ProcessStateIdentity { get; }

    public string BoundaryConditionIdentity { get; }

    public string EnvironmentIdentity { get; }

    public string ConfigurationIdentity { get; }

    public string NumericalPolicyIdentity { get; }

    public string UncertaintyPolicyIdentity { get; }

    public SimulationIdentity Identity { get; }

    private static string Require(string value, string parameterName) =>
        string.IsNullOrWhiteSpace(value)
            ? throw new ArgumentException(
                $"Simulation identity component cannot be empty.",
                parameterName)
            : value;
}

public enum SimulationResultStatus
{
    Completed,
    Failed,
    Invalid,
    Incomplete,
    Indeterminate
}

public enum SpatialRegionKind
{
    Influence,
    Affected,
    Exclusion,
    Deviation,
    MaterialPropertyChange,
    Custom
}

public sealed record SpatialRegion(
    CadId Id,
    SpatialRegionKind Kind,
    EngineeringFrameId Frame,
    SimulationIdentity SourceSimulation)
{
    public SpatialRegion
    {
        if (Id.Value.Length == 0)
            throw new ArgumentException("Spatial region ID cannot be empty.", nameof(Id));
    }
}

public sealed record ScalarFieldSample(
    double X,
    double Y,
    double Z,
    double Value)
{
    public ScalarFieldSample
    {
        if (!double.IsFinite(X) ||
            !double.IsFinite(Y) ||
            !double.IsFinite(Z) ||
            !double.IsFinite(Value))
        {
            throw new ArgumentOutOfRangeException(
                nameof(Value),
                "Scalar field samples must contain only finite values.");
        }
    }
}

public sealed record ScalarField
{
    public ScalarField(
        CadId id,
        EngineeringFrameId frame,
        string quantityId,
        IReadOnlyList<ScalarFieldSample> samples,
        SimulationIdentity sourceSimulation)
    {
        if (id.Value.Length == 0)
            throw new ArgumentException("Scalar field ID cannot be empty.", nameof(id));
        if (string.IsNullOrWhiteSpace(quantityId))
            throw new ArgumentException("Field quantity ID cannot be empty.", nameof(quantityId));

        ArgumentNullException.ThrowIfNull(samples);
        var copiedSamples = new ReadOnlyCollection<ScalarFieldSample>(samples.ToArray());

        Id = id;
        Frame = frame;
        QuantityId = quantityId;
        Samples = copiedSamples;
        SourceSimulation = sourceSimulation;
    }

    public CadId Id { get; }

    public EngineeringFrameId Frame { get; }

    public string QuantityId { get; }

    public IReadOnlyList<ScalarFieldSample> Samples { get; }

    public SimulationIdentity SourceSimulation { get; }
}

public sealed record SimulationResult
{
    public SimulationResult(
        SimulationRequest request,
        SimulationResultStatus status,
        IReadOnlyList<SpatialRegion> regions,
        IReadOnlyList<ScalarField> scalarFields)
    {
        ArgumentNullException.ThrowIfNull(request);
        ArgumentNullException.ThrowIfNull(regions);
        ArgumentNullException.ThrowIfNull(scalarFields);

        RequestIdentity = request.Identity;
        Status = status;
        Regions = new ReadOnlyCollection<SpatialRegion>(regions.ToArray());
        ScalarFields = new ReadOnlyCollection<ScalarField>(scalarFields.ToArray());
    }

    public SimulationIdentity RequestIdentity { get; }

    public SimulationResultStatus Status { get; }

    public IReadOnlyList<SpatialRegion> Regions { get; }

    public IReadOnlyList<ScalarField> ScalarFields { get; }

    public bool IsReusable(SimulationRequest request) =>
        request is not null &&
        Status == SimulationResultStatus.Completed &&
        RequestIdentity == request.Identity;
}
