# V6 S15 — Controlled Face-Removal Repair Contract

## Purpose

This contract defines a bounded, backend-neutral topology repair for one explicitly identified face in a pathological `ShellOrientationGraph`.

The repair is permitted only when the caller names the face to remove and the complete post-state satisfies the required topology and orientation invariants.

The operation is **not** a generic healing operation and does not infer intent from geometry or from native-kernel behavior.

## Authority

The semantic contract is authoritative. A native backend may provide evidence of topology pathology, but a native ability to produce a result does not authorize this repair.

The repair must remain deterministic, immutable with respect to its input, and independent of rendering, transport, persistence, or native-kernel state.

## Explicit repair authorization

The caller supplies one exact `FaceId`.

The implementation does not:

- detect or infer duplicate geometry;
- choose a face automatically;
- compare geometric proximity;
- weld vertices;
- close gaps;
- widen tolerances;
- delete degenerated edges;
- merge or split edges or faces;
- invoke a generic native `MakeValid` or healing procedure.

The selected face is removed only as an explicit topology operation.

## Accepted input pathology

The input graph is audited before any change.

The bounded repair accepts only a narrow pathology class:

1. the target face exists;
2. the target face has at least one edge use;
3. the input has no boundary edges;
4. the input has no face records without edge uses;
5. the input has no edge uses referring to unknown faces;
6. at least one edge used by the target face has more than two edge uses;
7. every pre-existing non-manifold edge is affected by the target face.

Any unrelated pathology fails closed as `UnsupportedPathology`.

A repeated `(edge, face)` incidence remains evidence rather than an automatic defect classification. This repair does not reinterpret such evidence as a duplicate face.

## Transformation

The repair creates an immutable clone of the input graph, removes the explicitly selected face, and removes all edge uses whose face is the selected face.

No other face, edge identity, or edge use is edited.

The returned `repaired_edges` set is exactly the set of edge identities that were non-manifold before repair and incident to the removed face.

The returned `removed_edge_use_count` is the exact number of removed edge-use records.

## Required postconditions

After transformation, the result is audited again.

The repair succeeds only when all of the following hold:

- there are no boundary edges;
- there are no non-manifold edges;
- there are no faces without edge uses;
- there are no unknown face references;
- there is exactly one connected face component;
- the topology audit reports a closed connected manifold;
- deterministic orientation propagation succeeds from the smallest remaining `FaceId`;
- graph validation succeeds.

Failure of any invariant returns a deterministic error and does not mutate the source graph.

## Failure semantics

The current bounded API exposes these explicit outcomes:

- `UnknownFace` — the caller selected a face not present in the graph;
- `FaceWithoutEdge` — the selected face has no edge use;
- `UnsupportedPathology` — the input contains pathology outside this repair's contract;
- `NoMultipleIncidence` — the selected face does not participate in any non-manifold incidence;
- `IntroducesBoundary` — removing the selected face leaves one or more boundary edges;
- `RemainsNonManifold` — one or more non-manifold edges remain;
- `DisconnectedAfterRepair` — the resulting face topology is disconnected;
- `ContradictoryOrientation` — the resulting graph cannot satisfy consistent orientation propagation.

These failures are hard outcomes. The implementation does not silently fall back to another repair strategy.

## Determinism and immutability

For the same graph and the same explicit `FaceId`, the result or failure is deterministic.

The source graph is never mutated. The successful result contains a new graph value and an evidence snapshot of the audited pre-state and post-state.

The deterministic orientation seed is the numerically smallest remaining `FaceId`; this seed choice is part of the contract and is not client-configurable in this bounded slice.

## Scope boundary

This contract establishes one controlled repair primitive only: explicit removal of a caller-selected face when the resulting topology is provably within the closed connected orientable manifold contract.

It does not establish general imported-model healing, geometric duplicate recognition, tolerance management, shell sewing, vertex welding, or automatic repair planning. Those require separate evidence classes, invariants, deterministic change sets, and independently tested contracts.
