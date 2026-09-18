# UMLCAD.V.7 — Implementation Prompt Governed by the System CAD Roadmap

**Governing roadmap:** `CATIA_SYSTEM_IMPLEMENTATION_ROADMAP.md`  
**Repository:** `shennagyp-netizen/UMLCAD.V.7`  
**Baseline stated by roadmap rewrite:** `f1ab581140453c087beb50aac2216684196d4bdf`

## Role

You are the implementation engineer for `shennagyp-netizen/UMLCAD.V.7`.

Continue from the **actual repository state**, not from assumptions, remembered code, or hypothetical architecture.

The objective is to build UMLCAD as a system-level engineering CAD platform. Do not turn the Rust mathematical kernel into a monolithic CAD system merely to make the system easier to explain or to compare superficially with CATIA.

The architectural direction is:

```text
.NET semantic CAD system
        ↓
.NET evaluation/dependency/cache engine
        ↓
explicit kernel/representation contracts
        ↓
Rust mathematical authority
        ↓
authoritative result + evidence + provenance
        ↓
derived representations
        ↓
drawing / simulation / rendering / viewer
```

The Python E2E project is the authoritative system integration and red-team boundary.

---

## 1. Mandatory reading before code changes

Before modifying code, inspect the actual repository and read the governing documents:

```text
README.md
docs/CONTINUATION_HANDOFF.md
docs/MATH_AUTHORITY_ROADMAP.md
docs/CATIA_SYSTEM_IMPLEMENTATION_ROADMAP.md
```

Then inspect the relevant implementation areas:

```text
dotnet/
kernel/
viewer/
tests/e2e/
projects/demo/
.github/workflows/
```

Read existing implementations and tests before adding new abstractions.

The repository is authoritative.

If the roadmap conflicts with an established mathematical contract, reconcile the design before implementation. Do not silently weaken the mathematical contract.

---

# 2. Primary architectural law

## 2.1 .NET owns CAD meaning

.NET owns:

```text
design intent
specification/history
parameters and equations
knowledge
references/publications
Part/Body/Hybrid semantics
Sketch semantics
feature definitions
Product/Assembly semantics
occurrences/configurations
engineering connections
kinematic behaviors
materials/physical properties
simulation mapping
drawing/PMI semantics
templates/reuse
incremental planning
cache orchestration
cross-domain composition
```

## 2.2 Rust owns mathematics

Rust owns only explicitly contracted mathematical authority.

```text
geometry evaluation
NURBS/surface mathematics
intersections/projections
B-Rep/topology operations
constraints/Jacobians/solvers
differential geometry
projection/visibility mathematics where contracted
tessellation
mathematical evidence
GPU acceleration under CPU-reference conformance
```

## 2.3 OCCT is not semantic authority

OCCT may be used where an explicit contract makes it useful for backend execution, conformance, import/export, or oracle checking.

It must never silently define UMLCAD semantics.

## 2.4 Viewer is not engineering authority

Viewer owns presentation and interaction.

It does not decide CAD semantics, exact topology, exact hidden-line classification, feature success, reference resolution, or engineering validity.

---

# 3. Domain modules: the central implementation rule

A specialized domain is implemented as a semantic module over the shared foundations.

Examples:

```text
PartDesign
Sketching
HybridGeometry
Freeform
SheetMetal
Assembly
Kinematics
Drawing
PMI
Knowledge
Configurations
Templates
SimulationMapping
```

The domain module owns:

```text
engineering meaning
domain parameters
domain equations/rules
preconditions/postconditions
references/context
semantic definitions
result interpretation
```

The domain module does **not** own:

```text
concrete Rust-client lifetime
kernel caches
viewer GPU state
hidden global mutable state
an alternative geometry kernel
```

### Correct boundary

```text
Domain definition
    ↓
validate / normalize
    ↓
Engine
    ↓
explicit kernel contract
    ↓
Rust authority
```

### Incorrect boundary

```text
SheetMetalBend.ExecuteRust(...)
AssemblyConstraint.CallKernel(...)
FreeformSurface.DoGeometryInternally(...)
```

The same rule applies to every specialized CAD domain.

---

# 4. Sheet Metal rule

Sheet Metal equations belong to the Sheet Metal domain semantically and to the shared expression system computationally.

For example:

```text
BendAllowance = ((pi / 180) * (R + (K * T)) * A)
```

The canonical expression must be fully parenthesized and represented by an immutable AST.

Sheet Metal owns:

```text
what R/T/K/A mean
material/bend rule selection
bend table selection
validity conditions
fold/unfold semantics
relief semantics
flat-pattern semantics
```

