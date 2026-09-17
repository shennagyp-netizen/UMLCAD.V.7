# UMLCAD V7 — Unified Kernel Architecture

## 1. Document authority

This is the single comprehensive architectural and development document for UMLCAD V7.

The previous V4/V5/V6 documentation is historical source material preserved in Git history. V7 does not maintain a parallel collection of active architecture, roadmap, execution-plan, contract, handoff, or AI-instruction documents.

The V7 repository is organized around one UMLCAD kernel with five explicit boundaries:

```text
kernel/
├── math/
├── contracts/
├── native/
├── occt/
└── api/
```

These five directories are architectural boundaries, not five independent kernels.

## 2. Core authority model

UMLCAD owns the meaning of geometry and engineering computation. External geometry libraries are implementations or conformance references.

The authority order is:

```text
mathematical authority
        ↓
semantic contracts
        ↓
native UMLCAD implementation
        ↓
optional OCCT realization / conformance
        ↓
public API boundary
```

OCCT is never the semantic definition of UMLCAD.

A client, renderer, transport, Git system, editor, LLM, cache, or native handle may provide inputs, execution infrastructure, or derived representations, but none of them may silently become engineering authority.

## 3. Five-directory kernel

### 3.1 kernel/math — mathematical authority

`math` contains the mathematics UMLCAD itself owns and must be able to specify independently of any backend.

This boundary includes, as applicable:

- vectors, points, matrices, frames, and transforms;
- numerical tolerances and numerical policy;
- curves and surfaces;
- B-spline and NURBS parameterization;
- homogeneous/projective evaluation;
- rational dehomogenization;
- analytic derivatives and differential geometry;
- parameter-domain mathematics;
- projection, distance, and intersection mathematics;
- geometric predicates;
- topological predicates that are mathematical rather than backend-specific;
- constraint equations and relation equations;
- Jacobians, rank, conditioning, and solver mathematics;
- independent mathematical regression oracles and adversarial fixtures.

The mathematical layer must not depend on OCCT, HTTP, WebSocket, .NET application state, browser state, renderer state, authentication, Git state, or a live project session.

The V6 mathematical-authority rules are retained: analytic definitions are preferred over sampled display geometry; finite-difference approximations must not replace available analytic derivatives; backend differences are classified instead of redefining the mathematics; tolerance widening is never a substitute for proof.

The existing V6 mathematical work on B-splines/NURBS, positive weights, homogeneous evaluation, differentiated control nets, quotient-rule derivatives, tensor-product surfaces, control-net bounds, and independent oracle testing becomes the starting mathematical corpus for V7.

### 3.2 kernel/contracts — semantic contract authority

`contracts` defines what a UMLCAD kernel operation means without prescribing a backend implementation.

It contains the backend-neutral semantic API and contract crates migrated from the V6 workspace, excluding the OCCT implementation itself.

The contract boundary covers, as applicable:

- geometric definitions;
- curve and surface definitions;
- freeform/NURBS semantics;
- intersection semantics;
- sweep/pipe semantics;
- offset, shell, thickness, fillet, and drafting semantics;
- topology and B-Rep semantics;
- mesh/tessellation contracts;
- exchange semantics;
- reference evolution and reference migration;
- operation provenance;
- immutable kernel integration snapshots;
- cancellation and structured operation errors;
- deterministic identity and canonicalization;
- diagnostics and evidence;
- backend-neutral capability discovery.

Contracts are semantic records and values. They must not leak OCCT classes, native pointers, renderer IDs, allocation addresses, or transport-specific session state.

A contract distinguishes at least these classes where relevant:

```text
valid
invalid
unsupported
ambiguous
indeterminate
backend-failure
```

The contract itself decides when a result is semantically accepted. A backend returning a displayable or valid native shape is not sufficient evidence by itself.

### 3.3 kernel/native — independent UMLCAD implementation

`native` contains UMLCAD's backend-independent native implementation and host-side kernel execution that can operate without OCCT semantics being authoritative.

The existing Rust kernel implementation is moved here as one implementation corpus rather than retained as a separately named `kernel_rust` project.

