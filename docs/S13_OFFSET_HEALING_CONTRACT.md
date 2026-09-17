# UMLCAD V6 — S13 Offset and Healing Contract

S13 preserves the V6 authority order:

```text
semantic mathematics -> backend-neutral contract -> native realization -> independent conformance -> topology
```

## Offset semantics implemented first

The bounded first slice is:

- oriented planar line-segment offset;
- oriented planar rectangular surface-patch offset.

For a directed line, positive distance follows `plane_normal × unit_tangent`; negative distance reverses the side; zero distance is exact identity.

For a planar patch, offset translates the origin along the oriented surface normal while preserving unit parameter directions and dimensions.

Non-finite values, degenerate geometry, incompatible line/plane declarations, non-unit/non-orthogonal surface directions, and non-finite distances are rejected before native construction.

## Native realization

The OCCT backend realizes:

- planar line offsets as independent curve geometry;
- planar surface offsets as a degree-1 tensor-product NURBS surface with one face.

The source definition is immutable and the native result is a new shape.

For native bounding-box conformance, the semantic expected geometry remains authoritative. The current OCCT 7.6.3 reference path has a measured planar-surface envelope on the order of `1e-7`; the backend conformance test therefore uses an explicit `1e-6` measurement envelope without changing the semantic/modeling tolerance.

## Explicit S13 boundary

S13 does not yet claim arbitrary NURBS offsets, offset self-intersection resolution, corner joining, offset trimming, shell/thickness, or general imported-shape healing. These require separate mathematical and topological contracts and must not be approximated by sampling or by accepting a displayable OCCT result.

## Healing authority

Future healing must declare before execution:

1. permitted defect classes;
2. permitted geometric/topological changes;
3. maximum tolerance budget;
4. invariants that must remain unchanged;
5. evidence proving which repair actions occurred;
6. explicit failure/unsupported outcomes.

A generic backend `make valid` call is not a UMLCAD healing contract.

## Required gates

```text
RED contract
-> smallest repair
-> post-repair validation
-> independent invariants
-> adversarial pathology tests
-> deterministic evidence
-> native conformance
-> full CI
```
