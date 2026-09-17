# UMLCAD V6 S15 — Topological Incidence Evidence Contract

## Purpose

This gate identifies one purely topological evidence pattern that can be proven from `ShellOrientationGraph` alone:

- `RepeatedEdgeFaceIncidence`: the same `(edge, face)` pair occurs more than once in the graph representation.

A repeated pair is evidence about the representation, not proof of an invalid seam. Because `EdgeUse` does not carry a distinct use identity, the semantic layer cannot distinguish a duplicated record from two legitimate uses of the same edge by one periodic/seam face. Therefore this condition is reported but is **not** by itself a degeneracy classification and does not make a graph non-manifold.

## Authority

The audit remains geometry-independent:

```text
topology graph -> explicit incidence evidence -> declared interpretation/repair policy
```

The evidence map is diagnostic only and must not be consumed as repair permission without a separate contract.

It does not use distances, native validity, mesh appearance, or surface evaluation to infer the meaning of the repeated incidence.

## Postconditions

For identical immutable input graphs, the incidence report is deterministic. The source graph is unchanged.

`is_manifold()` remains governed by the edge-use count rule already established by the shell topology contract. Repeated edge-face incidence is observable evidence but is not sufficient to invalidate manifold topology without additional use-identity or geometric evidence.

## Boundary

This gate does not claim detection of:

- zero-length geometric edges;
- zero-area faces;
- coincident but separately identified vertices or edges;
- collapsed parameter domains;
- geometric self-intersections;
- invalid surface parameterizations;
- native OCCT degeneracy classes.

Those require geometric/native evidence and separate contracts.

No repair is performed by the audit. A future repair gate must explicitly state which evidence classes it can change and what incidence/geometry invariants remain mandatory.

## Required gate

```text
RED repeated-incidence fixtures
-> deterministic evidence
-> explicit seam/same-face non-claim
-> unchanged source
-> no geometric inference
-> full debug/release/format/Clippy CI
```