This directory is the principal location for implementation that realizes V7 contracts from UMLCAD-owned mathematics. It includes the existing Rust API/host/service structure and the current implementation modules that were previously under `kernel_rust`.

The long-term objective is that V7 can perform the supported kernel computation through this native path without requiring OCCT to define or validate the semantics.

Native implementation requirements inherited from V6 include:

- immutable authoritative inputs and results;
- stateless evaluation with respect to live project/session state;
- deterministic semantic identity;
- fail-closed invalid and unsupported outcomes;
- explicit tolerance domains;
- structured diagnostics/evidence;
- source immutability for functional operations;
- no backend-derived semantic identity;
- no renderer-derived topology;
- no hidden mutable kernel session.

The current source code is preserved during this structural migration. The directory move does not claim that all existing native behavior has already reached the full V7 mathematical target.

### 3.4 kernel/occt — alternative realization and conformance backend

`occt` is an adapter/backend boundary only.

It contains the existing OCCT implementation and associated native bridge, tests, and build material previously under the V6 `rust/crates/occt-backend` package.

Its responsibilities are:

- realize UMLCAD contracts using OCCT;
- translate UMLCAD values to native OCCT objects;
- keep OCCT lifetimes and exceptions behind the native boundary;
- return backend-specific diagnostics without leaking native types upward;
- provide conformance evidence against UMLCAD semantic expectations;
- expose backend capability limits explicitly.

OCCT must never define:

- UMLCAD mathematical equations;
- semantic object identity;
- semantic tolerance policy;
- topology identity;
- candidate selection;
- acceptance criteria.

A backend failure is distinct from invalid UMLCAD input, mathematical degeneracy, unsupported UMLCAD semantics, or an indeterminate result.

V6 established this distinction repeatedly through NURBS surfaces, intersections, offsets, sweeps, B-Rep topology, sewing, pathology evidence, and reference-evolution work. V7 preserves that separation.

### 3.5 kernel/api — external boundary

`api` is the application-facing/API-facing boundary.

The existing `kernel` application/API tree is moved here unchanged as part of this structural migration. Its current transport and implementation details remain exactly as they are in the source files; their location now expresses the intended public boundary.

The API boundary is responsible for presenting kernel capabilities to external clients. It is not a second engineering engine.

External clients may include:

- .NET applications;
- Python tooling;
- CLI clients;
- browser applications;
- AI agents;
- future native clients.

The transport may be HTTP, WebSocket, or another mechanism outside the mathematical kernel. Transport state, sessions, authentication, authorization, Git collaboration, editor state, and renderer state remain outside the mathematical authority.

## 4. Immutable and stateless kernel model

The kernel follows the V6 invariants retained for V7:

```text
explicit immutable input
        ↓
functional evaluation
        ↓
candidate result
        ↓
validation / analysis / solve
        ↓
new immutable result
```

A rejected operation leaves the accepted input/result unchanged.

The kernel is stateless with respect to project/session/live application state. A request contains the authoritative inputs necessary to reproduce the operation. Internal caches are allowed only as optimizations; cache loss may change performance but never semantics.

Every mutable-looking change is understood as creation of a new immutable semantic state.

## 5. Semantic model

A V7 kernel evaluation is based on a semantic model rather than on render geometry.

The semantic state may contain, as applicable:

```text
parameters
geometry definitions
curves / surfaces / NURBS
typed constraints
geometric relations
dependency edges
provenance
persistent references
topology / B-Rep data
engineering rules
dimensions
spatial relations
validation evidence
```

Source code, SVG, GLB, DXF, STEP, IGES, PDF, screen coordinates, GPU objects, and cached artifacts are derived representations or consumers.

## 6. Mathematical verification discipline

Every mathematically meaningful operation should have an independently defined expectation whenever practical.

Preferred oracle order:

```text
closed-form analytic solution
        ↓
independent polynomial / rational identity
        ↓
geometric invariant
        ↓
independent implementation
        ↓
backend conformance
        ↓
numerical approximation only when necessary
```

