# UMLCAD V5 — Compiled 3D Viewer Architecture and Execution Plan

## 0. Purpose

Connect the compiled UMLCAD result to a browser-based React 3D viewer. The viewer is a renderer/navigator/inspector, not a CAD kernel or engineering authority.

The production flow is:

```text
C# CAD program
  ↓
.NET UMLCAD Framework
  ↓ Build()
immutable semantic snapshot
  ↓ BuildPackage
IRustKernelService
  ↓
Rust engineering kernel
  ├─ authoritative geometry/topology/evaluation
  ├─ 3D render projection
  └─ compiled-model manifest
  ↓
GLB + compiled-model manifest
  ↓
React + TypeScript + Three.js/React Three Fiber
  ├─ model tree
  ├─ 3D viewport
  ├─ inspector
  └─ selection/visibility/camera state
```

## 1. Architecture gate: current gap

The checked-in kernel specification currently defines a native 2D geometry surface. It does **not** establish a production 3D GLB export capability today. Therefore the plan must not pretend the viewer can be connected immediately to a real 3D kernel output. The work has two coordinated tracks:

```text
A. compiled-model contract + React viewer
B. Rust 3D evaluation/export capability required by A
```

A fixture GLB + manifest can unblock viewer development, but production integration is complete only when the Rust kernel produces the authoritative 3D result.

## 2. Viewer contract: general compiled model

Do not model the viewer around today's examples (`Assembly → Part → Sketch → Hole`). Those are examples only.

The viewer consumes a versioned `CompiledModelPackage`:

```text
CompiledModelPackage
├─ schema
├─ application
│  ├─ id
│  ├─ version
│  └─ buildIdentity
├─ render
│  ├─ assetIdentity
│  └─ GLB reference/payload
└─ model
   ├─ rootNodeIds[]
   ├─ nodes[]
   ├─ relationships[]
   ├─ representations[]
   ├─ topologyBindings[]
   ├─ sourceBindings[]
   └─ diagnostics[]
```

Recommended schema identifier: `uml-cad-compiled-model/1.0.0`.

The manifest is semantic/binding data. It is not a second geometry authority.

## 3. General node graph

Every compiled node should be addressable independently:

```text
CompiledNode
├─ id
├─ name
├─ kind
├─ parentId?
├─ childIds[]
├─ metadata
├─ relationshipIds[]
├─ representationIds[]
├─ sourceBinding?
└─ capabilities
```

### Identity

`id` is a stable semantic identity. It must not be a display name and must not depend on array position.

### Kind

`kind` is extensible, not a closed viewer enum. Possible kinds include Assembly, ComponentInstance, Part, Body, Solid, Surface, Feature, Sketch, Reference, Construction, Datum, Annotation, Drawing, Sheet, Face, Edge, Vertex, and future domain-specific kinds.

Unknown kinds must remain navigable as generic nodes.

### Recursion

`parentId`/`childIds[]` must support arbitrary hierarchy depth:

```text
Assembly
└─ Assembly
   └─ Assembly
      └─ Part
```

Large machines must not be constrained to one or two assembly levels.

## 4. Metadata model

Metadata must be extensible and typed. Do not create a fixed protocol containing every possible future CAD property.

Supported value classes should include:

- string;
- boolean;
- numeric scalar;
- quantity + unit;
- enum;
- list;
- structured object;
- semantic reference.

Optional metadata definitions should describe display label, type, unit, read-only/editable state, semantic category, and visibility policy.

Example:

```json
{
  "metadata": {
    "featureType": "Hole",
    "diameter": { "value": 12, "unit": "mm" },
    "depth": { "value": 30, "unit": "mm" },
    "through": true
  }
}
```

The first viewer is read-only, but the metadata contract must not prevent later authoritative edit-intent operations.

## 5. Relationships: hierarchy is not enough

CAD structure is a graph, not only a tree. The tree is used for navigation, while explicit relationships preserve semantic links:

