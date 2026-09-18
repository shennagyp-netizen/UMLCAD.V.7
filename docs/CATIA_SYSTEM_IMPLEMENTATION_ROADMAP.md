# UMLCAD V7 — System CAD Implementation Roadmap

**Repository:** `shennagyp-netizen/UMLCAD.V.7`  
**Mathematical baseline:** M0–M16 closed for their declared domains  
**Baseline used for this rewrite:** `f1ab581140453c087beb50aac2216684196d4bdf`  
**Document purpose:** Define the system-level implementation path from the existing V7 mathematical authority to a broad, CATIA-class mechanical/shape/product/drafting system without reproducing CATIA's internal architecture.

> **This is a system architecture and implementation roadmap, not a request to turn the Rust kernel into CATIA.**
>
> The central architectural decision is that UMLCAD is a layered engineering system. Rust remains a thin mathematical authority and execution substrate. .NET owns CAD meaning, design intent, domain semantics, orchestration, lifecycle, references, dependency analysis, deterministic evaluation, caching, and cross-domain composition. Viewer code owns presentation. Python E2E owns system-level acceptance and red-team integration.

---

# 1. Direction of the system

UMLCAD must be evaluated against CATIA at the **system capability level**, not by comparing one kernel to another.

The relevant question is not:

```text
Can the Rust kernel contain everything CATIA's kernel contains?
```

The relevant question is:

```text
Can UMLCAD provide the required engineering capabilities through a
coherent semantic CAD system whose mathematical operations are supplied
by a thin, authoritative mathematical substrate?
```

Therefore:

```text
CATIA functional capability
        ↓
UMLCAD semantic/domain contract
        ↓
.NET immutable design model
        ↓
.NET evaluation/dependency pipeline
        ↓
deterministic kernel contract
        ↓
Rust mathematical authority
        ↓
authoritative result + evidence + provenance
        ↓
derived representations
        ↓
drawing / simulation / rendering / downstream consumers
```

UMLCAD does not reproduce CATIA's historical implementation layers, internal object model, legacy V4/V5 architecture, or command-by-command class hierarchy.

It reproduces the required **engineering capability**, using UMLCAD's own architecture.

---

# 2. Non-negotiable architectural laws

## 2.1 .NET owns CAD meaning

.NET owns:

```text
design intent
specification/history semantics
parameters and equations
knowledge/rules/checks/reactions
references and publications
Part/Body/Hybrid Body semantics
Sketch semantics
feature and operation definitions
Product/Assembly semantics
occurrences and configurations
engineering connections
kinematic behavior definitions
materials and physical-property identity
simulation-model mapping
drawing/PMI semantics
templates and reusable semantic modules
design-review state
manufacturing-facing semantic contracts
incremental evaluation planning
cache policy and identity
persistence/build identity
cross-domain composition
CAD diagnostics and provenance
```

.NET is therefore the system-level semantic authority.

.NET must not become a second numerical geometry kernel.

## 2.2 Rust owns mathematical authority

Rust owns only mathematically contracted capabilities, including as applicable:

```text
curve/surface evaluation
NURBS mathematics
differential geometry
exact/contracted geometric construction
intersection/projection mathematics
B-Rep/topology operations within certified domains
constraint residuals and analytic Jacobians
solver mathematics
projection/visibility mathematics where contracted
tessellation mathematics
mesh-quality/error calculations where contracted
GPU acceleration under CPU-reference conformance
```

The current M0–M16 authority remains binding. A new mathematical capability requires a new explicit contract and validation; it must not be introduced by quietly broadening an old contract.

## 2.3 Domain modules do not own the kernel

A domain module such as Sheet Metal, Kinematics, Freeform, Assembly, Drawing, or Knowledge is **not** a second math engine.

Its responsibility is:

```text
engineering meaning
parameterization
domain equations
preconditions/postconditions
semantic composition
reference requirements
configuration/context
mapping to kernel contracts
result interpretation
```

Its exact geometric work is requested through the common Engine/Contract boundary.

This is the intended relation:

```text
SheetMetal semantic module
        ↓
Expressions + shared CAD semantics
        ↓
Engine
        ↓
SheetMetal kernel contract
        ↓
Rust mathematical operation
```

and **not**:

```text
SheetMetal → direct Rust-client calls scattered through domain classes
```

The same rule applies to Kinematics, Freeform, Part Design, Drawing, Simulation preparation, and future specialized modules.

## 2.4 Domain modules are independent of one another unless composition is real

The presence of an Assembly module does not mean the Part module depends on Assembly.

The presence of Sheet Metal does not mean Part Design depends on a concrete Sheet Metal assembly.

Instead:

