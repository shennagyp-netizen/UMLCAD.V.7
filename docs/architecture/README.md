# UMLCAD V7 Architecture Authority

This directory is the normative architectural companion to the system-CAD roadmap and implementation prompt.

The architecture is expressed in two complementary models:

1. Abstraction/dependency architecture: ownership, services, semantic domains, result flow, and allowed dependency directions.
2. C4 architecture: System Context, Containers, Components, and critical flows.

The mathematical M0–M16 roadmap remains authoritative for the Rust mathematical subsystem. This architecture does not expand that authority.

## Authority order

1. `docs/architecture/ABSTRACTION_AND_DEPENDENCY_MODEL.md`
2. `docs/architecture/architecture.json`
3. `docs/architecture/c4/`
4. semantic/domain contracts
5. concrete .NET project layout
6. implementation details

The project tree is an implementation projection of the architecture, not its definition.

## Core law

Lower abstractions provide general facts and reusable services. Higher abstractions provide engineering meaning and decisions.

A lower layer must not acquire semantic knowledge of an upper layer merely because the upper layer consumes its service.

A domain may consume another domain's published result/contract, but must not depend on the producer's private implementation.

## Verification

`tests/e2e/architecture_contract.py` is the executable architecture guard. It validates the machine-readable architecture manifest and the current repository projection.

The guard is deliberately a translation of the logical architecture into the current repository. It must never become the architecture definition itself.
