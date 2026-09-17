# V6 S16 — Topology Snapshot Identity Contract

## Purpose

This contract defines a deterministic semantic identity for a `ShellOrientationGraph` snapshot.

The identity is a SHA-256 digest computed entirely from the backend-neutral topology representation. It is suitable as a stable value for provenance, immutable snapshot comparison, cache keys, and later operation-graph integration.

## Semantic authority

The snapshot identity is defined from the UMLCAD semantic graph, not from an OCCT handle, pointer, allocation address, native shape identity, renderer state, filesystem location, or process-local state.

A native backend may reproduce the same topology, but native identity is never used as the semantic snapshot identifier.

## Canonicalization

The canonical digest input contains:

1. a fixed algorithm/domain tag;
2. the ordered `FaceId` set;
3. the number of edge-use records;
4. all edge-use records sorted by `(EdgeId, FaceId, forward)`.

The `BTreeSet` ordering of faces is already canonical. Edge-use storage order is not semantic and therefore does not affect the digest.

Duplicate edge-use records are retained in the canonical input. They remain identity-relevant even though repeated `(edge, face)` incidence is only diagnostic evidence in the current topology model.

## Identity properties

For a given semantic graph:

- repeated computation is deterministic;
- equivalent graphs differing only in edge-use insertion order have the same identity;
- any represented topology change changes the digest with cryptographic-hash collision resistance as the identity mechanism;
- the value is immutable and contains no mutable native state;
- the digest is exactly 32 bytes and has a deterministic 64-character lowercase hexadecimal representation.

## Immutability

`ShellOrientationGraph::snapshot_id()` takes `&self` and performs its canonicalization on a temporary copy of edge uses. The source graph is never reordered or otherwise modified.

## Scope boundary

This identity is a topology-snapshot identity only. It does not claim to identify geometry, an OCCT shape, an operation history, a document, a serialized file, or a complete part model.

Geometry-aware and operation-history identities require separate contracts that define their semantic inputs and canonical forms explicitly.

## S16 continuation

Later S16 integration may compose this identity into deterministic operation provenance and immutable kernel snapshots. Such composition must preserve the same authority order and must not make native backend identity semantically authoritative.
