# UMLCAD V5 — Architecture and Execution Plan

## 0. Purpose

UMLCAD V5 is a deliberate rewrite of the V4 product boundary.

V4 established valuable CAD-kernel principles—deterministic authoring, parametric evaluation, constraints, stable semantic identity, transactional regeneration, topology, dimensions, validation, and DXF projection—but its framework/viewer/runtime boundaries are no longer the target architecture for V5.

V5 is built around the following model:

```text
CAD source code
      ↓
UMLCAD framework + CAD kernel
      ↓
validated compiled CAD result
      ├── standard CAD artifact(s)
      └── UMLCAD semantic manifest
      ↓
browser client / viewer
      ↕
real-time WebSocket
      ↕
project server / live process manager
      ↓
source editor / LLM / compiler / kernel
```

The browser is the only required client application in V5. A native desktop viewer is not a V5 architectural dependency.

The UMLCAD framework itself is transport-independent. WebSocket, HTTP, Git service integration, container execution, authentication, collaboration, and server hosting are runtime/host concerns.

The project repository is Git. Each UMLCAD project is an independent Git repository. Live collaboration is deliberately separated from Git commit timing.

---

# 1. V5 architectural decisions — frozen starting point

## 1.1 Source code is the design authority

The authored UMLCAD source is the editable engineering program.

The framework executes/compiles that source into a validated CAD result. Generated CAD files are derived artifacts, not the original design language.

The system must never silently edit the generated DXF/STEP/GLB and pretend that the source program has changed.

```text
source
  ↓
framework/kernel
  ↓
compiled CAD result
```

The reverse direction is always an explicit **source-edit operation**.

## 1.2 The kernel is the validity authority

The CAD kernel decides whether a requested engineering operation is valid.

A browser, code editor, LLM, Git client, renderer, or collaboration server must not independently declare an engineering operation valid.

For a server operation:

```text
client intent
    ↓
server process
    ↓
kernel candidate validation
    ↓
ACCEPT or REJECT
```

Code mutation happens only as part of an accepted process.

After source mutation, the resulting source is compiled/evaluated again and the resulting CAD state is validated before the process is committed to the live state.

## 1.3 The viewer is semantic, not a dumb image viewer

The browser viewer does not infer CAD semantics from pixels.

The compiled result must provide enough semantic information for the viewer to know:

- what an object is;
- stable object identity;
- its properties;
- which properties are editable;
- available interaction capabilities;
- source/provenance binding;
- parent/child/component relationships;
- geometric/topological targets;
- constraint information where exposed;
- validation/diagnostic state.

The viewer may therefore display properties and request property changes directly.

The viewer is **not** the engineering authority. It asks the server/kernel whether the requested change is acceptable.

## 1.4 The compiled result is not one invented proprietary CAD format

V5 uses established interchange/runtime formats wherever an established format is appropriate.

Initial output targets:

- **2D engineering drawing:** DXF.
- **3D mechanical CAD:** STEP AP242 (ISO 10303-242:2025).
- **3D browser visualization cache:** glTF/GLB where needed for efficient rendering.
- **Document/print output:** PDF.

STEP is the engineering interchange artifact for 3D; glTF/GLB is a viewer delivery artifact, not the source-authoring format. DXF is the 2D exchange projection. PDF is presentation output.

The exact exporter capabilities are constrained by what the V5 kernel actually represents. An exporter must not invent missing engineering semantics.

## 1.5 UMLCAD semantic manifest

A standard CAD artifact is necessary but not sufficient for the browser experience.

V5 therefore emits a **versioned UMLCAD semantic manifest** alongside standard artifacts.

The manifest describes the semantic relationship between the compiled source design and the exported artifact, including, where available:

- stable UMLCAD object IDs;
- object type/kind;
- component identity and instance identity;
- source symbol/provenance;
- property definitions and current values;
- property types/units;
- property editability;
- supported interaction capabilities;
- references to geometry/topology in the artifact;
- constraint associations;
- dependency relationships;
- diagnostics;
- model revision/build identity.

The manifest is **not a second geometry authority**. It describes the semantics and bindings of the authoritative kernel result.

## 1.6 Viewer interaction is an intent protocol

The browser sends semantic intents such as:

```text
select object
inspect property
change property
move/drag target
measure
request process
undo process
redo process
open source target
```

Raw pointer events are local browser concerns and are not the server API.

The server receives semantic operations with stable target IDs and revision information.

## 1.7 WebSocket belongs outside the framework

