import { createHash } from "node:crypto";
import { buildCad, StandardCadProgram, queryGeometry, validateCadStateOrdered, analyzeConstraints as v4AnalyzeConstraints, solveConstraints as v4SolveConstraints, evaluateDimensions as v4EvaluateDimensions, spatialDistance, spatialRelation, validateSpatialRules, exportDxfIntegrated, migrateSemanticReference, type Drawing, type Geometry, type GeometryQuery, type DesignProgramSnapshot, type SourceModule, parseTypeScriptModule, resolveSourceModuleDependencies, sourceModuleEvaluationOrder } from "../../../vendor/UMLCAD.V.4/src/index.js";
import type { AssemblyBuildRequest, AssemblyBuildResult, AssemblyCache, AssemblyGeometryItem, BuildIdentity, ConstraintAnalysisRequest, ConstraintSolveRequest, DimensionEvaluationRequest, DxfExportRequest, GeometryQueryRequest, KernelApplication, PartBuildRequest, PartBuildResult, PartCache, PartSourceLoader, QueryRequest, SpatialAnalysisRequest } from "./contracts.js";

function canonical(value: unknown): string {
  if (value === null || typeof value !== "object") return JSON.stringify(value);
  if (Array.isArray(value)) return `[${value.map(canonical).join(",")}]`;
  const record = value as Record<string, unknown>;
  return `{${Object.keys(record).sort().map((key) => `${JSON.stringify(key)}:${canonical(record[key])}`).join(",")}}`;
}

export function buildIdentity(input: unknown): BuildIdentity { return { algorithm: "sha256", value: createHash("sha256").update(canonical(input)).digest("hex") }; }

export class InMemoryPartCache implements PartCache {
  private readonly values = new Map<string, PartBuildResult>();
  public get(identity: BuildIdentity): PartBuildResult | undefined { return this.values.get(identity.value); }
  public put(result: PartBuildResult): void { this.values.set(result.buildIdentity.value, result); }
  public clear(): void { this.values.clear(); }
}

export class InMemoryAssemblyCache implements AssemblyCache {
  private readonly values = new Map<string, AssemblyBuildResult>();
  public get(identity: BuildIdentity): AssemblyBuildResult | undefined { return this.values.get(identity.value); }
  public put(result: AssemblyBuildResult): void { this.values.set(result.buildIdentity.value, result); }
  public clear(): void { this.values.clear(); }
}

export class V4KernelApplication implements KernelApplication {
  public constructor(private readonly sourceLoader: PartSourceLoader, private readonly partCache: PartCache = new InMemoryPartCache(), private readonly assemblyCache: AssemblyCache = new InMemoryAssemblyCache()) {}

  public async buildPart(request: PartBuildRequest): Promise<PartBuildResult> {
    this.requireIdentifier(request.projectId, "projectId"); this.requireIdentifier(request.partId, "partId"); this.requireIdentifier(request.sourceRevision, "sourceRevision"); this.requireIdentifier(request.moduleId, "moduleId"); this.requireIdentifier(request.exportName, "exportName");
    const identity = buildIdentity({ projectId: request.projectId, partId: request.partId, sourceRevision: request.sourceRevision, moduleId: request.moduleId, exportName: request.exportName, buildProfile: request.buildProfile ?? null, kernel: "UMLCAD.V4@a1cd00d058653f0367af4d60240e99ba1bc5676c" });
    const cached = this.partCache.get(identity); if (cached) return cached;
    const drawing = await this.loadAndBuildDrawing(request.moduleId, request.exportName);
    const design = drawing.designSnapshot, validation = validateCadStateOrdered(design);
    const result: PartBuildResult = Object.freeze({ kind: "part-build", buildIdentity: identity, projectId: request.projectId, partId: request.partId, sourceRevision: request.sourceRevision, moduleId: request.moduleId, exportName: request.exportName, design, validation, authoringSettings: drawing.authoringSettings });
    this.partCache.put(result); return result;
  }

