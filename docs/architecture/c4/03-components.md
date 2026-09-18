# C4 Level 3 — UMLCAD V7 Engineering Runtime Components

The Engineering Runtime is the principal logical container. Components are grouped by responsibility and abstraction.

`
┌────────────────────────────────────────────────────────────────────┐
│ .NET ENGINEERING RUNTIME                                           │
│                                                                    │
│  Foundation / Mathematics                                         │
│      │                                                             │
│      ▼                                                             │
│  Science Services ────────────────┐                               │
│      │                             │                               │
│      │ Material / Units /          │ Phenomena Simulation Service  │
│      │ Physical Models             │                               │
│      │                             │                               │
│      ▼                             ▼                               │
│  CAD Engineering Core ◀──── Engineering Resource Model             │
│      │                             │                               │
│      │ Product/BOM                 │ Machine/Tool/Fixture/Process  │
│      │ References                  │ Capability                    │
│      │ Evaluation                  │                               │
│      │ Results                     │                               │
│      └──────────────┬──────────────┘                               │
│                     ▼                                              │
│            Specialized Engineering Domains                         │
│       PartDesign / SheetMetal / Freeform / Assembly                │
│       Kinematics / CAM / Drawing / PMI / Knowledge / ...           │
│                     │                                              │
│                     ▼                                              │
│            Representation orchestration                            │
│                                                                    │
└─────────────────────┬──────────────────────────────────────────────┘
                      ▼
              Explicit Math Contracts
                      ▼
             Rust Mathematical Authority
`

## Science components

Science owns reusable facts and services:

`
Material model
Quantity / Unit services
Physical models
Phenomena definitions
Phenomena Simulation Service
scientific/numerical policies
`

The Phenomena Simulation Service is a service with multiple implementations, not a peer application called "Simulation".

## CAD Engineering Core components

`
Product / Assembly product structure
BOM service/view
Part / Body semantics
References / Publications
Coordinates / Frames
Evaluation graph
Authoritative result integration
Topology provenance
Configuration/context
Knowledge integration
CAD diagnostics/provenance
`

## Engineering Resource Model

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

It is shared by CAM, Sheet Metal, manufacturing planning, inspection and future domains.

## Domain components

### Sheet Metal

Owns bend/flange/relief/unfold/flat-pattern semantics and material/process compatibility rules.

Consumes Science, CAD Core, Resource Model, expressions, and the Phenomena Simulation Service where applicable.

### CAM

Owns manufacturing operations, process planning, toolpaths, machining strategy, manufacturing constraints and postprocessing intent.

Consumes CAD results, material facts, machine/tool/process capabilities, and Phenomena Simulation.

### Drawing

Owns drafting semantics, views, sheets, dimensions, annotations, projections and drafting rules.

Consumes authoritative CAD results and scientific measurement/unit services.

### Assembly/Kinematics

Own product/occurrence relationships and motion semantics on top of CAD references, transforms, constraints and expressions.

### Freeform

Owns freeform design intent, control edits, continuity/matching/quality semantics. Exact geometric mathematics remains behind mathematical contracts.

## Provider components

Provider implementations are outside semantic ownership:

`
PhenomenaSimulationProvider
MachineControllerProvider
PLMProvider
ERPProvider
FileProvider
KernelClient
`

Providers implement contracts owned by UMLCAD.

## Critical dependency distinction

`
Domain → service                     allowed
Domain → published result/contract   allowed
Domain → private peer implementation forbidden by default
Service provider → consumer          forbidden
External technology → UMLCAD meaning forbidden
`
