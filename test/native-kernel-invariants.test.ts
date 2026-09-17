import assert from "node:assert/strict";
import test from "node:test";
import { DesignProgram, geometryEndpoint, evaluateDimensions, spatialAnalysis, buildTopology, type DesignProgramSnapshot } from "../kernel/src/native-index.js";

function lineProgram() {
  const p = new DesignProgram();
  const width = p.parameter("width", 10);
  p.addGeometry("base", ({ parameters }) => ({ kind: "line", start: { x: 0, y: 0 }, end: { x: parameters.get("width")!, y: 0 } }), [width]);
  p.addGeometry("side", ({ parameters }) => ({ kind: "line", start: { x: parameters.get("width")!, y: 0 }, end: { x: parameters.get("width")!, y: 5 } }), [width]);
  return { p, width };
}

test("DesignProgram snapshots are deterministic and deeply immutable at the exposed boundary", () => {
  const { p } = lineProgram();
  const a = p.snapshot(); const b = p.snapshot();
  assert.deepEqual(a, b); assert.ok(Object.isFrozen(a)); assert.ok(Object.isFrozen(a.geometry)); assert.ok(Object.isFrozen(a.geometry[0]));
  assert.ok(Object.isFrozen(a.geometry[0]!.geometry)); assert.ok(Object.isFrozen(a.parameters)); assert.ok(Object.isFrozen(a.values));
  assert.equal(a.geometry[0]!.id, "base"); assert.equal(a.geometry[1]!.id, "side");
});

test("parameter regeneration changes dependent geometry and restores state after invalid regeneration", () => {
  const { p } = lineProgram();
  const original = p.snapshot(); const changed = p.regenerate([{ parameter: "width", value: 25 }]);
  assert.equal((changed.geometry.find(g => g.id === "base")!.geometry as { end: { x: number } }).end.x, 25);
  assert.throws(() => p.regenerate([{ parameter: "width", value: Number.NaN }]), /must be finite/);
  assert.deepEqual(p.snapshot(), changed); assert.notDeepEqual(original, changed);
});

test("dependency traversal reaches downstream geometry and constraints without duplicate nodes", () => {
  const { p, width } = lineProgram();
  p.horizontal({ kind: "geometry", entityId: "base" });
  const downstream = p.downstreamFrom({ kind: "parameter", id: width.name });
  assert.deepEqual(downstream.map(node => `${node.kind}:${node.id}`), ["geometry:base", "geometry:side", "constraint:constraint-1"]);
});

test("invalid and degenerate geometry is rejected before a semantic snapshot is accepted", () => {
  const cases = [
    () => { const p = new DesignProgram(); p.addGeometry("zero", () => ({ kind: "line", start: { x: 0, y: 0 }, end: { x: 0, y: 0 } })); p.snapshot(); },
    () => { const p = new DesignProgram(); p.addGeometry("bad", () => ({ kind: "circle", center: { x: 0, y: 0 }, radius: 0 })); p.snapshot(); },
    () => { const p = new DesignProgram(); p.addGeometry("bad", () => ({ kind: "circle", center: { x: Number.POSITIVE_INFINITY, y: 0 }, radius: 1 })); p.snapshot(); },
    () => { const p = new DesignProgram(); p.addGeometry("bad", () => ({ kind: "arc", center: { x: 0, y: 0 }, radius: 1, startAngle: 0, endAngle: 0 })); p.snapshot(); },
  ];
  for (const makeInvalid of cases) assert.throws(makeInvalid);
});

test("duplicate identifiers and unknown references fail immediately", () => {
  const p = new DesignProgram(); p.parameter("x", 1); assert.throws(() => p.parameter("x", 2), /already exists/);
  p.addGeometry("g", () => ({ kind: "line", start: { x: 0, y: 0 }, end: { x: 1, y: 0 } }));
  assert.throws(() => p.addGeometry("g", () => ({ kind: "line", start: { x: 0, y: 0 }, end: { x: 2, y: 0 } })), /already exists/);
  assert.throws(() => p.horizontal({ kind: "geometry", entityId: "missing" }), /unknown geometry/);
  assert.throws(() => p.setParameter("missing", 4), /Unknown parameter/);
});

test("circle topology is a closed edge without endpoint vertices and line topology preserves incidence", () => {
  const p = new DesignProgram();
  p.addGeometry("circle", () => ({ kind: "circle", center: { x: 5, y: 5 }, radius: 2 }));
  p.addGeometry("a", () => ({ kind: "line", start: { x: 0, y: 0 }, end: { x: 1, y: 0 } }));
  p.addGeometry("b", () => ({ kind: "line", start: { x: 1, y: 0 }, end: { x: 1, y: 1 } }));
  const topology = buildTopology(p.snapshot()); const circle = topology.edges.find(e => e.geometryId === "circle")!;
  assert.equal(circle.closed, true); assert.equal(circle.startVertexId, null); assert.equal(circle.endVertexId, null);
  assert.equal(topology.edges.filter(e => e.geometryId === "a" || e.geometryId === "b").length, 2);
});

