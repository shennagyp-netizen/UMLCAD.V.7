# UMLCAD.V.7 — System-CAD Architecture

## 1. Authority and purpose

This document is the **active architectural authority for the System-CAD layer** of UMLCAD.V.7.

It is intentionally separate from `docs/MATH_AUTHORITY_ROADMAP.md`:

- `docs/MATH_AUTHORITY_ROADMAP.md` defines the certified mathematical authority.
- This document defines the System-CAD ownership, dependency direction, semantic contracts, C4 structure, canonical evaluation path, domain boundaries, and required architectural behavior built on that mathematical authority.
- `docs/UMLCAD_V7_MAIN_BRANCH_DEVELOPMENT_PROMPT.md` defines development discipline and validation law.
- `docs/doc.tex` is the published engineering handbook and must describe this architecture rather than invent a competing one.
- `docs/CONTINUATION_HANDOFF.md` records the current implementation/evidence state and points back to this document for architecture.

Repository audit baseline for this revision: exact `main` at `acb125ca3ad4ca01f5b87033ce63739f6a271163`.

The repository is the source of truth. Chat history is not an architectural source of truth.

---

## 2. Architectural objective

UMLCAD.V.7 is a **system-CAD architecture**, not a thin geometry application and not a copy of CATIA's internal implementation.

The target system is organized into **authority strata**, but those strata are **not a linear runtime dependency chain**. A domain may consume several authoritative producers without inheriting from them.

```
Authority strata (conceptual)
--------------------------------
Platform Foundation
Mathematical Authority
Science / Phenomena Services
CAD Engineering / Product Semantics
Engineering Resources
Specialized Engineering Domains
Application / Workflow / Presentation
```

The actual runtime dependency/consumption graph is defined separately in Section 10. The key distinction is:

```
Rust math                     .NET System-CAD
-----------                   ----------------
How the mathematics works    What the engineering system means
Geometry/math algorithms     Product/Part/Sketch/Feature meaning
Solver authority             Design intent and constraints as semantics
Topology mathematics         References and topology provenance
Numerical evidence           Evaluation planning/orchestration
                              Product/Assembly/BOM
                              Drawing/PMI
                              Sheet Metal
                              CAM
                              Knowledge/configuration
                              Lifecycle/collaboration
```

The Rust kernel is deliberately thin. The complete CAD system is therefore **not** compared kernel-to-kernel.

### Application Layer classification

All new production .NET libraries under `app/framework/libraries/` belong to one architectural **Application Layer**. The older `dotnet/` implementation remains legacy and is intentionally outside the new dependency graph. They own System-CAD and engineering meaning and may be internally decomposed into contracts, semantics, engines, domains, orchestration, presentation, and infrastructure-facing services.

The mathematical kernel is outside the Application Layer under `kernel/`. It is the mathematical authority, not another .NET application library.

Kernel access from the Application Layer is centralized through exactly one dedicated .NET kernel-access library. The active implementation gateway is `app/framework/libraries/UMLCAD.Kernel.Client`. Its Rust/HTTP/native mechanics are implementation details confined to that library and must not leak into other Application Layer libraries.

The Application Layer may contain many libraries and an internal dependency DAG, but that DAG must remain acyclic.

---

## 3. Authority hierarchy

### 3.1 Mathematical authority

The Rust mathematical layer is authoritative only for mathematical contracts it explicitly certifies.

CPU `f64` is the normative mathematical reference.

GPU implementations are accelerators. OCCT may challenge or realize a contract but never transfers semantic authority to itself.

### 3.2 System-CAD authority

The .NET System-CAD layer is authoritative for:

- CAD and engineering meaning;
- design intent;
- Product / Part / Body / Sketch / Feature semantics;
- Assembly / Occurrence / configuration semantics;
- references and publications;
- coordinate-frame semantics;
- dependency graphs;
- evaluation planning;
- deterministic semantic, evaluation and result identity;
- cache/invalidation behavior;
- immutable .NET-owned evaluation history/state;
- authoritative result integration;
- topology/reference provenance and evolution;
- Drawing / PMI meaning;
- Product Structure and derived BOM semantics;
- CAM meaning;
- Sheet Metal meaning;
- Knowledge / Configuration meaning;
- lifecycle and collaboration integration.

