import assert from "node:assert/strict";
import test from "node:test";
import { DesignProgram, appendRelations, evaluateRelation, isGeometricRelation, validateRelation, type GeometricRelation, type RelationPoint } from "../kernel/api/src/native-index.js";

function relationFixture() {
  const p = new DesignProgram();
  p.addGeometry("h1", () => ({ kind: "line", start: { x: 0, y: 0 }, end: { x: 10, y: 0 } }));
  p.addGeometry("h2", () => ({ kind: "line", start: { x: 0, y: 5 }, end: { x: 10, y: 5 } }));
  p.addGeometry("collinear", () => ({ kind: "line", start: { x: 2, y: 0 }, end: { x: 8, y: 0 } }));
  p.addGeometry("v1", () => ({ kind: "line", start: { x: 0, y: 0 }, end: { x: 0, y: 10 } }));
  p.addGeometry("diag", () => ({ kind: "line", start: { x: 0, y: 0 }, end: { x: 10, y: 10 } }));
  p.addGeometry("equal", () => ({ kind: "line", start: { x: 0, y: 20 }, end: { x: 6, y: 28 } }));
  p.addGeometry("circleA", () => ({ kind: "circle", center: { x: 0, y: 30 }, radius: 5 }));
  p.addGeometry("circleB", () => ({ kind: "circle", center: { x: 0, y: 30 }, radius: 5 }));
  return p.snapshot();
}
function point(geometryId: string, pointName: "start" | "end"): RelationPoint { return { kind: "endpoint", geometryId, point: pointName }; }

test("every supported geometric relation kind is recognized", () => {
  const relations: readonly GeometricRelation[] = [
    { kind: "parallel", firstGeometryId: "a", secondGeometryId: "b" }, { kind: "perpendicular", firstGeometryId: "a", secondGeometryId: "b" },
    { kind: "equal-length", firstGeometryId: "a", secondGeometryId: "b" }, { kind: "angle", firstGeometryId: "a", secondGeometryId: "b", radians: 0 },
    { kind: "collinear", firstGeometryId: "a", secondGeometryId: "b" }, { kind: "concentric", firstGeometryId: "a", secondGeometryId: "b" },
    { kind: "equal-radius", firstGeometryId: "a", secondGeometryId: "b" }, { kind: "radius", geometryId: "a", value: 1 },
    { kind: "diameter", geometryId: "a", value: 2 }, { kind: "tangent", firstGeometryId: "a", secondGeometryId: "b" },
    { kind: "midpoint", point: point("a", "start"), lineGeometryId: "b" }, { kind: "point-on-line", point: point("a", "start"), lineGeometryId: "b" },
    { kind: "point-on-circle", point: point("a", "start"), circleGeometryId: "b" }, { kind: "distance-points", first: point("a", "start"), second: point("b", "end"), value: 1 },
    { kind: "symmetric", first: point("a", "start"), second: point("b", "end"), about: { kind: "center", geometryId: "c" } },
  ];
  for (const relation of relations) assert.equal(isGeometricRelation(relation), true, relation.kind);
  assert.equal(isGeometricRelation({ kind: "bogus" }), false);
});

test("parallel, perpendicular, equal-length, angle and collinear residuals are zero for exact geometry", () => {
  const s = relationFixture();
  const exact: readonly GeometricRelation[] = [
    { kind: "parallel", firstGeometryId: "h1", secondGeometryId: "h2" }, { kind: "perpendicular", firstGeometryId: "h1", secondGeometryId: "v1" },
    { kind: "equal-length", firstGeometryId: "h1", secondGeometryId: "equal" }, { kind: "angle", firstGeometryId: "diag", secondGeometryId: "h1", radians: -Math.PI / 4 },
    { kind: "collinear", firstGeometryId: "h1", secondGeometryId: "collinear" },
  ];
  for (const relation of exact) assert.ok(evaluateRelation(s, relation).scaledNorm < 1e-12, relation.kind);
});

test("circle relations distinguish concentricity, equal radius, radius and diameter", () => {
  const s = relationFixture();
  const relations: readonly GeometricRelation[] = [
    { kind: "concentric", firstGeometryId: "circleA", secondGeometryId: "circleB" }, { kind: "equal-radius", firstGeometryId: "circleA", secondGeometryId: "circleB" },
    { kind: "radius", geometryId: "circleA", value: 5 }, { kind: "diameter", geometryId: "circleA", value: 10 },
  ];
  for (const relation of relations) assert.ok(evaluateRelation(s, relation).scaledNorm < 1e-12, relation.kind);
});

