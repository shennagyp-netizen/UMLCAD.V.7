# UMLCAD V6 S14 — Bounded Box All-Edge Fillet Contract

## Purpose

This gate extends S14 fillet/blend generation with one independently certifiable family: a rectangular box whose twelve straight edges receive the same positive circular fillet radius.

## Semantic definition

`BoxAllEdgesFillet` is defined by positive finite box dimensions `a`, `b`, `c` and a positive finite radius `r`.

The input box is the axis-aligned solid with extents:

```text
0 <= x <= a
0 <= y <= b
0 <= z <= c
```

All twelve box edges are filleted by the same radius `r`.

The first bounded family requires the strict geometric admissibility condition:

```text
0 < r < min(a,b,c) / 2
```

The strict bound is semantic. It prevents adjacent rounded features from collapsing into a zero-width planar region and avoids relying on backend healing to make an inadmissible definition successful.

## Analytic volume

The exact rounded-box volume is:

```text
V = abc
    - 2r(ab + ac + bc)
    + πr²(a + b + c)
    + (4π/3 - 8)r³
```

The expression is evaluated by the semantic layer. Native measurements are conformance evidence and do not define the result.

## Native realization

The reference backend realizes the semantic definition by constructing the exact source box and applying its native all-edge fillet operation. The backend operation is invoked only after semantic validation succeeds.

No automatic radius reduction, corner healing, edge skipping, or generic `make valid` operation is permitted to convert an invalid definition into a successful result.

## Topology evidence

The fillet must demonstrate a real topological change rather than merely report a successful backend call. The source box is a single solid/shell with 6 faces, 12 edges, and 8 vertices. A successful rounded result must remain a single solid/shell while increasing the face, edge, and vertex counts above those source values and satisfying the Euler characteristic for a closed connected orientable shell:

```text
V - E + F = 2
```

The contract deliberately does not prescribe exact edge/vertex counts for the native representation. Different valid B-Rep representations may split or merge topological entities without changing the represented geometry. The evidence requirement is therefore based on invariants, validity, manifoldness, and demonstrated topology change rather than backend-specific enumeration.

## Determinism and immutability

Repeated realization of the same immutable definition must produce identical topology counts and bounding-envelope measurements. The source box must remain unchanged.

The expected bounding envelope remains exactly the source box envelope within the validation tolerance:

```text
[0,a] × [0,b] × [0,c]
```

## Failure classification

The semantic layer rejects:

- non-finite dimensions or radius;
- any box dimension at or below modeling tolerance;
- radius at or below modeling tolerance;
- radius at or above half the minimum box dimension.

Native construction failure remains a backend failure/unsupported result. It is never silently converted to success.

## Scope boundary

This gate does not claim:

- selected-edge fillets;
- variable-radius fillets;
- edge-chain or tangent-chain propagation;
- face-set driven blends;
- rolling-ball blends on arbitrary B-Reps;
- fillets around imported/freeform topology;
- automatic self-intersection or corner repair;
- shell/thickness operations.

Those remain separate semantic contracts and green gates.