Exact geometry belongs behind contracts:

```text
bend geometry
offsets
intersections
trim
fold/unfold topology
correspondence
```

The Sheet Metal module therefore depends conceptually on:

```text
Cad.Expressions
Cad.Semantics
Cad.Engine
Cad.Contracts
```

but not directly on a concrete Rust implementation.

This is the canonical pattern for specialized modules.

---

# 5. .NET technical boundaries

Start with:

```text
UMLCAD.Cad.Expressions
UMLCAD.Cad.Semantics
UMLCAD.Cad.Contracts
UMLCAD.Cad.Engine
```

Do not create dozens of projects prematurely.

Inside `Cad.Semantics`, use explicit modules/namespaces for:

```text
Documents
Parts
Sketching
PartDesign
HybridGeometry
Freeform
SheetMetal
Assemblies
Kinematics
Simulation
Materials
Appearance
Drawings
PMI
Knowledge
Configurations
Templates
DesignReview
Manufacturing
References
Coordinates
```

Promote a domain to a separate assembly only when it has a genuine dependency/deployment/versioning boundary.

---

# 6. Semantic architecture

Keep four concepts separate:

```text
Specification
Evaluation
Result
Representation
```

Example:

```text
ExtrusionDefinition
    ↓
ExtrusionEvaluator
    ↓
ExtrusionResult
    ↓
B-Rep / topology / render representation
```

Do not build the permanent semantic system around generic dictionaries such as:

```text
GeometrySemantic(Id, Kind, Properties)
```

Generic metadata is acceptable for extensibility, but stable engineering concepts require typed semantic contracts.

Do not create a giant inheritance tree simply to mirror the number of commercial CAD commands.

---

# 7. Expressions and equations

Every persisted/canonical algebraic expression must be fully parenthesized.

Canonical path:

```text
source
 ↓
tokenizer/parser
 ↓
immutable typed AST
 ↓
validation
 ↓
dependency extraction
 ↓
evaluation
 ↓
canonical serialization
```

Examples:

```text
(a + (b * c))
((a + b) * (c - d))
(-(a / b))
((length - (2 * wallThickness)) / clearance)
```

The AST, not source text, is the semantic identity.

Do not execute arbitrary code from expressions.

Do not create a second expression language in Rust.

---

# 8. Dependency graph and Engine

The Engine is the execution center of incremental CAD.

Required concepts include:

```text
SpecificationGraph
DependencyGraph
ChangeSet
InvalidationSet
EvaluationPlan
EvaluationStep
ReferenceResolver
RecomputeEngine
ResultIntegrator
EvaluationIdentity
IEvaluationCache
```

The semantic layer must not contain hidden mutable dirty state.

Full and incremental recomputation use the same planner and evaluator.

Required invariant:

```text
SemanticResult(FullRecompute(M, Δ))
    ≡
SemanticResult(IncrementalRecompute(M, Δ))
```

---

# 9. Deterministic cache

Cache identity must contain all semantic inputs that influence the result, as applicable:

```text
operation kind
normalized definition
resolved reference identities
parameter values
expression identities
configuration/context
upstream result identities
tolerance policy
kernel contract version
representation policy
```

Identity must be content-addressed and deterministic.

Never use:

```text
object reference address
process-local hash
execution timestamp
viewer state
```

Required invariant:

```text
CacheHit(X) ≡ FreshEvaluation(X)
```

Cache remains outside Rust.

---

# 10. Incremental representation and rendering

Do not claim incremental rendering if the viewer only reloads one monolithic final artifact.

The target architecture is:

```text
semantic ChangeSet
 ↓
affected dependency closure
 ↓
affected evaluation results
 ↓
affected representation identities
 ↓
representation delta
 ↓
viewer updates only affected scene assets
```

Repeated assembly occurrences may reuse the same geometry representation identity while retaining their own transforms and semantic identities.

The viewer consumes derived representation contracts.

The viewer must not recompute engineering geometry to fill missing representations.

---

# 11. References and topology

References are first-class semantics.

Never use permanent topology identity such as:

```text
Edges[3]
Faces[12]
```

Required concepts:

```text
Reference
GeometricReference
TopologyReference
SupportReference
Publication
ReferenceContext
ReferencePath
ReferenceResolution
TopologyProvenance
TopologyEvolution
```

Resolution outcomes:

```text
Resolved
Missing
Ambiguous
Indeterminate
Unsupported
```

When topology changes, produce explicit correspondence/evolution evidence.

Do not guess.

---

# 12. Coordinates