  public async buildAssembly(request: AssemblyBuildRequest): Promise<AssemblyBuildResult> {
    this.requireIdentifier(request.projectId, "projectId"); this.requireIdentifier(request.assemblyId, "assemblyId"); this.requireIdentifier(request.sourceRevision, "sourceRevision");
    const keys = request.instances.map((instance) => instance.instanceId); if (new Set(keys).size !== keys.length) throw new Error("Assembly instance IDs must be unique");
    const identity = buildIdentity({ projectId: request.projectId, assemblyId: request.assemblyId, sourceRevision: request.sourceRevision, instances: request.instances });
    const cached = this.assemblyCache.get(identity); if (cached) return cached;
    const geometry: AssemblyGeometryItem[] = [];
    for (const instance of request.instances) {
      const part = this.partCache.get(instance.partBuildId); if (!part) throw new Error(`Referenced part build is unavailable: ${instance.partBuildId.value}`); if (part.partId !== instance.partId) throw new Error(`Part build ${instance.partBuildId.value} does not belong to part ${instance.partId}`);
      for (const item of part.design.geometry) geometry.push({ id: `${instance.instanceId}:${item.id}`, instanceId: instance.instanceId, sourcePartId: part.partId, sourceGeometryId: item.id, geometry: transformGeometry(item.geometry, instance.transform) });
    }
    const validationState: DesignProgramSnapshot = { parameters: {}, geometry: geometry.map((item) => ({ id: item.id, kind: "geometry", geometry: item.geometry })), provenance: [], constraints: [], relations: [], dependencyGraph: [] };
    const result: AssemblyBuildResult = Object.freeze({ kind: "assembly-build", buildIdentity: identity, projectId: request.projectId, assemblyId: request.assemblyId, sourceRevision: request.sourceRevision, instances: request.instances.map((instance) => ({ ...instance, transform: { ...instance.transform, translation: { ...instance.transform.translation } } })), geometry: Object.freeze(geometry.map((item) => Object.freeze({ ...item }))), validation: validateCadStateOrdered(validationState) });
    this.assemblyCache.put(result); return result;
  }

  public async analyzeConstraints(request: ConstraintAnalysisRequest) { return v4AnalyzeConstraints(request.state, request.residualTolerance, request.rankTolerance, request.conditionTolerance); }
  public async solveConstraints(request: ConstraintSolveRequest) { return v4SolveConstraints(request.state, request.options); }
  public async evaluateDimensions(request: DimensionEvaluationRequest) { return v4EvaluateDimensions(request.dimensions, request.state); }
  public async analyzeSpatial(request: SpatialAnalysisRequest) {
    const diagnostics = validateSpatialRules(request.geometry, request.rules, request.tolerance);
    const relations: { firstGeometryId: string; secondGeometryId: string; relation: ReturnType<typeof spatialRelation>; distance: number }[] = [];
    for (let i = 0; i < request.geometry.length; i += 1) for (let j = i + 1; j < request.geometry.length; j += 1) { const first = request.geometry[i]!, second = request.geometry[j]!; relations.push({ firstGeometryId: first.id, secondGeometryId: second.id, relation: spatialRelation(first.geometry, second.geometry, request.tolerance), distance: spatialDistance(first.geometry, second.geometry, request.tolerance) }); }
    return { diagnostics, relations };
  }
  public async resolveReference(request: { readonly source: import("../../../vendor/UMLCAD.V.4/src/index.js").SemanticReference; readonly migration: import("../../../vendor/UMLCAD.V.4/src/index.js").ReferenceMigrationMap }) { return migrateSemanticReference(request.source, request.migration); }
  public async exportDxf(request: DxfExportRequest) { return exportDxfIntegrated(request.snapshot, { export: request.options, ...(request.dimensions ? { dimensions: request.dimensions } : {}), ...(request.manufacturing ? { manufacturing: request.manufacturing } : {}) }); }

