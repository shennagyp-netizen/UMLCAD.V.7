using System.Net;
using System.Net.Http.Json;
using Microsoft.Extensions.Options;
using UMLCAD.Cad.Contracts;
using UMLCAD.Cad.Engine;
using UMLCAD.Kernel.Client;

namespace UMLCAD.Engineering.Tests;

public sealed class RustCadKernelEvaluatorTests
{
    [Fact]
    public async Task ProductionAdapterEvaluatesBoxThroughAuthoritativeGeometryService()
    {
        var box = new BoxFeatureSpecification(
            new CadId("box"),
            new CadFrame(new CadId("part-frame"), CadFrameKind.Part, 10d, 20d, 30d),
            2d,
            3d,
            4d);

        var geometry = new RecordingBoxGeometryService(
            new AxisAlignedBoxSolidKernelResult(
                GeometryKernelStatus.Succeeded,
                new ContractResultId("solid:box-001"),
                "evidence:box-001",
                new[]
                {
                    new AxisAlignedBoxSolidKernelTopology("Face", "f_top"),
                    new AxisAlignedBoxSolidKernelTopology("Face", "f_bottom"),
                    new AxisAlignedBoxSolidKernelTopology("Face", "f_right"),
                    new AxisAlignedBoxSolidKernelTopology("Face", "f_left"),
                    new AxisAlignedBoxSolidKernelTopology("Face", "f_front"),
                    new AxisAlignedBoxSolidKernelTopology("Face", "f_back")
                },
                24d,
                52d,
                new KernelVector3(11d, 21.5d, 32d),
                Array.Empty<string>()));

        var evaluator = new RustCadKernelEvaluator(geometry);

        var response = await evaluator.EvaluateAsync(
            new KernelEvaluationRequest(
                new CadId("evaluation-001"),
                box,
                null,
                Array.Empty<ReferenceResolution>(),
                Array.Empty<CadFeatureEvaluationResult>())
            {
                Tolerance = new KernelTolerance(1e-7, 1e-8)
            });

        Assert.Equal(CadEvaluationStatus.Succeeded, response.Status);
        Assert.NotNull(response.AuthoritativeResult);

        var result = response.AuthoritativeResult!;
        Assert.Equal("solid:box-001", result.ResultId.Value);
        Assert.Equal(24d, result.Volume);
        Assert.Equal(52d, result.SurfaceArea);
        Assert.Equal(new CadBoundingBox3(10d, 20d, 30d, 12d, 23d, 34d), result.Bounds);
        Assert.Equal(6, result.Topology.Entities.Count);

        var top = Assert.Single(result.Topology.Entities, x => x.Id.Value == "f_top");
        Assert.Equal(new CadVector3(0d, 0d, 1d), top.Normal);
        Assert.Equal(new CadVector3(11d, 21.5d, 34d), top.Point);
        Assert.Equal(6d, top.Measure);
        Assert.True(geometry.LastRequest is not null);
        Assert.Equal("evaluation-001", geometry.LastRequest!.OperationIdentity);
        Assert.Equal(new KernelTolerance(1e-7, 1e-8), geometry.LastRequest.Tolerance);
        Assert.Equal(new KernelVector3(10d, 20d, 30d), geometry.LastRequest.Min);
        Assert.Equal(new KernelVector3(12d, 23d, 34d), geometry.LastRequest.Max);
        Assert.Equal(new KernelTolerance(1e-7, 1e-8), geometry.LastRequest.Tolerance);
    }