The UMLCAD framework has no WebSocket dependency.

The WebSocket server belongs to the project runtime/host and exposes the already-defined semantic operation/session contracts.

The framework can be embedded in:

- the local project server;
- an online workspace runtime;
- CLI/build tooling;
- future other hosts.

The transport can later be WebSocket, HTTP, IPC, or another adapter without changing the CAD kernel.

For live browser collaboration, WebSocket is the first transport.

## 1.8 Live state is separate from Git commit state

Git is project history and durable source persistence.

The live collaborative design state is managed by the project server and can advance without waiting for a Git commit.

```text
LIVE REVISION 184
       ↓
WebSocket broadcast
       ↓
all connected clients see it

then, whenever a designer chooses:

LIVE REVISION 184
       ↓
Git commit
```

A browser must never wait for a Git commit merely to see an accepted live design change.

## 1.9 Every project is a Git repository

Each project is represented by one Git repository.

The repository contains, at minimum:

```text
project/
├── source/
├── components/
├── assemblies/
├── generated/       # optional tracked/generated artifacts by policy
├── umlcad.json
└── .git/
```

A hosted platform may store many repositories, but each project remains independently versioned and addressable as a Git repository.

Large generated artifacts may use Git LFS or external artifact storage without changing the project-level Git model.

## 1.10 Runtime isolation is per project execution

Project source is never executed directly inside the platform control process.

The code compilation/evaluation environment is isolated.

The first V5 runtime implementation should use Docker/container isolation behind a runtime abstraction, so the architecture is not permanently coupled to one isolation technology.

Conceptually:

```text
Project repository (persistent)
        ↓ mounted/checked-out
isolated project runtime
        ├── compiler
        ├── UMLCAD framework
        ├── CAD kernel
        ├── source editor bridge
        ├── process engine
        └── optional LLM tool bridge
```

The runtime is disposable; the repository and project history are not.

---

# 2. V5 system topology

```text
                             PLATFORM / LOCAL MACHINE

       persistent project repository
                    │
                    │ working tree / revision
                    ▼
          ┌───────────────────────┐
          │ isolated project     │
          │ runtime               │
          │                       │
          │ UMLCAD framework      │
          │ CAD kernel            │
          │ compiler/evaluator    │
          │ process manager       │
          │ source-edit engine    │
          │ artifact builder      │
          └───────────┬───────────┘
                      │
                live revision
                      │
                 WebSocket
                      │
                      ▼
          ┌───────────────────────┐
          │ browser client        │
          │                       │
          │ viewer                │
          │ selection             │
          │ property editor       │
          │ process/undo UI       │
          │ source editor UI      │
          └───────────────────────┘

External services (optional):
  Git remote / hosting
  LLM provider or local LLM runtime
  authentication provider
  artifact/object storage
```

The browser can also operate against a local project runtime without an online platform.

---

# 3. Repository architecture

V5 starts empty deliberately. Do not copy V4's entire tree into V5.

The V5 tree should be grown around explicit boundaries:

```text
UMLCAD.V.5/
├── framework/                 # transport-free CAD framework
│   ├── src/
│   └── test/
│
├── kernel/                    # geometry, constraints, topology, validation
│   ├── src/
│   └── test/
│
├── compiler/                  # source/project → compiled CAD result
│   ├── src/
│   └── test/
│
├── artifacts/                 # standard artifact projections
│   ├── dxf/
│   ├── step/
│   ├── gltf/
│   └── pdf/
│
├── semantic/                  # versioned UMLCAD semantic manifest/contracts
│   ├── src/
│   └── test/
│
├── protocol/                  # transport-neutral client/server semantic protocol
│   ├── src/
│   └── test/
│
├── runtime/                   # server/host, NOT part of framework
│   ├── src/
│   ├── test/
│   └── adapters/
│
├── isolation/                 # Docker/runtime isolation contract and images
│   ├── docker/
│   └── test/
│
├── collaboration/             # live revision/process/session infrastructure
│   ├── src/
│   └── test/
│
├── backward/                  # source-target/provenance/edit translation
│   ├── src/
│   └── test/
│
├── client/                    # browser-only application
│   ├── src/
│   └── test/
│
├── docs/
│
└── test/                      # cross-boundary integration tests where appropriate
```

This is a target structure, not a command to create every directory empty on day one. A directory becomes real when its first contract is implemented.

### Ownership rule

`framework/`, `kernel/`, and the mathematical CAD contracts must not import from `runtime/`, `client/`, `collaboration/`, Docker APIs, WebSocket libraries, browser APIs, Git APIs, or LLM APIs.

