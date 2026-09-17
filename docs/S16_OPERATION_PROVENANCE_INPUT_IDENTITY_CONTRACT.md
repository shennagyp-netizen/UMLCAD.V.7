# UMLCAD V6 — S16 Operation Provenance Input Identity Contract

## Purpose

Operation provenance stores topology snapshot identities as semantic references. A zero-valued `TopologySnapshotId` is reserved as an invalid/uninitialized value and must never enter an operation descriptor.

## Rule

Every operation input identity must be non-zero, just as the operation output identity must be non-zero.

For an input at zero-based position `index`, provenance construction rejects the descriptor with `ProvenanceError::ZeroInput(index)` before the descriptor can become part of provenance history.

## Determinism and immutability

The validation is value-based and deterministic. It does not inspect native backend state, allocation state, timestamps, transport state, or topology graph contents. Failed validation does not mutate the source provenance value.

## Scope boundary

This rule validates the identity value only. It does not prove that the referenced topology snapshot exists in storage, is active in an integration snapshot, or is geometrically valid; those concerns remain separate contracts.

## Required conformance

Tests must prove rejection of zero input identities at the first zero-valued input position and preservation of the original provenance value after rejection.