    [Fact]
    public async Task ProductionAdapterConsumesTheTypedBoxHttpContract()
    {
        var handler = new RecordingHttpHandler(async request =>
        {
            Assert.Equal(HttpMethod.Post, request.Method);
            Assert.Equal(
                "http://kernel.test/v1/geometry/box-solid",
                request.RequestUri!.ToString());

            var payload = await request.Content!.ReadFromJsonAsync<JsonElement>();
            Assert.Equal(
                AxisAlignedBoxSolidRequest.ContractSchema,
                payload.GetProperty("schema").GetString());
            Assert.Equal(
                "evaluation-http-001",
                payload.GetProperty("operationIdentity").GetString());
            Assert.Equal(
                1e-7,
                payload.GetProperty("tolerance").GetProperty("absolute").GetDouble());
            Assert.Equal(
                1e-8,
                payload.GetProperty("tolerance").GetProperty("relative").GetDouble());

            return new HttpResponseMessage(HttpStatusCode.OK)
            {
                Content = JsonContent.Create(new
                {
                    schema = AxisAlignedBoxSolidRequest.ContractSchema,
                    status = "succeeded",
                    succeeded = true,
                    resultId = "solid:http-001",
                    evidenceHash = "evidence:http-001",
                    topology = new[]
                    {
                        new { kind = "Face", key = "f_bottom" },
                        new { kind = "Face", key = "f_top" },
                        new { kind = "Face", key = "f_back" },
                        new { kind = "Face", key = "f_front" },
                        new { kind = "Face", key = "f_left" },
                        new { kind = "Face", key = "f_right" }
                    },
                    volume = 24d,
                    surfaceArea = 52d,
                    centroid = new { x = 1d, y = 1.5d, z = 2d },
                    diagnostics = Array.Empty<string>()
                })
            };
        });

        var httpClient = new HttpClient(handler)
        {
            BaseAddress = new Uri("http://kernel.test/")
        };

        var geometryService = new RustGeometryKernelService(
            httpClient,
            Options.Create(new RustKernelOptions
            {
                BaseAddress = new Uri("http://kernel.test/"),
                RequestTimeout = TimeSpan.FromSeconds(5)
            }));

        var evaluator = new RustCadKernelEvaluator(geometryService);

        var response = await evaluator.EvaluateAsync(
            new KernelEvaluationRequest(
                new CadId("evaluation-http-001"),
                new BoxFeatureSpecification(
                    new CadId("box"),
                    new CadFrame(new CadId("part"), CadFrameKind.Part, 0d, 0d, 0d),
                    2d, 3d, 4d),
                null,
                Array.Empty<ReferenceResolution>(),
                Array.Empty<CadFeatureEvaluationResult>())
            {
                Tolerance = new KernelTolerance(1e-7, 1e-8)
            });

        Assert.Equal(CadEvaluationStatus.Succeeded, response.Status);
        Assert.Equal("solid:http-001", response.AuthoritativeResult!.ResultId.Value);
        Assert.Equal(24d, response.AuthoritativeResult.Volume);
        Assert.Equal(6, response.AuthoritativeResult.Topology.Faces.Count);
    }

    [Fact]
    public void SemanticPlanarFaceReferenceResolvesIndependentOfTopologyArrayOrder()
    {
        var box = new BoxFeatureSpecification(
            new CadId("box"),
            new CadFrame(new CadId("part-frame"), CadFrameKind.Part, 0d, 0d, 0d),
            2d,
            3d,
            4d);

        var evaluator = new RustCadKernelEvaluator(
            new RecordingBoxGeometryService(
                new AxisAlignedBoxSolidKernelResult(
                    GeometryKernelStatus.Succeeded,
                    new ContractResultId("solid:box-002"),
                    "evidence:box-002",
                    new[]
                    {
                        new AxisAlignedBoxSolidKernelTopology("Face", "f_left"),
                        new AxisAlignedBoxSolidKernelTopology("Face", "f_bottom"),
                        new AxisAlignedBoxSolidKernelTopology("Face", "f_top"),
                        new AxisAlignedBoxSolidKernelTopology("Face", "f_back"),
                        new AxisAlignedBoxSolidKernelTopology("Face", "f_right"),
                        new AxisAlignedBoxSolidKernelTopology("Face", "f_front")
                    },
                    24d,
                    52d,
                    new KernelVector3(1d, 1.5d, 2d),
                    Array.Empty<string>())));

        var evaluation = await evaluator.EvaluateAsync(
            new KernelEvaluationRequest(
                new CadId("evaluation-002"),
                box,
                null,
                Array.Empty<ReferenceResolution>(),
                Array.Empty<CadFeatureEvaluationResult>()));

        var result = evaluation.AuthoritativeResult!;
        var reference = new CadReference(
            new CadId("support"),
            ReferenceKind.Support,
            new CadId("box"),
            TopologySelector.PlanarFaceByNormalAndPoint(
                new CadVector3(0d, 0d, 1d),
                new CadVector3(1d, 1.5d, 4d)),
            new ReferenceContext(CadFrameKind.Part, "default", result.ResultId));

        var resolution = new AuthoritativeCadReferenceResolver().Resolve(reference, result);

        Assert.Equal(ReferenceResolutionStatus.Resolved, resolution.Status);
        Assert.Equal("f_top", resolution.Candidates.Single().Value);
    }

