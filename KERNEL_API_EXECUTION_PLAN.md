# UMLCAD V5 — Kernel API Execution Plan and Architecture Evaluation

## 0. Purpose

This document defines the execution plan for exposing the existing UMLCAD V4 CAD kernel through a stable, RESTful, stateless Kernel API in UMLCAD V5.

The first architectural rule is non-negotiable:

> **V5 does not create a second CAD kernel. The V4 kernel is the engineering foundation. V5 extracts and exposes its existing capabilities through a versioned API boundary, then extends the existing kernel only where the new runtime requires capabilities that V4 does not yet provide.**

The following V4 implementation is therefore the starting authority:

```text
CadProject
    ↓
Drawing / Sheet
    ↓
DesignProgram
    ├── parameters
    ├── geometry definitions
    ├── constraints
    ├── geometric relations
    ├── dependency graph
    └── snapshots / regeneration
            ↓
constraint analysis / solver
            ↓
topology / references / spatial rules
            ↓
CAD validation
```

The V4 code already contains these authorities and must be reused rather than replicated.

---

# 1. Architecture evaluation

## 1.1 Evaluation of the current V4 kernel

The V4 kernel is architecturally suitable for service extraction because its important engineering concepts already have explicit types and functions rather than being hidden in a UI layer.

Verified V4 boundaries include:

- `CadProject` registers source-owned Drawing/Sheet entries and builds them through a `CadProgram`.
- `Drawing` owns the single canonical `DesignProgram` for the authored design.
- `DesignProgram` owns parameters, geometry definitions, constraints, relations, dependency graph, evaluation, snapshots, and transactional regeneration.
- `DependencyGraph` provides stable semantic dependency nodes and downstream traversal.
- Constraint analysis exposes residuals, Jacobian, rank, DOF, redundancy, contradictions, and numerical conditioning.
- The solver exposes candidate/accepted state, iteration data, termination, convergence, residuals, damping, and scaling policy.
- `validateCadState` provides structured geometry, reference, constraint, topology, spatial, and engineering-rule validation.
- Topology has explicit vertex, edge, wire, and face semantics.
- Geometry has explicit primitive semantics and degeneracy rejection.

These are appropriate Kernel API capabilities.

## 1.2 The important V4 limitation

The V4 kernel is **not yet an incremental evaluator in the strong sense required for a high-performance interactive V5 runtime**.

The dependency graph already exists, but the current `DesignProgram.evaluateParameters()` materializes all registered geometry definitions in deterministic order and evaluates the complete constraint/relation set. `regenerate()` is transactional, but currently evaluates a candidate complete snapshot rather than evaluating only the affected dependency closure.

Therefore V5 must not falsely advertise incremental evaluation as an already-complete V4 capability.

The execution plan separates:

```text
Phase A:
Expose existing V4 behavior correctly.

Phase B:
Add incremental evaluation to the existing V4 authority.
```

No second geometry engine, solver, topology engine, or validation engine may be introduced for Phase B.

## 1.3 Evaluation of REST/stateless architecture

A RESTful stateless API is appropriate for the kernel because it creates a stable boundary between the CAD computation engine and the surrounding routing/editor/Git/browser system.

The API contract is stateless:

```text
request + immutable input/revision
        ↓
deterministic kernel computation
        ↓
response
```

The implementation may maintain caches, compiled modules, dependency graphs, evaluated subgraphs, solver preparation data, and artifact caches. These are performance optimizations, not API state.

A cache loss must change performance only, never semantics.

## 1.4 Why Git must remain outside the kernel

Git identifies and stores source history. It is not the CAD semantic model.

The kernel must not parse Git commits, understand branches, perform merges, or derive CAD validity from textual diffs.

The surrounding runtime may translate Git/source changes into a source revision or kernel build request.

The boundary is:

```text
Git / workspace
      ↓ source revision or source input
Kernel API
      ↓ CAD computation
CAD result
```

A source patch may be an input to a build pipeline, but Git is never the Kernel API.

## 1.5 Why HTTP/REST must remain outside the kernel implementation

The mathematical/kernel implementation should expose a programmatic application interface. HTTP is an adapter over that interface.

Therefore:

```text
kernel-core
    ↓
Kernel Application API
    ↓
REST adapter
```

The kernel core must not import HTTP libraries, routing libraries, WebSocket libraries, Git libraries, browser APIs, or LLM APIs.

This preserves the ability to run the exact same kernel in tests, CLI tooling, a colocated server, a future worker pool, or another process without changing CAD semantics.

---

