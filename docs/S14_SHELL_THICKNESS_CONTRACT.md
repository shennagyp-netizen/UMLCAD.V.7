# UMLCAD V6 S14 — Bounded Closed Box Thickness Contract

## Purpose

This gate introduces a bounded shell/thickness family without adding backend-specific shell semantics: a concentric inner box is removed from a rectangular outer box, leaving a closed, uniform wall-thickness enclosure.

## Semantic definition

`ClosedBoxThickness` is defined by finite positive outer dimensions `a`, `b`, `c` and uniform wall thickness `t`.

The outer solid is:

```text
0 <= x <= a
0 <= y <= b
0 <= z <= c
```

The inner cavity is:

```text
t <= x <= a-t
 t <= y <= b-t
 t <= z <= c-t
```

The first gate requires the strict admissibility condition:

```text
0 < t < min(a,b,c) / 2
```

This guarantees all three inner dimensions are strictly positive. The rule is semantic and is checked before native realization; no automatic thinning, healing, or tolerance widening may turn an invalid wall into a successful result.

## Analytic quantities

The inner dimensions are:

```text
(a-2t, b-2t, c-2t)
```

Outer volume:

```text
V_outer = abc
```

Cavity volume:

```text
V_inner = (a-2t)(b-2t)(c-2t)
```

Material volume:

```text
V_material = V_outer - V_inner
```

These equations are semantic authority. Native geometry measurements are conformance evidence only.

## Native realization

The reference backend constructs the validated outer box, constructs the validated inner box, translates the inner box by `(t,t,t)`, and applies the existing backend-neutral Boolean cut operation.

The resulting geometry is expected to be a valid solid containing an internal cavity. No backend-specific shell constructor is required for this first bounded family, and no generic `make valid` operation is permitted.

## Evidence

A successful realization must demonstrate:

- `Solid` geometry kind;
- successful construction evidence;
- valid and manifold B-Rep;
- preserved outer bounding envelope `[0,a] × [0,b] × [0,c]` within validation tolerance;
- topology richer than the original 6-face box, consistent with an internal cavity;
- deterministic repeated topology and bounding-envelope results;
- unchanged source outer box when the source is separately retained.

Exact native entity counts are not prescribed because valid B-Rep representations may split or merge topological entities differently while preserving the same geometry.

## Failure classification

The semantic layer rejects:

- non-finite dimensions or thickness;
- any outer dimension at or below modeling tolerance;
- thickness at or below modeling tolerance;
- thickness at or above half the minimum outer dimension.

Native Boolean failure remains a backend failure/unsupported result and is never silently repaired.

## Scope boundary

This gate does not claim:

- open shells with a removed face;
- arbitrary-face shelling of existing B-Reps;
- variable wall thickness;
- offsetting freeform/NURBS faces to form thickness;
- automatic corner closure or self-intersection repair;
- imported-pathology healing;
- shelling of filleted or chamfered arbitrary topology.

Those remain separate contracts and green gates.