```text
Sketch --profiles--> Feature
Feature --creates--> Solid
Feature --references--> Face
AssemblyInstance --instantiates--> PartDefinition
Constraint --references--> Geometry
Annotation --targets--> Face
DrawingView --references--> Part
```

Use a separate relationship collection:

```text
Relationship
├─ id
├─ kind
├─ sourceId
├─ targetIds[]
└─ metadata
```

The viewer tree must never be forced to encode these relationships as fake parent/child structure.

## 6. Geometry/render bindings

A semantic node may map to zero, one, or many render objects. Never assume `one node == one Three.js Object3D`.

A `Representation` should identify:

```text
Representation
├─ id
├─ kind
├─ artifactId
├─ renderNodeId / render path
├─ bounds
└─ capabilities
```

Examples:

```text
Assembly  → aggregate/no geometry
Part      → one or more render nodes
Sketch    → curve representation
Feature   → generated representation
Solid     → mesh representation
Face      → selectable render primitive(s)
```

## 7. Face/topology identity

Face selection is a first-class requirement.

Required resolution path:

```text
viewport hit
  ↓
render primitive/sub-primitive identity
  ↓
authoritative face ID
  ↓
compiled node/face
  ↓
face name + nomenclature + metadata
```

A face record should support, as supplied by the kernel:

```text
Face
├─ id
├─ name
├─ nomenclature
├─ number/local index where meaningful
├─ parent topology/object
├─ metadata
└─ representation binding
```

React must never derive authoritative face names from triangle order or mesh topology heuristics.

Tessellation is a display representation; the kernel remains the authority for topology identity.

## 8. Source/provenance

The compiled model should optionally bind nodes to source information:

```text
source file
source symbol/declaration
source range
component source
feature declaration
build revision
```

This lets the viewer show where an item came from without making React a compiler or source editor.

## 9. Visibility and selection

Visibility is local viewer/session state, not a CAD mutation.

Every renderable node should expose enough capability information for:

```text
visible
hideable
selectable
focusable
```

Selection state must use compiled identifiers, never renderer object references:

```text
Selection
├─ targetKind
├─ targetId
└─ subTargetId?   # e.g. face/edge
```

Required behavior:

```text
Tree → select → highlight
Tree → focus → zoom to bounds
Viewport → hit → resolve object/face
Viewport → select → reveal tree path
Tree → hide/show → update render targets
```

## 10. Camera

Camera state is independent viewer state. It must never trigger build/kernel evaluation.

V1 controls:

- orbit/rotate;
- pan;
- zoom/wheel zoom;
- fit all;
- focus selected node;
- focus selected face/topology target;
- reset camera.

## 11. GLB strategy

Use glTF 2.0 / GLB as the browser rendering artifact. Three.js recommends glTF/GLB as the runtime delivery format and `GLTFLoader` supports glTF 2.0 plus relevant compression extensions. citeturn276445search0turn276445search3

The glTF specification supports application-specific `extras`, so node-level IDs or compact lookup hints may be embedded there. citeturn276445search24

However, GLB metadata is **not** the complete semantic authority. The compiled-model manifest is the viewer contract; glTF hierarchy/names/extras are rendering-side bindings/optimizations.

## 12. Package integrity

The viewer must never pair a GLB from build A with a manifest from build B.

Both must carry the same `buildIdentity`. The host/viewer must validate:

```text
package buildIdentity == manifest buildIdentity == render asset buildIdentity
```

Mismatch is a hard load failure or explicit invalid-state diagnostic.

## 13. .NET integration

The current .NET framework owns semantic build composition, parts, assemblies, drawings/sheets, semantic services, build history, and canonical build packages. Current semantic records already cover parts, assemblies, drawings, sheets, geometry, constraints, components, parameters, and references. fileciteturn121file0L2-L2

The viewer must consume an immutable compiled projection of that build, never mutable builder state.