    [Fact]
    public void DuplicateSemanticFaceEvidenceFailsClosedAsAmbiguous()
    {
        var evaluator = new RustCadKernelEvaluator(
            new RecordingBoxGeometryService(
                new AxisAlignedBoxSolidKernelResult(
                    GeometryKernelStatus.Succeeded,
                    new ContractResultId("solid:box-003"),
                    "evidence:box-003",
                    new[]
                    {
                        new AxisAlignedBoxSolidKernelTopology("Face", "f_bottom"),
                        new AxisAlignedBoxSolidKernelTopology("Face", "f_top"),
                        new AxisAlignedBoxSolidKernelTopology("Face", "f_back"),
                        new AxisAlignedBoxSolidKernelTopology("Face", "f_front"),
                        new AxisAlignedBoxSolidKernelTopology("Face", "f_left"),
                        new AxisAlignedBoxSolidKernelTopology("Face", "f_right")
                    },
                    24d,
                    52d,
                    new KernelVector3(1d, 1.5d, 2d),
                    Array.Empty<string>())));

        var feature = new BoxFeatureSpecification(
            new CadId("box"),
            new CadFrame(new CadId("part"), CadFrameKind.Part, 0d, 0d, 0d),
            2d, 3d, 4d);

        var response = await evaluator.EvaluateAsync(
            new KernelEvaluationRequest(
                new CadId("evaluation-003"),
                feature,
                null,
                Array.Empty<ReferenceResolution>(),
                Array.Empty<CadFeatureEvaluationResult>()));

        var result = response.AuthoritativeResult!;
        var duplicate = result.Topology.Entities.Single(x => x.Id.Value == "f_top") with
        {
            Id = new TopologyEntityId("f_top_duplicate")
        };

        var ambiguous = result with
        {
            Topology = new TopologySnapshot(
                result.Topology.KernelContractVersion,
                result.Topology.Entities.Append(duplicate).ToArray())
        };

        var reference = new CadReference(
            new CadId("support"),
            ReferenceKind.Support,
            feature.Id,
            TopologySelector.PlanarFaceByNormalAndPoint(
                new CadVector3(0d, 0d, 1d),
                new CadVector3(1d, 1.5d, 4d)),
            new ReferenceContext(CadFrameKind.Part, "default"));

        var resolution = new AuthoritativeCadReferenceResolver().Resolve(reference, ambiguous);

        Assert.Equal(ReferenceResolutionStatus.Ambiguous, resolution.Status);
        Assert.Equal(
            new[] { "f_top", "f_top_duplicate" },
            resolution.Candidates.Select(x => x.Value).ToArray());
    }

