# C4 Level 1 — UMLCAD V7 System Context

C4 Level 1 describes UMLCAD as a system in its environment. It does not prescribe the internal .NET project layout.

`
                         CAD / Engineering User
                                  │
                                  ▼
                         ┌───────────────────┐
                         │    UMLCAD V7      │
                         │ Engineering CAD   │
                         │ / CAM / Analysis  │
                         │ System            │
                         └─────┬───────┬─────┘
                               │       │
                 ┌─────────────┘       └──────────────┐
                 ▼                                     ▼
      External scientific/simulation         Manufacturing ecosystem
      applications / solvers                machines/controllers/tools
                 │                                     │
                 └────────────────┬────────────────────┘
                                  ▼
                         PLM / PDM / ERP /
                         external file systems
`

## External actors and systems

### Engineering User

Creates and modifies engineering specifications, reviews results, requests analysis/manufacturing operations, and consumes drawings and other representations.

### External scientific/simulation systems

Provide optional concrete implementations for phenomena simulation through UMLCAD-owned provider contracts.

### Manufacturing systems

Include machine/controller environments and external manufacturing equipment. UMLCAD owns machine/process/tool semantics where appropriate; concrete controller communication is an integration boundary.

### PLM/PDM/ERP and file systems

Are external systems integrated through adapters. They do not become semantic authorities merely because UMLCAD exchanges information with them.

## System-context laws

- Rust mathematical execution is internal to the UMLCAD system boundary; it is not the product.
- External solvers implement scientific contracts; they do not define UMLCAD's scientific semantics.
- External machines/controllers implement manufacturing integration; they do not define UMLCAD's machine model.
- External PLM/PDM/ERP systems consume or provide integration data through explicit contracts.
- The viewer/client is the user's presentation surface, not engineering authority.
