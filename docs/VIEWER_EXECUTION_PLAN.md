# UMLCAD V5 — Compiled 3D Viewer Execution Plan

This plan defines the browser viewer and its compiled-model contract. The viewer is a renderer/navigator/inspector, not a CAD kernel or engineering authority.

## Architecture

```text
C# CAD program
  ↓
.NET UMLCAD Framework → Build() → immutable semantic snapshot
  ↓
BuildPackage → IRustKernelService → Rust kernel
  ↓
authoritative evaluation + geometry/topology/export
  ├──────────────► GLB/glTF render artifact
  └──────────────► Compiled Model Manifest
                         ↓
                 React + TypeScript
                 Three.js/R3F viewer
```

.NET owns semantic application composition/build meaning. Rust owns engineering geometry/topology/validation and authoritative render export. React renders and navigates the compiled result.

## Critical current gap

The checked-in kernel specification currently defines a native **2D** geometry surface, so production 3D GLB export is not yet present in the current contract. The implementation therefore has two coordinated tracks:

1. compiled-model contract + React viewer;
2. Rust 3D/export capability required to produce the contract.

A deterministic fixture GLB+manifest may be used to develop React, but no client-side geometry synthesis is an acceptable substitute for kernel export.

## Compiled model package

Version the viewer contract separately from transport:

```text
uml-cad-compiled-model/1.1.0

CompiledModelPackage
├─ application: id/version/buildIdentity
├─ render: GLB artifact identity/reference
└─ model
   ├─ rootNodeIds[]
   ├─ nodes[]
   ├─ relationships[]
   ├─ representations[]
   ├─ topologyBindings[]
   ├─ sourceBindings[]
   └─ diagnostics[]
```

The package must be internally consistent: GLB, manifest and build identity must refer to the same compiled build. The render artifact must carry the same build identity and a stable asset identity; an optional SHA-256 integrity value may be supplied for hosted artifacts.

## General model graph

Do **not** hard-code a tree such as `Assembly → Part → Sketch → Hole`. Those are examples only.

Every node has the conceptual form:

```text
CompiledNode
├─ id                 globally unique within the compiled model
├─ name               display name
├─ kind               extensible/qualified kind
├─ parentId?
├─ childIds[]
├─ metadata
├─ relationshipIds[]
├─ representationIds[]
├─ sourceBinding?
├─ capabilities
└─ state
```

`kind` is open-ended. Assembly, ComponentInstance, Part, Body, Solid, Surface, Feature, Sketch, Reference, Datum, Annotation, Drawing, Sheet, Face, Edge, Vertex, etc. are examples, not a closed enum.

`parentId`/`childIds` support **arbitrary recursive depth**, including assemblies containing assemblies containing assemblies for very large machines.

Identity must never be an array index or display name. Definition identities and occurrence identities are distinct. A reusable definition may occur multiple times; an occurrence ID is path-qualified so identical local occurrence names remain distinct throughout a large recursive machine.

The primary navigation tree is occurrence-oriented. Definition nodes remain addressable through `instantiates` relationships and can be used by inspectors, BOM views or definition browsers without being duplicated as top-level tree roots.

## Relationships

CAD structure is a graph, not merely a tree. Preserve non-tree relationships separately:

```text
Sketch ──profiles──> Feature
Feature ──creates──> Solid
Feature ──references──> Face
AssemblyInstance ──instantiates──> PartDefinition
Constraint ──references──> Geometry
Annotation ──targets──> Face
DrawingView ──references──> Part
```

```text
Relationship
├─ id
├─ kind
├─ sourceId
├─ targetIds[]
└─ metadata
```

The model tree is for navigation; relationships preserve actual semantics.

## .NET framework assembly/part authoring

The framework must author **definitions** and **occurrences** explicitly.

```csharp
builder.AddPart("housing", "mechanical-part", part => { ... });

builder.AddAssembly("gearbox", "Gearbox", assembly =>
{
    assembly.Part("housing:1", "housing");
    assembly.Assembly("shaft-group:1", "shaft-group");
});
```

A definition may be referenced many times. An occurrence carries occurrence-specific state such as placement/transform, configuration, quantity, BOM structure, visibility, suppression, grounding, flexibility and custom metadata. Assembly nesting is recursive and is validated for unresolved references and cycles during `Build()`.

The framework must not maintain two competing authoritative assembly composition systems. Assembly occurrences are authoritative; legacy generic component-reference fields may remain only for compatibility and must not drive compiled navigation.

## Extensible metadata

Metadata is extensible rather than a fixed list of CAD concepts. The current framework exposes common CAD/document properties such as part number, description, material, manufacturer, vendor, revision, lifecycle state, author, document code and arbitrary custom properties.

The compiled-model contract is designed for richer typed values, including:

