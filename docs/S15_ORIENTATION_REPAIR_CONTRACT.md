# UMLCAD V6 S15 — Orientation-Only Repair Contract

## Purpose

This bounded repair slice converts a validated closed manifold shell graph with a consistent orientation solution into a canonically oriented graph by changing only face-use traversal directions.

The authority order is:

```text
validated topology -> orientation constraint solving -> declared repair -> post-repair evidence
```

The repair consumes the existing `ShellOrientationGraph::propagate` semantics. It never uses native OCCT validity or geometry as permission to alter topology.

## Permitted change

The repair may change only:

- `EdgeUse.forward` for every use belonging to a face whose propagated `face_flip` is `true`.

The repair must preserve exactly:

- the face-ID set;
- the edge-ID set;
- the number of edge uses;
- every `(edge, face)` incidence pair;
- the source graph value.

No face or edge is created, deleted, merged, split, projected, snapped, or geometrically moved.

## Preconditions

`repair_orientations(graph, seed)` first invokes the existing orientation propagation contract. Therefore the operation fails closed for:

- empty shells;
- unknown faces;
- faces without edge uses;
- non-manifold edges;
- disconnected face graphs;
- contradictory orientation cycles.

A contradictory cycle is not repaired by arbitrary local choices.

## Postconditions

The returned graph must remain a valid closed two-use-per-edge shell graph. Applying the existing orientation propagation to the returned graph with the same seed must succeed without contradiction.

`changed_faces` reports the exact number of face orientations inverted by the repair. A graph already satisfying the propagation constraints returns an unchanged value-equivalent graph and `changed_faces = 0`.

## Evidence and determinism

The repair result carries both the repaired graph and the complete propagated face-flip map. For identical input graphs and seed faces the result is deterministic.

The operation is immutable: the caller's input graph is never modified.

## Boundary

This is not general B-Rep healing. It does not:

- change geometric surfaces or curves;
- alter edge geometry or vertex positions;
- merge or split topology;
- close open boundaries;
- remove non-manifold uses;
- repair degenerate geometry;
- infer geometric coincidence;
- invoke arbitrary native healing.

Those changes require separate pathology classes and explicit contracts.

## Required gate

```text
RED orientation-damage fixtures
-> deterministic orientation solve
-> orientation-only repair
-> exact incidence preservation
-> post-repair propagation
-> contradictory-cycle rejection
-> source immutability
-> full debug/release/format/Clippy CI
```
