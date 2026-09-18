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

```text
Science
├── Material / physical properties
└── Phenomena Simulation Service
             ↑
             │ multiple providers
             │
Engineering Resources
├── Machine
├── Tool
├── Fixture
├── Process
└── Capabilities
             ↑
      ┌──────┴──────┐
      │             │
 Sheet Metal       CAM
      │             ├── Toolpath
      │             ├── Postprocessor
      │             └── deterministic G-code / NC
      │
      └── material/machine/process validation

CAD Product Structure
        ↓
       BOM
        ↓
Drawing / CAM / PLM
```

BOM is derived from authoritative product structure. CAM consumes authoritative CAD/product semantics, engineering resources, and applicable scientific/simulation services; CAM ultimately produces deterministic G-code/NC through its postprocessor boundary. Simulation is provider-neutral. The viewer is presentation-only.

## Mathematical authority

CPU mathematical behavior is normative for the repository's certified mathematical contracts. GPU execution is acceleration only. Do not duplicate mathematical authority in .NET or introduce alternative numerical semantics without an explicit contract and verification plan.

## Validation

Substantial changes follow:

```text
Inspect → Authoritative RED → Implement → GREEN
→ RED-TEAM → Python E2E → Full repository gates
→ Exact evidence
```

A test that did not execute is **not validated**, not PASS.