```text
Common semantic foundation
        ↑
        ├── Part Design
        ├── Sketch
        ├── Hybrid Surface
        ├── Freeform
        ├── Sheet Metal
        ├── Assembly/Product
        ├── Kinematics
        ├── Simulation Mapping
        ├── Drawing
        ├── PMI
        ├── Knowledge
        ├── Configuration
        └── Reuse/Templates
```

Real cross-domain relationships are expressed through explicit contracts and references, not accidental project dependencies.

## 2.5 Viewer owns presentation, not engineering truth

Viewer owns:

```text
camera
interaction
selection presentation
manipulation widgets
shading
lighting
textures
GPU resources
2D canvas presentation
render diagnostics
```

Viewer never decides:

```text
whether a feature succeeds
what a topology identity means
what an assembly constraint means
whether hidden geometry exists
what a drawing view mathematically contains
what a simulation constraint means
what a reference resolves to
```

---

# 3. The layer model

The system must be understood as six layers.

```text
L5  Client / Viewer / UI
        ↓
L4  Derived representations
        ↓
L3  CAD domain semantics + evaluation engine
        ↓
L2  Shared expressions / references / semantic foundation
        ↓
L1  Kernel contracts and numerical boundary
        ↓
L0  Rust mathematical authority
```

The layer meanings are fixed.

## L0 — Mathematical authority

Rust implementation of certified mathematical operations.

## L1 — Mathematical contracts

Typed, versioned requests/results that describe what the mathematical authority is being asked to calculate and what evidence it returns.

The contract is the abstraction boundary; the domain module does not know whether the result came from a specific internal algorithm.

## L2 — Shared CAD foundation

Common concepts required by every domain:

```text
identity
units
dimensions
expressions
coordinates
references
publications
semantic provenance
diagnostics
configuration context
```

## L3 — CAD semantics and Engine

The actual engineering system:

```text
Part Design
Sketch
Hybrid / Surface
Freeform
Sheet Metal
Assembly/Product
Kinematics
Simulation mapping
Drawing
PMI
Knowledge
Configurations
Templates
Design Review
Manufacturing contracts
```

The Engine is the cross-domain execution plane.

## L4 — Derived representations

```text
B-Rep result
lightweight shape
render mesh
simulation mesh
projected drawing geometry
PMI graphics
review snapshots
```

These are derived from semantic/result state.

## L5 — Client/viewer

Presentation, interaction, navigation, and editing orchestration only.

---

# 4. Project and library boundaries

Do not create a giant `UMLCAD_semanticCore` containing every concern.

Do not create dozens of one-class assemblies simply because the roadmap contains many subjects.

Start with these stable technical boundaries:

```text
UMLCAD.Cad.Expressions
UMLCAD.Cad.Semantics
UMLCAD.Cad.Contracts
UMLCAD.Cad.Engine
```

Then organize the domain model internally into explicit modules/namespaces:

```text
Cad.Semantics/
  Core/
  Documents/
  Expressions/
  Coordinates/
  References/
  Parts/
  Sketching/
  PartDesign/
  HybridGeometry/
  Freeform/
  SheetMetal/
  Assemblies/
  Kinematics/
  Simulation/
  Materials/
  Appearance/
  Drawings/
  PMI/
  Knowledge/
  Configurations/
  Templates/
  DesignReview/
  Manufacturing/
```

A subject becomes a separate assembly only when at least one of these is true:

```text
it has an independent public dependency boundary;
it must be consumed without the rest of Cad.Semantics;
it requires a separately versioned contract;
it creates a dependency cycle unless extracted;
it has a clear deployment/test ownership boundary.
```

The default is **module first, assembly later**.

---

# 5. Correct dependency direction

The intended dependency direction is:

```text
                 ┌──────────────────────┐
                 │ Cad.Expressions      │
                 └──────────┬───────────┘
                            ↓
                 ┌──────────────────────┐
                 │ Cad.Semantics        │
                 │ shared CAD model     │
                 └──────────┬───────────┘
                            ↓
          ┌─────────────────┴──────────────────┐
          │       domain semantic modules     │
          │                                   │
          │ Part / Sketch / Freeform /        │
          │ SheetMetal / Assembly / Kinematics│
          │ Drawing / PMI / Knowledge / ...   │
          └─────────────────┬──────────────────┘
                            ↓
                 ┌──────────────────────┐
                 │ Cad.Engine           │
                 │ planning/recompute   │
                 └──────────┬───────────┘
                            ↓
                 ┌──────────────────────┐
                 │ Cad.Contracts        │
                 └──────────┬───────────┘
                            ↓
                 ┌──────────────────────┐
                 │ Kernel.Client        │
                 └──────────┬───────────┘
                            ↓
                 ┌──────────────────────┐
                 │ Rust authority       │
                 └──────────────────────┘
```

There is an important implementation refinement:

```text
Domain Definition
      ↓
Domain Evaluator Adapter
      ↓
Engine
      ↓
Contract
      ↓
Kernel
```

The **definition itself must not depend on the concrete kernel client**.

This keeps design intent independent of execution machinery.

---

# 6. Specification, evaluation, result, representation

Every CAD operation is modeled as four distinct concepts.

```text
Specification
     ↓
Evaluation
     ↓
Result
     ↓
Representation
```

Example:

```text
SheetMetalBendDefinition
     ↓
SheetMetalBendEvaluator
     ↓
BendResult
     ↓
folded B-Rep / flat-pattern / render representation
```

Example:

```text
ExtrusionDefinition
     ↓
ExtrusionEvaluator
     ↓
ExtrusionResult
     ↓
B-Rep / topology / tessellation
```

Example:

```text
MotionBehaviorDefinition
     ↓
KinematicEvaluator
     ↓
KinematicStateResult
     ↓
animation / trace / simulation representation
```

Rules:

```text
Definition stores intent.
Evaluator performs an execution step.
Result records authoritative outcome and evidence.
Representation is derived for a consumer.
```

Never collapse these into one mutable `Feature` object with `Execute()`, `Dirty`, and hidden result state.

---

# 7. Shared mathematical and engineering foundation

The system needs one shared place for concepts that recur across all domains.

## 7.1 Expressions

Expressions are immutable typed ASTs.

Required pipeline:

```text
source
 ↓
tokenize
 ↓
parse
 ↓
typed AST
 ↓
unit/type validation
 ↓
dependency extraction
 ↓
evaluation
 ↓
canonical serialization
```

Every persisted/canonical algebraic expression is fully parenthesized.

Examples:

```text
(a + (b * c))
((a + b) * (c - d))
(-(a / b))
((R + (K * T)) * ((pi / 180) * A))
```

The AST is authoritative. Text is only a deterministic projection.

## 7.2 Domain equations

Engineering equations are allowed to live in domain modules as semantic formulas, but they execute through the common expression system.

For example, Sheet Metal may define:

```text
BendAllowance = ((pi / 180) * (R + (K * T)) * A)
```

That does **not** make Sheet Metal a numerical kernel.

The Sheet Metal module owns:

```text
what K means
what T means
what R means
what A means
which material/bend rule is active
which bend-table entry applies
which validity conditions apply
```

The expression subsystem owns:

```text
AST
units
dimensions
canonicalization
evaluation mechanics
dependency extraction
```

Exact geometric bend/unbend construction remains a kernel contract.

## 7.3 Coordinates and references

Explicit frames are first-class:

```text
WorldFrame
DocumentFrame
PartFrame
BodyFrame
SketchFrame
FaceFrame
OccurrenceFrame
DrawingViewFrame
SimulationFrame
```

References are semantic objects, not array positions.

---

# 8. Reference and topology law

Never use fragile identities such as:

```text
Edges[3]
Faces[12]
Surface[2]
```

as permanent semantic references.

A reference must carry sufficient provenance/context to resolve against a result.

Required concepts include:

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

Resolution status must distinguish:

```text
Resolved
Missing
Ambiguous
Indeterminate
Unsupported
```

Topology evolution is an explicit result of evaluation.

A downstream object either receives a certified correspondence or receives an explicit failure/ambiguity outcome.

No silent guessing.

---

# 9. Dependency graph and incremental evaluation

The semantic system maintains three related graphs.

```text
Specification graph
Dependency/evaluation graph
Result graph
```

They are related but not interchangeable.

## 9.1 Specification graph

Represents design intent and references.

```text
Sketch → Support Face
Pocket → Sketch Profile
Fillet → Edge Reference
Drawing View → Product Occurrence
Motion Equation → Parameters
```

## 9.2 Evaluation graph

Represents executable dependencies.

```text
resolve inputs
      ↓
validate
      ↓
evaluate
      ↓
validate result/evidence
      ↓
integrate topology/reference evolution
```

## 9.3 Result graph

Represents what evaluation produced.

```text
Result
 ├── solid
 ├── shell
 ├── surface
 ├── wire
 └── topology bindings
```

---

# 10. Deterministic cache is part of the architecture

Incremental recomputation is not a UI optimization.

It is the execution model.

The same planner/evaluator path must support both:

```text
Full recompute = maximal invalidation
Incremental recompute = actual invalidation
```

Required invariant:

```text
SemanticResult(FullRecompute(M, Δ))
    ≡
SemanticResult(IncrementalRecompute(M, Δ))
```

Another required invariant:

```text
CacheHit(EvaluationIdentity)
    ≡
FreshEvaluation(EvaluationIdentity)
```

## 10.1 Evaluation identity

Identity includes, as applicable:

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

Canonical identity is content-addressed.

No memory addresses, object hashes, execution timestamps, or viewer state.

