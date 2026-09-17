# UMLCAD V5 — Complete V4-Grounded Kernel API Execution Plan v2

## Status

This document supersedes the earlier `KERNEL_API_EXECUTION_PLAN.md` as the current execution authority for the Kernel API work. The earlier document is retained unchanged for history.

The plan was revised after a deeper audit of UMLCAD V4 `src/index.ts` and the underlying kernel modules.

## 1. Non-negotiable architecture

V5 does **not** create a second CAD engine.

V4 remains the single engineering authority for geometry, constraints, relations, numerical analysis, solving, topology, validation, spatial/engineering rules, references, dimensions, source-module rules, and DXF release behavior.

V5 adds:

- a transport-neutral Kernel Application API;
- explicit versioned DTO/schema contracts;
- a stateless REST adapter;
- cache management;
- part-level incremental builds;
- a separate assembly aggregation step;
- orchestration around existing V4 authorities.

The Kernel core must not depend on HTTP, WebSocket, Git, browser UI, authentication, authorization, editor UI, or LLM services.

---

# 2. V4 capability inventory — nothing may be silently omitted

The audited V4 `src/index.ts` exports these functional areas:

```text
authoring-api
 authoring-project
 authoring-settings
 authoring-config-json
 cad-component
 engineering-rules
 engineering-rules-json
 cad-program-json
 cad-validation
 validation-pipeline
 constraint-analysis
 constraint-normalization
 constraint-scaling
 degeneracy
 dimensions
 solver-objective
 solver-conditioning
 constraint-solver
 advanced-constraints
 constraint-relations
 geometric-variable-analysis
 relation-jacobian
 basic-shapes
 dependency-graph
 design-program
 design-program-snapshot-json
 geometry-query
 layers
 public-values
 references
 reference-migration
 source-module-analysis
 source-module-runtime
 spatial
 spatial-rules
 topology
 dxf-export
 dxf-semantic-adapter
 dxf-manufacturing-adapter
 dxf-integration
```

Each item must be classified during extraction as exactly one of:

```text
EXTERNAL SEMANTIC CAPABILITY
INTERNAL ENGINEERING AUTHORITY
SERIALIZATION/PUBLIC-VALUE HELPER
```

An exported symbol that is not classified fails the extraction phase.

The API does not have to expose every low-level helper as a separate URL, but every **externally meaningful capability** must have an explicit API representation or an explicit documented reason for remaining internal.

---

# 3. V4 engineering model that the API must preserve

## 3.1 Project and authoring

`CadProject` is source-oriented project composition. It registers drawings and sheets by stable IDs and source references, provides deterministic project snapshots, and builds registered entries through `CadProgram`.

`Drawing`/`Sheet` owns:

- exactly one canonical `DesignProgram`;
- resolved authoring settings;
- layer registry/default layer;
- drawing CAD items;
- mounted `CadComponent`s;
- final-build lifecycle;
- design snapshot after build.

Layer assignment, authoring settings, and mounted components are therefore part of build semantics, not merely viewer metadata.

## 3.2 Source-module analysis/runtime

V4 has a controlled source module pipeline:

- static TypeScript AST import/export extraction;
- no execution during static analysis;
- only explicit relative module resolution;
- explicit module manifest;
- deterministic dependency-first evaluation order;
- duplicate module ID rejection;
- import-cycle rejection;
- path normalization preventing root escape;
- dynamic import and `require()` rejected by the controlled static contract.

This subsystem must be represented in the V5 build input/analysis contract.

## 3.3 Geometry

V4 canonical geometry is:

```text
Point2D
line segment
circle
circular arc
```

Semantics include:

- finite coordinates;
- non-degenerate lines;
- positive finite circle/arc radii;
- exact circular arc semantics;
- counter-clockwise arc representation;
- explicit unwrapped start/end angles;
- arc span strictly greater than zero and strictly less than one revolution;
- stable geometry IDs;
- degeneracy validation.

The API must never make sampled SVG/display geometry the engineering representation.

