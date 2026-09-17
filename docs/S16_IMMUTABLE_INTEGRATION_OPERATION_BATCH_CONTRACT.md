# UMLCAD V6 — S16 Immutable Integration Operation Batch Contract

## Purpose

`IntegrationSnapshot::apply_operations` defines immutable functional composition of an ordered sequence of already-described semantic operations.

## Semantics

The batch is processed in the supplied order using the existing single-operation transition rules. Each successful intermediate value is immutable and remains private to the transition; only the final value is returned.

An empty operation sequence is valid and returns a value equal to the source snapshot.

Every operation must satisfy the existing operation provenance and active-root transition preconditions at the point where it is applied. Therefore later operations may consume roots produced by earlier operations in the same batch.

## Failure behavior

If any operation fails, the batch returns that failure and the original source snapshot remains unchanged. No partially updated snapshot escapes the method.

This property comes from immutable value composition. It must not be interpreted as transaction, rollback, or persistence semantics.

## Ordering

Operation ordering is semantic. Reordering a valid dependent sequence can change its validity or resulting snapshot. Root ordering follows the existing single-operation transition contract.

## Scope boundary

This batch contract does not execute geometry, invoke OCCT, define persistence, transactions, rollback, distributed synchronization, authorization, transport/service protocols, or assembly semantics.

## Required conformance

Tests must prove:

1. an empty batch returns the source value unchanged;
2. a multi-operation batch equals explicit sequential composition;
3. operations may consume outputs produced earlier in the same batch;
4. a later failure leaves the original source unchanged;
5. no partial intermediate value is observable through the public API.
