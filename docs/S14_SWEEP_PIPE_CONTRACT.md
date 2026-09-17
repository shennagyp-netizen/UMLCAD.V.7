# UMLCAD V6 S14 — Sweep / Pipe Contract

## 1. Purpose

S14 extends the validated freeform construction layer with feature-generation operations. The first S14 slice is deliberately bounded to a circular profile swept along a straight path.

The authority order is:

```text
analytic sweep semantics
        ↓
backend-neutral contract
        ↓
native realization
        ↓
independent conformance
        ↓
topology / validity evidence
```

OCCT is a realization backend. It does not define the semantic result.

## 2. Supported first slice

`LinearCircularSweep` consists of:

- a finite circular profile center;
- a finite profile normal;
- a strictly positive radius above the UMLCAD modeling tolerance;
- a finite non-degenerate linear path;
- profile center coincident with path start;
- profile normal parallel or anti-parallel to the path tangent, so the circular profile plane is perpendicular to the path.

The exact geometric result is the Minkowski sweep of the circular disk along the line segment. For a straight path this is a right circular cylinder with:

```text
length = |path.end - path.start|
volume = π r² length
```

The profile normal selects the profile plane; reversing that normal does not change the geometry of an ideal circular profile.

## 3. Failure semantics

The semantic layer rejects:

- non-finite coordinates;
- non-finite radius;
- radius at or below modeling tolerance;
- a zero/degenerate path;
- a zero/degenerate profile normal;
- a profile normal that is not parallel or anti-parallel to the path tangent within validation tolerance;
- a profile center that is not coincident with path start within validation tolerance.

These are contract failures, not requests to relax tolerance.

## 4. Native realization

The OCCT reference backend realizes the bounded slice through an explicit native bridge. The native operation constructs an equivalent cylinder aligned to the requested path, preserving the exact straight-path sweep geometry while remaining behind the opaque backend boundary.

The backend validates the UMLCAD definition before entering the native bridge. The native normal check is defensive and mirrors the backend-neutral contract; it is not an independent semantic authority.

## 5. Required evidence

A successful realization must expose:

- geometry kind `Solid`;
- successful backend evidence;
- valid/manifold native validation;
- finite ordered bounding box;
- deterministic topology counts for repeated identical requests.

The source definition is immutable and repeated evaluation must not mutate it.

## 6. Scope boundary

This slice does not claim:

- arbitrary multi-segment or curved sweep paths;
- path curvature/continuity management;
- profile orientation transport along curved paths;
- variable-radius sweeps;
- guide-curve sweeps;
- self-intersection resolution;
- shell/thickness semantics;
- generalized pipe-surface topology healing.

Those require separate contracts and independent red-green gates.

## 7. Required TDD sequence

```text
semantic contract
→ failing tests
→ smallest correct implementation
→ native conformance
→ adversarial cases
→ determinism
→ integration
→ release gate
→ CI gate
```
