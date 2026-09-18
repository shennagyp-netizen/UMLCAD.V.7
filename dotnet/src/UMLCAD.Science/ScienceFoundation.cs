namespace UMLCAD.Science;

public readonly record struct QuantityDimension(
    string LengthPower,
    string MassPower,
    string TimePower,
    string TemperaturePower)
{
    public static QuantityDimension Dimensionless => new("0", "0", "0", "0");
}

public readonly record struct Quantity(
    double SiValue,
    QuantityDimension Dimension,
    string UnitSymbol)
{
    public Quantity
    {
        if (!double.IsFinite(SiValue))
            throw new ArgumentOutOfRangeException(nameof(SiValue), "Quantity values must be finite.");

        if (string.IsNullOrWhiteSpace(UnitSymbol))
            throw new ArgumentException("UnitSymbol is required.", nameof(UnitSymbol));
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

public sealed record MaterialProperties(
    double DensityKgPerM3,
    double YoungsModulusPa,
    double YieldStrengthPa,
    double UltimateStrengthPa,
    double DuctilityPercent,
    double ThermalConductivityWPerMk,
    double ElectricalConductivitySiemensPerM)
{
    public MaterialProperties
    {
        ValidateFiniteNonNegative(DensityKgPerM3, nameof(DensityKgPerM3));
        ValidateFiniteNonNegative(YoungsModulusPa, nameof(YoungsModulusPa));
        ValidateFiniteNonNegative(YieldStrengthPa, nameof(YieldStrengthPa));
        ValidateFiniteNonNegative(UltimateStrengthPa, nameof(UltimateStrengthPa));
        ValidateFiniteNonNegative(DuctilityPercent, nameof(DuctilityPercent));
        ValidateFiniteNonNegative(ThermalConductivityWPerMk, nameof(ThermalConductivityWPerMk));
        ValidateFiniteNonNegative(ElectricalConductivitySiemensPerM, nameof(ElectricalConductivitySiemensPerM));

        if (YieldStrengthPa > UltimateStrengthPa && UltimateStrengthPa > 0d)
            throw new ArgumentException("Yield strength cannot exceed ultimate strength.");
    }

    private static void ValidateFiniteNonNegative(double value, string name)
    {
        if (!double.IsFinite(value) || value < 0d)
            throw new ArgumentOutOfRangeException(name, "Material properties must be finite and non-negative.");
    }
}

public sealed record Material(
    string Name,
    MaterialFamily Family,
    MaterialProperties Properties)
{
    public Material
    {
        if (string.IsNullOrWhiteSpace(Name))
            throw new ArgumentException("Material name is required.", nameof(Name));

        ArgumentNullException.ThrowIfNull(Properties);
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

public sealed record PhenomenaSimulationRequest(
    string RequestId,
    PhenomenonKind Phenomenon,
    IReadOnlyDictionary<string, double> Inputs)
{
    public PhenomenaSimulationRequest
    {
        if (string.IsNullOrWhiteSpace(RequestId))
            throw new ArgumentException("RequestId is required.", nameof(RequestId));

        ArgumentNullException.ThrowIfNull(Inputs);

        foreach (var pair in Inputs)
        {
            if (string.IsNullOrWhiteSpace(pair.Key))
                throw new ArgumentException("Simulation input names cannot be empty.", nameof(Inputs));
            if (!double.IsFinite(pair.Value))
                throw new ArgumentOutOfRangeException(nameof(Inputs), "Simulation inputs must be finite.");
        }

        Inputs = new Dictionary<string, double>(Inputs, StringComparer.Ordinal);
    }
}

public sealed record PhenomenaSimulationResult(
    string RequestId,
    PhenomenonKind Phenomenon,
    bool IsSuccessful,
    IReadOnlyDictionary<string, double> Outputs,
    string ProviderId,
    string Diagnostic)
{
    public PhenomenaSimulationResult
    {
        if (string.IsNullOrWhiteSpace(RequestId))
            throw new ArgumentException("RequestId is required.", nameof(RequestId));
        if (string.IsNullOrWhiteSpace(ProviderId))
            throw new ArgumentException("ProviderId is required.", nameof(ProviderId));

        Outputs = Outputs is null
            ? throw new ArgumentNullException(nameof(Outputs))
            : new Dictionary<string, double>(Outputs, StringComparer.Ordinal);

        foreach (var pair in Outputs)
        {
            if (!double.IsFinite(pair.Value))
                throw new ArgumentOutOfRangeException(nameof(Outputs), "Simulation outputs must be finite.");
        }

        Diagnostic ??= string.Empty;
    }
}

public interface IPhenomenaSimulationProvider
{
    string ProviderId { get; }

    bool Supports(PhenomenonKind phenomenon);

    PhenomenaSimulationResult Simulate(PhenomenaSimulationRequest request);
}