`protocol/` may depend on semantic contract types but must not depend on a specific transport.

`runtime/` owns transports and host lifecycle.

`client/` owns UI/rendering/camera behavior.

`backward/` owns the reverse mapping from semantic targets/intents to source-editable targets.

---

# 4. Preserve from V4, redesign where required

The V4 work is reference material, not an automatic source tree.

Preserve and revalidate these V4 principles:

- one engineering geometry authority;
- typed parameters;
- constraints as mathematical primitives;
- explicit residual/Jacobian/scaling contracts;
- transactional regeneration;
- stable semantic identity;
- semantic references rather than array indexes/coordinates;
- deterministic evaluation/order;
- topology derived from authoritative geometry/relations;
- dimensions as engineering semantics;
- structured diagnostics;
- invalid state rejection;
- deterministic exports;
- adversarial/property testing.

Redesign these V4 architectural assumptions:

- native macOS viewer as a required product path;
- browser as merely a passive projection;
- browser/client-independent backward package as an isolated future layer;
- WebSocket in or coupled to the framework;
- generated output treated as only an opaque projection;
- Git commit as the live synchronization mechanism;
- source editing dependent on an IDE/editor undo stack;
- server authorization being mixed into CAD contracts.

---

# 5. Compiled CAD result contract

## 5.1 Compilation output

A successful build produces a build directory/revision containing:

```text
build/<revision>/
├── manifest.json
├── semantic.json
├── 2d/...
├── 3d/...
├── visualization/...
└── diagnostics.json
```

The exact artifact set depends on project content.

Examples:

```text
2D drawing only:
  drawing.dxf
  drawing.pdf
  semantic.json

3D mechanical part:
  part.step
  part.glb
  semantic.json

Assembly:
  assembly.step OR referenced part artifacts + assembly manifest
  assembly.glb
  semantic.json
```

## 5.2 Build identity

Every successful compiled result gets:

- source revision ID;
- build ID;
- kernel/framework version;
- artifact hashes;
- semantic manifest version;
- project ID;
- deterministic diagnostics summary.

The browser must be able to prove which compiled result it is displaying.

## 5.3 Manifest binding

The semantic manifest must bind stable UMLCAD objects to their standard-artifact representations wherever the target format permits reliable binding.

Where an external format cannot preserve UMLCAD identity, the mapping must be explicit and deterministic in the manifest. It must never depend on screen coordinates or arbitrary rendering order.

---

# 6. Part and assembly model

## 6.1 Parts

A component/part may compile to its own standard artifact.

For 3D mechanical parts, STEP AP242 is the primary engineering artifact target.

## 6.2 Assembly

An assembly is not required to flatten every part into one file for browser viewing.

The project can maintain:

```text
parts/
  wheel.step
  brake.step
  chassis.step

assembly.json
```

The assembly information can identify:

- component instance ID;
- referenced part artifact/version;
- placement transform;
- component configuration;
- assembly relationships/constraints;
- semantic parent/child hierarchy;
- visibility/state.

The browser can load the parts and construct the live assembly presentation from those references.

A final monolithic STEP assembly export may also be generated where the kernel/exporter supports it.

## 6.3 Component reuse

One physical part definition may be instantiated multiple times.

Instance identity must be distinct from definition identity.

Changing the part definition updates all dependent instances only through the normal validated compilation/process path.

---

# 7. Browser viewer contract

The browser is an actual application in V5.

It performs:

- loading standard artifacts/visualization artifacts;
- semantic binding using the UMLCAD manifest;
- pan/zoom/orbit/view control;
- selection and hit testing;
- highlighting;
- property display/editing;
- constraint display;
- diagnostics display;
- process history;
- undo/redo requests;
- source-target navigation;
- live WebSocket synchronization.

It does **not** own:

- engineering validity;
- constraint solving authority;
- source truth;
- Git history authority;
- compilation truth;
- authorization authority.

## 7.1 Selection

Selection must resolve to a stable semantic object ID.

A viewer hit result may contain:

```text
artifact object
geometric subelement
semantic object ID
component instance ID
source target ID
```

The browser must never send only screen coordinates when a stable semantic identity is available.

## 7.2 Properties

The semantic manifest defines the properties the viewer may display/edit.

A property schema should include:

- property ID;
- type;
- unit;
- current value;
- range/domain where applicable;
- editable status;
- validation hints;
- target semantic object;
- source binding.