### 3.3 Derived representation

Viewer geometry, SVG, tessellations, drawing graphics, simulation visualization meshes and other presentation artifacts are derived representations.

They may be approximate for presentation.

They are never permitted to define engineering truth.

---

# 4. C4 Level 1 — System Context

C4 Level 1 describes UMLCAD.V.7 as one software system and shows only people and **external systems** around it. The Rust mathematical kernel is an internal container of UMLCAD.V.7 and therefore belongs in Level 2, not Level 1.

The external system context is:

```
                         Engineering User
                               |
                               v
                  +---------------------------+
                  |       UMLCAD.V.7          |
                  |       System-CAD           |
                  +-------------+-------------+
                                |
             +------------------+-------------------+
             |                  |                   |
             v                  v                   v
   Simulation / Solver     PLM / PDM / ERP     Manufacturing /
      Providers               Systems          NC Consumer
             ^                                     
             |                                     
        provider contract
```

The important system relationships are:

- The engineering user defines and inspects engineering intent through UMLCAD.
- The UMLCAD system invokes its internal mathematical authority through an explicit kernel contract.
- External simulation/solver providers implement provider-neutral phenomena contracts where applicable.
- PLM/PDM/ERP and other external systems integrate through explicit adapters.
- CAM ultimately emits deterministic machine-ready G-code/NC for an external NC/machine consumer.
- An external provider never becomes semantic authority merely because it performs a computation.

The Rust mathematical authority, native host, CAD Core, Science, resources, persistence, adapters, workflow, and presentation are **internal UMLCAD containers** and are decomposed in C4 Level 2.---

# 5. C4 Level 2 — Containers

The target containers are logical responsibilities; they do not require one project per box.

| Container | Authority / responsibility |
|---|---|
| Platform Foundation | IDs, canonicalization, serialization, diagnostics, cancellation, application infrastructure, shared primitives that do not own CAD meaning |
| Mathematical Authority | Rust CPU mathematical contracts and certified algorithms |
| Native Kernel / Contract Host | Exposes typed mathematical operations through process/native boundaries and manages computational adapters |
| Science | Physical quantities, material/physical properties, scientific models and phenomena semantics |
| Phenomena Simulation Service | Stable provider-neutral simulation service consumed by engineering domains |
| CAD Engineering Core | Product/Part/Body/Sketch/Feature meaning, references, frames, dependency graph, evaluation, result/provenance, topology bindings, configuration, knowledge and product structure |
| Engineering Resource Model | Machine, Tool, Fixture, Process and Capability facts |
| Specialized Engineering Domains | Sheet Metal, CAM, Kinematics, Drawing/PMI, and other peer engineering domains |
| Application / Workflow | User workflows, orchestration, commands, sessions and collaboration integration |
| Presentation | Viewer and derived presentation assets |
| Integration Adapters | External simulation, PLM/PDM/ERP, machine/postprocessor, file/exchange and other provider boundaries |
| Persistence | Durable specifications, revisions, lifecycle state, identities, evidence and cached derived data according to explicit persistence contracts |

**Important:** a specialized domain must consume CAD/Core truth through stable contracts. It must not acquire private access to a concrete kernel implementation.

---

# 6. C4 Level 3 — CAD Engineering Core Components

The canonical CAD semantic pipeline is composed of the following logical components.

```
CAD Specification
      |
      v
Reference / Publication Resolution
      |
      v
Dependency Graph
      |
      v
Evaluation Planner
      |
      v
Evaluation Engine
      |
      +----> Evaluation Cache / Invalidation
      |
      v
Kernel Contract Factory
      |
      v
Kernel Client / Native Boundary
      |
      v
Authoritative Mathematical Result
      |
      v
Result Integration
      |
      +----> Topology / Provenance / Evolution
      |
      v
Derived Representation
```

### 6.1 Specification model

Stores immutable semantic intent:

