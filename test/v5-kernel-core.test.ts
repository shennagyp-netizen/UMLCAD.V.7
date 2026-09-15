import assert from "node:assert/strict";
import test from "node:test";
import { DesignProgram, queryGeometry, validateSnapshot } from "../kernel/src/index.js";

test("V5 core builds a deterministic semantic snapshot without a V4 object", () => {
  const program = new DesignProgram();
  const width = program.parameter("width", 10);
  program.addGeometry("edge", ({ parameters }) => ({
    kind: "line",
    start: { x: 0, y: 0 },
    end: { x: parameters.get(width.name)!, y: 0 },
  }), [width]);
  program.horizontal({ kind: "geometry", entityId: "edge" });

  const snapshot = program.evaluate();
  assert.equal(snapshot.values.width, 10);
  assert.deepEqual(snapshot.geometry[0]?.geometry, { kind: "line", start: { x: 0, y: 0 }, end: { x: 10, y: 0 } });
  assert.equal(validateSnapshot(snapshot).length, 0);
});

test("V5 core regeneration is transactional", () => {
  const program = new DesignProgram();
  const width = program.parameter("width", 10);
  program.addGeometry("edge", ({ parameters }) => ({ kind: "line", start: { x: 0, y: 0 }, end: { x: parameters.get(width.name)!, y: 0 } }), [width]);

  const updated = program.regenerate([{ parameter: "width", value: 25 }]);
  assert.equal(updated.values.width, 25);
  assert.deepEqual(updated.geometry[0]?.geometry, { kind: "line", start: { x: 0, y: 0 }, end: { x: 25, y: 0 } });

  assert.throws(() => program.regenerate([{ parameter: "width", value: Number.NaN }]), /must be finite/);
  assert.equal(program.snapshot().values.width, 25);
});

test("dependency closure reaches geometry and its constraints", () => {
  const program = new DesignProgram();
  const radius = program.parameter("radius", 5);
  program.addGeometry("circle", ({ parameters }) => ({ kind: "circle", center: { x: 0, y: 0 }, radius: parameters.get(radius.name)! }), [radius]);
  const constraint = program.fixed("circle");
  const downstream = program.downstreamFrom({ kind: "parameter", id: "radius" });
  assert.deepEqual(downstream.map((node) => `${node.kind}:${node.id}`), [`geometry:circle`, `constraint:${constraint}`]);
});

test("geometry queries are semantic, not sampled render geometry", () => {
  const query = queryGeometry({ kind: "line", start: { x: 0, y: 0 }, end: { x: 10, y: 0 } });
  assert.deepEqual(query.pointAt(0.5), { x: 5, y: 0 });
  assert.deepEqual(query.tangentAt(0.25), { x: 1, y: 0 });
  assert.equal(query.parameterAt({ x: 5, y: 0 }), 0.5);
  assert.equal(query.distanceTo({ x: 5, y: 3 }), 3);
});