# 2. V4-to-V5 capability mapping

## 2.1 Project composition

### Existing authority

V4 `CadProject` owns source registration and builds a registered Drawing or Sheet through `CadProgram`.

### V5 exposure

Provide project/entry discovery and build operations without exposing internal class instances.

Conceptual API:

```text
POST /v1/build
POST /v1/query
```

A build request identifies the project source revision and authored entry.

## 2.2 Design program

### Existing authority

V4 `DesignProgram` owns:

- parameter definitions;
- geometry definitions;
- parameter dependencies;
- constraints;
- geometric relations;
- dependency graph;
- evaluation;
- snapshots;
- transactional regeneration.

### V5 exposure

These become immutable serialized model/result representations. The REST API must never expose mutable internal JavaScript objects.

## 2.3 Dependency graph

### Existing authority

V4 `DependencyGraph` has semantic nodes for parameters, geometry, constraints, and relations, rejects dependency cycles, exposes downstream traversal, and produces deterministic snapshots.

### V5 role

It becomes the basis for incremental evaluation and affected-set reporting.

Initial API access is read-only:

```text
POST /v1/query
query.kind = dependencies
```

Later incremental evaluation may consume the same graph internally.

## 2.4 Solver

### Existing authority

V4 `solveConstraints()` already implements damped least squares with finite-difference/analytic Jacobian paths, solver scaling, damping control, candidate states, acceptance, convergence and explicit termination.

### V5 exposure

Expose solver results as DTOs without exposing internal solver classes.

Conceptual endpoint:

```text
POST /v1/solve
```

The endpoint accepts a serialized candidate/evaluation identity and solver options within approved limits.

## 2.5 Constraint analysis

### Existing authority

V4 `analyzeConstraints()` exposes residuals, Jacobian, rank, degrees of freedom, redundancy, contradictions, numerical conditioning and classification.

### V5 exposure

```text
POST /v1/analyze/constraints
```

The response must retain enough structure for browser diagnostics and engineering tooling while remaining independent of TypeScript class names.

## 2.6 Validation

### Existing authority

V4 `validateCadState()` performs geometry validation, reference validation, constraint analysis, topology building, spatial-rule validation and structured diagnostics.

### V5 exposure

```text
POST /v1/validate
```

Validation remains an engineering authority. API serialization must not collapse structured diagnostics into strings.

## 2.7 Topology

### Existing authority

V4 exposes explicit topology vertices, edges, wires and faces derived from canonical geometry and relations.

### V5 exposure

Topology is returned as semantic data in snapshots, validation results, query responses and manifests as appropriate.

Topology is not reconstructed from rendered geometry at the API boundary.

## 2.8 Geometry

### Existing authority

V4 geometry currently includes line, circle and exact circular arc semantics with explicit degeneracy rules and query support.

### V5 exposure

Geometry is serialized as semantic primitives rather than SVG paths or sampled display points.

---

# 3. Kernel API contract

## 3.1 Design rule

The Kernel API is **capability-oriented, not CRUD-oriented**.

Do not design the public API as:

```text
PUT /geometry/{id}
PUT /constraint/{id}
DELETE /object/{id}
```

Those endpoints incorrectly model a CAD kernel as a database.

The public API models computation:

```text
build
query
analyze
validate
regenerate
solve
operation preview
operation evaluation
export/projection
```

## 3.2 Initial endpoint surface

### Build

```text
POST /v1/build
```

Build/evaluate an authored project entry from an explicitly identified source revision.

### Query

```text
POST /v1/query
```

Queries a supplied model/build identity.

Initial query kinds:

```text
snapshot
object
property
geometry
constraints
dependencies
provenance
topology
diagnostics
capabilities
```

The exact query schema is versioned rather than multiplying endpoint paths for every object type.

### Constraint analysis

```text
POST /v1/analyze/constraints
```

Maps to the existing V4 constraint-analysis authority.

### Validation

```text
POST /v1/validate
```

Maps to the existing V4 CAD validation authority.

### Regeneration

```text
POST /v1/regenerate
```

Provides a source/program parameter-change candidate using the existing transactional `DesignProgram.regenerate()` behavior.

### Solve

```text
POST /v1/solve
```

Runs the existing constraint solver against an explicitly identified candidate state.

### Operation preview

```text
POST /v1/operations/preview
```

A V5 orchestration-level semantic operation is translated into the existing V4 kernel operations. The preview returns affected targets, diagnostics, and candidate-state information. It does not mutate accepted state.

### Operation evaluation

```text
POST /v1/operations/evaluate
```

