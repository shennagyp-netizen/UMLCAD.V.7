export interface Point2D { readonly x: number; readonly y: number; }
export interface LineGeometry { readonly kind: "line"; readonly start: Point2D; readonly end: Point2D; }
export interface CircleGeometry { readonly kind: "circle"; readonly center: Point2D; readonly radius: number; }
export interface ArcGeometry { readonly kind: "arc"; readonly center: Point2D; readonly radius: number; readonly startAngle: number; readonly endAngle: number; }
export type Geometry = LineGeometry | CircleGeometry | ArcGeometry;

export interface ParameterDefinition { readonly id: string; readonly defaultValue: number; }
export interface ParameterHandle { readonly name: string; }
export interface GeometryContext { readonly parameters: ReadonlyMap<string, number>; }
export type GeometryFactory = (context: GeometryContext) => Geometry;
export interface GeometryItem { readonly id: string; readonly geometry: Geometry; readonly parameterDependencies: readonly string[]; }

export type Constraint =
  | { readonly kind: "horizontal"; readonly entityId: string }
  | { readonly kind: "vertical"; readonly entityId: string }
  | { readonly kind: "coincident"; readonly firstGeometryId: string; readonly firstPoint: "start" | "end"; readonly secondGeometryId: string; readonly secondPoint: "start" | "end" }
  | { readonly kind: "fixed"; readonly entityId: string }
  | { readonly kind: "distance"; readonly firstGeometryId: string; readonly secondGeometryId?: string; readonly value: number };

export type DependencyNode =
  | { readonly kind: "parameter"; readonly id: string }
  | { readonly kind: "geometry"; readonly id: string }
  | { readonly kind: "constraint"; readonly id: string };
export interface DependencyEdge { readonly from: DependencyNode; readonly to: DependencyNode; }
export interface DesignProgramSnapshot {
  readonly parameters: readonly ParameterDefinition[];
  readonly values: Readonly<Record<string, number>>;
  readonly geometry: readonly GeometryItem[];
  readonly constraints: readonly { readonly id: string; readonly constraint: Constraint }[];
  readonly dependencyGraph: readonly DependencyEdge[];
}

const EPSILON = 1e-9;
const finite = (value: number, label: string): void => { if (!Number.isFinite(value)) throw new Error(`${label} must be finite`); };
const distance = (a: Point2D, b: Point2D): number => Math.hypot(b.x - a.x, b.y - a.y);
const copyGeometry = (geometry: Geometry): Geometry => geometry.kind === "line"
  ? { kind: "line", start: { ...geometry.start }, end: { ...geometry.end } }
  : geometry.kind === "circle"
    ? { kind: "circle", center: { ...geometry.center }, radius: geometry.radius }
    : { kind: "arc", center: { ...geometry.center }, radius: geometry.radius, startAngle: geometry.startAngle, endAngle: geometry.endAngle };

function validateGeometry(geometry: Geometry, id: string): void {
  if (geometry.kind === "line") {
    finite(geometry.start.x, `${id}.start.x`); finite(geometry.start.y, `${id}.start.y`);
    finite(geometry.end.x, `${id}.end.x`); finite(geometry.end.y, `${id}.end.y`);
    if (distance(geometry.start, geometry.end) <= EPSILON) throw new Error(`Degenerate line geometry: ${id}`);
    return;
  }
  finite(geometry.center.x, `${id}.center.x`); finite(geometry.center.y, `${id}.center.y`); finite(geometry.radius, `${id}.radius`);
  if (geometry.radius <= EPSILON) throw new Error(`Invalid radius for geometry: ${id}`);
  if (geometry.kind === "arc") {
    finite(geometry.startAngle, `${id}.startAngle`); finite(geometry.endAngle, `${id}.endAngle`);
    if (Math.abs(geometry.endAngle - geometry.startAngle) <= EPSILON) throw new Error(`Degenerate arc geometry: ${id}`);
  }
}