## 3.4 Geometry queries

V4 `GeometryQuery` supports:

```text
start
end
boundingBox
parameterDomain
pointAt
 tangentAt
closestPoint
parameterAt
intersections
distanceTo
```

The implemented intersection paths include the line/circle/arc combinations present in V4.

These are semantic kernel operations and must be reachable through the API.

## 3.5 References

V4 stable references include:

```text
EntityRef
EdgeRef
VertexRef(entityId + start/end)
StableGeometryRef
```

`GeometryReferenceResolver` rejects stale/unknown identities and type-invalid references.

V4 reference migration explicitly supports:

```text
preserved
split
merged
invalidated
```

A missing migration must never silently mean "preserved".

## 3.6 DesignProgram

`DesignProgram` is the canonical parametric design authority. It owns:

- parameters;
- geometry definitions;
- parameter dependencies;
- parametric constraints;
- geometric relations;
- dependency graph;
- deterministic evaluation;
- snapshots;
- provenance;
- transactional regeneration.

Current parametric constraint kinds:

```text
horizontal
vertical
coincident
fixed
distance
```

Current geometric relation kinds:

```text
parallel
perpendicular
equal-length
angle
collinear
concentric
equal-radius
radius
diameter
tangent
midpoint
point-on-line
point-on-circle
distance-points
symmetric
```

Circle center is also a valid semantic relation point reference.

## 3.7 Advanced constraints

V4 has a distinct advanced-constraint evaluator for:

```text
parallel
perpendicular
equal-length
angle(parameter-driven)
```

It exposes normalized residuals and degeneracy status.

This must not be collapsed into a vague "constraint" boolean in the API.

## 3.8 Constraint analysis

V4 analysis exposes:

- residual components;
- per-constraint residuals;
- norms;
- maximum residual;
- geometric variables;
- variable materialization;
- contradiction diagnostics;
- Jacobian;
- numerical rank;
- degrees of freedom;
- redundant component count;
- numerical conditioning;
- satisfaction;
- classification.

Classifications:

```text
underconstrained
fully-constrained
overconstrained
unsatisfied
```

## 3.9 Numerical solver stack

V4 separately contains authorities for:

- constraint normalization;
- solver scaling;
- solver objective;
- solver conditioning;
- constraint solver;
- geometric-variable analysis;
- relation Jacobians.

The solver itself is the V4 numerical authority. It must not be duplicated in V5.

Solver results/termination/diagnostic structures must be serialized as DTOs where externally meaningful.

## 3.10 Dependency graph

V4 dependency nodes are:

```text
parameter
geometry
constraint
relation
```

The graph:

- rejects duplicate nodes;
- rejects cycles;
- records dependencies;
- provides deterministic snapshots;
- provides deterministic downstream traversal.

It becomes an **internal optimization mechanism** for V5, not the initial primary cache unit.

## 3.11 Dimensions

V4 dimensions support:

```text
distance
horizontal-distance
vertical-distance
radius
diameter
angle
```

with:

- stable IDs;
- geometry references;
- length/angular units;
- nominal target;
- asymmetric plus/minus tolerance;
- reference-only mode;
- evaluated value;
- satisfied status;
- deviation.

These semantics must be preserved because DXF semantic projection already depends on them.

## 3.12 Layers

V4 layer semantics include:

```text
id
name
classification
visible
locked
plottable
item membership
default layer
```

The API must expose resolved layer semantics as part of authoring/build results.

## 3.13 CAD components

V4 allows reusable `CadComponent`s to mount into the canonical design.

The API must preserve mounted-component identity/provenance and must not reduce a component to its display geometry.

## 3.14 Spatial and engineering rules

V4 spatial rules are:

```text
minimum-clearance
forbid-interaction
```

V4 engineering rule profiles include:

```text
profile ID/version
model length unit
spatial tolerance
spatial rules
acceptance severity policy
```

Rule definition and acceptance policy remain separate concepts.