- definitions;
- parameters and expressions;
- constraints;
- feature operations;
- references/publications;
- configuration/context;
- component/template relationships;
- product structure;
- drawing definitions;
- manufacturing definitions where owned by the corresponding domain.

Specification does not contain hidden mutable solver state or viewer state.

### 6.2 Reference and publication resolver

Resolves semantic references against declared producers and authoritative results.

A reference is evaluated using:

- producer identity;
- authoritative result identity;
- semantic target;
- topology/provenance meaning;
- selector constraints;
- frame/context requirements;
- applicable configuration.

Resolution states include:

```
Resolved
Missing
Ambiguous
Indeterminate
Unsupported
```

Array positions such as `Faces[12]` are never persistent semantic identity.

### 6.3 Dependency graph

Represents explicit dependency edges between semantic objects.

Examples:

```
Sketch
  -> Profile
  -> Feature
  -> Result

Face publication
  -> Sketch support reference

Part
  -> Occurrence
  -> Product Structure
  -> BOM projection
```

Dependency order is deterministic. Cycles are explicit invalid states.

### 6.4 Evaluation planner

Converts semantic dependencies into an explicit execution plan.

The planner decides:

- what must be evaluated;
- in what dependency order;
- what can be reused from a valid cache;
- what becomes invalid after a semantic change;
- which kernel contract is required;
- which result identities are expected.

It does not redefine mathematical algorithms.

### 6.5 Evaluation engine

Executes the semantic plan through typed evaluators/services.

Its responsibilities include:

- deterministic evaluation identity;
- dependency closure;
- reference resolution;
- frame transformation;
- contract selection;
- kernel dispatch;
- result validation;
- result integration;
- cache semantics;
- explicit diagnostics.

### 6.6 Kernel contract factory

Transforms a validated semantic operation into an explicit backend-neutral mathematical request.

A CAD domain depends on this contract, not on concrete Rust types or a particular numerical/backend implementation.

### 6.7 Kernel client / native boundary

Provides transport/process isolation where required. The kernel boundary is stateless from the System-CAD perspective: each request contains explicit operation inputs and each response contains a new mathematical result/evidence. The kernel does not persist or advance CAD evaluation history.

It validates:

- schema;
- contract version;
- operation identity;
- input identity;
- status/result consistency;
- completeness of authoritative evidence;
- cancellation and transport failure.

A syntactically valid response is not automatically an authoritative result.

### 6.8 Result integration

Maps mathematical results into System-CAD authoritative results while preserving:

- result identity;
- operation identity;
- semantic source identity;
- frame;
- topology;
- provenance;
- evidence;
- diagnostics;
- status.

A mathematical result that cannot be integrated without losing its semantic identity must fail closed.

The resulting evaluation-history snapshot is created and owned by .NET. It records the ordered operation/result/evidence state of the current evaluation and is replaced by a new immutable snapshot on a completed rebuild. It is not a kernel cache, kernel database, or mutable kernel session.

### 6.9 Topology and provenance

Topology is authoritative only when supported by a certified mathematical result.

Topology bindings carry semantic provenance sufficient for references and derived representations.

Topology evolution must be explicit. When correspondence cannot be established, the system reports ambiguity/indeterminacy instead of guessing.

### 6.10 Representation derivation

Derived representations include:

- viewer meshes;
- SVG and drawing graphics;
- simplified display geometry;
- manufacturing visualization;
- simulation visualization.

Representation generation is downstream of authoritative results and never feeds semantic truth backward. No FP operation node invokes representation generation as part of authoritative evaluation.

---

# 7. C4 Level 4 — Critical Runtime Paths

The most important runtime paths are defined explicitly.

## 7.1 Feature evaluation path

```
FeatureSpecification
 -> resolve references
 -> resolve frames
 -> close dependencies
 -> construct evaluation identity
 -> select evaluator
 -> create mathematical contract
 -> invoke kernel authority
 -> validate authoritative result
 -> integrate topology/provenance
 -> update cache/invalidation
 -> publish immutable .NET evaluation-history snapshot
 -> [representation request is a separate downstream operation]
 -> expose semantic selection
```