  public async query(request: QueryRequest): Promise<unknown> {
    const result = this.partCache.get(request.buildIdentity); if (!result) throw new Error(`Unknown or evicted part build: ${request.buildIdentity.value}`);
    switch (request.kind) {
      case "design": return result.design;
      case "validation": return result.validation;
      case "authoring": return result.authoringSettings;
      case "layers": return result.authoringSettings.layers;
      case "dependencies": return result.design.dependencyGraph;
      case "geometry": { if (!request.geometryId) throw new Error("geometryId is required for geometry query"); const item = result.design.geometry.find((candidate) => candidate.id === request.geometryId); if (!item) throw new Error(`Unknown geometry: ${request.geometryId}`); return item; }
    }
  }

  public async geometryQuery(request: GeometryQueryRequest): Promise<unknown> {
    const result = this.partCache.get(request.buildIdentity); if (!result) throw new Error(`Unknown or evicted part build: ${request.buildIdentity.value}`);
    const first = result.design.geometry.find((candidate) => candidate.id === request.geometryId); if (!first) throw new Error(`Unknown geometry: ${request.geometryId}`);
    return executeGeometryQuery(queryGeometry(first.geometry), request.operation, result.design.geometry);
  }

  private async loadAndBuildDrawing(moduleId: string, exportName: string): Promise<Drawing> {
    const candidate = await this.sourceLoader.loadDrawing(moduleId, exportName);
    if (typeof candidate !== "object" || candidate === null || typeof (candidate as { finalBuild?: unknown }).finalBuild !== "function") throw new Error(`Source export is not a V4 Drawing/Sheet buildable object: ${moduleId}#${exportName}`);
    return buildCad(candidate as Drawing, StandardCadProgram);
  }
  private requireIdentifier(value: string, name: string): void { if (!value.trim()) throw new Error(`${name} must not be empty`); }
}

function transformGeometry(geometry: Geometry, transform: { translation: { x: number; y: number }; rotationRadians: number }): Geometry {
  if (!Number.isFinite(transform.rotationRadians) || !Number.isFinite(transform.translation.x) || !Number.isFinite(transform.translation.y)) throw new Error("Assembly transform values must be finite");
  const c = Math.cos(transform.rotationRadians), s = Math.sin(transform.rotationRadians), apply = (point: { x: number; y: number }) => ({ x: c * point.x - s * point.y + transform.translation.x, y: s * point.x + c * point.y + transform.translation.y });
  if (geometry.kind === "line") return { kind: "line", start: apply(geometry.start), end: apply(geometry.end) };
  if (geometry.kind === "circle") return { kind: "circle", center: apply(geometry.center), radius: geometry.radius };
  return { kind: "arc", center: apply(geometry.center), radius: geometry.radius, startAngle: geometry.startAngle + transform.rotationRadians, endAngle: geometry.endAngle + transform.rotationRadians };
}

function executeGeometryQuery(query: GeometryQuery, operation: GeometryQueryRequest["operation"], geometry: DesignProgramSnapshot["geometry"]): unknown {
  switch (operation.kind) {
    case "metadata": return { start: query.start, end: query.end, boundingBox: query.boundingBox, parameterDomain: query.parameterDomain };
    case "pointAt": return query.pointAt(operation.parameter);
    case "tangentAt": return query.tangentAt(operation.parameter);
    case "closestPoint": return query.closestPoint(operation.point);
    case "parameterAt": return query.parameterAt(operation.point, operation.tolerance);
    case "distanceTo": return query.distanceTo(operation.point);
    case "intersections": { const other = geometry.find((candidate) => candidate.id === operation.otherGeometryId); if (!other) throw new Error(`Unknown geometry: ${operation.otherGeometryId}`); return query.intersections(queryGeometry(other.geometry)); }
  }
}

export function analyzeSourceModule(id: string, sourceText: string): ReturnType<typeof parseTypeScriptModule> { return parseTypeScriptModule(id, sourceText); }
export function orderSourceModules(modules: readonly SourceModule[]): readonly string[] { return sourceModuleEvaluationOrder(modules.map((module) => module.id), resolveSourceModuleDependencies(modules)); }
