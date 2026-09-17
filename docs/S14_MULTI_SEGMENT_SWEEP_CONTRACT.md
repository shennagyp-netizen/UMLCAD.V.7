# UMLCAD V6 S14 — Two-Segment Circular Sweep Contract

## Purpose

This gate extends the linear circular sweep to a deliberately bounded piecewise-linear path with one genuine corner. It is not a claim of generalized curved-path pipe support.

## Semantic object

`TwoSegmentCircularSweep` consists of:

- a finite circular profile center `P0`;
- a finite profile normal parallel or anti-parallel to the first tangent;
- a finite radius strictly greater than modeling tolerance;
- one corner `P1`;
- one endpoint `P2`;
- non-degenerate segments `P0→P1` and `P1→P2`;
- a genuine path corner, meaning the two segment tangents are not parallel within validation tolerance.

For the circular profile, orientation transport is geometrically invariant under rotation of the disk about its center. The exact bounded sweep is therefore defined as the union of the two exact straight-segment circular sweeps sharing the profile disk at `P1`.

## Validation authority

The semantic layer rejects:

- non-finite coordinates/radius;
- radius at or below modeling tolerance;
- either degenerate segment;
- an initial profile normal not aligned with the first path tangent;
- a collinear two-segment path, because this gate exists to certify the first non-trivial path-corner family.

These are explicit contract failures. Tolerance is not silently widened to make an invalid path acceptable.

## Native realization

The reference OCCT backend realizes each segment through the already-certified linear circular sweep and combines the resulting native solids through the existing Boolean-fuse boundary. OCCT acceptance remains implementation evidence; semantic admissibility is owned by UMLCAD.

## Required evidence

A successful operation must provide:

- `Solid` geometry kind;
- valid/manifold validation;
- finite envelope containing both path segments;
- deterministic repeated topology and bounding-box measurements;
- unchanged input definition.

## Scope exclusions

This gate does not claim:

- arbitrary numbers of path segments;
- curved paths;
- guide curves;
- generalized orientation transport for non-circular profiles;
- automatic corner blending/miter/healing;
- self-intersection handling;
- variable-radius sweep;
- shell/thickness or generalized pipe surfaces.

Each requires a separate semantic contract and green gate.

## TDD sequence

```text
semantic mathematics
→ backend-neutral validation
→ native segment realizations
→ native combination
→ adversarial tests
→ deterministic tests
→ release gate
→ CI gate
```
