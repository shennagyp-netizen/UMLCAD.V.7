namespace UMLCAD.Science;

public readonly record struct QuantityDimension(
    int LengthPower,
    int MassPower,
    int TimePower,
    int TemperaturePower)
{
    public static QuantityDimension Dimensionless => new(0, 0, 0, 0);
}

public readonly record struct Quantity
{
    public double SiValue { get; }
    public QuantityDimension Dimension { get; }
    public string UnitSymbol { get; }

    public Quantity(double siValue, QuantityDimension dimension, string unitSymbol)
    {
        if (!double.IsFinite(siValue))
            throw new ArgumentOutOfRangeException(nameof(siValue), "Quantity values must be finite.");

        if (string.IsNullOrWhiteSpace(unitSymbol))
            throw new ArgumentException("UnitSymbol is required.", nameof(unitSymbol));

        SiValue = siValue;
        Dimension = dimension;
        UnitSymbol = unitSymbol;
    }
}

public enum MaterialFamily
{
    Metal,
    Polymer,
    Ceramic,
    Composite,
    Wood,
    Stone,
    Other,
}

public sealed record MaterialProperties
{
    public double DensityKgPerM3 { get; }
    public double YoungsModulusPa { get; }
    public double YieldStrengthPa { get; }
    public double UltimateStrengthPa { get; }
    public double DuctilityPercent { get; }
    public double ThermalConductivityWPerMk { get; }
    public double ElectricalConductivitySiemensPerM { get; }

    public MaterialProperties(
        double densityKgPerM3,
        double youngsModulusPa,
        double yieldStrengthPa,
        double ultimateStrengthPa,
        double ductilityPercent,
        double thermalConductivityWPerMk,
        double electricalConductivitySiemensPerM)
    {
        ValidateFiniteNonNegative(densityKgPerM3, nameof(densityKgPerM3));
        ValidateFiniteNonNegative(youngsModulusPa, nameof(youngsModulusPa));
        ValidateFiniteNonNegative(yieldStrengthPa, nameof(yieldStrengthPa));
        ValidateFiniteNonNegative(ultimateStrengthPa, nameof(ultimateStrengthPa));
        ValidateFiniteNonNegative(ductilityPercent, nameof(ductilityPercent));
        ValidateFiniteNonNegative(thermalConductivityWPerMk, nameof(thermalConductivityWPerMk));
        ValidateFiniteNonNegative(electricalConductivitySiemensPerM, nameof(electricalConductivitySiemensPerM));

        if (yieldStrengthPa > ultimateStrengthPa && ultimateStrengthPa > 0d)
            throw new ArgumentException("Yield strength cannot exceed ultimate strength.");

        DensityKgPerM3 = densityKgPerM3;
        YoungsModulusPa = youngsModulusPa;
        YieldStrengthPa = yieldStrengthPa;
        UltimateStrengthPa = ultimateStrengthPa;
        DuctilityPercent = ductilityPercent;
        ThermalConductivityWPerMk = thermalConductivityWPerMk;
        ElectricalConductivitySiemensPerM = electricalConductivitySiemensPerM;
    }

    private static void ValidateFiniteNonNegative(double value, string name)
    {
        if (!double.IsFinite(value) || value < 0d)
            throw new ArgumentOutOfRangeException(name, "Material properties must be finite and non-negative.");
    }
}

public sealed record Material
{
    public string Name { get; }
    public MaterialFamily Family { get; }
    public MaterialProperties Properties { get; }

    public Material(string name, MaterialFamily family, MaterialProperties properties)
    {
        if (string.IsNullOrWhiteSpace(name))
            throw new ArgumentException("Material name is required.", nameof(name));

        ArgumentNullException.ThrowIfNull(properties);

        Name = name;
        Family = family;
        Properties = properties;
    }
}

public enum PhenomenonKind
{
    Deformation,
    StressStrainResponse,
    ThermalResponse,
    Vibration,
    CuttingProcessResponse,
}

public sealed record PhenomenaSimulationRequest
{
    public string RequestId { get; }
    public PhenomenonKind Phenomenon { get; }
    public IReadOnlyDictionary<string, double> Inputs { get; }

    public PhenomenaSimulationRequest(
        string requestId,
        PhenomenonKind phenomenon,
        IReadOnlyDictionary<string, double> inputs)
    {
        if (string.IsNullOrWhiteSpace(requestId))
            throw new ArgumentException("RequestId is required.", nameof(requestId));

        ArgumentNullException.ThrowIfNull(inputs);

        var normalized = new Dictionary<string, double>(inputs, StringComparer.Ordinal);
        foreach (var pair in normalized)
        {
            if (string.IsNullOrWhiteSpace(pair.Key))
                throw new ArgumentException("Simulation input names cannot be empty.", nameof(inputs));
            if (!double.IsFinite(pair.Value))
                throw new ArgumentOutOfRangeException(nameof(inputs), "Simulation inputs must be finite.");
        }

        RequestId = requestId;
        Phenomenon = phenomenon;
        Inputs = normalized;
    }
}

public sealed record PhenomenaSimulationResult
{
    public string RequestId { get; }
    public PhenomenonKind Phenomenon { get; }
    public bool IsSuccessful { get; }
    public IReadOnlyDictionary<string, double> Outputs { get; }
    public string ProviderId { get; }
    public string Diagnostic { get; }

    public PhenomenaSimulationResult(
        string requestId,
        PhenomenonKind phenomenon,
        bool isSuccessful,
        IReadOnlyDictionary<string, double> outputs,
        string providerId,
        string diagnostic)
    {
        if (string.IsNullOrWhiteSpace(requestId))
            throw new ArgumentException("RequestId is required.", nameof(requestId));
        if (string.IsNullOrWhiteSpace(providerId))
            throw new ArgumentException("ProviderId is required.", nameof(providerId));

        var normalized = outputs is null
            ? throw new ArgumentNullException(nameof(outputs))
            : new Dictionary<string, double>(outputs, StringComparer.Ordinal);

        foreach (var pair in normalized)
        {
            if (!double.IsFinite(pair.Value))
                throw new ArgumentOutOfRangeException(nameof(outputs), "Simulation outputs must be finite.");
        }

        RequestId = requestId;
        Phenomenon = phenomenon;
        IsSuccessful = isSuccessful;
        Outputs = normalized;
        ProviderId = providerId;
        Diagnostic = diagnostic ?? string.Empty;
    }
}

public interface IPhenomenaSimulationProvider
{
    string ProviderId { get; }

    bool Supports(PhenomenonKind phenomenon);

    PhenomenaSimulationResult Simulate(PhenomenaSimulationRequest request);
}
