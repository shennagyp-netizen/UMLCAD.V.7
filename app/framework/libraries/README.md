# Framework Libraries

The new Application Layer is intentionally decomposed into focused libraries.

| Library | Primary responsibility |
| --- | --- |
| UMLCAD.Kernel | Single concrete gateway to mathematical kernel authority |
| UMLCAD.Cad.Contracts | Stable CAD semantic contracts |
| UMLCAD.Cad.Expressions | Deterministic expression/value semantics |
| UMLCAD.Cad.Semantics | Authoritative CAD and product semantics |
| UMLCAD.Cad.Engine | Dependency planning and CAD evaluation orchestration |
| UMLCAD.Science | Scientific/material/phenomena semantics |
| UMLCAD.Engineering.Resources | Machines, tools, fixtures, processes, capabilities |
| UMLCAD.Engineering.SheetMetal | Sheet-metal semantics |
| UMLCAD.Engineering.Cam | Manufacturing semantics and NC generation |
| UMLCAD.Engineering.Drawing | Drawing and PMI semantics |
| UMLCAD.Integration.Simulation | External simulation integration boundary |
| UMLCAD.Framework | Application composition and cross-domain orchestration |

The table describes ownership, not inheritance. Dependencies are explicit and must remain acyclic.