test("geometryEndpoint distinguishes true endpoints from center semantics", () => {
  const line = { kind: "line" as const, start: { x: 1, y: 2 }, end: { x: 3, y: 4 } };
  assert.deepEqual(geometryEndpoint(line, "start"), line.start); assert.deepEqual(geometryEndpoint(line, "end"), line.end);
  const circle = { kind: "circle" as const, center: { x: 4, y: 5 }, radius: 2 };
  assert.throws(() => geometryEndpoint(circle, "start"), /no topological start\/end endpoint/);
});

test("analytic line, circle and arc lengths are invariant under rigid translation", () => {
  const p1 = new DesignProgram();
  p1.addGeometry("line", () => ({ kind: "line", start: { x: 0, y: 0 }, end: { x: 3, y: 4 } }));
  p1.addGeometry("circle", () => ({ kind: "circle", center: { x: 0, y: 0 }, radius: 2 }));
  p1.addGeometry("arc", () => ({ kind: "arc", center: { x: 0, y: 0 }, radius: 2, startAngle: 0, endAngle: Math.PI / 2 }));
  const p2 = new DesignProgram();
  p2.addGeometry("line", () => ({ kind: "line", start: { x: 100, y: -4 }, end: { x: 103, y: 0 } }));
  p2.addGeometry("circle", () => ({ kind: "circle", center: { x: 100, y: -4 }, radius: 2 }));
  p2.addGeometry("arc", () => ({ kind: "arc", center: { x: 100, y: -4 }, radius: 2, startAngle: 0, endAngle: Math.PI / 2 }));
  const dims = (s: DesignProgramSnapshot) => evaluateDimensions(s, [
    { id: "line", kind: "length", firstGeometryId: "line" }, { id: "circle", kind: "length", firstGeometryId: "circle" }, { id: "arc", kind: "length", firstGeometryId: "arc" },
  ]);
  const d1 = dims(p1.snapshot()); const d2 = dims(p2.snapshot());
  assert.deepEqual(d1.map(x => x.value), d2.map(x => x.value));
  assert.ok(Math.abs(d1[0]!.value - 5) < 1e-12); assert.ok(Math.abs(d1[1]!.value - 4 * Math.PI) < 1e-12); assert.ok(Math.abs(d1[2]!.value - Math.PI) < 1e-12);
});

test("spatial analysis never confuses overlapping AABBs with line intersection", () => {
  const first = { id: "a", geometry: { kind: "line" as const, start: { x: 0, y: 0 }, end: { x: 10, y: 10 } } };
  const second = { id: "b", geometry: { kind: "line" as const, start: { x: 0, y: 5 }, end: { x: 10, y: 15 } } };
  const result = spatialAnalysis([first, second])[0]!;
  assert.equal(result.intersects, false); assert.equal(result.method, "exact-2d"); assert.ok(result.distance > 0);
});

test("near-touching line and circle respect explicit tolerance", () => {
  const line = { id: "line", geometry: { kind: "line" as const, start: { x: -1, y: 1 + 1e-8 }, end: { x: 1, y: 1 + 1e-8 } } };
  const circle = { id: "circle", geometry: { kind: "circle" as const, center: { x: 0, y: 0 }, radius: 1 } };
  const strict = spatialAnalysis([line, circle], 1e-12)[0]!; const loose = spatialAnalysis([line, circle], 1e-7)[0]!;
  assert.equal(strict.intersects, false); assert.equal(loose.intersects, true);
});

test("spatial analysis is symmetric under entity ordering", () => {
  const a = { id: "a", geometry: { kind: "line" as const, start: { x: 0, y: 0 }, end: { x: 4, y: 0 } } };
  const b = { id: "b", geometry: { kind: "line" as const, start: { x: 6, y: 0 }, end: { x: 10, y: 0 } } };
  const ab = spatialAnalysis([a, b])[0]!; const ba = spatialAnalysis([b, a])[0]!;
  assert.equal(ab.intersects, ba.intersects); assert.equal(ab.distance, ba.distance);
});

test("snapshot copies do not share mutable geometry objects with factory results", () => {
  const source = { start: { x: 0, y: 0 }, end: { x: 1, y: 0 } }; const p = new DesignProgram();
  p.addGeometry("g", () => ({ kind: "line", start: source.start, end: source.end })); const snapshot = p.snapshot(); source.start.x = 99;
  assert.equal((snapshot.geometry[0]!.geometry as { start: { x: number } }).start.x, 0);
});

function acceptsSnapshot(snapshot: DesignProgramSnapshot): void { assert.ok(snapshot.geometry.length >= 0); }
test("native snapshot contract remains composable across analysis APIs", () => {
  const p = new DesignProgram(); p.addGeometry("g", () => ({ kind: "line", start: { x: 0, y: 0 }, end: { x: 2, y: 0 } })); const snapshot = p.snapshot();
  acceptsSnapshot(snapshot); assert.equal(evaluateDimensions(snapshot, [{ id: "g", kind: "length", firstGeometryId: "g" }])[0]!.value, 2);
});
