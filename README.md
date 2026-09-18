# UMLCAD.V.7

UMLCAD.V.7 is a system-CAD architecture in which the .NET layer owns CAD and engineering meaning while the Rust layer provides contracted mathematical authority.

## Development

The global development law for system-CAD work is:

- [Main-Branch Development Prompt](docs/UMLCAD_V7_MAIN_BRANCH_DEVELOPMENT_PROMPT.md)

The prompt defines architecture, authority boundaries, TDD, red-team/E2E validation, determinism, failure-closed behavior, provider boundaries, and documentation discipline. It intentionally does **not** contain a fixed comprehensive implementation roadmap.

The repository's current milestone, gap, handoff, architecture, and capability documents at the exact `main` head determine what should be implemented next.

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