A comparison against OCCT alone is not proof of UMLCAD correctness because both implementations may share a convention or defect.

When UMLCAD mathematics and a backend disagree:

```text
reproduce
→ classify the disagreement
→ derive an independent oracle
→ correct the faulty side
→ add regression coverage
```

Do not widen tolerances first and do not copy backend output into the semantic layer merely to obtain agreement.

## 7. NURBS and freeform foundation

V7 begins from the V6 freeform mathematical corpus.

The authoritative semantic representation includes, where applicable:

- control points;
- positive finite weights;
- degree;
- validated nondecreasing full knot vectors;
- explicit parameter domains;
- homogeneous/projective evaluation;
- rational dehomogenization;
- analytic first and second derivatives;
- tensor-product surface semantics;
- explicit UV-domain validation;
- conservative control-net bounds.

Positive-weight NURBS surface bounds may be used for certified broad-phase disjointness, but overlapping bounds are only potential contact and are not an intersection certificate.

The V6 bounded S12/S19 intersection work remains authoritative historical foundation: exact supported families are accepted; unsupported arbitrary freeform intersection tracing remains explicitly unsupported until an independently justified mathematical contract exists.

## 8. Constraints, relations, and solving

Constraints and geometric relations are semantic equations, not UI hints.

For a solver-related contract, define explicitly:

```text
meaning
domain
residual units
residual scale
variable parameterization
Jacobian
singular / degenerate cases
contradiction behavior
conditioning policy
acceptance predicate
```

A solver result alone is not acceptance. Candidate geometry, topology, references, dimensions, spatial rules, and engineering predicates must be checked according to the active contract.

V6's hardening discipline is retained: avoid unproved approximations, distinguish rank from conditioning, keep tolerance domains separate, and do not silently clamp invalid geometry into validity.

## 9. Topology and B-Rep authority

Topology is semantic connectivity, not drawing order.

Persistent references are semantic and backend-neutral. Native OCCT identities, traversal order, array indices, renderer object IDs, and pointer addresses are never semantic identity.

The V6 reference-evolution vocabulary remains explicit:

```text
one → one       preserved
one → zero      invalidated
one → many      split
many → one      merged
many → many     ambiguous
```

Multi-target outcomes are not automatically resolved to one target. Ambiguous topology remains observable evidence.

Future generalized B-Rep work must preserve the same fail-closed model for manifoldness, sewing, degeneration, imported pathology, face removal, orientation propagation, topology history, and reference migration.

## 10. Freeform operation boundary

V7 inherits the V6 rule that no numerical trace, renderer approximation, or native shape visibility may silently become topology.

Certified operation families are defined individually, including the bounded historical slices for:

- line/surface intersections;
- curve/surface isolated roots;
- affine-planar surface/surface intersections;
- positive-weight NURBS broad-phase disjointness;
- bounded bilinear and rational-bilinear patch/plane intersections;
- bounded straight and piecewise-linear circular sweeps;
- bounded variable-radius straight sweeps;
- bounded draft, shell/thickness, and fillet families;
- controlled topology sewing and pathology repair.

Generalized future families require new contracts and independent evidence before being promoted from unsupported to supported.

## 11. Tolerance discipline

V7 keeps separate tolerance concepts where they have different meanings:

```text
UMLCAD mathematical/modeling tolerance
backend/modeling tolerance
backend realizability limit
validation / acceptance tolerance
measurement / comparison tolerance
```

One tolerance must not silently replace another.

Every result materially affected by tolerance should expose enough evidence to identify the applicable tolerance context.

## 12. Failure model

Malformed requests are rejected before engineering evaluation where the contract requires prevalidation.

Structured failures include, as applicable:

```text
InvalidInput
Unsupported
Ambiguous
Indeterminate
SemanticInvariantViolation
BackendFailure
Cancelled
Internal
```

Cancellation is not a geometry failure and must not silently become partial success.

A backend failure is not rewritten as semantic invalidity. Likewise, a semantic failure is not hidden by a backend success.

## 13. Operation provenance and integration state