test("tangency covers line-circle and external/internal circle-circle cases", () => {
  const p = new DesignProgram();
  p.addGeometry("line", () => ({ kind: "line", start: { x: -10, y: 5 }, end: { x: 10, y: 5 } }));
  p.addGeometry("outer", () => ({ kind: "circle", center: { x: 0, y: 0 }, radius: 5 }));
  p.addGeometry("inner", () => ({ kind: "circle", center: { x: 3, y: 0 }, radius: 2 }));
  p.addGeometry("external", () => ({ kind: "circle", center: { x: 7, y: 0 }, radius: 2 }));
  const s = p.snapshot();
  assert.ok(evaluateRelation(s, { kind: "tangent", firstGeometryId: "line", secondGeometryId: "outer", mode: "external" }).scaledNorm < 1e-12);
  assert.ok(evaluateRelation(s, { kind: "tangent", firstGeometryId: "outer", secondGeometryId: "inner", mode: "internal" }).scaledNorm < 1e-12);
  assert.ok(evaluateRelation(s, { kind: "tangent", firstGeometryId: "outer", secondGeometryId: "external", mode: "external" }).scaledNorm < 1e-12);
});

test("point-based relations cover midpoint, incidence, point-on-circle and distance", () => {
  const p = new DesignProgram();
  p.addGeometry("target", () => ({ kind: "line", start: { x: 0, y: 0 }, end: { x: 10, y: 0 } }));
  p.addGeometry("pointCarrier", () => ({ kind: "line", start: { x: 5, y: 0 }, end: { x: 5, y: 5 } }));
  p.addGeometry("circle", () => ({ kind: "circle", center: { x: 10, y: 10 }, radius: 10 }));
  const s = p.snapshot();
  const relations: readonly GeometricRelation[] = [
    { kind: "midpoint", point: point("pointCarrier", "start"), lineGeometryId: "target" },
    { kind: "point-on-line", point: point("pointCarrier", "start"), lineGeometryId: "target" },
    { kind: "point-on-circle", point: point("target", "end"), circleGeometryId: "circle" },
    { kind: "distance-points", first: point("target", "start"), second: point("target", "end"), value: 10 },
  ];
  for (const relation of relations) assert.ok(evaluateRelation(s, relation).scaledNorm < 1e-12, relation.kind);
});

test("symmetric relation uses an explicit center and rejects line centers", () => {
  const p = new DesignProgram();
  p.addGeometry("left", () => ({ kind: "line", start: { x: 0, y: 0 }, end: { x: 1, y: 0 } }));
  p.addGeometry("right", () => ({ kind: "line", start: { x: 10, y: 0 }, end: { x: 9, y: 0 } }));
  p.addGeometry("circle", () => ({ kind: "circle", center: { x: 5, y: 0 }, radius: 1 }));
  const s = p.snapshot();
  assert.ok(evaluateRelation(s, { kind: "symmetric", first: point("left", "start"), second: point("right", "start"), about: { kind: "center", geometryId: "circle" } }).scaledNorm < 1e-12);
  assert.throws(() => validateRelation(s, { kind: "symmetric", first: point("left", "start"), second: point("right", "start"), about: { kind: "center", geometryId: "left" } }), /has no center/);
});

test("relation snapshots are immutable and preserve explicit dependencies", () => {
  const snapshot = appendRelations(relationFixture(), [{ kind: "parallel", firstGeometryId: "h1", secondGeometryId: "h2" }]);
  assert.equal(snapshot.relations.length, 1); assert.equal(snapshot.relations[0]!.id, "relation-1"); assert.equal(snapshot.relationDependencies.length, 2);
  assert.ok(Object.isFrozen(snapshot)); assert.ok(Object.isFrozen(snapshot.relations)); assert.ok(Object.isFrozen(snapshot.relations[0]));
  assert.deepEqual(snapshot.relationDependencies.map(x => x.from.id).sort(), ["h1", "h2"]);
});

test("invalid relation domains fail closed", () => {
  const p = new DesignProgram();
  p.addGeometry("line", () => ({ kind: "line", start: { x: 0, y: 0 }, end: { x: 1, y: 0 } }));
  p.addGeometry("circle", () => ({ kind: "circle", center: { x: 0, y: 0 }, radius: 1 }));
  const s = p.snapshot();
  assert.throws(() => validateRelation(s, { kind: "concentric", firstGeometryId: "line", secondGeometryId: "circle" }), /circular geometries/);
  assert.throws(() => validateRelation(s, { kind: "perpendicular", firstGeometryId: "line", secondGeometryId: "circle" }), /requires a line/);
  assert.throws(() => validateRelation(s, { kind: "radius", geometryId: "line", value: 1 }), /circular geometry/);
  assert.throws(() => validateRelation(s, { kind: "midpoint", point: { kind: "center", geometryId: "line" }, lineGeometryId: "line" }), /has no center/);
  assert.throws(() => validateRelation(s, { kind: "distance-points", first: point("line", "start"), second: point("line", "end"), value: -1 }), /non-negative/);
});

test("angle residual wraps continuously around the -pi/pi boundary", () => {
  const p = new DesignProgram();
  p.addGeometry("a", () => ({ kind: "line", start: { x: 0, y: 0 }, end: { x: -1, y: 1e-12 } }));
  p.addGeometry("b", () => ({ kind: "line", start: { x: 0, y: 0 }, end: { x: -1, y: -1e-12 } }));
  const residual = evaluateRelation(p.snapshot(), { kind: "angle", firstGeometryId: "a", secondGeometryId: "b", radians: -2e-12 });
  assert.ok(residual.scaledNorm < 1e-10);
});