```text
string
boolean
number
quantity + unit
enum
list
structured object
semantic reference
```

Example:

```json
{
  "featureType": "Hole",
  "diameter": {"value": 12, "unit": "mm"},
  "depth": {"value": 30, "unit": "mm"},
  "through": true
}
```

V1 viewer behavior is read-only. The model must not block future authoritative edit-intent APIs.

Unknown metadata and unknown node kinds must remain safely inspectable without requiring a viewer release for every new kernel concept.

## Geometry and topology bindings

A semantic node may map to zero, one or many render targets. Do not assume one semantic node equals one Three.js object.

```text
Representation
├─ id
├─ kind
├─ artifactId
├─ renderNodeId/path
├─ bounds
└─ capabilities
```

Face selection is first-class:

```text
viewport hit
  ↓
render primitive/sub-primitive
  ↓
authoritative face ID
  ↓
compiled face/node
  ↓
name + nomenclature + metadata
```

A face may expose `id`, `name`, `nomenclature`, `number/local index where meaningful`, parent topology/object, metadata and representation binding.

React must **never infer authoritative face identity from triangle order**. Tessellation is a display representation; the kernel is the topology authority.

The current React layer already supports the lookup/selection path; actual `CompiledTopologyBinding` production depends on the Rust topology/export workstream.

## Provenance

Nodes may bind to source file, source symbol/declaration, source range, component source, feature declaration and build revision. This allows navigation/inspection without making React a compiler or source editor.

## Visibility/selection/camera

Visibility is local viewer/session state, not a CAD mutation.

Renderable capabilities distinguish `visible`, `hideable`, `selectable`, and `focusable`.

Selection state references compiled IDs, never Three.js object references:

```text
Selection
├─ targetKind
├─ targetId
└─ subTargetId?
```

Required viewer operations:

- recursive tree navigation;
- select object/feature/body/solid/face/edge/vertex where available;
- tree → viewport highlight;
- viewport → tree selection/reveal;
- show/hide any renderable object at any hierarchy depth;
- focus selected object;
- focus selected face/topology target;
- fit all;
- orbit/rotate;
- pan;
- zoom/wheel zoom;
- reset camera;
- metadata inspection.

Camera state must never trigger build/kernel evaluation.

## GLB/glTF

Use glTF 2.0/GLB as the browser render artifact. The render artifact is a display representation only; it is not the semantic authority. Render-side IDs may be attached through the glTF scene graph/application metadata, but the compiled manifest remains authoritative.

## Security and resource boundaries

The viewer must treat a compiled package as untrusted input at the application boundary.

Before activation, validate at minimum:

```text
schema/version compatibility
package ↔ manifest build identity
package ↔ render artifact build identity
unique node IDs
parent/child consistency
relationship targets
representation references
topology references
hierarchy cycles
hierarchy depth
node/relationship count
metadata size
inline artifact size
render artifact format
artifact URI scheme
```

The browser viewer must not accept an arbitrary render URI separately from the compiled package. Artifact resolution must originate from the validated render artifact reference and the host's allowed transport/origin policy.

The .NET kernel client enforces request timeout/response limits and validates returned compiled identity before exposing the result to consumers. Public diagnostics intentionally avoid returning raw server exception text.

For deployment, the host should add explicit trusted-origin/TLS policy, request-size limits and authentication/authorization at the host/router layer. The viewer itself is not an authentication authority.

## Large-machine requirements

Support from the beginning:

- arbitrary recursive assemblies;
- globally unique stable IDs;
- occurrence-path identity for repeated subassemblies/parts;
- model index/maps for fast lookup;
- virtualized/lazy tree rendering;
- render instancing where supported;
- optional mesh compression;
- no duplicated large geometry in React state;
- separate model/selection/visibility/camera state.

Resource limits are mandatory to prevent malformed or hostile models from exhausting browser memory.

## Error/versioning rules

Distinguish at least:

```text
BUILD_FAILED
KERNEL_FAILED
EXPORT_FAILED
MANIFEST_INVALID
ARTIFACT_MISMATCH
MODEL_LOAD_FAILED
```

Unknown node kinds remain generic/navigable. Unknown metadata is safely ignored/preserved. Incompatible major schema versions are rejected. Build identity is mandatory.

## .NET integration

The .NET semantic snapshot covers parts, assemblies, drawings, sheets, geometry, constraints, components, parameters and references. Add the compiled-model projection only from the immutable `SemanticApplication`; never expose mutable builder state to React. Keep React/Three.js out of the framework.

The typed Rust client returns a transport-neutral `CompiledModelPackage` result. No React, Three.js, HTTP or WebSocket types belong in the semantic contract.

## React implementation

Initial stack:

```text
React + TypeScript
Three.js
React Three Fiber
Drei where useful
```

