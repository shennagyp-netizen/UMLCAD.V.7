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
CAD references to material/scientific data and material assignment semantics
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

# 3. Abstraction and dependency model

The system architecture is a directed acyclic graph of abstraction levels and reusable services.

The principal semantic direction is:

`
Platform Foundation
        ↓
Mathematics
        ↓
Science
        ↓
CAD Engineering Core
        ↓
Engineering Resource Model
        ↓
Specialized Engineering Domains
        ↓
Application / Workflow / Presentation
`

This is an abstraction model, not a requirement that every row become one .NET project.

Science provides reusable facts and services. CAD Core provides shared engineering meaning. Specialized domains consume those services and add domain meaning.

External technologies are reached through outward adapters:

`
UMLCAD-owned contract/service
        ↓
adapter/provider
        ↓
external application/controller/system
`

C4 terminology is separate from project terminology. A C4 Container is a major runtime/deployment/data boundary; it is not automatically a .NET assembly.

The normative details are in:
- `docs/architecture/ABSTRACTION_AND_DEPENDENCY_MODEL.md`
- `docs/architecture/c4/`
- `docs/architecture/architecture.json`

B-Rep is authoritative when produced by a certified operation. Render meshes, drawing graphics, simulation meshes, and other consumer assets are derived representations.
# 4. Project and library boundaries

Project boundaries must follow stable dependency responsibilities rather than the current feature list.

The current .NET foundation remains:

`
UMLCAD.Cad.Expressions
UMLCAD.Cad.Semantics
UMLCAD.Cad.Contracts
UMLCAD.Cad.Engine
`

These are an implementation projection of the larger architecture, not its permanent shape.

Do not create a giant semantic-core assembly containing every domain.

Do not create dozens of one-class assemblies without a genuine boundary.

A subject should become a separate library when it has an independent public/dependency boundary, requires separate versioning or test ownership, breaks a dependency cycle, or is a substantial bounded engineering discipline.

### Sheet Metal

Sheet Metal is explicitly a strong candidate for a separate engineering library because it owns a coherent manufacturing-specific semantic system:

`
Thickness
Bend
Flange
Relief
BendTable
KFactor
BendAllowance
Fold/Unfold
FlatPattern
material/process compatibility
machine/tool constraints
`

Its exact placement is governed by the architecture manifest, not by arbitrary project minimization.

A separate `UMLCAD.Engineering.SheetMetal` library is preferred once implementation begins, unless a demonstrated boundary analysis shows that keeping it within a broader project is architecturally cleaner.

The Sheet Metal library must consume shared Science, CAD Core, Engineering Resource, expression, and applicable Phenomena Simulation services. It must not become a private geometry kernel or a direct concrete Rust client.

### Other domains

The same rule applies to:

`
Part Design
Freeform
Assembly/Kinematics
CAM
Drawing
PMI
Knowledge
`

A domain is separated when the boundary is real, not merely because the roadmap names it.
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

Sheet Metal is a specialized engineering domain with its own material/process/machine behavior.

It owns semantic concepts such as:

`
SheetMetalPart
Thickness
BendDefinition
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
`

Its dependency structure is:

`
Sheet Metal
   → CAD Core
   → Science / Material
   → Engineering Resource Model
   → Expression / Quantity services
   → Phenomena Simulation Service when scientifically applicable
`

Its domain equations use the shared expression system. Its exact geometric operations use the common Engine → Math Contract → Rust boundary.

Sheet Metal must be capable of rejecting materially or mechanically incompatible operations before requesting exact geometry. The rejection must be based on material/process/machine/tool capabilities and rules, not hard-coded material-name cases.

A separate Sheet Metal library is the preferred implementation boundary once the domain is introduced in code.
# 13. Product, Assembly, and BOM

Product structure is part of the CAD Engineering Core.

Assembly owns product/occurrence semantics:

```
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
```

BOM is a CAD product-structure service/view over that authoritative structure.

```
Product Structure
      ↓
   BOM Service
      ↓
  ┌───┼───────────┐
  ↓   ↓           ↓
Drawing CAM      PLM/ERP
```