UI validation may provide immediate feedback but cannot replace kernel validation.

---

# 8. Semantic operation protocol

The protocol is transport-neutral.

Initial WebSocket transport messages are JSON objects conforming to the protocol contract.

Every mutating operation carries:

```text
project ID
session ID
client ID
base live revision
process/operation ID
operation kind
semantic target
requested change
```

The server response carries:

```text
accepted/rejected
process ID
base revision
result revision (when accepted)
diagnostics
source change summary
artifact/build identity
```

## 8.1 Example property edit

```json
{
  "kind": "property.change",
  "target": "hole-17",
  "property": "diameter",
  "value": 12,
  "baseRevision": 842
}
```

The server does not directly trust `value = 12` as valid CAD. It asks the kernel to evaluate the requested operation.

## 8.2 Revision rule

A mutating request based on stale live state must be rejected or explicitly rebased according to the declared conflict policy.

Do not silently apply stale requests to a newer design state.

---

# 9. Server process model

The server owns a live process engine.

Every accepted design change becomes a named process.

```text
Process
├── process ID
├── human-readable name
├── author/client
├── mode: human | llm | system
├── base live revision
├── requested intent
├── kernel candidate result
├── source patch
├── inverse/compensating patch
├── resulting revision
├── build ID
├── artifact hashes
└── diagnostics
```

The process is the unit of user-visible history, collaboration, and browser undo/redo.

## 9.1 Transaction pipeline

```text
browser intent
      ↓
create process
      ↓
resolve stable semantic target
      ↓
kernel candidate operation
      ↓
ACCEPT / REJECT
      ↓ accepted
source mutation
      ↓
compile/evaluate
      ↓
full kernel validation
      ↓
artifact + semantic manifest build
      ↓
commit live revision
      ↓
WebSocket broadcast
      ↓
optional Git checkpoint
```

Any failure after candidate acceptance must prevent publication of a new live revision and must leave the previous accepted live state intact.

---

# 10. LLM source-edit contract

The LLM is not the engineering validator.

The LLM may translate an accepted semantic operation into a source patch.

For machine-authored source changes, V5 requires **one-step exact patching** whenever the operation can be represented as a deterministic replacement.

Example:

```text
old:
hole(diameter = 10)

new:
hole(diameter = 12)
```

The process stores the exact old/new region and resulting source revision.

The LLM must not be allowed to:

- directly publish a live revision;
- bypass kernel validation;
- mutate generated artifacts as a substitute for source editing;
- silently change unrelated source;
- make undocumented additional design edits.

After patching, the source is compiled and validated. The resulting model must satisfy the original requested operation and all global validity rules.

If the generated code does not produce the requested valid result, the process is rejected/rolled back.

---

# 11. Human code-edit contract

Human editing is a first-class authoring path.

A human does not depend on the text editor's undo stack for CAD-process undo.

The server records human source changes as named processes when they enter the project runtime.

A process must record the exact source change or an explicit semantic compensating operation.

For example:

```text
Process: Add front suspension mounting function
Source change: add function createFrontMount(...)
Inverse: remove that exact function/change
```

The user may therefore undo the CAD process from the browser even if the code editor has been closed, restarted, or changed its local undo history.

When an inverse patch cannot safely be applied because the source has diverged, the server must reject the automatic undo and report the conflict rather than silently corrupt source.

Git remains an additional durable recovery mechanism, but browser process undo must not depend on the editor's in-memory undo buffer.

---

# 12. Undo/redo contract

Undo and redo operate on named CAD processes, not editor keystrokes.

```text
Browser
  ↓
UNDO Process P184
  ↓
server resolves inverse/compensating operation
  ↓
kernel validates resulting candidate
  ↓
source change
  ↓
compile
  ↓
validate
  ↓
new live revision
```

Undo does not erase history.

It produces another live revision describing the reversal.

Example:

```text
842  original
843  P184: hole 10 → 12
844  undo P184: hole 12 → 10
```

This preserves an auditable process history and makes collaborative state transitions deterministic.

Redo is similarly a new validated process unless conflict rules require explicit reapplication/rebase.

---

# 13. Live collaboration and concurrency

## 13.1 No Git wait for live updates

A successful process becomes visible to connected browsers as soon as the new live revision is validated and published.

Git commit is independent.

```text
process accepted
    ↓
new live revision
    ↓
WebSocket
    ↓
all connected browsers

later:
new live revision
    ↓
Git commit
```

## 13.2 Server ordering