## 7.2 Sketch-on-face path

```
Part/Body
 -> authoritative face publication
 -> semantic FaceReference
 -> Sketch definition
 -> sketch frame derived from referenced face
 -> sketch geometry + constraints
 -> constraint solver contract
 -> authoritative sketch result
 -> profile validation
 -> consuming feature evaluation
```

A sketch frame is derived from explicit face semantics, not viewer camera orientation.

## 7.3 Product/Assembly path

```
Part Definition
 -> Assembly Occurrence
 -> occurrence transform/context
 -> Product Structure
 -> BOM projection
 -> Drawing / CAM / PLM consumers
```

An occurrence transform places an instance. It does not mutate the source Part definition.

## 7.4 Drawing path

```
Authoritative Product/CAD state
 -> Drawing Definition
 -> Sheet
 -> View / Projection / Section / Detail
 -> Dimension / PMI / Annotation
 -> derived drawing graphics
```

Drawing elements refer to authoritative CAD/product state. SVG or viewer geometry is not reinterpreted as CAD truth.

## 7.5 CAM path

```
Authoritative CAD/Product state
 -> Manufacturing Definition
 -> Setup / Coordinate System
 -> Operation / Strategy
 -> Toolpath
 -> Postprocessor
 -> deterministic G-code / NC
```

A rendered toolpath is not equivalent to machine-ready NC.

## 7.6 Phenomena simulation path

```
Engineering domain
 -> IPhenomenaSimulationService
 -> IPhenomenaSimulationProvider
 -> simulation application / solver / implementation
 -> verified simulation result
 -> engineering consumer
```

The simulation adaptor/service represents **phenomena**, not a simulation of another application.

The service can dispatch to multiple implementations, including internal, external, CPU, GPU, specialized or remote simulation applications. Provider-specific types remain behind the adapter/provider boundary.

CAM may consume the simulation service directly when manufacturing behavior requires physical or process validation.

---

# 8. CAD Semantic Model

The CAD Core semantic model is organized around a few stable roots.

## 8.1 Document

A document is the semantic container for definitions and related engineering state.

It may contain:

- Part definitions;
- Product/Assembly definitions;
- Drawings;
- templates;
- component definitions;
- configurations;
- knowledge rules;
- publications;
- manufacturing definitions.

## 8.2 Part / Body / Sketch / Feature

Part and Body own design structure.

Sketch owns planar geometry/constraints and publishes semantically addressable profile/reference information.

Feature owns an operation and its declared semantic inputs/parameters.

A Feature does not itself become an authoritative geometric result merely because it has parameters.

## 8.3 Product / Assembly / Occurrence

Product Structure owns composition.

An Assembly contains Occurrences of definitions.

Each occurrence has:

- stable identity;
- definition identity;
- transform/context;
- configuration;
- quantity;
- suppression/visibility state;
- BOM participation;
- explicit parentage.

## 8.4 BOM

BOM belongs with CAD Product Structure as a **derived semantic projection of authoritative composition**.

```
Product Structure
      ↓
     BOM
      ↓
Drawing / CAM / PLM
```

BOM must never become a competing assembly authority.

## 8.5 Template / Component / Sheet / Canvas

Reusable templates and components are semantic definitions, not display-only files.

A component can be inserted into a document while preserving source-definition identity.

A sheet/canvas is a document-space container for drawing/presentation semantics. It is not an alternative geometric authority.

This model must support, for example:

```
Document
 ├── Part / Product definitions
 ├── inserted Components
 └── Drawing
      ├── Sheet 1
      │    ├── inserted drawing components/views
      │    └── annotations/details
      └── Sheet 2
```

Inserted instances retain semantic source/provenance identity.

---

# 9. Domain Boundaries

## 9.1 Science

Science owns reusable scientific facts and semantics:

- physical quantities;
- material/physical properties;
- phenomena definitions;
- scientific assumptions;
- simulation-service semantics.

Science does not become a CAD feature engine.

## 9.2 Phenomena Simulation Service

