# UMLCAD V6 — S18 System Integration Completion Gate

## Purpose

S18 is the **final V6 kernel completion gate**. It is not satisfied by isolated crate tests, isolated geometry-family tests, or by the existence of implemented code alone.

The S18 exit condition is a **full-system confirmation of the integrated V6 kernel** on `main`.

## Authoritative completion rule

S18 may be declared COMPLETE only when all of the following are simultaneously true:

1. Every in-scope V6 kernel capability required by the declared product boundary is implemented or explicitly fail-closed as unsupported.
2. Every participating workspace crate is built and tested in debug and release configurations.
3. Every participating kernel crate is built and tested in debug and release configurations.
4. All unit, contract, adversarial, determinism, topology, freeform, native-conformance, exchange, tessellation, and integration tests applicable to the declared kernel surface are green.
5. A dedicated system-integration suite exercises the **cross-crate/cross-layer behavior** of the kernel rather than merely testing each crate independently.
6. The authoritative CI workflow executes that complete system-integration suite and is green on the exact `main` commit being accepted as the S18 completion candidate.
7. No code from a required V6 kernel slice exists only on a development branch; the accepted implementation is reachable from `main`.

## System integration meaning

The system-integration suite must validate composition across the complete kernel path, including where applicable:

```text
semantic geometry
    -> freeform/NURBS
    -> surface operations
    -> offsets / sweeps / blends / shells / drafts
    -> topology / B-Rep
    -> mesh / tessellation
    -> STEP / IGES exchange
    -> reference evolution / migration
    -> immutable kernel integration snapshots
    -> native OCCT realization and independent conformance
```

The suite must test representative end-to-end scenarios that cross subsystem boundaries, not just individual public functions.

## Authority and independence

OCCT remains a realization/conformance backend. It does not define semantic truth, topology identity, tolerance policy, candidate selection, or acceptance criteria.

System integration tests must preserve the established authority order:

```text
mathematical / semantic authority
        -> backend-neutral contract
        -> independent semantic tests
        -> native realization
        -> independent native conformance
        -> cross-system integration tests
        -> adversarial + determinism + release CI
```

## Mainline requirement

A green feature branch is not sufficient for S18 completion. The completion candidate must be the `main` commit itself, with the complete kernel and the complete system-integration suite reachable from that commit.

Historical S19A/S19B/S19C labels are S18.x implementation identifiers and their merged functionality is part of this final gate.

## Current repository observation

The existing CI workflow already runs the Rust workspace and the separate `kernel_rust` test gates, but it does not yet constitute proof of exhaustive cross-system integration by name or scope. The `kernel-integration-api` currently contains an S17 stress integration test; additional S18 system-integration coverage must exist and be included explicitly in the authoritative completion workflow before S18 can be declared complete.

## Final status rule

Until the complete system-integration suite is present, executed on `main`, and green, S18 remains **INCOMPLETE**, regardless of how many individual kernel slices have already passed their own gates.
