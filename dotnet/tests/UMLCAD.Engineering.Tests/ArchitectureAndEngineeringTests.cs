using UMLCAD.Cad.Expressions;
using UMLCAD.Cad.Semantics;
using UMLCAD.Engineering.Cam;
using UMLCAD.Engineering.Drawing;
using UMLCAD.Engineering.Resources;
using UMLCAD.Engineering.SheetMetal;
using UMLCAD.Science;

namespace UMLCAD.Engineering.Tests;

public sealed class ArchitectureAndEngineeringTests
{
    [Fact]
    public void ExpressionCanonicalizationIsFullyParenthesizedAndDeterministic()
    {
        var expression = new BinaryExpression(
            BinaryOperator.Add,
            new VariableExpression("a"),
            new BinaryExpression(
                BinaryOperator.Multiply,
                new VariableExpression("b"),
                new VariableExpression("c")));

        Assert.Equal("(a + (b * c))", expression.ToCanonicalString());
        Assert.Equal(expression.ToCanonicalString(), ExpressionIdentity.Canonicalize(expression));
        Assert.Equal(ExpressionIdentity.Sha256(expression), ExpressionIdentity.Sha256(expression));
    }

    [Fact]
    public void ExpressionEvaluationIsFiniteAndUsesSuppliedVariables()
    {
        var expression = new BinaryExpression(
            BinaryOperator.Multiply,
            new VariableExpression("length"),
            new VariableExpression("scale"));

        Assert.Equal(12d, expression.Evaluate(
            new Dictionary<string, double>
            {
                ["length"] = 4d,
                ["scale"] = 3d,
            }));
    }

    [Fact]
    public void MaterialPropertiesAreScientificAndSheetMetalRejectsStone()
    {
        var granite = new Material(
            "Granite",
            MaterialFamily.Stone,
            new MaterialProperties(
                DensityKgPerM3: 2700d,
                YoungsModulusPa: 50e9,
                YieldStrengthPa: 0d,
                UltimateStrengthPa: 0d,
                DuctilityPercent: 0d,
                ThermalConductivityWPerMk: 2.5d,
                ElectricalConductivitySiemensPerM: 0d));

        var part = new SheetMetalPartDefinition(
            SemanticId.New(),
            granite,
            2d);

        var bend = new BendDefinition(SemanticId.New(), 90d, 2d, 0.33d);
        var result = SheetMetalValidator.Validate(part, bend);

        Assert.False(result.IsValid);
        Assert.Contains(result.Diagnostics, x => x.Contains("classified as Metal", StringComparison.Ordinal));
    }

    [Fact]
    public void SheetMetalAcceptsCompatibleMetalAndMachineTool()
    {
        var steel = new Material(
            "Steel",
            MaterialFamily.Metal,
            new MaterialProperties(
                DensityKgPerM3: 7850d,
                YoungsModulusPa: 200e9,
                YieldStrengthPa: 250e6,
                UltimateStrengthPa: 400e6,
                DuctilityPercent: 20d,
                ThermalConductivityWPerMk: 50d,
                ElectricalConductivitySiemensPerM: 6e6));

        var part = new SheetMetalPartDefinition(SemanticId.New(), steel, 2d);
        var bend = new BendDefinition(SemanticId.New(), 90d, 2d, 0.33d);
        var machine = new MachineDefinition(
            "PB-01",
            MachineKind.PressBrake,
            "PRESS-BRAKE-01",
            new[]
            {
                new MachineCapability(
                    ManufacturingProcessKind.SheetMetalBending,
                    0.5d,
                    10d),
            });
        var tool = new ToolDefinition(
            "PUNCH-10",
            ToolKind.PressBrakePunch,
            "PRESS-BRAKE-01",
            10d,
            5d,
            20d);

        var result = SheetMetalValidator.Validate(part, bend, machine, tool);

        Assert.True(result.IsValid);
        Assert.Empty(result.Diagnostics);
    }

    [Fact]
    public void BendAllowanceUsesSharedExpressionAst()
    {
        var expression = SheetMetalExpressions.BendAllowance();
        Assert.Equal(
            "(((pi / 180) * (R + (K * T))) * A)",
            expression.ToCanonicalString());

        var value = expression.Evaluate(
            new Dictionary<string, double>
            {
                ["pi"] = Math.PI,
                ["R"] = 2d,
                ["K"] = 0.33d,
                ["T"] = 2d,
                ["A"] = 90d,
            });

        Assert.True(double.IsFinite(value));
        Assert.InRange(value, 3.5d, 3.8d);
    }

    [Fact]
    public void SimulationServiceSelectsProviderDeterministically()
    {
        var provider = new TestSimulationProvider(
            "A.provider",
            PhenomenonKind.CuttingProcessResponse,
            42d);

        var later = new TestSimulationProvider(
            "B.provider",
            PhenomenonKind.CuttingProcessResponse,
            99d);

        var service = new PhenomenaSimulationService(new[] { later, provider });
        var result = service.Simulate(
            new PhenomenaSimulationRequest(
                "SIM-001",
                PhenomenonKind.CuttingProcessResponse,
                new Dictionary<string, double> { ["load"] = 10d }));

        Assert.Equal("A.provider", result.ProviderId);
        Assert.Equal(42d, result.Outputs["response"]);
    }