## 3.15 Topology

V4 derives semantic topology from canonical geometry/relations and exposes explicit semantic references for:

```text
vertices
edges
wires
faces
```

The browser/API must not reconstruct topology from SVG/GLB/DXF.

## 3.16 Validation

V4 validation covers:

```text
geometry
references
constraints
relations
constraint analysis
topology
spatial rules
engineering rules
```

The canonical diagnostic phase order is frozen as:

```text
geometry
constraints
topology
references
spatial
validation
```

Secondary ordering is deterministic. The API must preserve structured diagnostics and this ordering.

## 3.17 DXF

V4 already has a complete DXF release path:

```text
canonical DXF export
semantic dimension projection
manufacturing readiness validation
integrated release orchestration
```

The V5 DXF API must delegate to this V4 authority.

The audited V4 kernel does **not** currently establish STEP, GLB/glTF, or PDF as kernel exporters through `src/index.ts`; these are future V5 projection services, not existing V4 capabilities.

## 3.18 JSON/public values

V4 contains explicit snapshot/public-value JSON helpers. They must be used or wrapped during extraction so V5 does not blindly serialize JavaScript class graphs or closures.

---

# 4. Correct incremental-build strategy

## 4.1 Primary unit: complete part

The first V5 incremental system is intentionally simple:

```text
part source + configuration
        ↓
complete V4 kernel build
        ↓
complete validated PartBuildResult
        ↓
immutable cache
```

The cache unit is one complete part build, not one parameter, geometry node, or constraint.

## 4.2 Complete PartBuildResult

The immutable part result should contain, as applicable:

```text
part identity
source/build identity
source-module identity
resolved authoring settings
layer semantics
mounted components
parameters
canonical geometry
constraints
relations
provenance
stable references
reference migrations
dependency graph
dimensions
constraint analysis
solver result/diagnostics
topology
spatial diagnostics
engineering diagnostics
validation result
DXF/release artifacts when requested
semantic manifest
artifact identities
```

## 4.3 Part cache identity

The cache key must cover every semantic input capable of changing the part result:

```text
project/part identity
source revision/content identity
source module graph/content identity
authoring settings identity
engineering rule profile identity
kernel version
API/schema version
build profile
projection/export options when they affect the cached artifact
```

No cache hit is valid if relevant input identity differs.

## 4.4 Incremental rebuild

```text
source/configuration change
          ↓
identify affected parts
          ↓
rebuild affected parts completely
          ↓
reuse immutable cached results for unaffected parts
          ↓
assembly build
```

A full rebuild of a changed part is not a failure of incrementality. It is the intended first implementation.

## 4.5 Assembly is a separate end step

After required parts are built, V5 performs a separate assembly build.

```text
Part A build ─┐
Part B build ─┼──→ Assembly build ─→ Unified assembly result
Part C build ─┘
```

The assembly stage consumes immutable `partBuildId`s and explicit instance transforms/assembly semantics.

Assembly may perform assembly-level:

- placement;
- instance identity;
- cross-part spatial/interference checks;
- assembly relationships where the V5 assembly model explicitly supports them;
- aggregate semantic projections.

A part result is not rewritten by assembly.

Assembly has its own cache key derived from the exact ordered set of referenced part build identities plus assembly source/settings.

## 4.6 Fine-grained incrementality is optional later

Only after complete part caching and assembly are correct may V5 optimize inside a part with the V4 dependency graph.

Potential later path:

```text
part cache
  ↓
source-module cache
  ↓
affected dependency closure
  ↓
partial geometry evaluation
  ↓
partial constraint/relation analysis
  ↓
partial topology/validation
```

Any such optimization must prove semantic equivalence to a cold complete part build. Otherwise it falls back to complete evaluation.

---

# 5. Kernel Application API

The transport-neutral application API is the only boundary the kernel implementation needs to know.

Conceptual families:

