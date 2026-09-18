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

## Architecture authority

Before implementing any capability, treat the following as normative:

`
docs/architecture/ABSTRACTION_AND_DEPENDENCY_MODEL.md
docs/architecture/architecture.json
docs/architecture/c4/
`

The C4 model describes system/runtime boundaries. The abstraction model describes dependency and ownership direction. The machine-readable manifest is the executable architectural description consumed by the E2E architecture guard.

A .NET project is not automatically a C4 Container.

The project tree is an implementation projection of the logical architecture.

The key dependency law is:

`
Foundation → Mathematics → Science → CAD Core
                                      ↓
                           Engineering Resources
                                      ↓
                           Engineering Domains
`

with outward integration:

`
UMLCAD-owned service/contract → adapter/provider → external technology
`

Lower layers provide facts/services. Upper layers interpret them as engineering meaning.

Peer domains may exchange stable published results/contracts, but private implementation dependencies are forbidden by default.

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
material assignment/references; Science owns material physical properties
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

A domain module owns engineering meaning, rules, parameterization, semantic composition, and result interpretation.

It consumes lower-level facts and services:

`
Domain
  → Science services
  → CAD Core services/results
  → Engineering Resource services where applicable
  → shared expression/quantity services
  → Phenomena Simulation Service where applicable
`

It does not own the concrete kernel.

Exact geometry follows:

`
Domain Definition
      ↓
Domain Evaluator
      ↓
Evaluation Engine
      ↓
explicit mathematical contract
      ↓
Rust mathematical authority
`

### Result flow versus dependency

A domain may consume a stable published result from another domain:

`
Sheet Metal → published FlatPatternResult → CAM
`

This does not authorize:

`
CAM → private SheetMetal implementation/state
`

Data/result flow and compile-time implementation dependency are separate architectural concepts.

# 4. Sheet Metal rule

Sheet Metal is a bounded engineering discipline and should be implemented as a separate library when its public/dependency boundary is introduced.

Preferred boundary:

`
UMLCAD.Engineering.SheetMetal
`

It owns:

`
SheetMetalPart
Thickness
BendDefinition
BendRadius
BendAngle
KFactor
BendAllowance
BendDeduction
BendTable
ReliefDefinition
FlangeDefinition
CornerDefinition
FoldDefinition
UnfoldDefinition
FlatPatternDefinition
`

It uses shared Science material facts, Engineering Resource machine/tool capabilities, the expression system, CAD Core references/results, and the Phenomena Simulation Service when applicable.

Example semantic equation:

`
BendAllowance = ((pi / 180) * (R + (K * T)) * A)
`

The expression subsystem owns AST, units, dimensions, dependency extraction, canonicalization, and evaluation mechanics.

Sheet Metal owns the engineering meaning of K, T, R, A, bend tables, validity rules, material/process compatibility, and machine/tool constraints.

Exact bend/fold/unfold geometry is requested through the common Engine → Math Contract → Rust boundary.

The application must reject incompatible material/process/machine combinations from their declared properties/capabilities. Do not implement material compatibility as a collection of special-case names.

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

# 14. Phenomena Simulation Service and representation

Phenomena simulation is a reusable scientific capability, not a peer application that CAM depends upon.

The architecture is:

`
Phenomena Simulation Service
        ↓
provider contract
        ↓
multiple implementations/providers
        ↓
internal solver or external scientific application
`

The service models physical phenomena, subject to explicit scientific contracts.

Examples include, where contracted:

`
deformation
stress/strain response
thermal response
vibration
cutting-process response
`

CAM may consume the service:

`
CAM → Phenomena Simulation Service
`

Sheet Metal and other engineering domains may also consume it.

The provider is an implementation choice. CAM must not depend on a concrete solver application such as ANSYS or Abaqus.

Simulation meshes and other visualization/solver representations remain derived from authoritative engineering/scientific inputs. They do not replace the authoritative CAD result or scientific model.

# 15. Materials and physical properties

Material identity and scientifically defined material properties belong to the Science layer.

A shared material model may include:

`
composition/classification
density
elastic properties
plastic properties
thermal properties
electrical properties
hardness
other scientifically defined properties
`

Engineering domains interpret those facts.

For example:

`
Science:
    "This material has these properties."

Sheet Metal:
    "Given those properties, this bending process is valid/invalid."

CAM:
    "Given those properties, this manufacturing process is feasible
     under these machine/tool conditions."
`

The system must not duplicate material truth independently in Sheet Metal, CAM, or simulation implementations.

Material compatibility is therefore a rule over declared properties/capabilities, not an ad-hoc name check.
# 16. Drawing / PMI

Drawing is an upper engineering domain.

For a dimension:

`
Drawing
   ↓
Reference Resolution
   ↓
Authoritative CAD Result
   ↓
Science Measurement / Quantity / Unit services
   ↓
Drawing applies drafting rules
   ↓
dimension representation
`

Authority is separated:

`
CAD      → authoritative geometry/topology
Science  → measurement/unit/scientific calculation
Drawing  → drafting meaning and rules
Viewer   → presentation
`

Therefore Science does not depend on Drawing. Drawing consumes scientific services.
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

## CAM / G-code rule

CAM is a manufacturing engineering domain, not just a toolpath exporter.

Required production flow:

\`\`\`
CAD Result
  ↓
Manufacturing interpretation
  ↓
Material + Machine + Tool + Fixture capabilities
  ↓
Process planning
  ↓
optional Phenomena Simulation Service
  ↓
Toolpath
  ↓
Machine-specific postprocessor
  ↓
deterministic G-code / NC
\`\`\`

G-code/NC is an explicit manufacturing result. A postprocessor is an outward adapter/provider translating a UMLCAD-owned manufacturing-output contract into a machine/controller dialect.

The NC generator must never emit plausible output for an invalid/unsupported/ambiguous machine, tool, unit, or operation state. It must fail closed with structured diagnostics.


## CATIA-level drawing capability mapping

Drawing implementation must explicitly cover the CATIA drafting capability families relevant to UMLCAD.

At minimum map:

\`\`\`
projection/front/side/top/isometric views
sections
aligned/offset sections
detail views
circular/profiled details
clipping
auxiliary/unfolded/broken views where contracted
view axis/orientation/display mode
associative generation and update state
dimensions: linear/angular/radius/diameter/coordinate/baseline/chain
tolerances/limits/precision/units
texts/notes/leaders/balloons
GD&T
datums and datum targets
surface roughness
welding symbols
centerlines/axes/symmetry/thread lines
hatching/area-fill
2D dress-up geometry
sheet/border/title block/revision zones
BOM table from CAD Product Structure
assembly filtering in generated views
drawing standards such as ISO/ANSI/JIS
DXF/DWG boundary
2D structure editing/reuse
Knowledgeware/formula association and drafting validation
\`\`\`

These are capability targets, not permission to reproduce CATIA's internal object model.

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

`
one immutable CAD semantic/product model
+ shared mathematical/expression services
+ shared scientific/material/phenomena services
+ shared engineering resource model
+ independent engineering domains
+ one dependency/evaluation engine
+ deterministic caches
+ explicit mathematical contracts
+ thin Rust mathematical authority
+ authoritative engineering results
+ derived consumer representations
+ outward adapters/providers
+ semantic viewer/client
+ Python authoritative E2E/red-team
`

For any new domain:

`
1. identify the semantic owner;
2. identify reusable lower-level services;
3. identify the published result/contract;
4. identify consumers;
5. prohibit private peer implementation coupling;
6. identify external adapters/providers;
7. identify the authoritative layer for disagreements.
`

The product is the engineering system. The mathematical kernel is one authority inside it.



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