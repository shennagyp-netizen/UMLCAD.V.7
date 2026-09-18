# UMLCAD V7 — System CAD Testing Paradigms

## Purpose

This document makes testing a design constraint of system-CAD development. A capability is not complete because code exists or a local test passes. Each capability must be demonstrated across the applicable testing layers.

The governing cycle is:

```
RED
  ↓
smallest semantic implementation
  ↓
GREEN
  ↓
red-team attack
  ↓
metamorphic / property / differential checks where applicable
  ↓
full Python E2E
  ↓
full repository gates
  ↓
exact evidence
```

An infrastructure failure before test execution is recorded as **Not validated — infrastructure execution unavailable**. It is never converted into PASS or source-level FAIL.

## Testing pyramid and ownership

| Paradigm | Primary owner | Purpose | S1 requirement |
|---|---|---|---|
| Unit | Rust / .NET | Pure local invariants and failure semantics | Required |
| Contract | .NET / kernel API | Schema, status, identity, transport semantics | Required |
| Integration | .NET ↔ Rust | Adapter and service boundary correctness | Required |
| System E2E | `tests/e2e/` | Real multi-layer workflow | Required |
| Red-team / adversarial | `tests/e2e/` + component tests | Fail-closed behavior under hostile/invalid inputs | Required |
| Metamorphic | Rust / .NET / E2E | Invariance under semantics-preserving transformations | Required where an invariant exists |
| Property / generative | Rust / .NET / E2E | Broad input-space exploration without hand-picked cases only | Required for bounded generative domains |
| Differential / conformance | adapter/kernel tests | Compare independently implemented boundary behavior without creating a second authority | Required when an oracle exists |
| Architecture/static | architecture guard | Enforce dependency direction and authority boundaries | Required |
| Determinism/replay | Engine/E2E | Same semantic input produces same plan/result/identity | Required |
| Full vs incremental | Engine/E2E | Prove recomputation equivalence | Required |
| Fault injection / resilience | adapter/E2E | Transport failure, malformed response, timeout, cancellation, incomplete result | Required for external boundaries |
| Performance | repository gates | Detect unacceptable regression without changing semantic authority | Required where milestone gate exists |
| Hardware validation | hardware-specific workflow | Only for capabilities requiring hardware | Only when explicitly applicable |

## TDD law

For every new semantic capability:

1. Define the authoritative acceptance behavior.
2. Add the test before completing implementation.
3. Run RED and capture the reason.
4. Implement the smallest complete architecture-preserving path.
5. Run GREEN.
6. Add adversarial cases.
7. Run red-team GREEN.
8. Run complete E2E.
9. Run supporting tests and repository gates.
10. Record exact evidence.

A test added after implementation is supplementary unless it can demonstrably reproduce the pre-implementation defect.

## Red-team baseline

Every CAD capability must attack:

- malformed and non-finite inputs;
- unsupported feature types;
- missing, stale, ambiguous, and context-mismatched references;
- coordinate-frame inconsistencies;
- invalid or degenerate geometry;
- invalid tolerance values;
- conflicting constraints;
- cyclic dependencies;
- wrong upstream result identity;
- incomplete or contradictory kernel results;
- corrupted or stale cache entries;
- deterministic identity collisions;
- full/incremental divergence;
- topology correspondence ambiguity;
- incomplete derived representations;
- transport timeout/cancellation/invalid JSON/incorrect schema;
- forbidden architectural dependencies.

The expected behavior for unverifiable engineering state is explicit failure, not a guessed result.

## Metamorphic invariants

Metamorphic cases should be used where the transformation is semantically understood.

S1 examples:

- Reordering kernel topology records must not change semantic face-reference resolution.
- Reordering sketch circles/constraints must not change the canonical sketch meaning or evaluation identity when ordering is not semantic.
- Uniform translation must change world coordinates predictably while preserving intrinsic sketch geometry.
- A valid frame transformed into an equivalent rigid frame must preserve local geometric relationships.
- Full recompute and incremental recompute must produce equivalent semantic final results.
- Changing a tolerance or feature direction must change evaluation identity.
- Equivalent repeated evaluations must produce identical authoritative identities and representation identities.

## Property / generative testing

Use bounded deterministic generation for:

- finite frame bases;
- valid/invalid topology selector evidence;
- positive dimensions and tolerance boundaries;
- topology permutation;
- dependency graphs including generated acyclic graphs and injected cycles;
- cache identity input permutations;
- supported/unsupported status combinations;
- sketch entities within bounded coordinate/radius ranges.

Generated cases must use fixed seeds in authoritative CI when reproducibility is required.

## Differential / conformance testing

Differential tests compare a boundary against an independently defined expected behavior, not against another implementation that could silently become an authority.

Examples:

- raw Rust HTTP endpoint against the typed .NET adapter contract;
- typed contract result against canonical serialized schema;
- production adapter against an explicit hand-authored mathematical fixture;
- CPU reference against GPU acceleration when GPU is present.

Do not introduce OCCT, a second solver, or a second CAD kernel as an ungoverned semantic authority.

## Architecture testing

The executable architecture guard is a test, not documentation.

It must reject:

- lower-layer dependency on higher-layer implementation;
- peer-domain private implementation coupling;
- domain direct dependency on concrete Rust clients;
- viewer ownership of engineering truth;
- provider dependency on consumer applications;
- application/runtime composition being pushed downward into domain libraries.

## Determinism and replay

For every authoritative S1 result:

```
same specification
+ same references
+ same configuration
+ same tolerance
+ same contract version
+ same representation policy
        ⇒
same evaluation plan
+ same evaluation identities
+ same authoritative result identity
+ same representation identity
```

The test suite must exercise repeated runs and reordered non-semantic collections.

## Skip / execution population law

An authoritative .NET or Rust gate must report an executable test-result summary.

The E2E harness rejects:

- zero executed tests;
- nonzero failures;
- skipped tests in authoritative suites;
- missing summary output.

This prevents a green-looking command from hiding skipped or unexecuted tests.

## Milestone evidence

For each milestone increment record:

```
RED:
GREEN:
RED-TEAM:
METAMORPHIC:
PROPERTY/GENERATIVE:
DIFFERENTIAL/CONFORMANCE:
ARCHITECTURE:
DETERMINISM:
FULL-vs-INCREMENTAL:
E2E:
FULL REPOSITORY GATE:
INFRASTRUCTURE LIMITATIONS:
```

A milestone is GREEN only when every applicable gate has executed and passed.

For the current S1 vertical slice, the mandatory acceptance flow remains:

```
Create cube
 → select support face
 → create sketch
 → create circle A
 → create circle B
 → solve/validate
 → feature A in +face-normal direction
 → feature B in -face-normal direction
 → recompute
 → authoritative result
 → topology/reference provenance
 → derived representation
 → semantic viewer selection
```

No polygonal approximation of a circle is acceptable as authoritative geometry merely to satisfy this scenario.