    [Fact]
    public async Task UnsupportedFeatureDoesNotFallBackToLegacyBuildRoute()
    {
        var evaluator = new RustCadKernelEvaluator(
            new RecordingBoxGeometryService(
                new AxisAlignedBoxSolidKernelResult(
                    GeometryKernelStatus.Succeeded,
                    new ContractResultId("solid:unused"),
                    "evidence:unused",
                    new[]
                    {
                        new AxisAlignedBoxSolidKernelTopology("Face", "f_bottom"),
                        new AxisAlignedBoxSolidKernelTopology("Face", "f_top"),
                        new AxisAlignedBoxSolidKernelTopology("Face", "f_back"),
                        new AxisAlignedBoxSolidKernelTopology("Face", "f_front"),
                        new AxisAlignedBoxSolidKernelTopology("Face", "f_left"),
                        new AxisAlignedBoxSolidKernelTopology("Face", "f_right")
                    },
                    24d,
                    52d,
                    new KernelVector3(1d, 1.5d, 2d),
                    Array.Empty<string>())));

        var feature = new SketchFeatureSpecification(
            new CadId("sketch"),
            new CadReference(
                new CadId("support"),
                ReferenceKind.Support,
                new CadId("box"),
                TopologySelector.PlanarFaceByNormalAndPoint(
                    new CadVector3(0d, 0d, 1d),
                    new CadVector3(1d, 1.5d, 4d)),
                new ReferenceContext(CadFrameKind.Part, "default")),
            new CadFrame(new CadId("sketch-frame"), CadFrameKind.Sketch, 0d, 0d, 4d),
            new[]
            {
                new SketchCircle(new CadId("circle-a"), 0d, 0d, 1d)
            },
            new[]
            {
                new SketchConstraintSpecification(
                    new CadId("fixed-a"),
                    SketchConstraintKind.Fixed,
                    new CadId("circle-a"))
            });

        var response = await evaluator.EvaluateAsync(
            new KernelEvaluationRequest(
                new CadId("evaluation-004"),
                feature,
                null,
                Array.Empty<ReferenceResolution>(),
                Array.Empty<CadFeatureEvaluationResult>()));

        Assert.Equal(CadEvaluationStatus.Unsupported, response.Status);
        Assert.Contains(response.Diagnostics, x => x.Code == "KERNEL_FEATURE_UNSUPPORTED");
    }

    [Fact]
    public async Task CadEvaluationEngineCanUseProductionAdapterForCertifiedBoxSlice()
    {
        var evaluator = new RustCadKernelEvaluator(
            new RecordingBoxGeometryService(
                new AxisAlignedBoxSolidKernelResult(
                    GeometryKernelStatus.Succeeded,
                    new ContractResultId("solid:box-005"),
                    "evidence:box-005",
                    new[]
                    {
                        new AxisAlignedBoxSolidKernelTopology("Face", "f_right"),
                        new AxisAlignedBoxSolidKernelTopology("Face", "f_top"),
                        new AxisAlignedBoxSolidKernelTopology("Face", "f_bottom"),
                        new AxisAlignedBoxSolidKernelTopology("Face", "f_left"),
                        new AxisAlignedBoxSolidKernelTopology("Face", "f_front"),
                        new AxisAlignedBoxSolidKernelTopology("Face", "f_back")
                    },
                    24d,
                    52d,
                    new KernelVector3(1d, 1.5d, 2d),
                    Array.Empty<string>())));

        var engine = new CadEvaluationEngine(evaluator);
        var result = await engine.RecomputeAsync(
            new CadDocumentSpecification(
                new CadId("document"),
                "A",
                "default",
                new CadFeatureSpecification[]
                {
                    new BoxFeatureSpecification(
                        new CadId("box"),
                        new CadFrame(new CadId("part"), CadFrameKind.Part, 0d, 0d, 0d),
                        2d, 3d, 4d)
                }));

        Assert.True(result.Succeeded, result.Failure?.Message);
        Assert.Equal("solid:box-005", result.FinalAuthoritativeResult!.ResultId.Value);
        Assert.NotNull(result.Representation);
        Assert.Equal(
            result.FinalAuthoritativeResult.ResultId,
            new CadResultId(result.Representation!.SourceResultId.Value));
    }


