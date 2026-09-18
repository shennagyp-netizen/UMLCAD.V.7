# UMLCAD V7 — Abstraction and Dependency Model

## 1. Purpose

UMLCAD is too large to make the feature list or project tree the architectural authority.

For every concept, the architecture must answer:

- who owns its meaning;
- which general facts/services it consumes;
- which result/contract it publishes;
- who can consume that result;
- whether the consumer depends on implementation or only on a stable contract;
- which external adapter implements concrete technology;
- which layer is authoritative when results disagree.

## 2. Abstraction direction

The principal dependency direction is:

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

This is a directed acyclic architectural graph, not a requirement that every implementation live in a separate assembly.

External technology is reached through outward adapters:

`
UMLCAD-owned service/contract
        ↓
adapter/provider
        ↓
external application / controller / PLM / file system
`

## 3. Semantic ownership

### Science owns

`
quantities
units
dimensions
physical constants
materials
material properties
physical models
phenomena definitions
scientific constraints/policies
phenomena simulation service contracts
`

Science provides reusable facts and scientific services. It does not know Drawing, CAM, Sheet Metal, or other application meanings.

### CAD Engineering Core owns

`
Product / Assembly product structure
Part / Body
CAD specifications
References / Publications
Coordinates / Frames
Dependency / Evaluation graph
Authoritative CAD result integration
Topology provenance
Configuration / context
Knowledge integration
BOM semantics
CAD diagnostics / provenance
`

The product structure is authoritative for design/product relationships. BOM is a CAD product-structure service/view, not an independently invented CAM model.

### Engineering Resource Model owns

`
Machine
Tool
Fixture
Process
MachineCapability
ToolCapability
ProcessCapability
ManufacturingConstraint
`

This model is shared infrastructure for manufacturing-oriented domains. It does not depend on CAM.

### Specialized domains own

`
Part Design
Sketch
Freeform
Sheet Metal
Assembly/Kinematics
CAM
Drawing
PMI
Knowledge
Design Review
...
`

Each domain owns its own engineering language and rules.

## 4. Materials

Material identity and scientifically defined properties belong to Science.

Example:

`
Material
 ├── classification/composition
 ├── density
 ├── elastic properties
 ├── plastic properties
 ├── thermal properties
 ├── electrical properties
 └── other scientifically defined properties
`

A domain interprets those facts.

Therefore:

`
Science:
  "This material has these properties."

Sheet Metal:
  "Given these properties, this bend/process is valid or invalid."

CAM:
  "Given these properties, this process is feasible under these
   machine/tool conditions."
`

There must not be duplicated material truth in Sheet Metal, CAM, Simulation, and other domains.

This naturally rejects combinations such as a non-sheet material being passed to a Sheet Metal process because the Sheet Metal rules query the material's classification/capabilities rather than hard-coding material names.

## 5. Phenomena Simulation Service

This is a critical boundary.

UMLCAD does not define:

`
CAM → Simulation Application
`

It defines a reusable scientific capability:

`
Phenomena Simulation Service
`

The service models physical phenomena. It is not an engineering application.

Examples, only where explicitly contracted:

`
deformation
stress/strain response
thermal response
vibration
cutting-process response
...
`

Multiple implementations may exist:

`
                 Phenomena Simulation Service
                             │
                       provider contract
                             │
              ┌──────────────┼──────────────┐
              ↓              ↓              ↓
         internal solver  external app A  external app B
              provider      adapter         adapter
`

The provider/adapters implement the scientific service contract.

The service does not depend on CAM, Sheet Metal, or another consumer.

CAM may consume it:

`
CAM
 ├── CAD result
 ├── material service
 ├── machine/tool capability
 ├── process rules
 └── phenomena simulation service
`

Sheet Metal may consume it for applicable phenomena such as material response or springback, while retaining ownership of Sheet Metal interpretation.

## 6. Machines and tools

A machine is an engineered resource, not merely a CAM class.

Its semantic model may include:

`
machine family
kinematic structure
axes
work envelope
limits
interfaces
accuracy
process capability
controller characteristics
`

Families can include:

`
MachiningCenter
Lathe
WireEDM
PressBrake
LaserCutter
Waterjet
GrindingMachine
...
`

"CNC" is treated as a control/automation characteristic where appropriate, not as the universal machine taxonomy.

CAM and Sheet Metal consume machine/tool/process capability services.

Controller/postprocessor integration is an adapter boundary.

## 7. Sheet Metal

Sheet Metal is a real bounded engineering domain and is a strong candidate for its own library.

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
NeutralAxis
BendTable
ReliefDefinition
FlangeDefinition
CornerDefinition
FoldDefinition
UnfoldDefinition
FlatPatternDefinition
`

Its dependencies are:

`
Sheet Metal
   → CAD Core
   → Science / Material
   → Engineering Resource Model
   → Expression/quantity services
   → Phenomena Simulation Service, when applicable
`

For exact geometry:

`
SheetMetal Definition
      ↓
SheetMetal validation/rules
      ↓
CAD Evaluation Engine
      ↓
explicit mathematical contract
      ↓
Rust mathematical authority
`

The Sheet Metal module owns the meaning of a bend; the mathematical subsystem owns the exact contracted geometric computation.

## 8. Drawing

Drawing is an upper engineering domain.

For a dimension:

`
Drawing
   ↓
Reference Resolution
   ↓