export class DependencyGraph {
  private readonly edges = new Map<string, DependencyEdge>();
  public add(from: DependencyNode, to: DependencyNode): void {
    const key = `${from.kind}:${from.id}->${to.kind}:${to.id}`;
    this.edges.set(key, Object.freeze({ from: { ...from }, to: { ...to } }));
  }
  public downstreamFrom(node: DependencyNode): readonly DependencyNode[] {
    const result: DependencyNode[] = []; const queue: DependencyNode[] = [node]; const seen = new Set<string>();
    while (queue.length > 0) {
      const current = queue.shift()!;
      for (const edge of this.edges.values()) {
        if (edge.from.kind !== current.kind || edge.from.id !== current.id) continue;
        const key = `${edge.to.kind}:${edge.to.id}`;
        if (seen.has(key)) continue;
        seen.add(key); result.push(edge.to); queue.push(edge.to);
      }
    }
    return Object.freeze(result.map((item) => ({ ...item })));
  }
  public snapshot(): readonly DependencyEdge[] {
    return Object.freeze([...this.edges.values()].sort((a, b) => edgeKey(a).localeCompare(edgeKey(b))).map((edge) => Object.freeze({ from: { ...edge.from }, to: { ...edge.to } })));
  }
}
const edgeKey = (edge: DependencyEdge): string => `${edge.from.kind}:${edge.from.id}->${edge.to.kind}:${edge.to.id}`;

export class DesignProgram {
  private readonly definitions = new Map<string, ParameterDefinition>();
  private readonly values = new Map<string, number>();
  private readonly geometries = new Map<string, { readonly factory: GeometryFactory; readonly dependencies: readonly string[] }>();
  private readonly constraintsById = new Map<string, Constraint>();
  private readonly graph = new DependencyGraph();
  private nextConstraintId = 1;

  public parameter(id: string, defaultValue: number): ParameterHandle {
    requireId(id, "parameter"); finite(defaultValue, `Parameter ${id}`);
    if (this.definitions.has(id)) throw new Error(`Parameter already exists: ${id}`);
    this.definitions.set(id, Object.freeze({ id, defaultValue })); this.values.set(id, defaultValue);
    return Object.freeze({ name: id });
  }
  public setParameter(handleOrId: ParameterHandle | string, value: number): void {
    const id = typeof handleOrId === "string" ? handleOrId : handleOrId.name;
    if (!this.definitions.has(id)) throw new Error(`Unknown parameter: ${id}`); finite(value, `Parameter ${id}`); this.values.set(id, value);
  }
  public addGeometry(id: string, factory: GeometryFactory, dependencies: readonly ParameterHandle[] = []): void {
    requireId(id, "geometry");
    if (this.geometries.has(id)) throw new Error(`Geometry already exists: ${id}`);
    const ids = [...new Set(dependencies.map((handle) => handle.name))].sort();
    for (const dependency of ids) if (!this.definitions.has(dependency)) throw new Error(`Unknown geometry parameter dependency: ${dependency}`);
    this.geometries.set(id, Object.freeze({ factory, dependencies: ids }));
    for (const dependency of ids) this.graph.add({ kind: "parameter", id: dependency }, { kind: "geometry", id });
  }
  public horizontal(entity: { readonly kind: "geometry"; readonly entityId: string }): string { return this.addConstraint({ kind: "horizontal", entityId: entity.entityId }); }
  public vertical(entity: { readonly kind: "geometry"; readonly entityId: string }): string { return this.addConstraint({ kind: "vertical", entityId: entity.entityId }); }
  public coincident(firstGeometryId: string, firstPoint: "start" | "end", secondGeometryId: string, secondPoint: "start" | "end"): string { return this.addConstraint({ kind: "coincident", firstGeometryId, firstPoint, secondGeometryId, secondPoint }); }
  public fixed(entityId: string): string { return this.addConstraint({ kind: "fixed", entityId }); }
  public distance(firstGeometryId: string, value: number, secondGeometryId?: string): string {
    finite(value, "Distance"); if (value < 0) throw new Error("Distance cannot be negative");
    return this.addConstraint({ kind: "distance", firstGeometryId, ...(secondGeometryId === undefined ? {} : { secondGeometryId }), value });
  }
  private addConstraint(constraint: Constraint): string {
    const id = `constraint-${this.nextConstraintId++}`; this.validateConstraintReferences(id, constraint);
    this.constraintsById.set(id, Object.freeze({ ...constraint }));
    for (const geometryId of constraintGeometryIds(constraint)) this.graph.add({ kind: "geometry", id: geometryId }, { kind: "constraint", id });
    return id;
  }
  public evaluate(): DesignProgramSnapshot {
    const geometry: GeometryItem[] = [];
    for (const [id, definition] of [...this.geometries.entries()].sort(([a], [b]) => a.localeCompare(b))) {
      const item = definition.factory({ parameters: new Map(this.values) }); validateGeometry(item, id);
      geometry.push(Object.freeze({ id, geometry: Object.freeze(copyGeometry(item)), parameterDependencies: Object.freeze([...definition.dependencies]) }));
    }
    const geometryIds = new Set(geometry.map((item) => item.id));
    for (const [id, constraint] of this.constraintsById) for (const geometryId of constraintGeometryIds(constraint)) if (!geometryIds.has(geometryId)) throw new Error(`Constraint ${id} references unknown geometry: ${geometryId}`);
    const snapshot: DesignProgramSnapshot = Object.freeze({
      parameters: Object.freeze([...this.definitions.values()].sort((a, b) => a.id.localeCompare(b.id)).map((item) => ({ ...item }))),
      values: Object.freeze(Object.fromEntries([...this.values.entries()].sort(([a], [b]) => a.localeCompare(b)))),
      geometry: Object.freeze(geometry),
      constraints: Object.freeze([...this.constraintsById.entries()].sort(([a], [b]) => a.localeCompare(b)).map(([id, constraint]) => Object.freeze({ id, constraint: { ...constraint } }))),
      dependencyGraph: this.graph.snapshot(),
    });
    detectCycle(snapshot.dependencyGraph); return snapshot;
  }
  public snapshot(): DesignProgramSnapshot { return this.evaluate(); }
  public downstreamFrom(node: DependencyNode): readonly DependencyNode[] { return this.graph.downstreamFrom(node); }
  public regenerate(changes: readonly { readonly parameter: string; readonly value: number }[]): DesignProgramSnapshot {
    const previous = new Map(this.values);
    try { for (const change of changes) this.setParameter(change.parameter, change.value); return this.evaluate(); }
    catch (error) { this.values.clear(); for (const [id, value] of previous) this.values.set(id, value); throw error; }
  }
  private validateConstraintReferences(id: string, constraint: Constraint): void {
    for (const geometryId of constraintGeometryIds(constraint)) if (!this.geometries.has(geometryId)) throw new Error(`Constraint ${id} references unknown geometry: ${geometryId}`);
  }
}