```text
project/inspect
source/analyze
source/dependencies
build/part
build/assembly
query
analyze/constraints
analyze/dimensions
analyze/spatial
analyze/solver
validate
regenerate
solve
operations/preview
operations/evaluate
references/resolve
references/migrate
export/dxf
export/dxf/validate
```

This list describes capability families, not an instruction that every family must be a separate physical microservice.

---

# 6. REST API surface

## Build

```http
POST /v1/build/part
POST /v1/build/assembly
```

`/v1/build` may be a compatibility alias but must carry an explicit build kind.

## Source

```http
POST /v1/source/analyze
POST /v1/source/dependencies
```

## Query

```http
POST /v1/query
```

Query kinds must cover the full semantic result surface, including:

```text
project
drawing/sheet/entry
part
assembly
source metadata
authoring settings
layers
components
parameters
geometry
geometry-query
constraints
relations
advanced constraints
dimensions
references
reference migrations
dependencies
provenance
constraint analysis
solver result
topology
spatial relations/rules
diagnostics
validation
public values
artifacts/manifests
capabilities/schema
```

## Constraint analysis/solver

```http
POST /v1/analyze/constraints
POST /v1/analyze/solver
POST /v1/solve
```

## Dimensions/spatial

```http
POST /v1/analyze/dimensions
POST /v1/analyze/spatial
```

## Validation

```http
POST /v1/validate
```

## Regeneration

```http
POST /v1/regenerate
```

The initial contract maps to the currently implemented V4 parameter-change transaction. It is not an arbitrary source mutation endpoint.

## Semantic operations

```http
POST /v1/operations/preview
POST /v1/operations/evaluate
```

These are V5 orchestration-level operations and must delegate to explicit V4 authorities. No `mutate-anything` command is permitted.

## References

```http
POST /v1/references/resolve
POST /v1/references/migrate
```

## DXF

```http
POST /v1/export/dxf
POST /v1/export/dxf/validate
```

These delegate to V4 DXF export/integration/manufacturing authorities.

---

# 7. REST request identity

Every computational request must be reproducible without hidden session state.

Part build identity must include at least:

```text
projectId
partId
sourceRevision/contentIdentity
source module identity
resolved authoring settings identity
engineering rule profile identity
kernel version
schema version
build profile
```

Assembly build additionally includes:

```text
assemblyId
assembly source/configuration identity
ordered part instances:
  partId
  partBuildId
  instanceId
  transform
  assembly semantics
```

Operations that require retry/idempotency carry an explicit operation ID.

Git is an input identity source, not a kernel semantic dependency.

---

# 8. DTO/schema rules

No REST response may serialize V4 classes directly.

Minimum explicit schema families:

```text
ProjectDescriptor
DrawingDescriptor
SheetDescriptor
PartBuildRequest
PartBuildResult
AssemblyBuildRequest
AssemblyBuildResult
BuildIdentity
SourceModule
SourceDependency
AuthoringSettings
LayerSpec
CadComponentDescriptor
Point2D
Geometry
GeometryItem
GeometryQueryRequest
GeometryQueryResult
EntityRef
EdgeRef
VertexRef
TopologyVertexRef
TopologyEdgeRef
WireRef
FaceRef
ReferenceMigration
Parameter
ParametricConstraint
GeometricRelation
AdvancedConstraint
ConstraintResidual
ConstraintAnalysis
SolverRequest
SolverResult
DimensionSpec
EvaluatedDimension
DependencyNode
DependencyEdge
DesignProgramSnapshot
Provenance
SpatialRule
SpatialRuleDiagnostic
EngineeringRuleProfile
CadDiagnostic
CadValidationResult
TopologyModel
DxfExportRequest
DxfExportResult
DxfSemanticProjection
DxfManufacturingValidationResult
SemanticManifest
ArtifactIdentity
```

Requirements:

- no closures/functions;
- no mutable class identity;
- stable semantic IDs;
- deterministic collection ordering;
- explicit schema version;
- explicit numerical/tolerance semantics.

---

# 9. Application build pipeline

For one part:

