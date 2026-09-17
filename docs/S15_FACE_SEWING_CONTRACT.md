# UMLCAD V6 S15 — Boundary Sewing Contract

## Purpose

This gate defines the semantic topology operation used to sew two previously open shell boundaries. It deliberately separates topological coincidence from geometric coincidence.

The topology layer may only consume an explicit declaration that two boundary edges are the same semantic edge. A backend-specific geometric algorithm must establish that declaration from geometry; topology must not infer it from proximity alone.

## Input model

`BoundarySewingGraph` contains a finite non-empty set of faces and edge uses.

An edge may have:

- exactly one use: an open boundary edge;
- exactly two uses: an already connected manifold edge.

An edge with more than two uses is non-manifold and is rejected. Every face must participate in at least one edge use, and every referenced face must exist in the face set.

## Sewing operation

`sew_boundary_edges` accepts explicit pairs `(left_edge, right_edge)`.

For every pair:

1. both edge IDs must exist;
2. both must currently be boundary edges with exactly one use;
3. the two IDs must differ;
4. neither edge may be reused by another pair;
5. the right edge is deterministically replaced by the left edge in its edge uses.

The operation is immutable: the input graph is not modified.

After all replacements, the result must satisfy the closed-shell topology contract: every edge has exactly two uses. Otherwise the operation fails with `ResultNotClosed` rather than inventing missing topology.

## Orientation

Once sewing produces a closed shell, the existing `ShellOrientationGraph::propagate` authority is applied unchanged. Shared edges must have opposite effective traversal directions under the assigned face flips.

`sew_and_orient` therefore composes:

```text
validated boundary pairing
        ↓
deterministic edge identity merge
        ↓
closed-shell validation
        ↓
deterministic orientation propagation
```

## Geometry boundary

This contract does **not** claim that two paired edges are geometrically coincident. No tolerance-based vertex merge, gap closure, projection, snapping, healing, or proximity guess is performed here.

The native OCCT sewing gate must establish geometric admissibility separately and then prove that its resulting topology conforms to these semantic invariants. OCCT's `BRepBuilderAPI_Sewing` is therefore a realization mechanism, not the semantic authority.

## Failure classification

The semantic layer explicitly rejects:

- empty input;
- unknown face references;
- faces without edge uses;
- non-manifold edges;
- missing boundary edges;
- attempts to sew an already internal edge;
- attempts to sew an edge to itself;
- reuse of a boundary edge in multiple sewing pairs;
- incomplete closure after the declared sewing pairs;
- orientation contradictions after closure.

All failures are explicit. No native repair or tolerance widening may convert a semantic failure into success.

## Determinism and immutability

For identical immutable input graphs and identical ordered sewing pairs, the produced topology and orientation assignments are deterministic. The source graph remains unchanged.

## Scope boundary

This gate does not yet implement:

- automatic geometric edge matching;
- arbitrary face-to-face sewing driven from native geometry;
- gap healing;
- degenerate-edge repair;
- non-manifold sewing;
- cavity containment/classification;
- imported-pathology repair.

Those capabilities remain separate S15 gates.