The service is stable and provider-neutral.

It defines what a domain requests from a simulation capability without defining a specific solver application as semantic truth.

A provider may invoke an external application, a local implementation, a specialized solver or another service.

Provider selection is an implementation concern unless explicitly declared semantic input.

Simulation results must identify:

- result identity;
- source inputs;
- assumptions;
- applicable frame/context;
- provider/contract identity where relevant;
- numerical/status evidence;
- provenance.

## 9.3 Engineering Resources

Engineering Resources owns reusable facts:

- Machine;
- Tool;
- Fixture;
- Process;
- Capability.

Resources are not hidden CAM state.

## 9.4 Sheet Metal

Sheet Metal owns sheet-metal semantics:

- thickness;
- bends;
- flanges;
- relief;
- folding/unfolding;
- flat-pattern;
- manufacturing constraints and validation.

It consumes CAD, Science and Engineering Resource contracts.

## 9.5 CAM

CAM owns manufacturing semantics:

- manufacturing definitions;
- setups;
- operations;
- strategies;
- toolpaths;
- postprocessors;
- machine-ready output.

It consumes authoritative CAD/Product/BOM data, resources, material/process information, applicable Sheet Metal results and phenomena simulation services.

## 9.6 Drawing / PMI

Drawing owns:

- sheets;
- views;
- projections;
- sections;
- details;
- dimensions;
- tolerances;
- GD&T;
- PMI;
- annotations;
- BOM views;
- standards.

Every drawing object that refers to engineering meaning must retain an authoritative semantic source/reference.

## 9.7 Kinematics

Kinematics owns behavioral/motion semantics and consumes Product/Assembly structure, constraints and mathematical authority.

It does not become a second assembly or solver authority.

## 9.8 Knowledge / Configuration

Knowledge and Configuration own:

- parameters;
- expressions;
- rules;
- checks;
- reactions;
- variants;
- templates;
- configuration-dependent semantics.

They affect evaluation identity when they affect authoritative results.

## 9.9 Lifecycle / PLM

Lifecycle/PLM owns:

- revision;
- release;
- workflow;
- collaboration;
- document/product integration.

It may govern lifecycle truth while remaining downstream of CAD geometry authority.

## 9.10 Viewer / Presentation

The viewer may:

- display representations;
- select semantic targets;
- focus/hide/show;
- inspect derived metadata.

The viewer may not:

- resolve CAD references independently;
- invent topology;
- define feature direction;
- convert display geometry into authoritative geometry;
- change manufacturing or physical truth.

---

# 10. Dependency Rules

## 10.1 Application Layer dependency law

The repository enforces the following executable architectural rules:

- every new production .NET framework project must live under `app/framework/libraries/` and is classified as Application Layer;
- production ProjectReferences must resolve only to other Application Layer production projects;
- the complete production .NET ProjectReference graph must be acyclic;
- exactly one production .NET library is designated as the kernel-access gateway;
- kernel transport, native-process, Rust-kernel implementation, and current kernel-host endpoint knowledge may exist only inside that gateway;
- no other Application Layer library may call the native kernel directly or reproduce the kernel transport;
- a kernel gateway may be consumed by Application Layer services, but it must not depend back on consumers in a way that creates a reverse architectural cycle;
- these rules are checked by `app_e2e` against the new `app/` tree and are required in CI.

A written dependency diagram is therefore not the enforcement mechanism. The repository's architecture gate is.

Two different diagrams are required and must never be conflated.

### 10.1 Authority strata

The conceptual strata are:

```
Platform Foundation
Mathematical Authority
Science / Phenomena Services
CAD Engineering Core / Product Structure
Engineering Resources
Specialized Engineering Domains
Application / Workflow / Presentation
```

These strata describe **ownership and conceptual placement**, not a requirement that every layer depend on the next layer only.

### 10.2 Actual dependency / consumption graph

For the dependency graph below, A → B means **A consumes an explicit contract/authority supplied by B**.