```text
source modules
    ↓
static AST analysis
    ↓
controlled import resolution
    ↓
deterministic module order
    ↓
CadProject composition
    ↓
Drawing/Sheet + authoring settings/layers/components
    ↓
DesignProgram
    ↓
constraints + relations
    ↓
analysis/solver as required
    ↓
dimensions
    ↓
topology
    ↓
spatial/engineering rules
    ↓
ordered CAD validation
    ↓
immutable PartBuildResult
```

The pipeline is orchestration. The actual engineering algorithms remain V4 algorithms.

Assembly then consumes the immutable part results.

---

# 10. Exact execution phases

## KAPI-01 — Freeze the V4 capability inventory

Deliver:

1. a machine-readable inventory of V4 exported symbols;
2. explicit classification of every export;
3. complete capability-to-DTO/API mapping;
4. no silent omissions;
5. a CI completeness test.

## KAPI-02 — Define extraction boundary

Package/extract V4 kernel for V5 while preserving exact semantics.

Do not import UI/server concerns into the kernel.

## KAPI-03 — Build the transport-neutral Kernel Application API

Define immutable request/response types and adapters around V4 authority.

## KAPI-04 — Implement complete DTO schemas

Add validation, serialization, deterministic ordering, versioning and invalid-input tests.

## KAPI-05 — Implement REST adapter

REST handles decoding, validation, protocol errors, idempotency, status mapping and serialization only.

## KAPI-06 — Implement `/v1/build/part`

Build one complete validated part from explicit source/build identity.

## KAPI-07 — Implement complete query surface

Expose geometry queries, dimensions, references, topology, analysis, layers, authoring settings, components, diagnostics, provenance and artifacts.

## KAPI-08 — Expose analysis, normalization/scaling/solver and regeneration

Preserve V4 numerical semantics and candidate/accepted transactions.

## KAPI-09 — Expose validation and DXF release

Preserve V4 diagnostic ordering and DXF manufacturing gates.

## KAPI-10 — Implement immutable part cache

Cache complete `PartBuildResult` objects by complete build identity.

## KAPI-11 — Implement affected-part discovery

At first, operate at whole-part granularity. Rebuild complete affected parts; reuse complete unaffected parts.

## KAPI-12 — Implement assembly build

Consume exact part build identities and perform the final unified assembly step.

## KAPI-13 — Implement assembly-level validation and cache

Keep cross-part consequences separate from part-level authority.

## KAPI-14 — Optional internal incremental evaluation

Only after KAPI-01 through KAPI-13 are green, use the V4 dependency graph for intra-part optimization.

---

# 11. Error semantics

The API must distinguish:

```text
protocol/transport error
schema/input error
source/module error
build/evaluation error
engineering rejection
validation failure
solver termination
reference invalidation
export/release rejection
internal defect
```

Engineering rejection is not an HTTP 500 by default.

Structured diagnostics are preserved.

---

# 12. Cache correctness

Required invariants:

1. identical complete build input → identical semantic result;
2. cold build ≡ warm cache result;
3. invalid/corrupt cache → safe recomputation;
4. changed part → changed part rebuilt;
5. unchanged part → cached result reused;
6. one changed part invalidates the assembly but not unrelated part caches;
7. assembly references exact part build IDs;
8. changing kernel/settings/rules/source identity invalidates affected cache entries;
9. cache eviction changes performance only;
10. no mutable cross-request kernel state determines semantics.

---

# 13. V4 parity testing

For every externally mapped capability:

```text
V4 direct call
     ≡
V5 Kernel Application API
     ≡
V5 REST DTO
```

within explicitly declared numerical/serialization tolerance.

Golden fixtures must cover:

- geometry;
- parameter regeneration;
- all parametric constraints;
- all geometric relations;
- advanced constraints;
- analysis/Jacobian/rank/DOF/conditioning;
- solver termination and accepted/rejected paths;
- reference resolution/migration;
- dimensions;
- layers/settings;
- spatial rules;
- topology;
- validation diagnostic order;
- DXF semantic/manufacturing pipeline;
- source module dependency resolution.