Evaluates a resulting source/candidate against a declared base revision and returns the validated result. The endpoint does not itself know whether a human or LLM produced the source.

### Export

```text
POST /v1/export/{format}
```

Initial formats:

```text
dxf
step

gltf/pdf where implemented and supported
```

Export consumes an identified validated model/build result and never becomes an alternative engineering authority.

---

# 4. Request identity and statelessness

Every request that can affect or evaluate CAD must carry enough identity to be reproducible.

Minimum conceptual identity:

```text
projectId
sourceRevision or sourceContentIdentity
entryId where applicable
model/build identity where applicable
operationId for mutation/evaluation requests
```

The API must not depend on an implicit per-session `current model`.

Example:

```json
{
  "projectId": "car-x",
  "sourceRevision": "git:abc123",
  "entry": {
    "kind": "drawing",
    "id": "vehicle"
  }
}
```

For a semantic operation:

```json
{
  "projectId": "car-x",
  "baseModel": "model:842",
  "operationId": "op:902",
  "operation": {
    "kind": "property.change",
    "target": "wheel-fl",
    "property": "radius",
    "value": 340
  }
}
```

Operation IDs must support idempotent retry at the service boundary.

---

# 5. Response architecture

Responses must separate the following concepts:

```text
status of request
engineering validity
model identity
changed semantic objects
full diagnostics
artifacts/manifests
```

A successful HTTP response must not imply engineering validity automatically.

For example:

```json
{
  "request": {
    "operationId": "op:902"
  },
  "result": {
    "accepted": true,
    "modelRevision": "model:843"
  },
  "validation": {
    "valid": true,
    "diagnostics": []
  }
}
```

A syntactically successful request can therefore still return a valid CAD response with warnings, or a rejected engineering candidate with structured diagnostics, while HTTP status communicates protocol-level success/failure according to the API contract.

---

# 6. Semantic serialization

The REST API must not serialize internal object graphs blindly.

Create explicit DTO/schema types for:

```text
Point2D
Geometry
GeometryItem
Parameter
ParameterValue
Constraint
Relation
DependencyNode
DependencyEdge
DesignSnapshot
ConstraintResidual
ConstraintAnalysis
ConstraintSolveResult
TopologyModel
CadDiagnostic
CadValidationResult
BuildIdentity
ArtifactIdentity
SemanticManifest
```

All collections must have deterministic ordering.

Object identities must be stable semantic identifiers, never array positions or generated serialization order.

No serialized object may contain functions/closures such as V4 geometry evaluator functions.

Source provenance must be represented declaratively.

---

# 7. Build and cache architecture

## 7.1 Immutable input identity

The build layer must derive a content/revision identity from authoritative source and configuration inputs.

Conceptually:

```text
source content
+ authoring configuration
+ framework version
+ kernel version
+ relevant build profile
        ↓
BuildInputIdentity
```

## 7.2 Caches

The first cache layers should be separated:

```text
source/module compilation cache
CAD dependency graph cache
evaluated model cache
constraint/solver preparation cache
semantic manifest cache
artifact cache
```

Cache keys must be explicit and reproducible.

A cache entry must never be used when its input identity is incompatible.

## 7.3 Cache failure behavior

A cache miss means recompute.

A corrupted cache entry means invalidate and recompute.

Cache state must never override source truth or kernel semantics.

---

# 8. Incremental evaluation execution plan

## KAPI-I1 — Baseline extraction

### Goal
Expose complete V4 behavior without incremental evaluation claims.

Tasks:

1. Inventory the V4 public kernel classes/functions used by `CadProject`, `Drawing`, `DesignProgram`, analysis, solver, topology, validation and geometry query.
2. Define transport-neutral application interfaces around those capabilities.
3. Create V5 DTOs and schemas.
4. Implement an in-process adapter to the V4 kernel authority.
5. Implement REST serialization above the adapter.
6. Add golden request/response fixtures.

Acceptance:

- Every exposed result can be traced to an existing V4 authority.
- No duplicated geometry/constraint/solver logic exists in the API adapter.
- No kernel type leaks through REST.

## KAPI-I2 — Deterministic build identity

Tasks:

1. Define source revision identity.
2. Define project entry identity.
3. Define build input identity.
4. Define model/build/artifact identities.
5. Add deterministic hashing and schema versioning.
6. Add repeated-build equality tests.

Acceptance:

Equivalent input produces equivalent semantic output and deterministic identity.

## KAPI-I3 — Query surface

Implement:

- snapshot;
- object lookup;
- properties;
- geometry;
- constraint list;
- relations;
- dependency graph;
- provenance;
- topology;
- diagnostics.

Acceptance:

The browser can understand an evaluated model without reconstructing CAD semantics itself.

## KAPI-I4 — Analysis and validation

Expose existing V4 constraint analysis and CAD validation exactly enough that no browser-side numerical validity logic is required.

Acceptance:

Golden diagnostics and numerical-analysis fixtures match V4 semantics.

## KAPI-I5 — Regeneration and solve

Map existing V4 transactional regeneration and constraint solving to explicit requests/responses.

Acceptance:

- accepted candidate produces an immutable result identity;
- rejected regeneration leaves prior accepted state unchanged;
- solver termination is deterministic within declared numerical tolerance;
- solver options are bounded and validated.

## KAPI-I6 — Operation preview/evaluation

Introduce a semantic operation DTO above the V4 kernel.

Supported initial operation kinds should correspond to existing V4 capabilities, not hypothetical future CAD operations.

Examples:

```text
parameter.change
constraint.add where V4 authority supports it
constraint.remove where source-level support exists
```

Do not create a generic `mutate-anything` endpoint.

## KAPI-I7 — Incremental dependency closure

Extend the existing V4 dependency graph so a changed parameter/source definition can produce a deterministic affected closure.

Required output:

```text
changed root
→ affected geometry
→ affected constraints
→ affected relations
→ affected topology/validation dependencies
```

Acceptance:

Unrelated nodes are provably excluded from the affected closure.

## KAPI-I8 — Incremental geometry/materialization

Refactor `DesignProgram` evaluation so unaffected geometry definitions may reuse valid evaluated values.

Rules:

- identical dependency inputs may reuse their evaluated result;
- changed definitions are reevaluated;
- downstream dependents are reevaluated;
- deterministic ordering is preserved;
- a dependency ambiguity causes safe broader reevaluation rather than an incorrect partial result.

Acceptance:

Full evaluation and incremental evaluation produce equivalent semantic snapshots.

## KAPI-I9 — Incremental constraint evaluation

Constraint analysis and solver preparation should consume only the affected constraint closure where mathematically valid.

The solver remains the existing V4 solver authority.

Do not create a second incremental solver.

Where a local change invalidates the assumptions required for local solving, expand to the necessary broader set or fall back to full solve.

Acceptance:

Incremental and full solve produce equivalent accepted results within declared tolerances.

## KAPI-I10 — Incremental topology and validation

Determine the topology and validation closure affected by changed geometry.

Unchanged topology may be reused only when its dependency identity proves validity.

Acceptance:

Incremental topology/validation cannot silently retain stale references.

## KAPI-I11 — Delta response

Once incremental evaluation is correct, add semantic delta responses:

```text
added objects
removed objects
changed objects
changed properties
changed topology
changed diagnostics
```

The browser can then update only affected representations.

## KAPI-I12 — Performance characterization

Benchmark:

- single parameter change;
- geometry change with one dependent constraint;
- large dependency fan-out;
- top-level/global change;
- full rebuild;
- cache hit;
- cache miss.

The goal is not to force every operation to be incremental. The goal is to make local changes substantially cheaper while guaranteeing semantic equivalence.

---

# 9. API error model

Protocol errors and engineering errors must be distinct.

## Protocol/request errors

Examples:

```text
invalid JSON
unsupported schema version
missing source revision
unknown endpoint
invalid operation envelope
```

## CAD engineering errors

Examples already represented by V4 diagnostics include:

```text
geometry-invalid
geometry-duplicate-id
constraint-unsatisfied
constraint-contradiction
constraint-underconstrained
constraint-redundant
constraint-analysis-failed
topology-invalid
reference-invalid
spatial-clearance-violation
spatial-interference
validation-internal-error
no-geometry
```

The API must preserve these structured diagnostics rather than flattening them into generic HTTP error strings.

---

# 10. Security and resource limits

The Kernel API is a computation boundary.

Every request must have bounded resource policy, including where appropriate:

- maximum source payload;
- maximum model size;
- maximum geometry count;
- maximum constraint count;
- solver iteration limit;
- maximum wall time;
- maximum exported artifact size;
- cancellation/timeout behavior.

Authentication and authorization remain outside the kernel. The surrounding routing/service layer supplies an already-authorized context.

The kernel must nevertheless validate every request structurally and must never trust client-side validation.

---

# 11. API versioning

Use:

```text
/v1/...
```

The API schema version is independent of the internal V4/V5 implementation version.