A project live session has one authoritative ordered revision stream.

Every mutating operation is attached to exactly one base revision.

The server serializes operations at the project boundary or uses an equivalent transaction mechanism that produces one deterministic total order.

## 13.3 Conflict policy

If two users modify independent objects/properties:

```text
A: wheel diameter
B: brake thickness
```

the server may accept both in sequence.

If two users modify the same semantic property from the same base revision:

```text
A: wheel diameter 18 → 20
B: wheel diameter 18 → 22
```

the second request must not silently overwrite the first. It is rejected, rebased explicitly, or resolved through a semantic conflict process.

Text merge alone is insufficient for CAD conflicts.

## 13.4 Local Git workflow

Each project is a Git repository.

A designer can:

- continue live work without committing;
- make local Git commits whenever desired;
- push/sync to a remote when desired;
- recover or inspect durable project history independently of live collaboration.

The live server uses the repository revision/checkpoint information but does not require a Git commit for every live operation.

---

# 14. Runtime isolation

## 14.1 Goal

Execute untrusted/project-owned code and compiler workloads separately from the control plane.

## 14.2 Initial implementation

Use Docker as the first runtime isolation implementation.

Introduce a runtime interface so the project server depends on:

```text
ProjectRuntime
  create
  start
  execute
  compile
  stop
  destroy
```

The Docker adapter implements that contract.

Do not scatter Docker API calls throughout CAD or collaboration code.

## 14.3 Isolation rules

The isolated runtime must have:

- project working tree access;
- required framework/kernel/compiler binaries/modules;
- restricted network by default;
- bounded CPU/memory/time;
- separate temporary storage;
- deterministic environment version;
- explicit artifact output directory.

LLM access, if enabled, must use a controlled server/tool boundary rather than unrestricted shell execution.

## 14.4 Persistence

The container is disposable.

Persistent project state is outside the container:

```text
Git repository
process/history storage
artifact storage
project configuration
```

The runtime can be recreated from these inputs.

---

# 15. Git and artifact policy

## 15.1 Source

Source code is always versioned in Git.

## 15.2 Standard artifacts

Generated artifacts may be:

- tracked directly when small and text-friendly;
- stored through Git LFS when large;
- stored in an artifact/object store with hash/version references when appropriate.

The project Git metadata must always be able to identify exactly which artifact belongs to which source revision/build.

## 15.3 Artifact reproducibility

A committed source revision plus declared UMLCAD/kernel/compiler version must reproduce the same semantic/build identity within declared deterministic tolerances.

A build must not silently depend on a mutable machine-wide CAD installation.

---

# 16. Security and authorization boundary

The V5 execution plan intentionally does not make authentication/authorization part of the CAD framework.

The project server is the security boundary for runtime operations.

For an online deployment, GitHub/Git hosting or another explicitly chosen identity provider may supply repository identity/authorization. The server must verify the established authorization context before acting on a project.

The browser is not itself an authorization authority.

The CAD kernel does not know user passwords/roles.

The LLM does not receive project authority merely because it produced text.

---

# 17. Framework API boundary

The framework must expose transport-neutral operations for:

- create/build project;
- evaluate source;
- create/query objects;
- parameter values;
- constraints;
- geometry queries;
- topology;
- dimensions;
- engineering validation;
- source/provenance binding;
- semantic model projection;
- artifact projection.

It must not expose:

- WebSocket server objects;
- HTTP request/response objects;
- browser DOM objects;
- Git client implementation;
- Docker APIs;
- editor APIs;
- LLM APIs.

---

# 18. Kernel architecture

V5 kernel work starts from the verified V4 mathematical contracts but is rebuilt into a clean dependency structure.

Suggested internal order:

```text
numeric primitives
      ↓
geometry
      ↓
constraints
      ↓
solver
      ↓
topology/references
      ↓
dimensions/engineering rules
      ↓
validation
```

No renderer or viewer package may become an upstream dependency.

## 18.1 Geometry

Initial V5 geometry coverage should begin with the exact primitives already established in V4 and expand only under explicit contracts.

Every primitive needs:

- canonical parameterization;
- point/tangent queries where meaningful;
- closest-point/parameter queries where meaningful;
- intersections;
- bounding box;
- endpoint/periodic semantics;
- degeneracy policy.

## 18.2 Constraints

Retain explicit mathematical contracts for:

- residual;
- units;
- scaling;
- Jacobian;
- valid domain;
- branch behavior;
- degeneracy behavior;
- acceptance;
- diagnostics.

