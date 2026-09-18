using UMLCAD.Cad.Contracts;
using UMLCAD.Cad.Semantics;

namespace UMLCAD.Cad.Engine;

public sealed class AxisAlignedBoxSolidEvaluator
{
    private readonly IAuthoritativeGeometryService _geometry;

    public AxisAlignedBoxSolidEvaluator(IAuthoritativeGeometryService geometry)
    {
        _geometry = geometry ?? throw new ArgumentNullException(nameof(geometry));
    }

    public async Task<AuthoritativeCadResult> EvaluateAsync(
        AxisAlignedBoxSolidSpecification specification,
        string configurationContext,
        KernelTolerance tolerance,
        CancellationToken cancellationToken = default)
    {
        ArgumentNullException.ThrowIfNull(specification);

        if (string.IsNullOrWhiteSpace(configurationContext))
            throw new ArgumentException("Configuration context is required.", nameof(configurationContext));

        var request = new AxisAlignedBoxSolidRequest(
            OperationIdentity: specification.FeatureId.Value.ToString("D"),
            Min: new KernelVector3(
                specification.MinXmm,
                specification.MinYmm,
                specification.MinZmm),
            Max: new KernelVector3(
                specification.MaxXmm,
                specification.MaxYmm,
                specification.MaxZmm),
            Tolerance: tolerance,
            ContractVersion: new ContractVersion("1.0"));

        var kernelResult = await _geometry.BuildAxisAlignedBoxSolidAsync(
            request,
            cancellationToken);

        var result = ResultIntegrator.Integrate(
            specification.FeatureId,
            kernelResult);

        if (result.Status == AuthoritativeResultStatus.Succeeded &&
            result.TopologyBindings.Count != 6)
        {
            throw new InvalidOperationException(
                "A successful axis-aligned box evaluation must expose six topology bindings.");
        }

        return result;
    }
}
