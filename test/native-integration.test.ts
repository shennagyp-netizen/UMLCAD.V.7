import assert from "node:assert/strict";
import test from "node:test";
import { DesignProgram, buildTopology, evaluateDimensions, spatialAnalysis, exportDxf, validateEngineering, solveAndValidateEngineering, classifyConstraintSystem, relationConsistency, analyzeConstraints, solveConstraints } from "../kernel/src/native-index.js";

function engineeringFixture() {
  const p = new DesignProgram();
  p.addGeometry("bottom", () => ({ kind: "line", start: { x: 0, y: 0 }, end: { x: 10, y: 0 } }));
  p.addGeometry("right", () => ({ kind: "line", start: { x: 10, y: 0 }, end: { x: 10, y: 10 } }));
  p.addGeometry("top", () => ({ kind: "line", start: { x: 10, y: 10 }, end: { x: 0, y: 10 } }));
  p.addGeometry("left", () => ({ kind: "line", start: { x: 0, y: 10 }, end: { x: 0, y: 0 } }));
  p.addGeometry("hole", () => ({ kind: "circle", center: { x: 5, y: 5 }, radius: 2 }));
  p.horizontal({ kind: "geometry", entityId: "bottom" }); p.vertical({ kind: "geometry", entityId: "right" });
  p.horizontal({ kind: "geometry", entityId: "top" }); p.vertical({ kind: "geometry", entityId: "left" });
  return p;
}

test("full native engineering pipeline composes structure, topology, dimensions, spatial checks and deterministic export", () => {
  const snapshot = engineeringFixture().snapshot();
  const topology = buildTopology(snapshot);
  assert.ok(topology.wires.length >= 1); assert.ok(topology.wires.some(w => w.closed));
  const dimensions = evaluateDimensions(snapshot, [{ id: "outer-edge", kind: "length", firstGeometryId: "bottom" }, { id: "hole", kind: "length", firstGeometryId: "hole" }]);
  assert.equal(dimensions[0]!.value, 10); assert.ok(Math.abs(dimensions[1]!.value - 4 * Math.PI) < 1e-12);
  const spatial = spatialAnalysis(snapshot.geometry); assert.ok(spatial.every(item => Number.isFinite(item.distance)));
  const dxf = exportDxf(snapshot); assert.match(dxf, /SECTION/); assert.match(dxf, /EOF/); assert.equal(exportDxf(snapshot), dxf);
});

test("engineering evidence exposes every trust dimension and stays true for a valid closed design", () => {
  const evidence = validateEngineering(engineeringFixture().snapshot());
  assert.equal(evidence.structuralValidity, true); assert.equal(evidence.constraintValidity, true); assert.equal(evidence.relationValidity, true);
  assert.equal(evidence.numericalConditioning, true); assert.equal(evidence.referenceValidity, true); assert.equal(evidence.topologyValidity, true);
  assert.equal(evidence.spatialValidity, true); assert.equal(evidence.engineeringRuleValidity, true); assert.equal(evidence.exportValidity, true);
});

test("engineering acceptance is fail-closed for contradictory relations even when raw geometry is valid", () => {
  const p = new DesignProgram();
  p.addGeometry("a", () => ({ kind: "line", start: { x: 0, y: 0 }, end: { x: 10, y: 0 } }));
  p.addGeometry("b", () => ({ kind: "line", start: { x: 0, y: 1 }, end: { x: 10, y: 1 } }));
  const relations = [{ kind: "parallel", firstGeometryId: "a", secondGeometryId: "b" } as const, { kind: "perpendicular", firstGeometryId: "a", secondGeometryId: "b" } as const];
  assert.deepEqual(classifyConstraintSystem(p.snapshot(), relations).contradictoryRelationIndexes, [0, 1]);
  assert.equal(relationConsistency(p.snapshot(), relations).length, 0);
  const evidence = validateEngineering(p.snapshot(), relations); assert.equal(evidence.relationValidity, false); assert.equal(evidence.engineeringRuleValidity, false);
});

test("duplicate constraints are classified as redundant without being silently discarded", () => {
  const p = new DesignProgram(); p.addGeometry("edge", () => ({ kind: "line", start: { x: 0, y: 0 }, end: { x: 10, y: 0 } }));
  p.horizontal({ kind: "geometry", entityId: "edge" }); p.horizontal({ kind: "geometry", entityId: "edge" });
  const classification = classifyConstraintSystem(p.snapshot()); assert.equal(classification.redundantConstraintIds.length, 1); assert.equal(classification.contradictoryConstraintIds.length, 0);
  assert.ok(classification.warnings.some(w => /duplicates/.test(w)));
});

test("incompatible scalar distance constraints are diagnosed before numerical acceptance", () => {
  const p = new DesignProgram(); p.addGeometry("edge", () => ({ kind: "line", start: { x: 0, y: 0 }, end: { x: 10, y: 0 } }));
  p.distance("edge", 10); p.distance("edge", 20);
  const classification = classifyConstraintSystem(p.snapshot()); assert.equal(classification.contradictoryConstraintIds.length, 2);
  const analysis = analyzeConstraints(p.snapshot()); assert.equal(analysis.valid, true); assert.equal(analysis.satisfied, false);
});

test("relation-aware solver keeps input immutable and exposes relation equations separately", () => {
  const p = new DesignProgram();
  p.addGeometry("a", () => ({ kind: "line", start: { x: 0, y: 0 }, end: { x: 10, y: 0 } }));
  p.addGeometry("b", () => ({ kind: "line", start: { x: 0, y: 5 }, end: { x: 10, y: 5 } }));
  const before = p.snapshot(); const relation = { kind: "parallel", firstGeometryId: "a", secondGeometryId: "b" } as const;
  const solved = solveConstraints(before, { relations: [relation], maxIterations: 20 });
  assert.equal(solved.analysis.relationCount, 1); assert.equal(solved.analysis.relationEquationCount, 1); assert.deepEqual(before.geometry, p.snapshot().geometry);
});

test("accepted engineering result carries a validated materialized snapshot distinct from source state", () => {
  const p = new DesignProgram(); p.addGeometry("edge", () => ({ kind: "line", start: { x: 2, y: 3 }, end: { x: 12, y: 3 } })); p.horizontal({ kind: "geometry", entityId: "edge" });
  const source = p.snapshot(); const result = solveAndValidateEngineering(source);
  assert.equal(result.accepted, true); assert.notEqual(result.snapshot.geometry, source.geometry); assert.equal((result.snapshot.geometry[0]!.geometry as { start: { x: number } }).start.x, 2);
  assert.ok(Object.isFrozen(result)); assert.ok(Object.isFrozen(result.evidence));
});

test("engineering validation rejects stale semantic references instead of accepting otherwise valid geometry", () => {
  const result = validateEngineering(engineeringFixture().snapshot(), [], [{ kind: "geometry", id: "deleted-geometry" }]);
  assert.equal(result.structuralValidity, true); assert.equal(result.referenceValidity, false); assert.equal(result.engineeringRuleValidity, false); assert.ok(result.diagnostics.some(d => d.code.includes("reference")));
});
