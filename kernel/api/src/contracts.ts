import type {
  CadValidationResult,
  ConstraintAnalysis,
  ConstraintEvaluationState,
  DesignProgramSnapshot,
  DxfExportOptions,
  DxfIntegrationResult,
  ExtendedConstraintEvaluationState,
  GeometryQuery,
  Point2D,
  ReferenceMigrationMap,
  SemanticReference,
  SolverOptions,
  ConstraintSolveResult,
  DimensionSpec,
  EvaluatedDimension,
  SpatialRule,
  SpatialRelation,
} from "../../vendor/UMLCAD.V.4/src/index.js";

export interface BuildIdentity { readonly algorithm: "sha256"; readonly value: string; }
export interface PartBuildRequest { readonly projectId: string; readonly partId: string; readonly sourceRevision: string; readonly moduleId: string; readonly exportName: string; readonly buildProfile?: string; }
export interface PartBuildResult { readonly kind: "part-build"; readonly buildIdentity: BuildIdentity; readonly projectId: string; readonly partId: string; readonly sourceRevision: string; readonly moduleId: string; readonly exportName: string; readonly design: DesignProgramSnapshot; readonly validation: CadValidationResult; readonly authoringSettings: import("../../vendor/UMLCAD.V.4/src/index.js").ResolvedAuthoringSettings; }
export interface AssemblyTransform2D { readonly translation: Point2D; readonly rotationRadians: number; }
export interface AssemblyPartInstance { readonly instanceId: string; readonly partId: string; readonly partBuildId: BuildIdentity; readonly transform: AssemblyTransform2D; }
export interface AssemblyBuildRequest { readonly projectId: string; readonly assemblyId: string; readonly sourceRevision: string; readonly instances: readonly AssemblyPartInstance[]; }
export interface AssemblyGeometryItem { readonly id: string; readonly instanceId: string; readonly sourcePartId: string; readonly sourceGeometryId: string; readonly geometry: import("../../vendor/UMLCAD.V.4/src/index.js").Geometry; }
export interface AssemblyBuildResult { readonly kind: "assembly-build"; readonly buildIdentity: BuildIdentity; readonly projectId: string; readonly assemblyId: string; readonly sourceRevision: string; readonly instances: readonly AssemblyPartInstance[]; readonly geometry: readonly AssemblyGeometryItem[]; readonly validation: CadValidationResult; }
export interface ConstraintAnalysisRequest { readonly state: ConstraintEvaluationState; readonly residualTolerance?: number; readonly rankTolerance?: number; readonly conditionTolerance?: number; }
export interface ConstraintSolveRequest { readonly state: ExtendedConstraintEvaluationState; readonly options?: SolverOptions; }
export interface DimensionEvaluationRequest { readonly state: ConstraintEvaluationState; readonly dimensions: readonly DimensionSpec[]; }
export interface SpatialAnalysisRequest { readonly geometry: readonly import("../../vendor/UMLCAD.V.4/src/index.js").GeometryItem[]; readonly rules: readonly SpatialRule[]; readonly tolerance: number; }
export interface DxfExportRequest { readonly snapshot: DesignProgramSnapshot; readonly options: DxfExportOptions; readonly dimensions?: readonly DimensionSpec[]; readonly manufacturing?: import("../../vendor/UMLCAD.V.4/src/index.js").DxfManufacturingValidationOptions; }

export type QueryKind = "design" | "validation" | "geometry" | "dependencies" | "authoring" | "layers";
export interface QueryRequest { readonly buildIdentity: BuildIdentity; readonly kind: QueryKind; readonly geometryId?: string; }
export interface GeometryQueryRequest { readonly buildIdentity: BuildIdentity; readonly geometryId: string; readonly operation: { readonly kind: "pointAt"; readonly parameter: number } | { readonly kind: "tangentAt"; readonly parameter: number } | { readonly kind: "closestPoint"; readonly point: Point2D } | { readonly kind: "parameterAt"; readonly point: Point2D; readonly tolerance?: number } | { readonly kind: "distanceTo"; readonly point: Point2D } | { readonly kind: "intersections"; readonly otherGeometryId: string } | { readonly kind: "metadata" }; }

export interface PartCache { get(identity: BuildIdentity): PartBuildResult | undefined; put(result: PartBuildResult): void; clear(): void; }
export interface AssemblyCache { get(identity: BuildIdentity): AssemblyBuildResult | undefined; put(result: AssemblyBuildResult): void; clear(): void; }
export interface PartSourceLoader { loadDrawing(moduleId: string, exportName: string): Promise<unknown>; }
export interface KernelApplication {
  buildPart(request: PartBuildRequest): Promise<PartBuildResult>;
  buildAssembly(request: AssemblyBuildRequest): Promise<AssemblyBuildResult>;
  analyzeConstraints(request: ConstraintAnalysisRequest): Promise<ConstraintAnalysis>;
  solveConstraints(request: ConstraintSolveRequest): Promise<ConstraintSolveResult>;
  evaluateDimensions(request: DimensionEvaluationRequest): Promise<readonly EvaluatedDimension[]>;
  analyzeSpatial(request: SpatialAnalysisRequest): Promise<{ readonly diagnostics: readonly import("../../vendor/UMLCAD.V.4/src/index.js").SpatialRuleDiagnostic[]; readonly relations: readonly { readonly firstGeometryId: string; readonly secondGeometryId: string; readonly relation: SpatialRelation; readonly distance: number }[] }>;
  resolveReference(request: { readonly source: SemanticReference; readonly migration: ReferenceMigrationMap }): Promise<readonly SemanticReference[]>;
  exportDxf(request: DxfExportRequest): Promise<DxfIntegrationResult>;
  query(request: QueryRequest): Promise<unknown>;
  geometryQuery(request: GeometryQueryRequest): Promise<unknown>;
}

export function isDrawingLike(value: unknown): value is { finalBuild: unknown } { return typeof value === "object" && value !== null && typeof (value as { finalBuild?: unknown }).finalBuild === "function"; }
export type GeometryQueryFactory = (geometry: NonNullable<DesignProgramSnapshot["geometry"]>[number]["geometry"]) => GeometryQuery;
