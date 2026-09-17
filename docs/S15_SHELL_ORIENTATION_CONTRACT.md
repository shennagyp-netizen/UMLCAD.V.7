# UMLCAD V6 S15 — Shell Orientation Propagation Contract

## Purpose

This first S15 gate establishes backend-neutral shell-topology semantics before native sewing is expanded. A shell is represented as a finite set of oriented face uses connected through shared edges.

## Semantic model

Each `FaceId` is connected to edge uses. An `EdgeUse` records:

- the shared `EdgeId`;
- the incident `FaceId`;
- the local traversal direction of that edge on the face boundary.

For the bounded manifold-shell family in this gate, every edge must have exactly two incident face uses and every face must participate in at least one edge use.

A connected shell must admit one globally consistent orientation assignment. Starting from a deterministic seed face, a face may be retained or flipped. Across every shared edge, the two effective edge traversals must be opposite.

Define the effective traversal direction of an edge use as `dir XOR flip`. The shared-edge requirement is:

```text
(dir(A) XOR flip(A)) XOR (dir(B) XOR flip(B)) = true
```

Equivalently, the propagation rule is:

```text
flip(B) = flip(A) XOR dir(A) XOR dir(B) XOR true
```

where `dir` records each local edge traversal direction.

## Propagation

Orientation is propagated by deterministic breadth-first traversal over the shared-edge graph. The seed face is assigned `flip = false`; neighboring assignments are derived from the shared-edge constraint.

A cycle that requires a face to have two different assignments is an explicit `ContradictoryOrientation` failure. A graph that passes local edge validation but cannot reach every face from the seed is an explicit `DisconnectedShell` failure.

## Failure classification

The semantic layer rejects:

- an empty shell;
- an unknown face reference;
- a face with no edge use;
- an edge incident to anything other than exactly two faces;
- contradictory orientation constraints;
- disconnected face components.

No tolerance, native healing, geometric guess, or orientation “fix” may turn any of these failures into success.

## Determinism

Face and edge identifiers are ordered semantic keys. Propagation therefore produces deterministic assignments for identical immutable input graphs.

## Scope boundary

This gate does not yet claim native OCCT sewing, arbitrary geometric gap closure, tolerance-based vertex merging, shell classification, cavity nesting, imported-pathology repair, or arbitrary non-manifold topology. Those are separate S15 gates.

The semantic orientation graph is intentionally independent of OCCT. Native sewing, when added, must conform to these invariants rather than define them.