    [Fact]
    public async Task ProductionAdapterEvaluatesSketchThroughTypedSketchService()
    {
        var sketch = new RecordingSketchGeometryService(
            new SketchKernelResult(
                GeometryKernelStatus.Succeeded,
                new ContractResultId("sketch:001"),
                "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee",
                new[]
                {
                    new SketchCircleResult("circle-a", 20d, 20d, 5d),
                    new SketchCircleResult("circle-b", 70d, 30d, 4d)
                },
                true,
                "Converged",
                1,
                0d,
                0d,
                0d,
                0,
                6,
                6,
                Array.Empty<string>()));

        var evaluator = new RustCadKernelEvaluator(
            new RecordingBoxGeometryService(
                new AxisAlignedBoxSolidKernelResult(
                    GeometryKernelStatus.Succeeded,
                    new ContractResultId("solid:unused"),
                    "evidence:unused",
                    new[]
                    {
                        new AxisAlignedBoxSolidKernelTopology("Face", "f_bottom"),
                        new AxisAlignedBoxSolidKernelTopology("Face", "f_top"),
                        new AxisAlignedBoxSolidKernelTopology("Face", "f_back"),
                        new AxisAlignedBoxSolidKernelTopology("Face", "f_front"),
                        new AxisAlignedBoxSolidKernelTopology("Face", "f_left"),
                        new AxisAlignedBoxSolidKernelTopology("Face", "f_right")
                    },
                    24d,
                    52d,
                    new KernelVector3(1d, 1.5d, 2d),
                    Array.Empty<string>())),
            sketch);

        var specification = new SketchFeatureSpecification(
            new CadId("sketch-001"),
            new CadReference(
                new CadId("support"),
                ReferenceKind.Support,
                new CadId("base"),
                TopologySelector.PlanarFaceByNormalAndPoint(
                    new CadVector3(0d, 0d, 1d),
                    new CadVector3(1d, 1d, 10d)),
                new ReferenceContext(CadFrameKind.Part, "default")),
            new CadFrame(new CadId("sketch-frame"), CadFrameKind.Sketch, 0d, 0d, 10d),
            new[]
            {
                new SketchCircle(new CadId("circle-a"), 20d, 20d, 5d),
                new SketchCircle(new CadId("circle-b"), 70d, 30d, 4d)
            },
            new[]
            {
                new SketchConstraintSpecification(
                    new CadId("fixed-a"),
                    SketchConstraintKind.Fixed,
                    new CadId("circle-a")),
                new SketchConstraintSpecification(
                    new CadId("fixed-b"),
                    SketchConstraintKind.Fixed,
                    new CadId("circle-b"))
            });

        var response = await evaluator.EvaluateAsync(
            new KernelEvaluationRequest(
                new CadId("sketch-evaluation-001"),
                specification,
                null,
                Array.Empty<ReferenceResolution>(),
                Array.Empty<CadFeatureEvaluationResult>())
            {
                Tolerance = new KernelTolerance(1e-9, 1e-9)
            });

        Assert.Equal(CadEvaluationStatus.Succeeded, response.Status);
        Assert.Null(response.AuthoritativeResult);
        Assert.NotNull(response.SketchResult);
        Assert.Equal(new CadResultId("sketch:001"), response.SketchResult!.ResultId);
        Assert.Equal(2, response.SketchResult.Circles.Count);
        Assert.True(response.SketchResult.Converged);
        Assert.Equal(0, response.SketchResult.DegreesOfFreedom);
        Assert.Equal(new CadFrame(new CadId("sketch-frame"), CadFrameKind.Sketch, 0d, 0d, 10d), response.SketchResult.Frame);
    }

