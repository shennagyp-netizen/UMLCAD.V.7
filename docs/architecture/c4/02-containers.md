# C4 Level 2 — UMLCAD V7 Containers

## C4 terminology

A C4 Container is a major runtime/deployment/data boundary. It is not automatically a .NET project, namespace, class, or Docker container.

Several .NET libraries can be Components inside one runtime Container. Conversely, one architectural boundary may later justify multiple deployable containers.

## Container model

`
┌───────────────────────────────────────────────────────────────┐
│                         UMLCAD V7                             │
│                                                               │
│  ┌──────────────────────┐     ┌────────────────────────────┐ │
│  │ Application / Client │────▶│ Engineering Runtime        │ │
│  │ Host                 │     │ .NET                        │ │
│  └──────────────────────┘     └─────────────┬──────────────┘ │
│                                             │                 │
│                                             ▼                 │
│                               ┌────────────────────────────┐  │
│                               │ Mathematical Authority     │  │
│                               │ Rust kernel host/service   │  │
│                               └────────────────────────────┘  │
│                                                               │
│  ┌──────────────────────┐     ┌────────────────────────────┐ │
│  │ Persistence / Cache  │◀───▶│ Representation / Asset     │ │
│  │ Storage              │     │ Delivery                    │ │
│  └──────────────────────┘     └────────────────────────────┘ │
│                                                               │
│  ┌──────────────────────────────────────────────────────────┐ │
│  │ Integration / Provider Boundary                          │ │
│  │ simulation providers, machine adapters, PLM/PDM/ERP,    │ │
│  │ file adapters and other external-system implementations  │ │
│  └──────────────────────────────────────────────────────────┘ │
└───────────────────────────────────────────────────────────────┘
`

## 1. Application / Client Host

Owns interaction, commands, workflows, session coordination, and presentation orchestration.

It may call engineering services, but it does not become the source of engineering truth.

## 2. Engineering Runtime

The central .NET engineering container.

Its principal Components are:

`
Expressions
Science
Phenomena Simulation Service
CAD Engineering Core
Product Structure / BOM
References
Evaluation Engine
Engineering Resource Model
Engineering Domains
Knowledge
Configuration
Representation orchestration
`

The current project split is only an implementation projection of these Components.

## 3. Mathematical Authority

The Rust kernel host executes explicit mathematical contracts.

It may be accelerated by CPU/GPU implementations under the established mathematical-authority rules.

It must not import or depend on Sheet Metal, CAM, Drawing, BOM, Assembly, or other upper engineering semantics.

## 4. Persistence / Cache Storage

Stores durable project state, histories and deterministic cache entries as required.

Storage does not define semantic identity.

## 5. Representation / Asset Delivery

Produces/transports consumer-facing derived representations, such as render assets, drawing projection data, simulation meshes, and review representations.

Authoritative engineering results remain upstream.

## 6. Integration / Provider Boundary

Implements UMLCAD-owned contracts against concrete external technologies.

Examples:

`
SimulationProvider → external solver
MachineControllerProvider → machine/controller
PLMProvider → PLM/PDM system
ERPProvider → ERP
FileProvider → external file format
KernelClient → Rust authority
`

## Container dependency rules

`
Application Host
      ↓
Engineering Runtime
      ↓
Math Authority

Engineering Runtime
      ↔
Persistence / Cache

Engineering Runtime
      ↓
Representation / Asset Delivery

Engineering Runtime
      ↓
Integration / Provider Boundary
      ↓
External technology
`

The external provider never becomes the semantic owner of the calling domain.
