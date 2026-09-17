# UMLCAD V6 — S18 Full-System Kernel Test Matrix

## Status

S18 is the final numbered V6 station. This document defines the evidence matrix required before S18 can be declared COMPLETE.

A green individual crate test is necessary but insufficient. The final candidate must exercise the supported kernel as one integrated system and the exact accepted commit must be on `main` with authoritative CI green.

## Test layers

| Layer | Required evidence |
| --- | --- |
| Semantic mathematics | Independent formulas, predicates, differential rules, domain rules, and deterministic classification |
| Backend-neutral contracts | Public API invariants, immutable results, invalid/unsupported behavior |
| Feature realization | Geometry, freeform, surface operations, offsets, sweeps, blends/fillets, shells, drafts |
| B-Rep topology | Connectivity, orientation, sewing, pathology/degeneracy evidence, controlled repair |
| Mesh/tessellation | Deterministic valid mesh generation and mesh-contract rejection cases |
| Exchange | STEP and IGES export/import, validation after import, supported round-trip evidence |
| References | Evolution cardinalities, migration classification, identity-safety rules |
| Integration state | Immutable snapshots, provenance, deterministic root transitions, no mutation of prior state |
| Cross-system scenarios | Multiple subsystems composed in one scenario rather than isolated calls |
| Adversarial/determinism | Boundary, degeneracy, repeatability, fail-closed behavior, scale-sensitive cases |
| Native conformance | OCCT realization compared with independently defined UMLCAD semantic expectations |
| Release gate | Debug + release tests, Clippy, format, exact accepted `main` commit |

## Required cross-system scenarios

1. **Primary solid path**: primitive solid → feature modification → topology validation → tessellation → STEP/IGES exchange → import → validation/topology evidence.
2. **Freeform path**: NURBS definition → differential semantics → native realization → supported curve/surface and point/surface operations → intersection evidence where the declared solver family applies.
3. **Feature-family path**: shell/thickness → draft → linear sweep → two-segment sweep → variable-radius sweep → planar offset, with explicit validation at every boundary.
4. **Topology/reference path**: topology snapshot → immutable operation/provenance → reference evolution → migration classification → deterministic identity.
5. **Failure path**: malformed or unsupported inputs must fail explicitly without creating an invalid semantic result or mutating the preceding snapshot.
6. **Repeat path**: representative end-to-end scenarios repeated many times must produce identical semantic evidence and stable integration identities.

## Completion rule

S18 cannot be marked COMPLETE while any required scenario, applicable contract, or authoritative CI gate is absent, untested, failing, or only available on a non-main branch.

The declared V6 product boundary determines which cases are applicable. Unsupported cases remain explicit and fail closed; they are not silently replaced by rendering approximations or native-kernel guesses.

## Mainline proof

The final completion evidence must record:

- accepted `main` commit SHA;
- authoritative CI run ID for that exact SHA;
- complete workspace debug/release results;
- complete kernel debug/release results;
- dedicated S18 system-integration debug/release results;
- Clippy/format results;
- exact cross-system scenarios exercised;
- any explicitly unsupported capability families remaining outside the declared boundary.