    [Fact]
    public async Task ProductionAdapterFailsClosedOnSketchGeometryIdentityMismatch()
    {
        var sketch = new RecordingSketchGeometryService(
            new SketchKernelResult(
                GeometryKernelStatus.Succeeded,
                new ContractResultId("sketch:bad"),
                "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
                new[] { new SketchCircleResult("wrong-circle", 0d, 0d, 1d) },
                true,
                "Converged",
                1,
                0d,
                0d,
                0d,
                0,
                3,
                3,
                Array.Empty<string>()));

        var evaluator = new RustCadKernelEvaluator(
            new RecordingBoxGeometryService(
                new AxisAlignedBoxSolidKernelResult(
                    GeometryKernelStatus.Succeeded,
                    new ContractResultId("solid:unused"),
                    "evidence:unused",
                    new[]
                    {
                        new AxisAlignedBoxSolidKernelTopology("Face", "f_bottom"),
                        new AxisAlignedBoxSolidKernelTopology("Face", "f_top"),
                        new AxisAlignedBoxSolidKernelTopology("Face", "f_back"),
                        new AxisAlignedBoxSolidKernelTopology("Face", "f_front"),
                        new AxisAlignedBoxSolidKernelTopology("Face", "f_left"),
                        new AxisAlignedBoxSolidKernelTopology("Face", "f_right")
                    },
                    24d,
                    52d,
                    new KernelVector3(1d, 1.5d, 2d),
                    Array.Empty<string>())),
            sketch);

        var specification = new SketchFeatureSpecification(
            new CadId("sketch-001"),
            new CadReference(
                new CadId("support"),
                ReferenceKind.Support,
                new CadId("base"),
                TopologySelector.PlanarFaceByNormalAndPoint(
                    new CadVector3(0d, 0d, 1d),
                    new CadVector3(1d, 1d, 10d)),
                new ReferenceContext(CadFrameKind.Part, "default")),
            new CadFrame(new CadId("sketch-frame"), CadFrameKind.Sketch, 0d, 0d, 10d),
            new[] { new SketchCircle(new CadId("circle-a"), 20d, 20d, 5d) },
            Array.Empty<SketchConstraintSpecification>());

        var response = await evaluator.EvaluateAsync(
            new KernelEvaluationRequest(
                new CadId("sketch-evaluation-bad"),
                specification,
                null,
                Array.Empty<ReferenceResolution>(),
                Array.Empty<CadFeatureEvaluationResult>()));

        Assert.Equal(CadEvaluationStatus.KernelFailure, response.Status);
        Assert.Null(response.SketchResult);
        Assert.Contains(response.Diagnostics, x => x.Code == "KERNEL_SKETCH_RESULT_IDENTITY_MISMATCH");
    }


    private sealed class RecordingSketchGeometryService(
        SketchKernelResult result) : ISketchGeometryService
    {
        public SketchSolveRequest? LastRequest { get; private set; }

        public Task<SketchKernelResult> SolveAsync(
            SketchSolveRequest request,
            CancellationToken cancellationToken = default)
        {
            cancellationToken.ThrowIfCancellationRequested();
            LastRequest = request;
            return Task.FromResult(result);
        }
    }

    private sealed class RecordingHttpHandler(
        Func<HttpRequestMessage, Task<HttpResponseMessage>> responder) : HttpMessageHandler
    {
        protected override Task<HttpResponseMessage> SendAsync(
            HttpRequestMessage request,
            CancellationToken cancellationToken) =>
            responder(request);
    }

    private sealed class RecordingBoxGeometryService(
        AxisAlignedBoxSolidKernelResult result) : IAuthoritativeGeometryService
    {
        public AxisAlignedBoxSolidRequest? LastRequest { get; private set; }

        public Task<AxisAlignedBoxSolidKernelResult> BuildAxisAlignedBoxSolidAsync(
            AxisAlignedBoxSolidRequest request,
            CancellationToken cancellationToken = default)
        {
            cancellationToken.ThrowIfCancellationRequested();
            LastRequest = request;
            return Task.FromResult(result);
        }
    }

}