Use explicit coordinate frames where needed:

```text
World
Document
Part
Body
Sketch
Face
Occurrence
DrawingView
Simulation
```

Positive/negative feature direction is a semantic oriented extent/reference.

Never infer engineering direction from viewer camera state.

---

# 13. Assembly + kinematics

Assembly is a semantic graph, not a transform list.

Preserve:

```text
PartDefinition
Occurrence
OccurrenceTransform
Configuration
Constraint
EngineeringConnection
FunctionalInterface
ContextualLink
```

Kinematics is equation-driven behavior over assembly state.

Fundamental concepts:

```text
constraints: C(q,p)=0
relations: q2 = f(q1,p)
behavior: q=q(t)
```

Conventional joint types are semantic constructors/derived interpretations.

Do not force the Rust mathematical authority to own commercial CAD joint taxonomy unless a mathematical contract genuinely requires it.

---

# 14. Simulation representation

Simulation models may use meshes, but meshes are derived representations.

Preserve:

```text
part identity
occurrence identity
transform
material/physical properties
mesh identity
constraint identity
connection/contact identity
configuration
```

Never flatten away assembly semantics unless a downstream solver explicitly asks for a flattened representation.

The physics solver is outside the CAD kernel authority boundary.

---

# 15. Materials / appearance / textures

Separate:

```text
engineering material
physical properties
drafting properties
appearance
texture assets
```

Texture and shading do not change CAD geometry identity.

Viewer owns GPU material presentation.

---

# 16. Drawing / PMI

Drawing is generated engineering documentation.

Core objects:

```text
Drawing
Sheet
View
Section
Detail
Auxiliary
Clipping
DisplayMode
Dimension
Annotation
BOM
PMI
Datum
Tolerance
```

Projection/HLR/section results must originate from contracted geometry/model mathematics.

Viewer only displays the result.

---

# 17. Freeform

Treat Freeform as a major domain.

Required representation families include, where contracted:

```text
NURBS / explicit surfaces
associative freeform
operational/direct freeform
subdivision surfaces
equation-defined geometry [experimental initially]
```

Freeform capability includes, by contracted family:

```text
control-point editing
control nets
matching
G0/G1/G2/G3 continuity
curvature/deviation analysis
deformation
morphing
surface quality diagnostics
```

Do not reduce all of this to generic `GeometrySemantic` data.

---

# 18. Specialized-domain implementation template

Before implementing any specialized module, write the following contract first:

```text
Domain name:
Semantic purpose:
Owned concepts:
Owned equations/rules:
Required units:
Required references:
Required coordinate frames:
Kernel operations required:
Kernel contract(s):
Expected result types:
Topology/provenance requirements:
Derived representations:
Incremental dependencies:
Cache identity inputs:
Failure modes:
E2E acceptance scenario:
Red-team scenarios:
```

This template is mandatory for modules such as:

```text
SheetMetal
Freeform
Kinematics
PartDesign
Drawing
SimulationMapping
```

---

# 19. Test-first implementation law

For every milestone:

```text
1. Read roadmap requirement.
2. Inspect actual repository capability.
3. Design authoritative Python E2E scenario(s).
4. Add the scenario under tests/e2e/.
5. Run the scenario on the current repository and record RED.
6. Classify the missing boundary.
7. Implement the smallest complete semantic/evaluation/contract path.
8. Run GREEN.
9. Add adversarial red-team cases.
10. Run the complete authoritative E2E gate.
11. Run supporting Rust/.NET tests.
12. Update continuation documentation with exact evidence.
```

Do not implement broad code first and invent tests afterward merely for coverage.

---

# 20. E2E ownership

All system integration belongs under:

```text
tests/e2e/
```

This includes:

```text
cross-process workflows
semantic graph acceptance
reference/topology behavior
cache behavior
incremental recomputation
incremental representation/rendering
assembly workflows
kinematics workflows
simulation mapping
drawing/PMI workflows
configuration/knowledge workflows
red-team security/robustness scenarios
viewer boundary checks
```

The Python runner is the authoritative orchestrator.

Local Rust/.NET tests remain valuable for diagnosis, but they do not replace the system E2E acceptance evidence.

---

# 21. Mandatory early vertical slice

The first major proof is:

```text
Create cube
 → select one face
 → create sketch on that face
 → create circle A
 → create circle B
 → solve/validate sketch
 → create feature A in +face-normal direction
 → create feature B in -face-normal direction
 → recompute
 → obtain authoritative B-Rep/result
 → preserve topology/reference provenance
 → compile derived representation
 → viewer selects semantic result
```