## 10.2 Cache layers

Use layers only where the system needs them:

```text
specification normalization/resolution
kernel evaluation
topology/reference mapping
tessellation
simulation mesh
drawing projection/HLR
compiled/render representation
```

The cache is outside Rust.

## 10.3 Invalidation

Changes are explicit:

```text
ChangeSet
 ↓
affected specification nodes
 ↓
dependency closure
 ↓
evaluation plan
 ↓
cache reuse or evaluation
 ↓
result integration
```

No hidden mutable `Dirty` network is the source of truth.

---

# 11. Incremental rendering is a first-class consequence

The current whole-artifact viewer pattern is not sufficient to claim true incremental rendering.

The target is:

```text
semantic change
    ↓
affected specification nodes
    ↓
affected evaluation results
    ↓
affected representations
    ↓
affected render assets
    ↓
viewer replaces only affected scene data
```

A large assembly must not require regeneration of unrelated render data merely because one part changed.

Render identities therefore derive from the authoritative representation identity and relevant display policy.

The viewer consumes a scene manifest or equivalent representation graph that allows independently addressable renderables/instances.

Repeated assembly occurrences should reuse geometry representation identity while retaining occurrence-specific transforms and semantic identity.

The viewer never becomes the incremental evaluator. It only applies the resulting representation delta.

---

# 12. Domain architecture

This roadmap defines domains by engineering responsibility.

## 12.1 Part Design

Owns additive/subtractive design intent and body semantics.

Examples:

```text
Extrusion / Pad
Pocket
Revolution / Shaft
Groove
Rib
Slot
Hole
Shell
Draft
Fillet / Blend
Chamfer
Patterns
Boolean operations
```

The exact geometry is delegated to contracted kernel operations.

## 12.2 Sketch

Owns:

```text
SketchDefinition
SketchSupport
SketchFrame
Geometry
ProfileLoop
Constraints
SolveState
Publications
```

The existing analytic Jacobian/constraint authority is reused.

## 12.3 Hybrid geometry / GSD

Owns wireframe, surfaces, hybrid combinations, associative construction, surface networks, and surface healing semantics.

## 12.4 Freeform

Freeform is a **major domain**, not a handful of surface commands.

The architecture must support multiple representations:

```text
Explicit/NURBS
Equation-defined geometry [initially experimental]
Subdivision surface
Associative freeform feature
Direct freeform editing
```

Freeform semantics include:

```text
control-point edits
surface patches
boundary networks
matching
continuity
curvature/deviation analysis
deformation
morphing
quality diagnostics
```

Exact surface mathematics remains in the kernel contract layer.

## 12.5 Equation-defined geometry

Equation geometry is supported as a deliberate research/advanced-design path, not as a replacement for NURBS.

Examples:

```text
P(t) = (x(t), y(t), z(t))
S(u,v) = (x(u,v), y(u,v), z(u,v))
F(x,y,z) = 0
```

A domain is production-valid only after explicit contracts cover as applicable:

```text
domain
continuity
regularity
derivatives
singularity handling
self-intersection
boundedness
orientation
trim participation
projection
intersection
topological conversion
```

Unsupported cases fail closed.

## 12.6 Sheet Metal

Sheet Metal is a specialized semantic engineering module.

It owns:

```text
SheetMetalPart
SheetMetalParameters
MaterialRule
Thickness
BendRadius
BendAngle
KFactor
BendAllowance
BendDeduction
NeutralAxis
BendTable
ReliefDefinition
FlangeDefinition
CornerDefinition
FoldDefinition
UnfoldDefinition
FlatPatternDefinition
ManufacturingData
```

Its relation to mathematics is explicit:

```text
Sheet Metal engineering equations
        ↓
Cad.Expressions / typed units
        ↓
Sheet Metal domain validation
        ↓
Engine
        ↓
bend/unfold/offset/intersection topology contract
        ↓
Rust mathematics
```

Therefore Sheet Metal **depends on the shared mathematical contract layer, not directly on the concrete Rust implementation**.

Its equations and engineering rules do not belong inside Rust merely because they are formulas.

Its geometric deformation/unfolding operations do not belong as ad-hoc C# geometry either.

This split applies to every specialized domain.

---

# 13. Product and Assembly

Assembly is a semantic graph.

It contains:

```text
Product
SubProduct
PartOccurrence
PartDefinition
OccurrenceTransform
Configuration
AssemblyConstraints
EngineeringConnections
FunctionalInterfaces
ContextualLinks
BOM semantics
Flexible behavior
```

The occurrence and the part definition are different identities.

Repeated occurrences may share one evaluated part result while retaining occurrence-specific transform and context.

Assembly-level features must be modeled as semantic operations over a product context rather than as unexplained shape mutations.

---