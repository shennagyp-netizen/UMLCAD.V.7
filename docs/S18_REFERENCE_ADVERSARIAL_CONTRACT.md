# V6 S18 — Reference Evolution Adversarial and Determinism Contract

## Purpose

This contract closes the S18 reference-evolution/migration regression gate for the bounded semantic vocabulary. It verifies boundary behavior and determinism without introducing topology-history inference or candidate selection.

## Required adversarial coverage

The gate covers:

- zero-source rejection;
- single-source disappearance (`1 → 0`) as explicit invalidation;
- multi-source disappearance (`many → 0`) as explicit rejection;
- all successful cardinality families;
- minimum nonzero counts;
- extreme `usize` cardinalities without arithmetic overflow;
- repeated identical evaluation;
- preservation of the source and target cardinalities in returned evidence;
- preservation of the rule that only `1 → 1` is automatic unique-identity-safe.

## Determinism

For identical semantic cardinality inputs, classification and migration evidence must be byte-for-byte equivalent as values. No geometry, tolerance, backend, native handle, storage order, renderer order, allocation order, or process state participates.

## Fail-closed behavior

No adversarial case may turn a missing or multi-target mapping into an inferred preserved identity. Unsupported cardinality cases remain explicit errors.

## Scope boundary

This gate does not add:

- geometric correspondence;
- topology-history reconstruction;
- candidate ranking;
- automatic reference repair;
- persistent storage;
- native-backend semantics.

It is a regression gate for the already-defined S18 semantic contracts only.