function constraintGeometryIds(constraint: Constraint): readonly string[] {
  switch (constraint.kind) {
    case "horizontal": case "vertical": case "fixed": return [constraint.entityId];
    case "coincident": return [constraint.firstGeometryId, constraint.secondGeometryId];
    case "distance": return constraint.secondGeometryId === undefined ? [constraint.firstGeometryId] : [constraint.firstGeometryId, constraint.secondGeometryId];
  }
}
function requireId(id: string, kind: string): void { if (!id.trim() || id.includes("/")) throw new Error(`${kind} id is invalid`); }
function detectCycle(edges: readonly DependencyEdge[]): void {
  const adjacency = new Map<string, string[]>();
  for (const edge of edges) { const from = nodeKey(edge.from); const values = adjacency.get(from) ?? []; values.push(nodeKey(edge.to)); adjacency.set(from, values); }
  const visiting = new Set<string>(); const visited = new Set<string>();
  const visit = (node: string): void => { if (visiting.has(node)) throw new Error(`Dependency cycle detected at ${node}`); if (visited.has(node)) return; visiting.add(node); for (const child of adjacency.get(node) ?? []) visit(child); visiting.delete(node); visited.add(node); };
  for (const node of adjacency.keys()) visit(node);
}
const nodeKey = (node: DependencyNode): string => `${node.kind}:${node.id}`;

export interface GeometryQuery { readonly start: Point2D; readonly end: Point2D; readonly parameterDomain: readonly [number, number]; pointAt(parameter: number): Point2D; tangentAt(parameter: number): Point2D; closestPoint(point: Point2D): Point2D; parameterAt(point: Point2D, tolerance?: number): number; distanceTo(point: Point2D): number; }
const normalize = (point: Point2D): Point2D => { const length = Math.hypot(point.x, point.y); if (length <= EPSILON) throw new Error("Cannot normalize zero vector"); return { x: point.x / length, y: point.y / length }; };
const dot = (a: Point2D, b: Point2D): number => a.x * b.x + a.y * b.y;

