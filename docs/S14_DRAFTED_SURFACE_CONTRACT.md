# UMLCAD V6 S14 — Bounded Rectangular Draft Construction Contract

## Purpose

This final S14 gate adds a bounded draft construction family with exact profile mathematics and explicit native conformance. The supported geometry is a rectangular prismatic body whose four side walls are drafted uniformly about the vertical axis.

## Semantic definition

`DraftedRectangularSolid` is defined by finite positive base width `a`, base depth `b`, positive height `h`, and a draft angle `θ` in radians.

The lower rectangular profile is centered at the origin at `z = 0`. The draft angle is measured from the vertical direction. Positive angle expands the upper profile; negative angle contracts it.

The lateral displacement per side is:

```text
δ = h tan(θ)
```

Therefore the exact upper dimensions are:

```text
A = a + 2δ
B = b + 2δ
```

The semantic domain requires finite dimensions, `h > 0`, and:

```text
|θ| < π/2
A > 0
B > 0
```

These restrictions are evaluated before native construction. They are not relaxed to accommodate a backend.

## Analytic volume

The exact lower and upper profile areas are:

```text
S0 = ab
S1 = AB
```

The exact frustum volume is:

```text
V = h (S0 + sqrt(S0 S1) + S1) / 3
```

At `θ = 0`, the upper profile equals the lower profile and the result reduces exactly to the rectangular prism `V = abh`.

The analytic equations are semantic authority. Native OCCT output is independent conformance evidence.

## Native realization

The reference backend realizes the definition by constructing the lower and upper rectangle profiles and invoking the existing backend-neutral `loft_between_polygons` operation.

The semantic layer therefore determines the exact profile geometry, while the native backend remains only a realization mechanism.

No automatic angle clamping, profile shrinking, healing, or renderer approximation is permitted.

## Evidence

A successful realization must demonstrate:

- `Solid` geometry kind;
- successful construction evidence;
- valid and manifold B-Rep;
- lower and upper profile dimensions consistent with the exact `h tan(θ)` displacement;
- finite ordered bounding envelope;
- deterministic topology and bounding-envelope measurements on repeated identical evaluation;
- exact prismatic-limit behavior at zero draft angle.

The contract does not prescribe backend-specific internal edge/vertex counts.

## Failure classification

The semantic layer rejects:

- non-finite dimensions or angle;
- non-positive base dimensions or height;
- `|θ| >= π/2`;
- any draft whose computed upper width or depth is at or below modeling tolerance;
- non-finite draft displacement.

Native loft failure remains a backend failure/unsupported result and is never silently repaired.

## Scope boundary

This gate does not claim:

- arbitrary face-selected drafting on existing B-Reps;
- multiple independent draft angles on different faces;
- curved/freeform drafted surfaces;
- automatic neutral-plane detection;
- variable draft along height;
- blended/drafted fillet intersections;
- imported topology healing.

These are later S15/S18 kernel work where applicable.
