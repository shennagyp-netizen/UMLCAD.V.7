# UMLCAD V6 — S16 Immutable Integration Operation Transition Contract

## Purpose

`IntegrationSnapshot::apply_operation` defines the bounded immutable transition from one in-memory integration snapshot to the next after a semantic operation has produced a topology snapshot identity.

## Preconditions

The supplied `OperationDescriptor` must satisfy the existing `OperationProvenance` validation rules: positive strictly increasing operation identifier, non-empty ordered inputs, and non-zero output identity.

Every operation input must be an active root of the current integration snapshot. Each input identity must occur at most once in the operation input list.

The output identity may equal one of the consumed inputs, which permits an explicit semantic identity-preserving operation. Otherwise, an output already active as an unconsumed root is rejected rather than silently deduplicated.

## Transition

For a successful operation:

1. validate the descriptor without mutating the source;
2. verify all ordered inputs are active roots and unique;
3. reject an already-active unconsumed output;
4. append the operation to a new immutable provenance value;
5. remove the consumed input roots while preserving the relative order of all remaining roots;
6. append the operation output as the new active root;
7. return the new immutable `IntegrationSnapshot`.

The source snapshot is never mutated. There is no partial state: a failed validation returns an error and the source remains unchanged.

## Ordering

Root ordering is semantic. Consumption removes matching roots from their existing positions; unrelated roots retain their relative order. The new output is appended at the end.

Input ordering remains part of the operation descriptor and therefore remains part of provenance identity.

## Failure modes

The bounded transition reports structured failures for:

- invalid operation provenance data;
- an input identity that is not an active root;
- repeated input identity within one operation;
- output identity already active outside the consumed input set.

No automatic root deduplication, input inference, operation renumbering, or hidden state mutation is permitted.

## Scope boundary

This transition does not execute geometry, invoke OCCT, perform persistence, define transactions or rollback, reconcile distributed branches, authorize operations, or define assembly semantics. It records an already-described semantic operation result into an immutable in-memory integration state.

## Required conformance

Tests must prove successful multi-input replacement, preservation of unrelated root ordering, source immutability, provenance append, and all fail-closed preconditions above.
