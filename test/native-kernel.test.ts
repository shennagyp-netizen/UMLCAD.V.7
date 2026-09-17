import assert from "node:assert/strict";
import test from "node:test";
import { Kernel, type PartDefinition } from "../kernel/src/native-index.js";

const makePart: PartDefinition = (builder) => {
  const width = builder.parameter("width", 10);
  builder.addGeometry("edge", ({ parameters }) => ({
    kind: "line",
    start: { x: 0, y: 0 },
    end: { x: parameters.get(width.name)!, y: 0 },
  }), [width]);
  builder.horizontal({ kind: "geometry", entityId: "edge" });
};

test("native V5 Kernel builds and caches immutable parts", () => {
  const kernel = new Kernel();
  const first = kernel.buildPart("p", "part-a", "rev-1", makePart);
  const second = kernel.buildPart("p", "part-a", "rev-1", makePart);
  assert.deepEqual(first.buildIdentity, second.buildIdentity);
  assert.equal(first.snapshot.values.width, 10);
  assert.deepEqual(first.snapshot.geometry[0]?.geometry, { kind: "line", start: { x: 0, y: 0 }, end: { x: 10, y: 0 } });
  assert.equal(first.diagnostics.length, 0);
});

test("native V5 Kernel regenerates from a prior part build", () => {
  const kernel = new Kernel();
  const original = kernel.buildPart("p", "part-a", "rev-1", makePart);
  const changed = kernel.regeneratePart(original.buildIdentity, [{ parameter: "width", value: 25 }], "p", "part-a", "rev-2");
  assert.notDeepEqual(changed.buildIdentity, original.buildIdentity);
  assert.equal(changed.snapshot.values.width, 25);
  assert.deepEqual(changed.snapshot.geometry[0]?.geometry, { kind: "line", start: { x: 0, y: 0 }, end: { x: 25, y: 0 } });
  assert.equal(original.snapshot.values.width, 10);
});

test("native V5 Kernel assembles immutable part builds", () => {
  const kernel = new Kernel();
  const part = kernel.buildPart("p", "part-a", "rev-1", makePart);
  const assembly = kernel.buildAssembly("p", "assembly-a", "rev-1", [
    { instanceId: "left", partId: "part-a", partBuildId: part.buildIdentity, transform: { translation: { x: 0, y: 0 }, rotationRadians: 0 } },
    { instanceId: "right", partId: "part-a", partBuildId: part.buildIdentity, transform: { translation: { x: 20, y: 0 }, rotationRadians: 0 } },
  ]);
  assert.equal(assembly.geometry.length, 2);
  assert.deepEqual(assembly.geometry[0]?.geometry, { kind: "line", start: { x: 0, y: 0 }, end: { x: 10, y: 0 } });
  assert.deepEqual(assembly.geometry[1]?.geometry, { kind: "line", start: { x: 20, y: 0 }, end: { x: 30, y: 0 } });
});