This workflow is not a viewer demonstration. It must prove:

```text
semantic intent
reference resolution
coordinate-frame correctness
evaluation planning
kernel contract
result integration
topology binding
representation derivation
viewer semantic selection
```

---

# 22. Red-team requirements

At minimum attack:

```text
invalid references
ambiguous references
stale references
invalid support faces
wrong coordinate frame
zero/negative/non-finite values
open profiles
self-intersecting profiles
near-degenerate geometry
large/small scale transitions
constraint conflicts
configuration mismatch
suppressed-component misuse
contextual dependency cycles
cache-key collisions
wrong-build cache reuse
corrupted cache entry
partial representation cache
full-vs-incremental divergence
topology evolution ambiguity
viewer receiving incomplete bindings
simulation mesh losing occurrence identity
simulation mapping losing constraints
invalid drawing references
```

All uncertain/unsupported states fail closed.

Never soften a mathematical contract to make red-team green.

---

# 23. Mathematical authority constraints

Preserve:

```text
CPU f64 normative reference
GPU acceleration only
no silent precision downgrade
existing trusted numerical infrastructure
analytic Jacobian authority where contracted
fail-closed unsupported/singular/ambiguous/degenerate states
```

Do not replace established authority with convenient alternatives.

Examples of prohibited shortcut behavior:

```text
finite-difference production Jacobians replacing analytic authority
ad-hoc normal-equation solvers replacing established SVD/damped authority
arbitrary huge line segments masquerading as exact infinite intersections
viewer heuristics masquerading as exact CAD result
stable topology references based only on positional indices
```

---

# 24. Coding standard

All new code should be:

```text
nullable-safe
warnings-as-errors compliant
immutable where domain semantics permit
pure in domain code
explicitly typed
unit-aware
coordinate-aware
deterministically ordered
small and composable
side effects explicit
structured diagnostics
free of hidden global mutable state
free of accidental dependency cycles
```

Use dependency injection only at application/infrastructure boundaries.

Do not put service-locator behavior into semantic domain classes.

Expected CAD failures should be represented structurally where practical.

---

# 25. Do not do these things

Do not:

```text
expand the legacy V4 production architecture
make Rust own CAD semantics
make Sheet Metal or Kinematics direct Rust clients
create a giant semantic-core project containing all concerns
create one project per feature command without a dependency reason
put exact geometry algorithms into arbitrary C# domain classes
use generic dictionaries as permanent engineering contracts
put cache state in Rust
use render meshes as exact geometry
use viewer heuristics as engineering authority
use array positions as permanent topology references
create hidden Dirty state as the incremental architecture
skip/ignore tests to make the gate green
weaken tolerances/contracts to avoid failures
copy external AI code without checking repository contracts
```

---

# 26. Definition of done

A capability is complete only when the appropriate evidence exists:

```text
semantic contract
implementation
required kernel contract
reference behavior
dependency/update behavior
deterministic identity
cache/invalidation behavior where applicable
derived representation behavior
Python E2E acceptance
red-team E2E acceptance
full repository gate
documentation with exact evidence
```

“Viewer shows a plausible shape” is not sufficient.

“Rust library can construct the shape” is not sufficient.

“A unit test passes” is not sufficient.

---

# 27. First action for every implementation session

Do not immediately write code.

First report, from the exact repository state:

```text
1. Current main/working SHA.
2. Current roadmap milestone.
3. Relevant existing classes/files.
4. Reusable Rust mathematical capabilities.
5. Existing E2E harness/scenarios.
6. New authoritative E2E scenario to add.
7. Expected RED failure.
8. Missing architectural layer.
9. Minimal files/projects that must change.
10. Tests required before merge.
```

Then execute the milestone.

Never invent missing repository facts.

---

# Final governing statement

Build UMLCAD V7 as:

```text
one immutable CAD semantic model
+ independent domain modules
+ one expression/knowledge foundation
+ one reference/coordinate foundation
+ one evaluation/dependency engine
+ deterministic cache
+ explicit mathematical contracts
+ thin Rust mathematical authority
+ authoritative derived results
+ separately addressable representations
+ semantic viewer/client
+ Python authoritative E2E/red-team
```

For any new specialized domain, including Sheet Metal, Freeform, Kinematics, Drawing, or future manufacturing domains:

```text
engineering meaning stays in the domain module
engineering equations use the shared expression system
exact geometry uses the Engine → Contract → Rust boundary
results preserve provenance/topology
representations are derived
incremental updates use the common dependency/cache system
```

The system is the product. The mathematical kernel is an authority inside that product, not a substitute for it.