import { createHash } from "node:crypto";
import { DesignProgram, queryGeometry, validateSnapshot, type DesignProgramSnapshot, type Geometry, type GeometryFactory, type ParameterHandle, type Point2D } from "./core-v5.js";

export interface BuildIdentity { readonly algorithm: "sha256"; readonly value: string; }
export interface PartDefinitionContext {
  parameter(id: string, defaultValue: number): ParameterHandle;
  addGeometry(id: string, factory: GeometryFactory, dependencies?: readonly ParameterHandle[]): void;
  horizontal(entity: { readonly kind: "geometry"; readonly entityId: string }): string;
  vertical(entity: { readonly kind: "geometry"; readonly entityId: string }): string;
  coincident(firstGeometryId: string, firstPoint: "start" | "end", secondGeometryId: string, secondPoint: "start" | "end"): string;
  fixed(entityId: string): string;
  distance(firstGeometryId: string, value: number, secondGeometryId?: string): string;
}
export type PartDefinition = (program: PartDefinitionContext) => void;
export interface KernelDiagnostic { readonly code: string; readonly message: string; readonly severity: "error" | "warning"; }
export interface PartBuild { readonly kind: "part-build"; readonly buildIdentity: BuildIdentity; readonly projectId: string; readonly partId: string; readonly sourceRevision: string; readonly snapshot: DesignProgramSnapshot; readonly diagnostics: readonly KernelDiagnostic[]; }
export interface PartInstance { readonly instanceId: string; readonly partId: string; readonly partBuildId: BuildIdentity; readonly transform: { readonly translation: Point2D; readonly rotationRadians: number }; }
export interface AssemblyGeometryItem { readonly id: string; readonly instanceId: string; readonly sourcePartId: string; readonly sourceGeometryId: string; readonly geometry: Geometry; }
export interface AssemblyBuild { readonly kind: "assembly-build"; readonly buildIdentity: BuildIdentity; readonly projectId: string; readonly assemblyId: string; readonly sourceRevision: string; readonly instances: readonly PartInstance[]; readonly geometry: readonly AssemblyGeometryItem[]; readonly diagnostics: readonly KernelDiagnostic[]; }

interface StoredPart { readonly identity: BuildIdentity; readonly program: DesignProgram; readonly projectId: string; readonly partId: string; readonly sourceRevision: string; }

export class Kernel {
  private readonly parts = new Map<string, StoredPart>();
  private readonly assemblies = new Map<string, AssemblyBuild>();

  public buildPart(projectId: string, partId: string, sourceRevision: string, definition: PartDefinition, buildProfile = "default"): PartBuild {
    requireIdentifier(projectId, "projectId"); requireIdentifier(partId, "partId"); requireIdentifier(sourceRevision, "sourceRevision");
    const identity = buildIdentity({ kind: "part", projectId, partId, sourceRevision, buildProfile, kernel: "v5-native-1" });
    const cached = this.parts.get(identity.value); if (cached) return this.result(cached);
    const program = new DesignProgram(); definition(program);
    const stored: StoredPart = Object.freeze({ identity, program, projectId, partId, sourceRevision }); this.parts.set(identity.value, stored);
    return this.result(stored);
  }

  public regeneratePart(baseBuildId: BuildIdentity, changes: readonly { readonly parameter: string; readonly value: number }[], projectId: string, partId: string, sourceRevision: string, buildProfile = "default"): PartBuild {
    const base = this.parts.get(baseBuildId.value); if (!base) throw new Error(`Unknown part build: ${baseBuildId.value}`);
    const snapshot = base.program.regenerate(changes);
    const identity = buildIdentity({ kind: "part", projectId, partId, sourceRevision, buildProfile, baseBuildId, snapshot });
    const stored: StoredPart = Object.freeze({ identity, program: base.program, projectId, partId, sourceRevision }); this.parts.set(identity.value, stored);
    return this.result(stored, snapshot);
  }