V6 operation provenance and integration snapshots are retained as the basis for deterministic integration.

An operation records explicit identity, kind, ordered semantic inputs, and output topology identity. Provenance is immutable and value-derived.

Integration snapshots contain immutable operation provenance and ordered active roots. Applying an operation returns a new snapshot; failures never expose partial state.

Snapshot and provenance identities are semantic SHA-256 values derived from canonical UMLCAD data, never from backend handles or runtime state.

## 14. AI contract

AI is a client of the semantic kernel, not a kernel authority.

An AI client must:

1. discover capabilities before requesting unsupported operations;
2. identify the exact immutable revision/build on which the operation is based;
3. address semantic IDs rather than array positions or screen coordinates;
4. propose the smallest explicit semantic operation;
5. consume solver, topology, diagnostics, dimensions, and validation results as evidence;
6. never infer engineering correctness from SVG, screenshots, pixels, GLB, or DXF text;
7. treat rejected operations as non-mutating;
8. treat cache artifacts as non-authoritative;
9. remain outside transport, authorization, Git, and source-editing authority.

Natural-language claims such as “the model looks correct” are not engineering proof.

## 15. Client and viewer boundary

The browser or other client is a semantic consumer and renderer.

A viewer may provide:

- navigation;
- selection;
- property inspection;
- visibility controls;
- camera operations;
- compiled-model visualization.

A viewer must not reconstruct authoritative topology from triangles, infer face identity from render order, or replace the kernel with client-side geometry logic.

A compiled render artifact is a projection of an authoritative build. Manifest and render artifact identity must correspond to the same immutable build.

## 16. Source, Git, security, and session boundaries

Source editing and Git are outside the mathematical kernel.

Authentication and authorization are host/application concerns. For GitHub-backed resources, GitHub remains the identity/repository/file-access authority unless a future explicit architecture says otherwise.

The kernel receives an already-authorized invocation context at its boundary; it does not implement a parallel repository ACL system.

Collaboration/session state is not kernel semantic state.

## 17. Testing philosophy

The V6 TDD gates are retained as V7 engineering discipline.

Every new capability should proceed through the applicable sequence:

```text
semantic contract
        ↓
independent mathematical tests
        ↓
backend-neutral contract tests
        ↓
native implementation
        ↓
native conformance
        ↓
adversarial tests
        ↓
determinism tests
        ↓
cross-layer integration
        ↓
release / performance validation
        ↓
authoritative CI
```

A feature is not complete merely because code compiles, a sample renders, or one backend returns a shape.

Negative tests are first-class tests.

Tolerances are not widened simply to make tests pass.

Performance optimization is subordinate to correctness and determinism.

## 18. V6 station inheritance

V7 does not discard the completed V6 engineering work. It absorbs the station history into the unified kernel model.

### S8
Interchange and visualization boundary hardening established the initial 3D/OCCT boundary, analytic primitives, transforms, Booleans, extrusion, revolution, loft, fillet/chamfer, NURBS, mesh, and exchange surfaces.

### S9
Native tensor-product NURBS surface realization established deterministic OCCT construction behind an opaque backend boundary.

### S10
Independent surface differential mathematics, quotient rules, normals, continuity policy, and backend conformance were established.

### S11
Trimmed NURBS face construction established UV-domain, loop, hole, closure, orientation, simplicity, and native p-curve constraints.

### S12
Representative curve/surface and surface/surface operations established conservative broad-phase behavior, exact supported families, deterministic root ordering, and fail-closed unsupported handling.

### S13
Bounded offset and healing contracts established explicit semantic offset direction, bounded planar realizations, and controlled repair policy.

### S14
Bounded sweeps, variable-radius sweeps, multi-segment circular sweeps, fillet, shell/thickness, and draft families were added under separate semantic contracts.

### S15
Topology orientation, boundary sewing, native conformance, pathology evidence, degeneracy evidence, and controlled explicit face-removal repair were established as separate evidence/repair boundaries.

### S16
Deterministic topology snapshot identities, immutable operation provenance, cancellation/error semantics, immutable integration snapshots, and atomic operation transitions were established.

