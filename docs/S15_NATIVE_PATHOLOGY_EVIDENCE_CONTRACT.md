# UMLCAD V6 S15 — Native Pathology Evidence Contract

## Purpose

This gate defines a read-only native-reference evidence snapshot for B-Rep topology/pathology observations. It is diagnostic evidence only; it does not replace the backend-neutral topology authority and it does not authorize repair.

## Evidence snapshot

For an existing native shape the reference backend reports, as one deterministic observation:

- native `BRepCheck_Analyzer` validity;
- the narrow UMLCAD native manifold predicate used by the current reference backend;
- edges with exactly one face use (`free_edges`);
- edges with more than two face uses (`multiple_edges`);
- native degenerated-edge count from `BRep_Tool::Degenerated`;
- unique solid/shell/face/edge/vertex topology counts.

The snapshot is observational and must not mutate the shape.

## Interpretation boundary

`valid`, `manifold`, and the individual evidence counts are deliberately separate facts. In particular:

- a native-degenerated edge is not automatically an invalid shape;
- an open shell may be valid as a shell while exposing free boundary edges and therefore failing the current closed-solid manifold predicate;
- native validity does not create UMLCAD semantic topology identity;
- the native evidence does not infer geometric coincidence, merge vertices, close gaps, remove degenerate edges, or repair imported geometry.

The reference backend's manifold predicate is intentionally narrow: for a `SOLID`, every indexed edge must have exactly two face ancestors. Shapes outside that family are not promoted to `manifold = true` by inference.

## Controlled pathology fixture

The gate includes a deterministic open-box-shell fixture made from five of the six faces of an axis-aligned box. The fixture exists only to prove that the evidence path distinguishes an open boundary from the regular closed-box case. It is not a generic imported-file repair model.

Expected evidence for the fixture is:

```text
valid = true
manifold = false
free_edges = 4
multiple_edges = 0
degenerated_edges = 0
solids = 0
shells = 1
faces = 5
edges = 12
vertices = 8
```

## Output initialization and failure behavior

All output locations are initialized to neutral values before input validation. Null input therefore cannot leave stale caller-owned data behind.

Invalid arguments and null native shapes remain explicit failures. No fallback result is synthesized.

## Determinism and immutability

Repeated snapshots of the same immutable native shape must return identical evidence. The source shape is never modified by this contract.

## Boundary

This gate does not implement:

- automatic imported-file repair;
- geometric tolerance-based edge/vertex matching;
- gap closure or sewing driven by proximity;
- degenerate-edge deletion or replacement;
- self-intersection resolution;
- semantic topology ID generation from native traversal order.

Those require separate backend-neutral contracts and independent evidence.

## Required gate

```text
regular solid -> zero free/multiple evidence
periodic analytic solid -> degenerated evidence may be non-zero while valid
open shell -> free-boundary evidence is explicit
same snapshot -> deterministic
null/invalid input -> initialized outputs + explicit failure
full debug/release/format/Clippy CI
```