```
CAD Engineering Core → Mathematical Contracts
CAD Engineering Core → Science Contracts (where required)
CAD Engineering Core → Configuration / Knowledge

Sheet Metal → CAD Core
Sheet Metal → Science
Sheet Metal → Engineering Resources
Sheet Metal → Phenomena Simulation Service (where required)

CAM → CAD Core / Product Structure / BOM
CAM → Engineering Resources
CAM → Sheet Metal (where applicable)
CAM → Science / Material / Process semantics
CAM → Phenomena Simulation Service (where required)

Drawing/PMI → CAD Core / Product Structure
Kinematics → Product / Assembly Structure
Kinematics → Mathematical Contracts

Lifecycle/PLM → CAD / Product / Configuration state

Application/Workflow → CAD Core and domain/application contracts
Presentation → Derived Result / Representation contracts

Simulation Provider → Phenomena Simulation Service
Integration Adapter → external provider/system
```

Platform Foundation is shared infrastructure consumed by the relevant internal components; it is not an upstream business-domain parent.

No specialized domain is required to depend on another domain's private implementation. Cross-domain use occurs through explicit stable contracts.

### Forbidden dependencies

A domain must not:

- depend directly on a concrete Rust mathematical type when a contract exists;
- embed its own solver/geometry kernel;
- use viewer state as semantic input;
- use topology array position as persistent identity;
- make BOM independent from Product Structure;
- make Drawing reconstruct CAD meaning from meshes;
- make CAM invent product structure;
- allow a simulation provider to redefine Science semantics;
- allow one specialized domain to acquire another domain's private state;
- use a compatibility/legacy API as a second canonical semantic engine.

### Allowed dependency form

```
Consumer domain
    ↓
Stable semantic contract
    ↓
Authoritative producer / provider / evaluator
    ↓
Concrete implementation
```

This graph is the architectural dependency rule. It is intentionally different from project-reference topology and from implementation order.---

# 11. Identity, Frames, Provenance and Lifecycle

Every authoritative semantic result has distinct identities for:

- semantic definition;
- revision;
- evaluation;
- authoritative result;
- representation;
- external/provider identity;
- lifecycle state.

Do not collapse these into one timestamp, database key or process-local hash.

Every authoritative result is evaluated in explicit frames.

Relevant frames include:

```
World
Document
Part
Body
Sketch
Face
Occurrence
Drawing View
Manufacturing Setup
Simulation
```

Frame identity and orientation are semantic inputs.

Topology/provenance is retained with results rather than reconstructed from presentation artifacts.

---

# 12. Specification, Evaluation, Result, Representation

The system must preserve:

```
Specification
     ↓
Evaluation
     ↓
Authoritative Result
     ↓
Representation
```

### Specification

What the engineer means.

### Evaluation

How dependencies are executed.

### Authoritative Result

What the system has proved.

### Representation

How the result is consumed or displayed.

This separation is mandatory even when a simple capability currently collapses several implementation projects into one assembly.

---

# 13. Cache and Recompute Architecture

Cache is a semantics-preserving optimization.

```
FreshEvaluation(E)
    ==
CacheHit(E)
```

For equivalent model states:

```
FullRecompute(M)
    ==
IncrementalRecompute(M)
```

Incremental evaluation uses explicit dependency/change closure. The .NET evaluation history records the resulting operation/result commitments and is replaced by a new immutable snapshot. Kernel evaluation does not mutate that history.

Invalidation cannot be inferred from incidental execution order.

Cache identity must include every semantic input that can change the authoritative result, including relevant:

- feature/specification inputs;
- resolved references;
- frame/configuration;
- upstream result identities;
- mathematical contract version;
- tolerance policy;
- provider/backend contract identity;
- manufacturing context.

---

# 14. Mandatory Early Vertical Slice

The first system-level CAD proof must establish the semantic architecture before broad feature expansion.

Required scenario:

```
Create Cube
   ↓
Select a Face semantically
   ↓
Create Sketch on that Face
   ↓
Create Circle A
Create Circle B
   ↓
Apply/solve supported constraints
   ↓
Validate sketch/profile
   ↓
Feature A along +FaceNormal
   ↓
Feature B along -FaceNormal
   ↓
Recompute
   ↓
Authoritative B-Rep/result
   ↓
Topology + reference provenance
   ↓
Derived representation
   ↓
Viewer semantic selection
```