## 18.3 Solver

Preserve the V4 distinction between engineering/raw Jacobian semantics and solver preconditioning.

The solver objective must not silently change because of numerical normalization.

## 18.4 Topology

Stable semantic topology is kernel-owned.

The viewer must never invent topology from screen proximity.

## 18.5 References

Identity must never depend on:

- array position;
- coordinate value;
- render ordering;
- DXF handle allocation.

---

# 19. Validation architecture

The authoritative validation pipeline is:

```text
geometry
   ↓
constraints
   ↓
numerical solve
   ↓
topology
   ↓
references
   ↓
engineering rules
   ↓
overall validity
```

A build is publishable only if its required validation profile succeeds.

Diagnostics must distinguish at least:

- invalid geometry;
- unsatisfied constraint;
- contradiction;
- numerical singularity;
- underconstraint;
- overconstraint/redundancy;
- stale reference;
- topology invalidity;
- engineering-rule violation;
- source/build failure;
- unsupported export state.

---

# 20. Export architecture

Exports are projections from one validated kernel result.

## 20.1 DXF

DXF emission must use authoritative 2D geometry, layers, dimensions, and annotations where supported.

Do not infer engineering meaning from SVG or browser geometry.

## 20.2 STEP AP242

STEP AP242 is the primary 3D engineering exchange target.

The V5 STEP exporter must define its supported semantic subset explicitly and reject/diagnose unsupported model states instead of emitting misleading geometry.

## 20.3 glTF/GLB

glTF/GLB is a browser delivery/visualization projection.

It may be optimized for fast viewer rendering and can contain a scene hierarchy, meshes, materials, and IDs/bindings needed by the viewer.

It is not a substitute for the parametric source or engineering kernel.

## 20.4 PDF

PDF is a presentation/document projection.

PDF output must be derived from validated drawing/sheet semantics.

---

# 21. Semantic manifest architecture

The manifest is versioned independently of individual UI implementations.

Example conceptual shape:

```json
{
  "schema": "uml-cad-semantic/1",
  "project": "car",
  "revision": 843,
  "build": "build-843",
  "objects": [
    {
      "id": "wheel-fl",
      "kind": "component-instance",
      "properties": {
        "diameter": { "value": 540, "unit": "mm", "editable": true }
      },
      "capabilities": ["select", "inspect", "edit-property"],
      "artifactBindings": ["..."],
      "source": {
        "file": "source/wheel.ts",
        "symbol": "Wheel"
      }
    }
  ]
}
```

The exact schema must be designed from real kernel/build outputs, not invented independently of the model.

Tests must verify that semantic identity remains stable across deterministic recompilation.

---

# 22. Backward/source-edit architecture

Backward editing is a first-class V5 subsystem.

Its responsibility is:

```text
semantic target
    ↓
source/provenance target
    ↓
accepted semantic operation
    ↓
source patch
    ↓
framework rebuild
    ↓
kernel validation
    ↓
new compiled result
```

The backward subsystem may use AST parsing/rewrite tools, source maps, symbol indexing, or structured source bindings.

It must not perform blind text replacement when the target can be represented semantically.

LLM-generated patches still pass through the same source-edit and validation gates.

---

# 23. Code editor architecture

The code editor is a client feature and/or a server-side workspace service, but its undo buffer is not the CAD transaction authority.

The editor must expose source mutations to the server process manager as explicit revisions/patches.

The server records:

```text
source base revision
source after revision
patch
process ID
author
reason/intent when available
```

The server is therefore able to undo a process independently of editor state.

---

# 24. LLM architecture

The LLM is an optional authoring assistant.

It may receive:

- user intent;
- semantic target/property;
- relevant source context;
- relevant diagnostics;
- allowed source-edit scope.

It should return a constrained source-edit proposal.

The server then:

1. validates patch shape;
2. applies it in isolation;
3. compiles;
4. runs kernel validation;
5. verifies that the requested semantic effect occurred;
6. commits the live process only if all gates pass.

LLM success is therefore **not** synonymous with CAD validity.

---

# 25. Browser/server API lifecycle

Initial connection:

```text
browser
  ↓
connect WebSocket
  ↓
establish project/session
  ↓
receive current live revision + manifest/artifact references
  ↓
load viewer
```

Live update:

```text
process accepted
  ↓
live revision N+1
  ↓
broadcast event
  ↓
browsers fetch changed artifact/manifest or incremental update
  ↓
viewer switches to N+1
```

