# UMLCAD V6 S14 — Linear Variable-Radius Sweep Contract

## Purpose

This gate adds the first variable-radius sweep family while retaining a closed mathematical form: a circular profile whose radius changes linearly along a straight path.

## Semantic definition

`LinearVariableRadiusSweep` consists of finite endpoints `P0`, `P1`, a finite profile normal, and positive radii `r0`, `r1` at the endpoints.

The profile normal must be parallel or anti-parallel to the path tangent. Both endpoint profiles are therefore circular sections perpendicular to the common path axis.

For a straight path, the exact result is a right circular cone when one radius is zero (outside this first gate) or a conical frustum when both radii are strictly positive. The current contract requires both radii to exceed modeling tolerance so the native result remains a proper solid with two circular boundary faces.

Let

```text
L = |P1 - P0|
```

Then the exact volume is

```text
V = π L (r0² + r0 r1 + r1²) / 3
```

Equal radii are the exact cylinder limit:

```text
r0 = r1 = r  =>  V = π r² L
```

This equality is semantic, not tolerance-based: only exactly equal endpoint radii select the cylinder limit. Distinct radii remain a frustum even when their numerical difference is small.

## Validation

The semantic layer rejects:

- non-finite endpoints, normal, or radii;
- either radius at or below modeling tolerance;
- a degenerate path;
- a zero/degenerate profile normal;
- a profile normal that is not parallel or anti-parallel to the path tangent within validation tolerance.

No tolerance widening or backend healing is permitted to turn these into successful results.

## Native realization

The OCCT reference backend realizes the mathematical family according to its exact special case:

- when `r0 == r1`, it uses the validated cylinder primitive;
- when `r0 != r1`, it uses the validated cone primitive with endpoint radii `r0` and `r1`.

Both cases then use the same deterministic axis alignment as the linear circular sweep. The equal-radius branch is not a backend workaround: it is the exact geometric cylinder limit of the conical-frustum formula and avoids asking a cone constructor to represent a constant-radius solid.

OCCT is only the realization backend. The analytic frustum/cylinder volume and admissibility rules remain UMLCAD semantic authority.

## Evidence

A successful realization must expose:

- `Solid` geometry kind;
- valid and manifold validation;
- finite ordered bounding box;
- agreement with the analytic volume formula;
- deterministic topology and measurements on repeated identical evaluation.

The source definition remains immutable.

## Scope boundary

This gate does not claim:

- zero-radius cone endpoints;
- curved or multi-segment variable-radius paths;
- arbitrary non-circular profiles;
- guide-curve sweeps;
- automatic corner blending;
- self-intersection repair;
- shell/thickness semantics.

Each remains a separate contract and green gate.
