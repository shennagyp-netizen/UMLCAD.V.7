# C4 Critical Flows — UMLCAD V7

These sequences make the dependency direction explicit.

## 1. Drawing measurement

`
Drawing
  │
  ├── resolve reference ─────────▶ CAD Reference Service
  │                                      │
  │                                      ▼
  │                               authoritative CAD result
  │                                      │
  └── request measurement ───────▶ Science Measurement Service
                                         │
                                         ▼
                                  quantity/unit result
                                         │
                                         ▼
                                   Drawing Rules
                                         │
                                         ▼
                                  dimension representation
`

Authority:

- CAD: authoritative geometry/topology.
- Science: measurement, units, scientific calculation.
- Drawing: drafting meaning and rules.
- Viewer: presentation.

Science never depends on Drawing.

## 2. CAM using phenomena simulation

`
CAM
 │
 ├──▶ CAD Result
 ├──▶ Material Service
 ├──▶ Machine/Tool Capability Service
 ├──▶ Process Rules
 └──▶ Phenomena Simulation Service
                 │
                 ▼
          Provider contract
                 │
          ┌──────┼─────────┐
          ▼      ▼         ▼
       internal Solver A  Solver B
       provider adapter  adapter
          │      │         │
          └──────┼─────────┘
                 ▼
          phenomenon result
                 │
                 ▼
                CAM
                 │
                 ▼
             Toolpath
`

The provider may call an external scientific application, but that application does not become the CAM dependency.

## 3. Sheet Metal

`
SheetMetal Definition
        │
        ├──▶ Material Science
        ├──▶ Machine/Tool Capability
        ├──▶ Sheet Metal rules
        └──▶ Phenomena Simulation Service (optional)
                 │
                 ▼
             validation
                 │
                 ▼
          CAD Evaluation Engine
                 │
                 ▼
          explicit math contract
                 │
                 ▼
        Rust mathematical result
                 │
                 ▼
       Bend / FlatPattern result
`

The architecture can therefore reject incompatible material/process combinations before exact geometry generation.

## 4. BOM

`
Product Structure
       │
       ▼
   BOM Service
       │
   ┌───┼──────────┐
   ▼   ▼          ▼
Drawing CAM     PLM/ERP
`

BOM consumers query a stable product-structure view. They do not redefine the product structure.

## 5. Generic CAD geometry operation

`
Domain Definition
       ↓
Domain Evaluator
       ↓
Evaluation Engine
       ↓
Explicit Math Contract
       ↓
Kernel Client
       ↓
Rust Mathematical Authority
       ↓
Authoritative Result + Evidence + Provenance
       ↓
Representation / Drawing / CAM / other consumers
`

## 6. Result flow versus compile-time dependency

This is a core architectural distinction.

Allowed:

`
Sheet Metal
     ↓
published FlatPatternResult
     ↓
CAM
`

Not the default:

`
CAM
     ↓
private SheetMetal classes/state
`

A result may travel upward or sideways. That does not authorize a lower or peer implementation dependency.

## 7. Architectural invariants

- Lower layers do not know upper engineering domains.
- Shared service providers do not depend on their consumers.
- Peer domains do not depend on private peer implementations by default.
- Published results/contracts may be consumed across domains.
- Product Structure/BOM is owned by CAD.
- Phenomena Simulation is a reusable scientific service with multiple implementations.
- Machine/Tool/Process is shared engineering-resource semantics, not a CAM-private model.
- Adapters point outward.
- B-Rep is authoritative when produced by a certified operation; downstream meshes/graphics are derived.
- Rust owns mathematical authority, not engineering semantics.