The framework should add a typed compiled-model projection/contract. It must not import React, Three.js, or browser dependencies.

## 14. Rust/API integration

The current `.NET` kernel client returns a generic `string Result`, which is insufficient for this viewer. fileciteturn114file0L2-L2

Evolve the transport-neutral result toward:

```text
KernelEvaluationResult
├─ succeeded
├─ compiledModelArtifact?
│  ├─ buildIdentity
│  ├─ manifest
│  └─ renderArtifact reference/payload
└─ diagnostics[]
```

The contract must remain independent of HTTP, WebSocket, React, Three.js, and other hosts.

## 15. React architecture

Initial stack:

```text
React
TypeScript
Three.js
React Three Fiber
selected Three.js addons / Drei where useful
```

Suggested modules:

```text
viewer/
├─ model/
│  ├─ compiled-model.ts
│  ├─ model-index.ts
│  └─ metadata.ts
├─ renderer/
│  ├─ glb-loader.ts
│  ├─ render-binding.ts
│  └─ selection-picking.ts
├─ state/
│  ├─ selection.ts
│  ├─ visibility.ts
│  └─ camera.ts
├─ components/
│  ├─ ModelTree
│  ├─ Viewport3D
│  ├─ Inspector
│  └─ Diagnostics
└─ app/
   └─ ViewerShell
```

React contains no CAD solver, constraint solver, topology engine, or authoritative measurement implementation.

## 16. Large-machine requirements

Design for large hierarchical machines from the beginning:

- arbitrary recursive depth;
- stable IDs, never array indexes as identity;
- virtualized/lazy model tree;
- compact node indexes/maps for O(1)-style lookup after indexing;
- instancing when supported by the render artifact;
- optional mesh compression;
- no large geometry duplicated in React state;
- camera/selection/visibility state separated from model data;
- bounded selection payloads.

The full model graph can be indexed in memory while UI tree rows remain virtualized.

## 17. Failure states

The viewer must distinguish at least:

```text
BUILD_FAILED
KERNEL_FAILED
EXPORT_FAILED
MANIFEST_INVALID
ARTIFACT_MISMATCH
MODEL_LOAD_FAILED
```

A failed build/export must never appear as a valid engineering model.

## 18. Versioning

The manifest is an API contract and must be explicitly versioned.

Rules:

- additive changes should remain backward-compatible where possible;
- unknown metadata should be preserved/ignored safely;
- unknown node kinds remain generic/navigable;
- incompatible major schema versions are rejected;
- authoritative IDs cannot be silently replaced by renderer IDs.

## 19. Testing strategy

### Contract tests

Verify:

- manifest schema validation;
- globally unique node IDs within a compiled result;
- parent/child resolution;
- arbitrary recursive assemblies;
- relationship target resolution;
- render binding resolution;
- face binding resolution;
- build-identity agreement.

### Kernel/export tests

Verify:

- deterministic export for equivalent compiled input;
- stable object identity where semantic identity is preserved;
- correct nested assembly transforms;
- authoritative face bindings;
- structured failure for unsupported 3D operations/features;
- no client-generated engineering geometry.

### React tests

Verify:

- recursive tree navigation;
- unknown kinds remain navigable;
- metadata inspector handles unknown fields;
- tree→viewport selection;
- viewport→tree selection;
- object/face focus;
- show/hide at arbitrary hierarchy levels;
- mismatched package rejection;
- camera changes do not modify model state.

### Required end-to-end fixture

Create a deterministic fixture approximately like:

```text
Top Assembly
├─ Subsystem Assembly
│  ├─ Nested Assembly
│  │  ├─ Part
│  │  │  ├─ Sketch
│  │  │  ├─ Feature
│  │  │  └─ Solid
│  │  └─ repeated Part instance
│  └─ Part
└─ Independent Assembly
```

Include several metadata classes, selectable faces, hidden/non-renderable nodes, repeated instances, and at least one non-tree relationship.

## 20. Execution phases