  public buildAssembly(projectId: string, assemblyId: string, sourceRevision: string, instances: readonly PartInstance[]): AssemblyBuild {
    requireIdentifier(projectId, "projectId"); requireIdentifier(assemblyId, "assemblyId"); requireIdentifier(sourceRevision, "sourceRevision");
    const seen = new Set<string>();
    for (const instance of instances) {
      if (seen.has(instance.instanceId)) throw new Error(`Duplicate assembly instance: ${instance.instanceId}`); seen.add(instance.instanceId);
      if (!this.parts.has(instance.partBuildId.value)) throw new Error(`Unknown part build: ${instance.partBuildId.value}`);
      if (![instance.transform.rotationRadians, instance.transform.translation.x, instance.transform.translation.y].every(Number.isFinite)) throw new Error(`Invalid transform for assembly instance: ${instance.instanceId}`);
    }
    const identity = buildIdentity({ kind: "assembly", projectId, assemblyId, sourceRevision, instances });
    const cached = this.assemblies.get(identity.value); if (cached) return cached;
    const geometry: AssemblyGeometryItem[] = []; const diagnostics: KernelDiagnostic[] = [];
    for (const instance of instances) {
      const stored = this.parts.get(instance.partBuildId.value)!; const snapshot = stored.program.snapshot();
      diagnostics.push(...validateSnapshot(snapshot).map((item) => ({ ...item, message: `${instance.instanceId}: ${item.message}` })));
      for (const item of snapshot.geometry) geometry.push(Object.freeze({ id: `${instance.instanceId}:${item.id}`, instanceId: instance.instanceId, sourcePartId: instance.partId, sourceGeometryId: item.id, geometry: transformGeometry(item.geometry, instance.transform) }));
    }
    const result: AssemblyBuild = Object.freeze({ kind: "assembly-build", buildIdentity: identity, projectId, assemblyId, sourceRevision, instances: Object.freeze(instances.map(cloneInstance)), geometry: Object.freeze(geometry), diagnostics: Object.freeze(diagnostics) });
    this.assemblies.set(identity.value, result); return result;
  }

  public queryGeometry(buildId: BuildIdentity, geometryId: string, operation: GeometryQueryOperation): unknown {
    const stored = this.parts.get(buildId.value); if (!stored) throw new Error(`Unknown part build: ${buildId.value}`);
    const item = stored.program.snapshot().geometry.find((candidate) => candidate.id === geometryId); if (!item) throw new Error(`Unknown geometry: ${geometryId}`);
    const query = queryGeometry(item.geometry);
    switch (operation.kind) {
      case "metadata": return { start: query.start, end: query.end, parameterDomain: query.parameterDomain };
      case "pointAt": return query.pointAt(operation.parameter);
      case "tangentAt": return query.tangentAt(operation.parameter);
      case "closestPoint": return query.closestPoint(operation.point);
      case "parameterAt": return query.parameterAt(operation.point, operation.tolerance);
      case "distanceTo": return query.distanceTo(operation.point);
    }
  }

  public clear(): void { this.parts.clear(); this.assemblies.clear(); }
  private result(stored: StoredPart, snapshot = stored.program.snapshot()): PartBuild { return Object.freeze({ kind: "part-build", buildIdentity: stored.identity, projectId: stored.projectId, partId: stored.partId, sourceRevision: stored.sourceRevision, snapshot, diagnostics: Object.freeze(validateSnapshot(snapshot)) }); }
}

export type GeometryQueryOperation =
  | { readonly kind: "metadata" }
  | { readonly kind: "pointAt"; readonly parameter: number }
  | { readonly kind: "tangentAt"; readonly parameter: number }
  | { readonly kind: "closestPoint"; readonly point: Point2D }
  | { readonly kind: "parameterAt"; readonly point: Point2D; readonly tolerance?: number }
  | { readonly kind: "distanceTo"; readonly point: Point2D };

function transformGeometry(geometry: Geometry, transform: PartInstance["transform"]): Geometry {
  const c = Math.cos(transform.rotationRadians); const s = Math.sin(transform.rotationRadians);
  const apply = (point: Point2D): Point2D => ({ x: c * point.x - s * point.y + transform.translation.x, y: s * point.x + c * point.y + transform.translation.y });
  if (geometry.kind === "line") return { kind: "line", start: apply(geometry.start), end: apply(geometry.end) };
  if (geometry.kind === "circle") return { kind: "circle", center: apply(geometry.center), radius: geometry.radius };
  return { kind: "arc", center: apply(geometry.center), radius: geometry.radius, startAngle: geometry.startAngle + transform.rotationRadians, endAngle: geometry.endAngle + transform.rotationRadians };
}
function cloneInstance(instance: PartInstance): PartInstance { return Object.freeze({ instanceId: instance.instanceId, partId: instance.partId, partBuildId: { ...instance.partBuildId }, transform: { rotationRadians: instance.transform.rotationRadians, translation: { ...instance.transform.translation } } }); }
function requireIdentifier(value: string, label: string): void { if (!value.trim()) throw new Error(`${label} must not be empty`); }
function buildIdentity(input: unknown): BuildIdentity { return Object.freeze({ algorithm: "sha256", value: createHash("sha256").update(canonical(input)).digest("hex") }); }
function canonical(value: unknown): string { if (value === null || typeof value !== "object") return JSON.stringify(value); if (Array.isArray(value)) return `[${value.map(canonical).join(",")}]`; const record = value as Record<string, unknown>; return `{${Object.keys(record).sort().map((key) => `${JSON.stringify(key)}:${canonical(record[key])}`).join(",")}}`; }
