# UMLCAD.V.7

UMLCAD.V.7 is a system-CAD architecture in which the .NET layer owns CAD and engineering meaning while the Rust layer provides contracted mathematical authority.

## Development

The governing System-CAD documents are:

- [System-CAD Architecture](docs/SYSTEM_CAD_ARCHITECTURE.md) — active authority for ownership, C4 structure, canonical evaluation flow, domain boundaries, and architectural gaps.
- [Main-Branch Development Prompt](docs/UMLCAD_V7_MAIN_BRANCH_DEVELOPMENT_PROMPT.md) — development and validation law.
- [Continuation Handoff](docs/CONTINUATION_HANDOFF.md) — current implementation/evidence state.
- [Mathematical Authority Roadmap](docs/MATH_AUTHORITY_ROADMAP.md) — Rust mathematical authority and certified domains.
- [Architecture Handbook](docs/doc.tex) — published comprehensive reference; it must remain consistent with the active architecture document.

The architecture document defines **how the system is structured**. The handoff/gap state defines **what is implemented and what should be done next**. The prompt defines **how changes are developed and validated**.

## Architectural invariants

The architecture distinguishes conceptual authority strata from the actual dependency graph.

```
Authority strata
----------------
Platform Foundation
Mathematical Authority
Science / Phenomena Services
CAD / Product Semantics
Engineering Resources
Specialized Engineering Domains
Application / Workflow / Presentation
```

Actual consumption is contract-driven:

```
CAD Core → Mathematical Contracts
CAM → CAD / Product Structure / BOM
CAM → Engineering Resources / Science / Simulation
Sheet Metal → CAD / Science / Engineering Resources
Drawing / PMI → CAD / Product Structure
Kinematics → Product / Assembly / Mathematical Contracts
Lifecycle / PLM → CAD / Product / Configuration
```

```
CAD Product Structure → BOM → Drawing / CAM / PLM
Simulation Provider → Phenomena Simulation Service
```

BOM is derived from authoritative Product Structure. CAM consumes authoritative CAD/product semantics, engineering resources, and applicable scientific/simulation services; CAM ultimately produces deterministic G-code/NC through its postprocessor boundary. Simulation is provider-neutral. The viewer is presentation-only.

## Mathematical authority

CPU mathematical behavior is normative for the repository's certified mathematical contracts. GPU execution is acceleration only. Do not duplicate mathematical authority in .NET or introduce alternative numerical semantics without an explicit contract and verification plan.

## Application Layer and kernel boundary

All new production .NET framework libraries under `app/framework/libraries/` are the UMLCAD Application Layer. The previous `dotnet/` implementation is legacy and remains untouched. The mathematical authority remains outside that layer under `kernel/`. Exactly one dedicated .NET library, `app/framework/libraries/UMLCAD.Kernel`, owns kernel-access implementation details. Other Application Layer libraries consume its concrete API and must not know Rust, native transport, or kernel-host details.

The executable architecture gate is the Python project [`app_e2e`](app_e2e/README.md). It checks the real new `app/` project graph, rejects circular dependencies, verifies framework/test/application placement, and rejects kernel transport/native implementation leakage outside the dedicated gateway.

## Validation

Substantial changes follow:

```text
Inspect → Authoritative RED → Implement → GREEN
→ RED-TEAM → Python E2E → Full repository gates
→ Exact evidence
```

A test that did not execute is **not validated**, not PASS.

