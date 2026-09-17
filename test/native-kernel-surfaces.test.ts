import assert from "node:assert/strict";
import test from "node:test";
import { Kernel, type PartDefinition } from "../kernel/api/src/native-index.js";

const definition: PartDefinition = (program) => {
  const width = program.parameter("width", 10);
  program.addGeometry("edge", ({ parameters }) => ({ kind: "line", start: { x: 0, y: 0 }, end: { x: parameters.get(width.name)!, y: 0 } }), [width]);
  program.horizontal({ kind: "geometry", entityId: "edge" });
};

test("native Kernel exposes all engineering projections from one immutable part build", () => {
  const kernel = new Kernel();
  const build = kernel.buildPart("p", "part", "rev-1", definition);
  assert.equal(kernel.analyzePart(build.buildIdentity).satisfied, true);
  assert.equal(kernel.solvePart(build.buildIdentity).converged, true);
  assert.equal(kernel.topologyPart(build.buildIdentity).edges.length, 1);
  assert.equal(kernel.dimensionsPart(build.buildIdentity, [{ id: "length", kind: "length", firstGeometryId: "edge" }])[0]!.value, 10);
  assert.equal(kernel.spatialPart(build.buildIdentity).length, 0);
  assert.match(kernel.dxfPart(build.buildIdentity), /LINE/);
  assert.equal(kernel.manifestPart(build.buildIdentity).modelIdentity, build.buildIdentity.value);
});