function lineQuery(geometry: LineGeometry): GeometryQuery {
  const delta = { x: geometry.end.x - geometry.start.x, y: geometry.end.y - geometry.start.y }; const lengthSquared = dot(delta, delta);
  const projection = (point: Point2D): number => dot({ x: point.x - geometry.start.x, y: point.y - geometry.start.y }, delta) / lengthSquared;
  const pointAt = (parameter: number): Point2D => ({ x: geometry.start.x + delta.x * parameter, y: geometry.start.y + delta.y * parameter });
  return { start: { ...geometry.start }, end: { ...geometry.end }, parameterDomain: Object.freeze([0, 1]), pointAt, tangentAt: () => normalize(delta), closestPoint: (point) => pointAt(Math.max(0, Math.min(1, projection(point)))), parameterAt: (point, tolerance = 1e-8) => { const parameter = projection(point); if (parameter < -tolerance || parameter > 1 + tolerance || distance(pointAt(parameter), point) > tolerance) throw new Error("Point is not on line segment within tolerance"); return parameter; }, distanceTo: (point) => distance(point, pointAt(Math.max(0, Math.min(1, projection(point))))) };
}

function circleOrArcQuery(geometry: CircleGeometry | ArcGeometry): GeometryQuery {
  const startAngle = geometry.kind === "arc" ? geometry.startAngle : 0;
  const span = geometry.kind === "arc" ? geometry.endAngle - geometry.startAngle : 2 * Math.PI;
  const pointAt = (parameter: number): Point2D => { const angle = startAngle + span * parameter; return { x: geometry.center.x + geometry.radius * Math.cos(angle), y: geometry.center.y + geometry.radius * Math.sin(angle) }; };
  const start = pointAt(0); const end = pointAt(1);
  const parameterAt = (point: Point2D, tolerance = 1e-8): number => { const radial = { x: point.x - geometry.center.x, y: point.y - geometry.center.y }; if (Math.abs(Math.hypot(radial.x, radial.y) - geometry.radius) > tolerance) throw new Error("Point is not on curve within tolerance"); if (geometry.kind === "circle") return ((Math.atan2(radial.y, radial.x) - startAngle) / span + 1) % 1; const rawAngle = Math.atan2(radial.y, radial.x); for (const candidate of [rawAngle, rawAngle + 2 * Math.PI, rawAngle - 2 * Math.PI]) { const parameter = (candidate - startAngle) / span; if (parameter >= -tolerance && parameter <= 1 + tolerance) return parameter; } throw new Error("Point is outside arc domain"); };
  const closestPoint = (point: Point2D): Point2D => { const radial = normalize({ x: point.x - geometry.center.x, y: point.y - geometry.center.y }); if (geometry.kind === "circle") return { x: geometry.center.x + radial.x * geometry.radius, y: geometry.center.y + radial.y * geometry.radius }; const raw = parameterAt({ x: geometry.center.x + radial.x * geometry.radius, y: geometry.center.y + radial.y * geometry.radius }); if (raw >= 0 && raw <= 1) return pointAt(raw); const a = pointAt(0); const b = pointAt(1); return distance(a, point) <= distance(b, point) ? a : b; };
  return { start, end, parameterDomain: Object.freeze([0, 1]), pointAt, tangentAt: (parameter) => { const angle = startAngle + span * parameter; const sign = span >= 0 ? 1 : -1; return normalize({ x: -Math.sin(angle) * sign, y: Math.cos(angle) * sign }); }, closestPoint, parameterAt, distanceTo: (point) => distance(point, closestPoint(point)) };
}

export function queryGeometry(geometry: Geometry): GeometryQuery { validateGeometry(geometry, "query"); return geometry.kind === "line" ? lineQuery(geometry) : circleOrArcQuery(geometry); }

export interface Diagnostic { readonly code: string; readonly message: string; readonly severity: "error" | "warning"; }
export function validateSnapshot(snapshot: DesignProgramSnapshot): readonly Diagnostic[] {
  const diagnostics: Diagnostic[] = []; const geometryIds = new Set<string>();
  for (const item of snapshot.geometry) { if (geometryIds.has(item.id)) diagnostics.push({ code: "duplicate-geometry-id", message: `Duplicate geometry id: ${item.id}`, severity: "error" }); geometryIds.add(item.id); try { validateGeometry(item.geometry, item.id); } catch (error) { diagnostics.push({ code: "invalid-geometry", message: error instanceof Error ? error.message : String(error), severity: "error" }); } }
  for (const item of snapshot.constraints) for (const geometryId of constraintGeometryIds(item.constraint)) if (!geometryIds.has(geometryId)) diagnostics.push({ code: "invalid-constraint-reference", message: `Constraint ${item.id} references unknown geometry ${geometryId}`, severity: "error" });
  return Object.freeze(diagnostics);
}