The first implementation may reload the affected artifact rather than inventing a complex delta protocol.

Incremental semantic deltas are a later optimization, not a prerequisite for V5 correctness.

---

# 26. Initial implementation strategy

V5 implementation must proceed in bounded phases.

## Phase 0 — architecture freeze

Create:

- `README.md`;
- `ARCHITECTURE.md`;
- `EXECUTION_PLAN.md` (this document);
- `DEVELOPMENT_METHOD.md`;
- `VERIFICATION.md`;
- protocol/semantic contracts documentation.

Acceptance:

- no contradictory framework/runtime definitions;
- no desktop client dependency;
- WebSocket explicitly outside framework;
- live state vs Git checkpoint explicitly defined;
- runtime isolation boundary explicit.

## Phase 1 — repository/build skeleton

Implement:

- root package/build configuration;
- strict TypeScript configuration;
- framework package;
- kernel package;
- test harness;
- CI baseline;
- deterministic build IDs/version metadata.

Acceptance:

- clean checkout installs;
- type checks;
- tests execute;
- no browser/runtime dependency enters kernel/framework.

## Phase 2 — kernel foundation

Rebuild/port the verified V4 mathematical kernel contracts into the V5 dependency structure.

Order:

1. numeric primitives;
2. geometry primitives;
3. references/identity;
4. constraints;
5. solver;
6. topology;
7. dimensions;
8. validation.

No viewer is built during this phase except test fixtures.

Acceptance:

- deterministic tests;
- adversarial cases;
- transactional regeneration;
- no parallel geometry authority.

## Phase 3 — source/program/compiler

Implement the code-driven project model:

```text
project source
   ↓
module resolution
   ↓
framework program
   ↓
kernel state
   ↓
validation
```

Add deterministic source/project registration and build identity.

Acceptance:

- sample part compiles from source;
- invalid source/build does not publish;
- repeated compile is deterministic.

## Phase 4 — semantic manifest

Implement the versioned semantic result generated from the validated build.

Acceptance:

- stable object identity;
- properties and editability;
- source provenance;
- artifact bindings;
- deterministic serialization;
- schema validation.

## Phase 5 — standard artifact exporters

Implement the minimum viable projections:

1. DXF for 2D;
2. STEP AP242 for supported 3D model subset;
3. glTF/GLB visualization projection where required;
4. PDF for supported drawing/sheet subset.

Acceptance:

- golden fixtures;
- deterministic export;
- invalid/unsupported state rejection;
- source-to-artifact identity traceability.

## Phase 6 — browser viewer

Build the client-only browser application.

Capabilities in order:

1. load build;
2. camera/view controls;
3. semantic selection;
4. property inspection;
5. property editing request UI;
6. diagnostics;
7. process history;
8. undo/redo request UI;
9. source navigation.

The browser must not contain CAD-solving authority.

## Phase 7 — transport-neutral protocol + WebSocket runtime

Implement protocol types first, transport adapter second.

Then implement server WebSocket transport outside the framework.

Acceptance:

- connect;
- current revision handshake;
- semantic selection/query;
- property edit request;
- validated result broadcast;
- stale revision rejection;
- clean disconnect/reconnect.

## Phase 8 — process manager and transactional source editing

Implement:

- process IDs/names;
- base/result revisions;
- source patches;
- inverse/compensating patches;
- build artifacts;
- process history;
- browser undo/redo.

Acceptance:

- human source edit produces a named process;
- LLM patch produces one-step process where applicable;
- undo never uses editor-local undo;
- rollback preserves prior live state;
- conflict is explicit.

## Phase 9 — isolated runtime

Implement project runtime interface and Docker adapter.

Acceptance:

- project code runs only inside isolated runtime;
- runtime can be destroyed/recreated;
- repository persists;
- resource/network restrictions are enforced;
- artifact output is captured deterministically.

## Phase 10 — Git project integration

Implement:

- repository initialization;
- working tree/revision management;
- project-level commit checkpoint;
- artifact association;
- optional LFS/object storage adapter;
- branch/remote operations as separate services.

Acceptance:

- a project can be fully recovered from Git + declared artifacts;
- live revisions do not require commits;
- a chosen live revision can be committed with its exact build identity.

## Phase 11 — collaboration

Implement:

- multi-client sessions;
- ordered live revisions;
- semantic conflict detection;
- reconnect/resync;
- process attribution;
- client notification.

Acceptance:

- two clients observe accepted operations in the same order;
- simultaneous conflicting property edits never silently overwrite;
- a reconnecting client can recover the current live revision.

