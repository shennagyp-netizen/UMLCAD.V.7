# UMLCAD V6 — S16 Immutable Integration Snapshot Contract

## Purpose

`IntegrationSnapshot` is the bounded in-memory state value that groups an ordered set of active semantic topology roots with the immutable operation provenance that produced the current integration state.

## Semantics

An integration snapshot contains exactly:

- an `OperationProvenance` value;
- an ordered list of `TopologySnapshotId` roots.

Root order is semantic and therefore contributes to snapshot identity. The API does not sort, deduplicate, infer, or otherwise normalize roots.

An empty root list is valid because an empty integration state is a valid in-memory value; no synthetic topology identity is created for it.

Every supplied root identity must be non-zero. Invalid identities are rejected before the snapshot value is created.

## Immutability

Construction and `with_roots` return new values. Existing snapshots and their provenance remain unchanged. Referenced topology identities are value types and no topology graph is owned or mutated by the snapshot.

## Identity

`IntegrationSnapshot::identity()` is a deterministic SHA-256 value derived from:

1. a fixed domain tag;
2. the complete provenance identity;
3. the root count;
4. the ordered root identities.

No native backend handle, allocation address, clock value, process identifier, filesystem path, or transport identifier contributes to identity.

## Scope boundary

This snapshot is deliberately not:

- a persisted document format;
- a transaction or rollback mechanism;
- a distributed synchronization primitive;
- an authorization object;
- a geometry snapshot beyond the topology identities it contains;
- an OCCT/native state container;
- an assembly or multi-part model format.

Branch reconciliation, persistence, transport/service protocols, and full geometric/part identity remain separate contracts.

## Required conformance tests

An implementation must prove deterministic identity, order sensitivity, zero-root rejection, immutability of prior values, provenance preservation when roots change, and absence of native backend dependencies.