Authoritative CAD Result
   ↓
Science Measurement Service
   ↓
Quantity / Unit Service
   ↓
Drawing applies drafting rules
   ↓
Dimension representation
`

Authority is separated:

`
CAD      → geometry/topology truth
Science  → measurement/unit/scientific truth
Drawing  → drafting semantics and rules
Viewer   → presentation
`

Therefore:

`
Science → Drawing       FORBIDDEN
Drawing → Science       ALLOWED through public services/contracts
`

Science can return a scientific validation/calculation consumed by Drawing without knowing that Drawing exists.

## 9. CAM

CAM consumes:

`
CAD authoritative results
Material Science
Machine/Tool/Fixture models
Process capabilities
Phenomena Simulation Service
`

CAM owns:

`
ManufacturingOperation
ProcessPlanning
Toolpath
MachiningStrategy
manufacturing constraints
postprocessing intent
`

CAM does not own the machine model, material model, CAD product structure, or phenomena simulation engine.

## 10. BOM and product structure

Product structure belongs to CAD.

`
Product
 ├── Component
 ├── Occurrence
 ├── quantity
 ├── part number
 ├── revision
 └── configuration
          ↓
       BOM Service
          ↓
   ┌──────┼────────┐
   ▼      ▼        ▼
 Drawing  CAM     PLM/ERP
`

Consumers receive a stable BOM/result view.

They do not redefine the authoritative product structure.

## 11. Domain result flow versus dependency

These are different.

A producer can publish a result that another domain consumes without the consumer depending on the producer's private implementation.

Correct:

`
Sheet Metal
    ↓
published FlatPattern / Manufacturing result
    ↓
CAM
`

Incorrect as the default architecture:

`
CAM
    ↓
private SheetMetal classes/state
`

The same principle applies to Drawing consuming CAD results and simulation consuming CAD/scientific inputs.

## 12. Specification → Evaluation → Result → Representation

Every engineering operation retains four concepts:

`
Specification
      ↓
Evaluation
      ↓
Authoritative Result
      ↓
Derived Representation
`

Examples:

`
SheetMetalBendDefinition → evaluator → BendResult → folded/flat/render representations

ExtrusionDefinition → evaluator → ExtrusionResult → B-Rep/tessellation representations

MotionBehaviorDefinition → evaluator → KinematicResult → animation/trace representations
`

B-Rep is authoritative when it is produced by a certified operation. Render mesh, drawing graphics, simulation mesh, and review assets are derived representations.

## 13. Service dependency

Reusable services are consumed from above.

Examples:

`
Drawing
    → MeasurementService
    → UnitService
    → ReferenceResolver

CAM
    → MaterialPropertyService
    → MachineCapabilityService
    → ToolCapabilityService
    → PhenomenaSimulationService

Sheet Metal
    → MaterialPropertyService
    → MachineCapabilityService
    → GeometryQueryService
    → PhenomenaSimulationService
`

The service provider must not add a dependency back to the consumer.

## 14. Dependency matrix

Default allowed direction:

| Unit | Foundation | Math | Science | CAD Core | Resources | Peer domain |
|---|---:|---:|---:|---:|---:|---:|
| Mathematics | yes | — | — | — | — | — |
| Science | yes | yes | — | — | — | — |
| CAD Core | yes | yes | yes | — | — | — |
| Resource Model | yes | yes | yes | yes where CAD contracts are needed | — | — |
| Part Design | yes | yes | yes | yes | no | no private dependency |
| Sheet Metal | yes | yes | yes | yes | yes | no private dependency |
| Assembly/Kinematics | yes | yes | yes | yes | no | no private dependency |
| CAM | yes | yes | yes | yes | yes | no private dependency |
| Drawing | yes | yes | yes | yes | no | no private dependency |
| Freeform | yes | yes | yes | yes | no | no private dependency |

A direct peer-domain implementation dependency requires an explicit architecture decision and a stable contract boundary.

## 15. Adapters

Adapters point outward.

Examples:

`
Simulation provider
Machine/controller adapter
PLM/PDM adapter
ERP adapter
File-format adapter
Rust kernel client
`

Pattern:

`
UMLCAD-owned contract
        ↓
adapter/provider
        ↓
external technology
`

The external implementation never becomes the source of UMLCAD semantic meaning.

## 16. Rust boundary

Exact mathematical work follows:

`
Domain Definition
      ↓
Evaluator / Engine
      ↓
explicit math contract
      ↓
Kernel Client adapter
      ↓
Rust mathematical authority
      ↓
authoritative mathematical result/evidence
`

No domain directly depends on the concrete Rust client.

CPU f64 remains the normative mathematical reference. GPU remains acceleration only. Unsupported, ambiguous, singular, degenerate, non-finite, and indeterminate cases fail closed.

## 17. Architectural invariants

1. Lower layers do not know upper engineering domains.
2. Shared services are consumed from above.
3. Service providers do not depend on their consumers.
4. Peer domains do not depend on private peer implementations by default.
5. Published results may cross domain boundaries.
6. Product Structure/BOM is authoritative in CAD.
7. Phenomena Simulation is a reusable scientific service with multiple implementations.
8. Machine/Tool/Process is shared engineering infrastructure rather than a CAM-private model.
9. Representations never become semantic authority.
10. Rust never owns CAD/SIM/CAM/Sheet Metal/Drawing semantics.