Internal refactoring must not silently alter a published API schema.

Breaking API changes require a new API major version.

Semantic model/manifest schema versions must also be explicit.

---

# 12. Testing strategy

## 12.1 Contract tests

For every endpoint:

- valid request;
- missing required field;
- invalid type;
- invalid identity;
- unsupported schema version;
- deterministic serialization;
- stable error format.

## 12.2 Golden kernel equivalence

For each representative V4 case:

```text
V4 direct kernel invocation
vs.
V5 REST → adapter → V4 kernel
```

must produce semantically equivalent results.

This is the most important extraction test.

## 12.3 Determinism tests

Repeat identical requests and compare:

- model identity;
- object ordering;
- geometry values within declared tolerance;
- diagnostics;
- constraint analysis;
- solver result;
- topology;
- manifest.

## 12.4 Transaction tests

Verify rejected candidates do not modify accepted state.

## 12.5 Idempotency tests

Repeat the same operation ID and verify that the same result is returned without double application.

## 12.6 Incremental equivalence tests

For every supported incremental path:

```text
incremental(base, change)
        ≡
fullBuild(base + change)
```

within explicitly declared numerical tolerances and identical semantic identity rules.

This equivalence test is mandatory before enabling incremental evaluation for a change class.

---

# 13. Execution order

The implementation sequence is deliberately conservative:

```text
1. V4 kernel inventory and boundary freeze
        ↓
2. Transport-neutral Kernel Application API
        ↓
3. Explicit V5 DTO/schema layer
        ↓
4. Stateless REST adapter
        ↓
5. build/query/analysis/validation endpoints
        ↓
6. regenerate/solve endpoints
        ↓
7. operation preview/evaluation
        ↓
8. deterministic identity and cache infrastructure
        ↓
9. dependency-closure analysis
        ↓
10. incremental geometry evaluation
        ↓
11. incremental constraints/solver preparation
        ↓
12. incremental topology/validation
        ↓
13. semantic delta responses
        ↓
14. exporters/artifact integration
        ↓
15. performance qualification
```

No browser, WebSocket, Git, editor, LLM, or routing implementation is required to establish Kernel correctness. Those components consume the stable API after its contract is proven.

---

# 14. Definition of GREEN

The Kernel API is GREEN only when:

1. Every published endpoint maps to an explicit existing V4 authority or to a documented V5 extension of that authority.
2. No duplicate geometry, constraint, solver, topology, or validation engine exists.
3. REST DTOs are explicit and deterministic.
4. Protocol errors and CAD diagnostics are distinct.
5. Repeated identical requests are semantically deterministic.
6. Idempotent operation handling is tested.
7. Transaction rejection cannot mutate accepted state.
8. V5 REST results are proven equivalent to direct V4 kernel results for the extracted capability set.
9. Incremental behavior is enabled only for change classes with full-build equivalence tests.
10. Cache loss changes performance only, never correctness.
11. Resource limits and cancellation behavior are deterministic.
12. Full kernel/API tests and all applicable V5 checks pass.

---

# 15. Explicit non-goals

The Kernel API implementation must not:

- contain WebSocket logic;
- contain browser UI logic;
- contain Git logic;
- call an LLM;
- implement source-editor behavior;
- implement authentication/authorization;
- implement project collaboration policy;
- depend on a particular container runtime;
- expose internal V4 class instances as the API schema;
- create a second CAD geometry or solver system;
- infer topology from rendered pixels.

These belong to surrounding V5 services or clients.

---

# 16. Final architecture

```text
                         BROWSER
                            │
                         WebSocket
                            │
                            ▼
                    ROUTING / SESSION
                            │
                 ┌──────────┼──────────┐
                 │          │          │
                 ▼          ▼          ▼
             Git Server   Editor/LLM   CAD REST API
                                         │
                                         ▼
                                Kernel Application API
                                         │
                           ┌─────────────┴─────────────┐
                           │                           │
                    V4 Kernel Authority       V5 Incremental Engine
                           │                           │
                    geometry/constraints        dependency closure
                    solver/topology             cache/evaluation
                    validation                   delta computation
                           └─────────────┬─────────────┘
                                         ▼
                                  Semantic CAD Result
                                         │
                              ┌──────────┼──────────┐
                              ▼          ▼          ▼
                           Manifest    DXF/STEP   GLB/PDF
```

The V4 kernel remains the mathematical and engineering foundation. V5's Kernel API is the stable service boundary around that authority. Incremental evaluation is an extension of the same kernel, not a parallel implementation.