The acceptance proof must establish:

1. semantic intent is preserved;
2. face references resolve by semantic evidence rather than topology-array position;
3. sketch coordinates use the referenced face frame;
4. evaluation dependencies are deterministic;
5. the mathematical kernel is reached through an explicit contract;
6. the returned result is integrated as authoritative system state;
7. topology/reference provenance is retained;
8. representation is derived from authoritative state;
9. viewer selection resolves back to semantic identity.

### Exact-geometry rule

A circular profile must not be polygonized to make this scenario pass.

If exact circular-profile solid construction or the required cube/cylinder Boolean topology is not yet certified by the mathematical layer, the system remains explicitly unsupported at that boundary until a new mathematical contract is established.

---

# 15. Current Main vs Target Architecture

At the audited main baseline:

- the Rust mathematical/native foundation is established;
- the existing .NET Framework provides semantic/build/compiled-model infrastructure and compatibility boundaries;
- the existing kernel client exposes the current typed transport boundary;
- the viewer and legacy TypeScript API remain compatibility/presentation surfaces;
- the full canonical System-CAD evaluation architecture described above is **target architecture**, not a claim that all corresponding projects already exist.

The architecture document therefore distinguishes:

```
CURRENT
  Verified files and behavior on main

TARGET
  Required System-CAD architecture to be implemented through
  explicit contracts, evaluation paths and evidence
```

No class, interface or package name in this target architecture is evidence of implementation until it exists and passes the repository's acceptance gates.

---

# 16. Architectural Gaps That Must Not Be Hidden

The following are architectural boundaries, not reasons to bypass the architecture:

- exact curved-face / circular-profile solid construction;
- unrestricted topology-aware Boolean construction;
- broader topology evolution/correspondence;
- complete drawing semantics and graphics mapping;
- complete Product/Assembly/BOM contracts beyond the current Framework surface;
- full Knowledge/Configuration domain;
- complete Sheet Metal domain;
- complete CAM domain and deterministic postprocessor chain;
- phenomena simulation service/provider implementations;
- lifecycle/PLM integration;
- production viewer semantic selection against authoritative CAD results.

Each gap requires its own semantic contract and validation evidence.

---

# 17. Architectural Rules for Future Development

1. Build the System-CAD meaning first; call mathematical authority through contracts.
2. Keep specialized domains independent peers around stable contracts.
3. Keep reusable Science and Engineering Resource facts outside private domain state.
4. Keep Product Structure authoritative and BOM derived.
5. Keep phenomena simulation provider-neutral and consumable as a service.
6. Keep CAM responsible for toolpaths and deterministic postprocessed G-code/NC.
7. Keep Drawing/PMI tied to authoritative CAD references.
8. Keep templates/components/sheets/canvases semantic and provenance-aware.
9. Keep topology and geometry authority separate from representation.
10. Keep cache, identity, invalidation and incremental evaluation explicit.
11. Never introduce approximate geometry as authoritative output.
12. Never turn a legacy/compatibility path into the canonical System-CAD architecture.
13. Do not add a new mathematical contract until reuse of existing authority has been audited.
14. Do not claim architecture completion merely because interfaces or placeholder projects exist.

---

# 18. Architectural Completion Meaning

A System-CAD architecture increment is complete only when the affected path has:

```
Semantic ownership
+ explicit contract
+ dependency direction
+ reference/frame semantics
+ deterministic identity
+ authoritative result
+ provenance/topology behavior
+ representation boundary
+ supporting implementation
+ adversarial validation
+ repository evidence
```

The architecture is successful when an engineering capability can be traced deterministically from:

```
engineering intent
 → semantic contract
 → dependency/evaluation plan
 → mathematical contract where required
 → authoritative result
 → topology/provenance
 → derived representation
 → downstream domain consumption
```

That trace is the architectural backbone of UMLCAD.V.7.