Manufacturing, drawing, PLM/ERP, purchasing, and service views may derive specialized BOM interpretations, but they do not redefine the underlying CAD product structure.

The occurrence and the part definition are different identities. Repeated occurrences may share an evaluated part result while retaining occurrence-specific transform and context.

Assembly-level features are semantic operations over product context, not unexplained shape mutations.

# 14. CAM, Manufacturing, and G-code

CAM is an engineering domain that consumes authoritative CAD/product results, material science, and the shared Engineering Resource Model.

Its responsibilities include:

\`\`\`
ManufacturingSetup
StockDefinition
ManufacturingFeature
ManufacturingOperation
MachiningStrategy
CuttingCondition
Toolpath
OperationSequence
SimulationRequest
NCProgram
PostprocessingIntent
\`\`\`

CAM does not stop at toolpath generation.

The production path is:

\`\`\`
Authoritative CAD Result
        ↓
Manufacturing interpretation
        ↓
Process planning
        ↓
Machine / Tool / Fixture capability checks
        ↓
optional Phenomena Simulation Service
        ↓
Toolpath
        ↓
Machine-specific postprocessor
        ↓
deterministic NC / G-code program
\`\`\`

G-code/NC is therefore a real downstream engineering result, not a viewer/export convenience.

The semantic model must distinguish:

\`\`\`
Toolpath          = machine-independent or machine-constrained path intent
Postprocessor     = translation to a controller dialect
NC/G-code         = executable machine program representation
Machine controller= external execution environment
\`\`\`

A postprocessor is an adapter/provider implementing an explicit manufacturing-output contract.

It must not alter manufacturing semantics silently.

The G-code generation contract must preserve deterministic ordering, explicit units/modes, tool/operation identity, machine context, and validation diagnostics.

The application must be able to report:

\`\`\`
valid
invalid
unsupported
ambiguous
non-postprocessable
\`\`\`

rather than emitting plausible but unsafe NC output.

## 14.1 Machine-aware CAM

The Engineering Resource Model provides:

\`\`\`
Machine
Tool
Fixture
Process
MachineCapability
ToolCapability
ProcessCapability
\`\`\`

CAM combines these with material and geometry.

For example:

\`\`\`
Material
   +
Machine capability
   +
Tool capability
   +
Operation constraints
   +
Geometry
        ↓
Manufacturing plan
        ↓
Toolpath
        ↓
Postprocessor
        ↓
G-code / NC
\`\`\`

Machine/controller integration is external to the semantic manufacturing model.

---

# 15. Drawing capability map

Drawing must be mapped against the real CATIA drafting capability surface, not reduced to "views + dimensions".

Dassault Systèmes documents Generative Drafting capabilities including associative generation from 3D parts, assemblies, surfaces, hybrid parts and sheet-metal definitions; front/side/top/isometric views; section, aligned/offset section, detail, circular/profiled detail and clipping views; associative dimensions and annotations; dress-up; assembly filtering; BOM generation in drawings; and standards/interoperability such as ANSI/ISO/JIS and DXF/DWG. Interactive Drafting adds interactive 2D design, dimensioning, associative annotations, GD&T, balloons, roughness symbols, notes, centerlines/axes/thread lines/area-fill/mark-up arrows, and drawing structure editing. These are treated as capability requirements to map into UMLCAD's own semantics, not as a command-by-command CATIA object model.

The UMLCAD Drawing domain therefore needs, by capability family:

### Drawing document structure

\`\`\`
DrawingDocument
DrawingSheet
SheetFormat
Border
TitleBlock
RevisionBlock
Zone
DrawingTree
ExternalReference
\`\`\`

### Generated views

\`\`\`
ProjectionView
FrontView
RearView
TopView
BottomView
LeftView
RightView
IsometricView
AuxiliaryView
SectionView
AlignedSectionView
OffsetSectionView
DetailView
CircularDetailView
ProfiledDetailView
ClippingView
BrokenView
UnfoldedView
\`\`\`

### View semantics

\`\`\`
ViewSource
ViewOrientation
ViewAxis
ProjectionMethod
DisplayMode
HiddenLinePolicy
SectionDefinition
ClippingDefinition
AssemblyFilter
GenerativeUpdateState
AssociativityState
\`\`\`

### Dimensions

At minimum:

\`\`\`
Length
Distance
Angle
Radius
Diameter
Chamfer
Coordinate/ordinate
Baseline/chain
Hole/thread-related dimension
Angular/linear associative dimensions
\`\`\`

Dimension semantics must support:

\`\`\`
tolerances
limits
units
precision
prefix/suffix
stacking
reference/driving distinction
associativity
placement constraints
dimension standards
\`\`\`

### Annotation / GD&T

\`\`\`
Text
RichText
Note
Leader
Balloon
Datum
DatumTarget
GeometricTolerance
FeatureControlFrame
SurfaceTexture/Roughness
WeldingSymbol
FlagNote
\`\`\`

Where standards are supported, the domain must explicitly encode the applicable standard rather than treating formatting as arbitrary viewer text.

### Dress-up / 2D geometry

\`\`\`
Centerline
AxisLine
SymmetryLine
ThreadLine
AreaFill
Hatching
BreakLine
ConstructionGeometry
MarkUpArrow
2D geometry
\`\`\`

### Sheet presentation

\`\`\`
line types
line weights
fonts
symbols
layers
view-specific display properties
standards
sheet scaling
view positioning
\`\`\`

### BOM on drawing

Drawing must consume the CAD Product Structure/BOM service and support:

\`\`\`
BOMTable
BOMItem
ItemNumber
Quantity
PartNumber
Revision
Description
\`\`\`

The drawing BOM is a representation of the CAD product structure; it does not redefine assembly truth.

### Associativity

The key contract is:

\`\`\`
3D authoritative result
        ↓
Drawing View specification
        ↓
projection / section / hidden-line result
        ↓
drawing dimensions / annotations / dress-up
\`\`\`

When the model changes, the drawing records which specifications remain associative, which require update, and which become missing/ambiguous/invalid.

No viewer heuristic may silently repair broken drawing references.

### Drawing standards

The domain must support an explicit standard/context model, initially allowing the architectural space for:

\`\`\`
ISO
ANSI
JIS
project/customer-specific standards
\`\`\`

The selected standard affects drafting semantics and presentation rules; it must not change the underlying CAD geometry.

The detailed CATIA capability mapping used for this scope is based on Dassault Systèmes' Generative Drafting and Interactive Drafting product descriptions. See the architecture source references recorded with this roadmap.

---

# 16. Science, phenomena simulation, and engineering consumers

Phenomena simulation is below CAM and Sheet Metal as a reusable scientific capability.

The service answers:

\`\`\`
"What happens physically under these inputs?"
\`\`\`

The consuming engineering domain answers:

\`\`\`
"What engineering decision should be made from that result?"
\`\`\`

Thus:

\`\`\`
Science
  └── Phenomena Simulation Service
             ↑
             ├── CAM
             ├── Sheet Metal
             ├── Kinematics / other domains where applicable
             └── future engineering domains
\`\`\`

An implementation provider may use an internal solver or an external scientific application.

The provider is interchangeable behind the service contract.

---

# 17. Library boundary strategy

The architecture distinguishes logical units from physical assemblies.

The following are strong candidates for independent libraries when their implementation begins:

\`\`\`
UMLCAD.Science
UMLCAD.Engineering.Resources
UMLCAD.Engineering.SheetMetal
UMLCAD.Engineering.CAM
UMLCAD.Engineering.Drawing
\`\`\`

This is not a requirement to create all five before their public boundaries are real.

The promotion rule is:

\`\`\`
logical domain
      ↓
stable public semantic/service boundary
      ↓
independent assembly when justified
\`\`\`

The important rule is that **a strong bounded domain must not remain buried in a generic project merely to reduce project count**.

At the same time, the solution must not be fragmented into one project per command.

---

# 18. Capability traceability

Every large engineering domain receives a capability matrix:

\`\`\`
External capability
    ↓
UMLCAD semantic capability
    ↓
required services
    ↓
required mathematical contracts
    ↓
authoritative result
    ↓
derived representations
    ↓
E2E scenario
\`\`\`

A CATIA comparison is therefore made by capability and evidence, not by counting command names.