## Phase 12 — code editor + LLM bridge

Implement the optional browser/server code editing and LLM authoring path.

Acceptance:

- editor changes are process-tracked;
- LLM changes are constrained to allowed scope;
- every change compiles and validates;
- failure never publishes a partial state;
- undo/redo is independent of editor history.

## Phase 13 — online deployment model

Only after local project runtime and collaboration are solid:

- persistent project repositories;
- isolated runtime orchestration;
- artifact storage;
- WebSocket session routing;
- authorization integration;
- quotas/resource controls;
- observability.

Docker is the initial runtime packaging mechanism; orchestration is a platform concern, not a framework concern.

---

# 27. Testing strategy

Every architectural boundary gets contract tests.

## Kernel

- mathematical unit tests;
- invariance/property tests;
- degeneracy/adversarial tests;
- solver stress tests;
- deterministic repeated execution.

## Compiler

- source fixtures;
- deterministic output;
- failed build atomicity;
- provenance correctness.

## Manifest

- schema tests;
- identity stability;
- artifact binding tests;
- property capability tests.

## Protocol

- request/response schema tests;
- revision semantics;
- stale-request behavior;
- deterministic error codes.

## Runtime

- isolation tests;
- resource-bound tests;
- runtime recreation;
- artifact collection.

## Collaboration

- ordered revision tests;
- concurrent edits;
- conflict detection;
- reconnect/resync;
- undo/redo process ordering.

## LLM/source editing

Use deterministic fake LLM responses in tests. Do not depend on a live LLM service for the canonical CI suite.

Test:

- valid one-step patch;
- unrelated patch rejection;
- syntactically invalid patch;
- compile failure;
- kernel rejection;
- semantic-effect mismatch;
- rollback;
- inverse patch.

---

# 28. Definition of GREEN for V5

A V5 subsystem is GREEN only when:

1. its boundary is explicitly documented;
2. it has one authoritative owner;
3. ordinary cases are tested;
4. invalid and degenerate cases are tested;
5. failure is transactional where required;
6. output is deterministic where promised;
7. no adjacent layer has silently become a second authority;
8. focused tests pass;
9. the complete V5 verification suite passes;
10. the execution-state documentation matches the actual repository.

A browser demo is not enough.

---

# 29. Explicit anti-goals

V5 must not become:

- an OCCT-style application object hierarchy copied wholesale into UMLCAD;
- a browser-only CAD kernel;
- a WebSocket-dependent framework;
- a GUI-driven geometry database;
- a generated-file-as-source CAD editor;
- an LLM-authority CAD engine;
- an editor-undo-dependent process system;
- a Git-commit-per-mouse-operation collaboration system;
- a desktop-app-required product;
- a Docker-dependent framework API.

Docker is an implementation of runtime isolation, not a UMLCAD programming concept.

WebSocket is an implementation of live transport, not a UMLCAD framework concept.

Git is durable project history, not the live CAD synchronization protocol.

---

# 30. First implementation slice

Do not start by implementing WebSocket or the browser UI.

The first coding slice after this plan is:

```text
V5 repository/build skeleton
        ↓
framework/kernel boundary
        ↓
minimal source → kernel evaluation
        ↓
validated semantic manifest
        ↓
one deterministic sample CAD artifact
```

Only after this slice is GREEN should the project add the client/server runtime.

The first sample should prove the entire foundational concept with a deliberately small model:

```text
source code
   ↓
parameterized object(s)
   ↓
kernel validation
   ↓
semantic object + editable property
   ↓
3D artifact / 2D artifact as applicable
   ↓
manifest
```

This is the architectural proof that V5 is a **code-driven CAD compiler + semantic result + live client**, rather than a conventional GUI CAD program.

---

# 31. Execution discipline

For every future implementation item:

1. Read this plan and the relevant current V5 contracts.
2. Inspect existing code before creating a new authority.
3. Write/freeze the exact contract for the item.
4. Implement the smallest coherent slice.
5. Add focused tests.
6. Add adversarial/edge tests.
7. Run full verification.
8. Update documentation/status.
9. Commit the bounded change.
10. Do not implement the next architectural layer merely because a demo can be made to work.

The V5 repository should always make it possible to answer:

```text
What is authoritative?
Where does validity live?
What changed?
What revision am I seeing?
Can this process be undone?
Which source produced this artifact?
Which runtime executed it?
```

If those questions cannot be answered deterministically, the corresponding subsystem is not GREEN.