Suggested boundaries:

```text
viewer/
├─ model/       compiled-model, index, validation, metadata
├─ renderer/    GLB loader, bindings, picking
├─ state/       selection, visibility, camera
├─ components/  ModelTree, Viewport3D, Inspector, Diagnostics
└─ app/         ViewerShell
```

No CAD solving, topology computation, engineering measurement authority or source mutation exists in React.

## Testing

### Contract

- schema validation;
- globally unique stable node IDs;
- valid recursive parent/child graph;
- relationship targets resolve;
- representations resolve;
- topology/face bindings resolve;
- build identity agrees across manifest and render artifact;
- package/resource limits are enforced;
- incompatible schema versions are rejected.

### Framework

- nested assembly definitions and occurrences;
- repeated occurrence IDs in different parents;
- unresolved references rejected;
- circular assembly references rejected;
- assembly/part metadata survives Build();
- metadata changes affect build identity;
- equivalent builds remain deterministic.

### Rust/export

- deterministic GLB for equivalent builds;
- correct nested transforms;
- stable semantic object bindings;
- authoritative face bindings;
- unsupported 3D capability produces structured diagnostics rather than invented geometry.

### React

- recursive hierarchy;
- occurrence-path navigation;
- unknown kind navigation;
- arbitrary metadata display;
- tree ↔ viewport selection;
- object/face focus/zoom;
- visibility toggling at arbitrary depth;
- package mismatch rejection;
- resource-limit rejection;
- camera operations do not mutate compiled model state.

### End-to-end fixture

Use a non-trivial recursive machine fixture with nested assemblies, repeated instances, part/sketch/feature/solid examples, multiple metadata shapes, selectable faces, hidden/non-renderable nodes and at least one non-tree relationship.

## Execution phases

### Phase 1 — freeze contract

1. Define schema and JSON examples.
2. Define stable ID rules.
3. Define general node graph and recursion.
4. Define extensible metadata.
5. Define relationships.
6. Define render/geometry bindings.
7. Define face/topology bindings.
8. Define provenance.
9. Define build identity and diagnostics.
10. Add contract and resource-validation tests.

**Gate:** deterministic schema/examples/validation are green.

### Phase 2 — .NET projection

1. Add typed compiled-model records/interfaces.
2. Add definitions/occurrences authoring APIs.
3. Validate references and cycles during Build().
4. Project only from immutable `SemanticApplication`.
5. Preserve canonical build identity/determinism.
6. Add recursive occurrence graph tests.

**Gate:** .NET build and tests green.

### Phase 3 — Rust 3D/export

1. Define typed transport-neutral kernel result.
2. Define supported 3D compiled/export surface.
3. Implement authoritative geometry/topology render projection.
4. Export deterministic GLB.
5. Emit manifest and bindings.
6. Implement nested assembly transforms/instances.
7. Implement face bindings where topology is available.
8. Validate identities and diagnostics.

**Gate:** real deterministic GLB + manifest produced from a kernel-approved fixture.

### Phase 4 — React viewer

1. Create React/TypeScript viewer.
2. Add GLB loading.
3. Build manifest↔render binding index.
4. Build recursive/virtualized occurrence tree.
5. Add picking/selection.
6. Add inspector.
7. Add visibility.
8. Add pan/zoom/orbit/fit/focus.
9. Add diagnostics/package-integrity handling.

**Gate:** fixture fully navigable/selectable in browser.

### Phase 5 — scale/performance

1. Virtualize large trees.
2. Optimize indexes.
3. Measure artifact size/load/render time.
4. Add compression/instancing only where measurement justifies it.
5. Profile deep assemblies and memory.

**Gate:** explicit large-machine performance targets pass.

### Phase 6 — host integration

1. Replace fixture with host-provided compiled package.
2. Connect `.NET → Rust → GLB+manifest → React`.
3. Serve artifact through the selected host transport.
4. Validate build identity at the viewer boundary.
5. Surface structured diagnostics.

**Gate:** a real C# CAD program can Build → kernel evaluate/export → browser render without manual conversion.

### Phase 7 — CI/release

```text
.NET build/test
 ↓
contract/schema tests
 ↓
Rust kernel/export tests
 ↓
React typecheck/test/build
 ↓
end-to-end fixture validation
```

Production-ready means the complete chain is green.

## V1 non-goals

No CAD solving, sketch/feature/topology editing, source mutation, independent engineering validation, Git operations, authentication/authorization, or collaboration protocol lives in the first viewer.

## Final invariant

```text
SOURCE
 ↓
.NET SEMANTIC BUILD
 ↓
RUST AUTHORITATIVE ENGINEERING RESULT
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

React knows what exists, how the compiled result is organized, and how a rendered target maps back to an authoritative object/face. It does not know how the engineering result was calculated.