    [Fact]
    public void BomIsDeterministicAndGroupsOccurrences()
    {
        var engine = new ProductComponent(
            SemanticId.New(),
            "ENG-02",
            "A",
            "Engine Mount");

        var bolt = new ProductComponent(
            SemanticId.New(),
            "BOLT-01",
            "B",
            "Fastener");

        var product = new ProductDefinition(
            SemanticId.New(),
            "ASM-100",
            "A",
            new[] { engine, bolt },
            new[]
            {
                new ProductOccurrence(SemanticId.New(), engine.ComponentId, 1, "Top"),
                new ProductOccurrence(SemanticId.New(), bolt.ComponentId, 4, "Top"),
                new ProductOccurrence(SemanticId.New(), bolt.ComponentId, 2, "Bottom"),
            });

        var bom = BomService.Generate(product);

        Assert.Equal(2, bom.Count);
        Assert.Equal("BOLT-01", bom[0].PartNumber);
        Assert.Equal(6, bom[0].Quantity);
        Assert.Equal("ENG-02", bom[1].PartNumber);
        Assert.Equal(1, bom[1].Quantity);
        Assert.Equal("1", bom[0].ItemNumber);
        Assert.Equal("2", bom[1].ItemNumber);
    }

    [Fact]
    public void GCodeGenerationIsDeterministicAndMachineAware()
    {
        var machine = new MachineDefinition(
            "MC-01",
            MachineKind.MachiningCenter,
            "HSK-63",
            new[]
            {
                new MachineCapability(ManufacturingProcessKind.Milling, 0.5d, 100d),
            });

        var tool = new ToolDefinition(
            "T10",
            ToolKind.EndMill,
            "HSK-63",
            10d,
            6d,
            20d);

        var operation = new ManufacturingOperation(
            SemanticId.New(),
            ManufacturingProcessKind.Milling,
            "T10",
            5d,
            new[]
            {
                new ToolpathPoint(0d, 0d, 5d),
                new ToolpathPoint(10d, 0d, 5d),
                new ToolpathPoint(10d, 10d, 2d),
            });

        var post = new DeterministicGCodePostprocessor();
        var first = post.Generate(operation, machine, tool);
        var second = post.Generate(operation, machine, tool);

        Assert.Equal(first.Serialize(), second.Serialize());
        Assert.Equal(first.ContentHash(), second.ContentHash());
        Assert.Equal(64, first.ContentHash().Length);
        Assert.Contains("G21", first.Lines);
        Assert.Contains("G90", first.Lines);
        Assert.Contains("(TOOL T10 DIA 10)", first.Lines);
        Assert.Equal("M30", first.Lines[^2]);
    }

    [Fact]
    public void GCodeRefusesUnsupportedProcess()
    {
        var machine = new MachineDefinition(
            "LATHE-01",
            MachineKind.Lathe,
            "CAPTO",
            new[]
            {
                new MachineCapability(ManufacturingProcessKind.Turning, 0.5d, 100d),
            });

        var tool = new ToolDefinition(
            "TN-01",
            ToolKind.EndMill,
            "CAPTO",
            10d,
            5d,
            20d);

        var operation = new ManufacturingOperation(
            SemanticId.New(),
            ManufacturingProcessKind.Turning,
            "TN-01",
            5d,
            new[] { new ToolpathPoint(0d, 0d, 0d) });

        var post = new DeterministicGCodePostprocessor();

        Assert.Throws<NotSupportedException>(() => post.Generate(operation, machine, tool));
    }

    [Fact]
    public void DrawingCapabilityProfileContainsDetailedDraftingSurface()
    {
        var profile = CatiaDraftingCapabilityProfile.Baseline(DrawingStandard.Iso);

        Assert.Contains(DrawingViewKind.Section, profile.ViewKinds);
        Assert.Contains(DrawingViewKind.AlignedSection, profile.ViewKinds);
        Assert.Contains(DrawingViewKind.OffsetSection, profile.ViewKinds);
        Assert.Contains(DrawingViewKind.CircularDetail, profile.ViewKinds);
        Assert.Contains(DrawingViewKind.ProfiledDetail, profile.ViewKinds);
        Assert.Contains(DrawingViewKind.Clipping, profile.ViewKinds);
        Assert.Contains(DimensionKind.Diameter, profile.DimensionKinds);
        Assert.Contains(DimensionKind.Baseline, profile.DimensionKinds);
        Assert.Contains(AnnotationKind.GeometricTolerance, profile.AnnotationKinds);
        Assert.Contains(AnnotationKind.SurfaceRoughness, profile.AnnotationKinds);
        Assert.Contains(DressUpKind.ThreadLine, profile.DressUpKinds);
        Assert.Contains(DressUpKind.AreaFill, profile.DressUpKinds);
    }

    [Fact]
    public void DrawingViewScaleMustBeDimensionless()
    {
        var source = SemanticId.New();

        Assert.Throws<ArgumentException>(() =>
            new DrawingView(
                SemanticId.New(),
                DrawingViewKind.Front,
                source,
                new Quantity(1d, new QuantityDimension(1, 0, 0, 0), "mm")));
    }

    private sealed class TestSimulationProvider(
        string providerId,
        PhenomenonKind phenomenon,
        double response) : IPhenomenaSimulationProvider
    {
        public string ProviderId => providerId;

        public bool Supports(PhenomenonKind requested) => requested == phenomenon;

        public PhenomenaSimulationResult Simulate(PhenomenaSimulationRequest request) =>
            new(
                request.RequestId,
                request.Phenomenon,
                true,
                new Dictionary<string, double> { ["response"] = response },
                ProviderId,
                "test");
    }
}
