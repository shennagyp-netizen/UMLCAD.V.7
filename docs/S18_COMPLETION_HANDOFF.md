# UMLCAD V6 — S18 Completion Handoff

## Station status

S18 is the **final numbered station** and the **full V6 kernel/system completion gate**.

The original S18 bounded reference-evolution/migration contract is complete and remains authoritative. It established deterministic, immutable, backend-neutral reference evolution and migration semantics with explicit adversarial coverage.

S18 itself is not considered fully complete merely because that bounded reference contract passed, nor because individual geometry/kernel crates have passed their own tests. The same S18 completion gate owns the remaining integrated V6 kernel implementation and the complete cross-system validation required by the declared product boundary.

## Completed S18 reference contract

The S18 reference-evolution vocabulary is:

- `OneToOne`
- `OneToZero`
- `OneToMany`
- `ManyToOne`
- `ManyToMany`

Zero-source mappings are rejected. A single source disappearing is represented explicitly as `OneToZero`; multiple source matches disappearing is rejected as `MultipleTargetsLost` rather than being converted into an inferred invalidation.

The migration vocabulary is:

- `Preserved` — only `OneToOne`
- `Invalidated` — `OneToZero`
- `Split` — `OneToMany`
- `Merged` — `ManyToOne`
- `Ambiguous` — `ManyToMany`

Only `Preserved` is eligible for automatic unique-identity continuation.

## Reference adversarial and determinism gate

The completed reference gate covers:

1. zero-source rejection;
2. single-source disappearance;
3. multi-source disappearance;
4. all successful cardinality families;
5. minimum non-zero cardinalities;
6. `usize::MAX` cardinalities without arithmetic overflow;
7. repeated identical evaluation;
8. preservation of authoritative before/after cardinalities in evidence;
9. exclusion of automatic identity safety from all non-`OneToOne` outcomes.

The gate does not use geometry, renderer state, native handles, allocation order, storage order, tolerance heuristics, process state, or modeling-history inference.

## S18.x implementation rule

Future V6 kernel work is performed as **S18.x implementation slices**. S19 and later are not new stations.

The already-merged S19A/S19B/S19C changes are historical implementation identifiers developed after the bounded S18 reference gate. They are treated as part of the ongoing S18 completion work and do not change the station map.

Every S18.x slice must preserve:

```text
semantic mathematics
    -> backend-neutral contract
    -> independent semantic tests
    -> native realization/conformance
    -> cross-system integration
    -> adversarial + determinism + release CI
```

## Final-station completion meaning

The final S18 gate closes only when the applicable V6 kernel capabilities are implemented **as one integrated system** and the full authoritative validation matrix is green on the exact `main` commit being accepted.

This includes, as applicable, geometry, B-Rep, freeform, intersection, sweep/pipe, offset/healing, tessellation, exchange, topology, reference evolution/migration, and kernel integration, together with their cross-crate and cross-layer interactions.

A dedicated system-integration suite must exercise representative end-to-end compositions rather than relying only on independent crate tests. The gate must include all applicable unit, contract, adversarial, determinism, native-conformance, release, workspace, kernel, and system-integration tests.

S18 cannot be declared complete while required system-integration coverage is absent, excluded from CI, or green only on a non-main branch.

This is not a claim that every theoretically possible industrial CAD operation exists. Unsupported cases remain explicit and fail closed. Assemblies, kinematics, drawings, FEA, and machine-design application behavior remain outside the V6 part-geometry-kernel boundary unless explicitly brought into the V6 scope by a separate roadmap revision.

## Current validation state

The current CI workflow runs the Rust workspace tests, the separate `kernel_rust` tests, formatting, and Clippy gates. The workspace contains the V6 kernel crates, including `kernel-integration-api`. The integration crate currently exposes an S17 stress integration test. These existing gates are necessary but are not, by themselves, proof of the final S18 full-system integration requirement.

Therefore **S18 remains INCOMPLETE until the complete S18 system-integration matrix is implemented, explicitly executed by authoritative CI, and passes on `main`.**

## Exit evidence for the reference portion

- PR #86: explicit topology reference evolution semantics.
- PR #88: immutable reference evolution evidence.
- PR #89: explicit reference migration classification.
- PR #90: adversarial and determinism regression gate.
- Authoritative V6 OCCT TDD run #733 passed after correcting the adversarial expectation for many-to-zero disappearance.

The reference portion is therefore closed; the final S18 gate remains open until full integrated-system implementation and validation are green.
