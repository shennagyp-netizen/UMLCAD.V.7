# UMLCAD V6 S15 — Topology Pathology Evidence Contract

## Purpose

This gate adds a read-only topology audit for imported or intermediate shell graphs. Its purpose is to expose explicit topology evidence before any repair decision is made.

The authority order remains:

```text
semantic topology invariants -> backend-neutral evidence -> native conformance -> controlled repair
```

The audit is intentionally geometry-independent. It never interprets spatial proximity, native validity, or renderer output as evidence of topological correctness.

## Evidence model

`topology::pathology::audit` reports:

- declared face count;
- distinct edge count;
- total edge-use count;
- boundary edges, where an edge has exactly one known or unknown use;
- non-manifold edges, where an edge has more than two uses;
- declared faces with no edge uses;
- edge uses that reference faces outside the declared face set;
- deterministic face connected components derived only from shared edge identity.

The report is immutable and does not modify the source graph.

## Interpretation

The predicates are deliberately narrow:

- `is_manifold` means no edge has more than two uses. It does not mean closed.
- `is_closed` requires no boundary edges, no non-manifold edges, no faces without uses, and no unknown-face uses.
- `is_connected` means exactly one face component.
- `is_closed_connected_manifold` combines those explicit predicates.

A boundary is therefore observable without being treated as a repairable defect automatically.

## Failure and repair boundary

This gate does not repair anything. It does not merge edges, close gaps, create missing faces, delete extra uses, alter orientation, or infer geometric coincidence.

A later controlled-repair gate may consume this evidence only after declaring:

1. the accepted pathology classes;
2. the exact allowed topological/geometric changes;
3. a tolerance budget, where applicable;
4. invariants that must remain unchanged;
5. deterministic repair evidence;
6. explicit unsupported/failure outcomes.

The topology audit itself is always observational.

## Imported-pathology use

The report is suitable as a first semantic boundary for imported data because it distinguishes several failure classes without silently repairing them. In particular, disconnected components remain visible rather than being interpreted as a single shell, and non-manifold edges remain explicit rather than being coerced into manifold topology.

This contract does not claim geometric defect detection, self-intersection detection, invalid surface parameterization detection, sewing tolerance selection, gap measurement, or arbitrary B-Rep healing.

## Determinism

For identical immutable graphs, the complete report, edge ordering, and connected-component ordering are deterministic. Repeated auditing must leave the source graph byte-for-byte equivalent at the Rust value level.

## Required gate

```text
RED pathological fixtures
-> deterministic read-only report
-> independent predicate checks
-> no source mutation
-> imported-pathology adversarial tests
-> full debug/release/format/Clippy CI
```

No repair is permitted in this station until the pathology evidence is explicit and testable.