---

# 14. Adversarial testing

At minimum:

```text
stale reference
invalid reference type
reference split/merge/invalidation
zero-length line
zero/negative/non-finite radius
invalid arc span
constraint contradiction
constraint redundancy
under/over-constrained state
solver singularity/ill-conditioning
cycle in dependency graph
cycle in source modules
unresolved source import
invalid module path traversal
invalid layer
locked layer
invalid dimension unit/tolerance
spatial clearance violation
forbidden interaction
topology failure
invalid engineering profile
DXF manufacturing gate failure
cache corruption
cache identity mismatch
assembly part-build mismatch
```

---

# 15. Resource/security limits

Kernel API limits must cover:

- source bytes;
- module count/depth;
- geometry count;
- constraint/relation count;
- matrix dimensions;
- solver iterations;
- query payload/result size;
- assembly part count;
- export size/time.

The controlled V4 source-module contract must not be weakened by the service boundary.

Authentication/authorization is outside kernel mathematics and routing decisions.

---

# 16. Explicit non-goals

The Kernel API does not own:

```text
Git operations
Git merges
branch management
browser sessions
WebSocket session state
authentication
authorization
LLM prompting
LLM orchestration
source editor UX
chat UX
native client UI
viewer rendering state
application command routing
```

Those are surrounding platform/runtime responsibilities.

---

# 17. GREEN definition

The Kernel API is GREEN only when:

### Preservation

- V4 is still the sole engineering authority.
- No duplicate CAD geometry/solver/topology/validation engine exists.
- V4 DXF remains the DXF authority.

### Completeness

- every V4 public capability is classified;
- every externally meaningful capability is represented by the API;
- no unexplained omission exists.

### Part-level incrementality

- complete parts build independently;
- complete results are immutable/cacheable;
- changed parts rebuild;
- unchanged parts reuse exact results;
- cache identity is deterministic.

### Assembly

- assembly is a separate final step;
- assembly consumes exact immutable part builds;
- assembly cache invalidation is correct;
- cross-part diagnostics remain separate from part authority.

### Statelessness

- every request carries sufficient immutable identity;
- caches are performance state only;
- no hidden current-model state exists.

### Semantic fidelity

- stable references survive the boundary;
- dimensions/layers/components/provenance/topology are preserved;
- numerical diagnostics are preserved;
- validation ordering is deterministic;
- source-module constraints are preserved;
- DXF semantics/manufacturing gating are preserved.

### Verification

- V4 parity tests green;
- cold/warm cache equivalence green;
- changed-part/unaffected-part tests green;
- assembly tests green;
- adversarial tests green;
- schema/version tests green.

---

# 18. Final architecture

```text
                      SOURCE / GIT / EDITOR / LLM
                                  │
                                  ▼
                         source revision/modules
                                  │
                                  ▼
                       ┌─────────────────────┐
                       │   Kernel REST API   │
                       │       adapter       │
                       └──────────┬──────────┘
                                  │
                         Kernel Application API
                                  │
                    ┌─────────────┴─────────────┐
                    │                           │
                 PART BUILD                  QUERY/ANALYSIS
                    │                           │
                    ▼                           │
              Existing V4 kernel ◄─────────────┘
                    │
                    ▼
             PartBuildResult
                    │
                    ▼
              IMMUTABLE CACHE
                    │
          ┌─────────┴──────────┐
          │                    │
     unchanged parts      changed parts
          │                    │
          └─────────┬──────────┘
                    ▼
             ASSEMBLY BUILD
                    │
                    ▼
          Unified Assembly Result
```

The central optimization is therefore deliberately simple and safe:

> **Build complete parts, cache complete parts, rebuild only affected parts, and make assembly the separate final aggregation step.**

The V4 dependency graph remains available for later intra-part optimization, but V5 does not need to make every internal kernel calculation incremental before the architecture is valid.