### S17
Deterministic stress and scale validation was completed for the declared bounded slices. Performance remains subordinate to semantic correctness.

### S18
S18 is the V6 final system-completion gate. It remains the historical final gate for the integrated V6 capability surface. Generalized remaining work remains capability-by-capability under the same semantic-first, contract-first, independent-conformance discipline.

### Historical S19A/S19B/S19C
These are retained as S18.x historical implementation slices rather than new V6 stations:

- bounded bilinear freeform/planar intersection;
- deterministic two/four-root multi-segment bilinear intersection;
- rational bilinear freeform/planar intersection with positive weights.

They do not establish unrestricted NURBS/NURBS intersection tracing.

## 19. V7 development direction

V7 changes the organization of the kernel rather than treating the repository as a collection of historical projects.

The sequence is:

```text
math
  ↓
contracts
  ↓
native implementation
  ↕
OCCT realization/conformance
  ↓
API boundary
```

The immediate V7 architectural objective is to complete and harden the UMLCAD-owned mathematical layer, then progressively move semantic authority and backend-independent implementations into the unified boundaries.

The first major architectural work is mathematical completeness, not UI expansion and not uncontrolled feature proliferation.

## 20. Repository structure mapping

The structural migration is:

```text
old                                      new
────────────────────────────────────────────────────────
kernel_rust/src/functions                kernel/math/functions
kernel_rust/src/api                      kernel/native/src/api
kernel_rust/src/bin                      kernel/native/src/bin
kernel_rust/src/services                 kernel/native/src/services
kernel_rust/src/lib.rs                   kernel/native/src/lib.rs
kernel_rust/Cargo.toml                   kernel/native/Cargo.toml
kernel_rust/Cargo.lock                   kernel/native/Cargo.lock
kernel_rust/tests                         kernel/native/tests

rust/crates/* except occt-backend        kernel/contracts/crates/*
rust/Cargo.toml                           kernel/contracts/Cargo.toml

rust/crates/occt-backend                 kernel/occt

kernel                                   kernel/api
```

No implementation content is intentionally rewritten by this structural migration.

The former V6 documentation corpus and temporary planning files are consolidated into this README. Git history remains the historical record of the removed documents.

## 21. Structural-migration invariant

This repository change is deliberately limited to organization and documentation consolidation.

The following are not changed by this migration:

- source algorithms;
- mathematical formulas already implemented;
- test logic;
- backend behavior;
- public source contents;
- existing project/viewer/application code;
- dependency versions;
- compiler settings;
- API route behavior.

Only file paths, directory ownership, and the active documentation entry point are changed.

Because source contents are intentionally not edited, path references inside existing build/configuration files may temporarily refer to their former locations until a later implementation change explicitly performs that migration. This structural commit must not be interpreted as proof that the moved projects already build from their new paths.

## 22. Non-negotiable V7 rules

1. There is one UMLCAD kernel architecture, not separate competing `kernel_rust` and `rust` authorities.
2. `math` owns UMLCAD mathematical meaning.
3. `contracts` owns backend-neutral semantic meaning.
4. `native` is an independent implementation path.
5. `occt` is an alternative realization/conformance backend.
6. `api` is the external boundary and is not a second solver.
7. Renderer output is never engineering truth.
8. Backend success is never semantic proof by itself.
9. Unsupported and ambiguous cases fail closed.
10. Immutable input produces immutable result; prior accepted state is never mutated.
11. Cache and session state are never semantic authority.
12. Every significant mathematical claim should have an independent verification basis where practical.
13. Every new capability must have a semantic contract before being promoted to supported behavior.
14. Future code changes must keep this document authoritative unless the architecture is deliberately revised.

## 23. Active documentation policy

There is one active documentation file at the repository root: this `README.md`.

Historical V4/V5/V6 documents are no longer part of the active V7 source tree. Their content has been consolidated here at the level required for continuing engineering work, and their individual files remain recoverable through Git history.

Future changes should update this document when they change architecture, mathematical authority, contract boundaries, development gates, or V7 roadmap direction.
