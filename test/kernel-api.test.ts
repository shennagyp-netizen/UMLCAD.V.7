import assert from "node:assert/strict";
import test from "node:test";
import { Drawing, type PartSourceLoader } from "../vendor/UMLCAD.V.4/src/index.js";
import {
  V4KernelApplication,
  InMemoryPartCache,
  InMemoryAssemblyCache,
  buildIdentity,
  analyzeSourceModule,
  orderSourceModules,
} from "../kernel/api/src/index.js";

class SimplePart extends Drawing {
  protected configure(): void {
    const width = this.design.parameter("width", 10);
    this.design.addGeometry("edge", ({ parameters }) => ({
      kind: "line",
      start: { x: 0, y: 0 },
      end: { x: parameters.get(width.name)!, y: 0 },
    }), [width]);
    this.design.horizontal({ kind: "edge", entityId: "edge" });
  }
}

class Loader implements PartSourceLoader {
  public loads = 0;
  public async loadDrawing(_moduleId: string, _exportName: string): Promise<unknown> {
    this.loads += 1;
    return new SimplePart();
  }
}

function request() {
  return {
    projectId: "p",
    partId: "part-a",
    sourceRevision: "rev-1",
    moduleId: "part.ts",
    exportName: "createPart",
  } as const;
}

test("part build delegates to V4 and caches complete result", async () => {
  const loader = new Loader();
  const app = new V4KernelApplication(loader, new InMemoryPartCache());
  const first = await app.buildPart(request());
  const second = await app.buildPart(request());
  assert.equal(loader.loads, 1);
  assert.deepEqual(first.buildIdentity, second.buildIdentity);
  assert.deepEqual(first.design, second.design);
  assert.equal(first.design.geometry[0]?.id, "edge");
  assert.equal(first.validation.valid, true);
});

test("assembly consumes immutable part build identities and is separately cacheable", async () => {
  const loader = new Loader();
  const app = new V4KernelApplication(loader, new InMemoryPartCache(), new InMemoryAssemblyCache());
  const part = await app.buildPart(request());
  const assemblyRequest = {
    projectId: "p",
    assemblyId: "asm-1",
    sourceRevision: "rev-1",
    instances: [
      { instanceId: "left", partId: "part-a", partBuildId: part.buildIdentity, transform: { translation: { x: 0, y: 0 }, rotationRadians: 0 } },
      { instanceId: "right", partId: "part-a", partBuildId: part.buildIdentity, transform: { translation: { x: 20, y: 0 }, rotationRadians: 0 } },
    ],
  } as const;
  const first = await app.buildAssembly(assemblyRequest);
  const second = await app.buildAssembly(assemblyRequest);
  assert.deepEqual(first.buildIdentity, second.buildIdentity);
  assert.equal(first.geometry.length, 2);
  assert.deepEqual(first.geometry[0]?.geometry, { kind: "line", start: { x: 0, y: 0 }, end: { x: 10, y: 0 } });
  assert.deepEqual(first.geometry[1]?.geometry, { kind: "line", start: { x: 20, y: 0 }, end: { x: 30, y: 0 } });
  assert.equal(loader.loads, 1);
});

test("geometry query uses V4 geometric query authority", async () => {
  const app = new V4KernelApplication(new Loader());
  const build = await app.buildPart(request());
  const point = await app.geometryQuery({
    buildIdentity: build.buildIdentity,
    geometryId: "edge",
    operation: { kind: "pointAt", parameter: 0.5 },
  });
  assert.deepEqual(point, { x: 5, y: 0 });
});

test("build identity is deterministic", () => {
  assert.deepEqual(buildIdentity({ b: 2, a: 1 }), buildIdentity({ a: 1, b: 2 }));
});

test("source analysis rejects dynamic imports", () => {
  assert.throws(() => analyzeSourceModule("a.ts", "const x = import(name);"), /dynamic import/);
});

test("source module order is dependency-first and deterministic", () => {
  const order = orderSourceModules([
    { id: "a.ts", imports: ["./b.ts"] },
    { id: "b.ts", imports: [] },
  ]);
  assert.deepEqual(order, ["b.ts", "a.ts"]);
});