### Phase 1 — freeze the compiled-model contract

1. Define `uml-cad-compiled-model/1.0.0`.
2. Define node identity and arbitrary recursion rules.
3. Define typed/extensible metadata.
4. Define explicit graph relationships.
5. Define render representations and bindings.
6. Define face/topology bindings.
7. Define provenance.
8. Define package/build identity.
9. Define diagnostics/errors.
10. Add schema, examples, and contract tests.

**Gate:** deterministic schema/examples/validation are green.

### Phase 2 — align the .NET framework

1. Add typed compiled-model projection interfaces/records.
2. Project from immutable `SemanticApplication` only.
3. Preserve canonical build identity and deterministic serialization.
4. Add tests for equivalent build → equivalent compiled projection.
5. Keep Microsoft.Extensions infrastructure unchanged.

**Gate:** `.NET` build/tests green; no mutable viewer state leaks into semantic builds.

### Phase 3 — Rust 3D/export boundary

1. Define transport-neutral typed kernel result.
2. Define the required 3D compiled-model/export input/output.
3. Implement authoritative 3D geometry/topology projection for the supported kernel surface.
4. Export deterministic GLB.
5. Emit compiled-model manifest/bindings.
6. Implement nested assembly transforms/instances.
7. Bind selectable faces/topology where supported.
8. Validate build identity across outputs.
9. Add structured export diagnostics.

**Gate:** deterministic fixture produces valid GLB + manifest with no browser-side geometry construction.

### Phase 4 — React viewer

1. Create React/TypeScript viewer project.
2. Add Three.js/R3F and GLB loader.
3. Build manifest/renderer binding index.
4. Implement recursive/virtualized model tree.
5. Implement selection and viewport picking.
6. Implement object/face inspector.
7. Implement visibility.
8. Implement orbit/pan/zoom/fit/focus.
9. Handle diagnostics and package mismatch.

**Gate:** fixture model is fully navigable and selectable in browser.

### Phase 5 — large-model hardening

1. Virtualize tree rendering.
2. Optimize lookup maps.
3. Measure GLB size/load/render time.
4. Add compression/instancing where measurements justify it.
5. Profile deep assembly recursion.
6. Bound memory use of UI state.

**Gate:** explicit performance targets pass on a large-machine fixture.

### Phase 6 — host integration

1. Replace fixture package with host-provided compiled model.
2. Connect `.NET → Rust` typed result.
3. Serve GLB + manifest through the host transport.
4. Validate package/build identity.
5. Surface kernel/export diagnostics.

**Gate:** one real C# CAD program can Build → Rust evaluate/export → React render without manual conversion.

### Phase 7 — CI/release gate

CI should execute:

```text
.NET restore/build/test
        ↓
compiled-model schema/contract tests
        ↓
Rust kernel/export tests
        ↓
React typecheck/lint/test/build
        ↓
end-to-end fixture validation
```

The viewer is production-ready only when the complete chain is green.

## 21. V1 non-goals

The first viewer does not implement:

- CAD solving;
- sketch/feature/topology editing;
- source mutation;
- independent authoritative measurement calculation;
- engineering validation;
- Git/authentication/authorization;
- WebSocket collaboration semantics.

Those belong to the .NET/kernel/host layers. V1 is a rich compiled-result renderer, navigator, selector, and inspector.

## 22. Final invariant

```text
SOURCE
  ↓
.NET SEMANTIC BUILD
  ↓
RUST AUTHORITATIVE ENGINEERING EVALUATION
  ↓
COMPILED MODEL PACKAGE
  ├─ render artifact
  └─ semantic/binding manifest
  ↓
REACT VIEWER
  ├─ render
  ├─ navigate
  ├─ select
  ├─ inspect
  ├─ show/hide
  └─ focus/zoom
```

React knows **what exists, how it is organized, and how a rendered target maps back to an authoritative compiled object/face**. It does not know how the engineering result was calculated.
