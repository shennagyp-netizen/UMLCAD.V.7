# V6 S19A — Bilinear Freeform / Planar Surface Intersection Contract

## Purpose

S19A adds one bounded non-planar surface-intersection family to the existing conservative surface-operations front end:

- one unweighted degree-1 × degree-1 NURBS patch (bilinear surface), and
- one unweighted degree-1 × degree-1 planar NURBS patch.

The result is a finite collection of curve segments represented by deterministic endpoints. The slice does not claim unrestricted NURBS/NURBS intersection tracing.

## Mathematical authority

For normalized parameters `(s,t) ∈ [0,1]^2`, an unweighted bilinear patch is

`P(s,t) = P00 + s A + t B + s t C`.

For a plane with origin `O` and normal `N`, the signed plane equation evaluated on the patch is

`F(s,t) = a + b s + c t + d s t`.

The intersection is therefore the zero set of a bilinear polynomial. Boundary roots are solved analytically; each accepted endpoint is evaluated directly from the patch definition and mapped back to its native parameter domain.

## Certified family

The implementation accepts only:

- valid, finite NURBS definitions;
- degree `(1,1)` and control grid `(2,2)`;
- unit positive weights;
- non-degenerate planar patch;
- non-degenerate bilinear patch Jacobian where the trace is evaluated.

Coplanar / identically-zero equations are classified as underdetermined. Singular or non-representable point contacts are not converted into curve segments.

## Result semantics

- `NoIntersection`: no analytically identified boundary roots exist in the certified box.
- `Unique`: exactly one bounded curve segment is represented by two distinct endpoints.
- `Ambiguous`: the zero set has more than one bounded branch or only an isolated contact that the segment-only result type cannot represent.

No endpoint candidate is selected by geometric proximity, renderer behavior, native identity, or heuristic scoring.

## Determinism requirements

For identical immutable inputs and tolerance, the operation must return byte-for-byte equivalent floating-point fields under the same Rust execution semantics, including deterministic endpoint ordering.

## Native conformance boundary

OCCT remains a realization/conformance backend only. The semantic intersection equation and endpoint construction are owned by UMLCAD. Native conformance must compare the semantic result against an independently constructed OCCT section result; OCCT output must not define semantic topology or endpoint identity.

## Explicit exclusions

This slice does not certify:

- arbitrary rational NURBS surface/surface intersection;
- higher-degree surface intersection;
- unrestricted multi-branch tracing;
- tangential/contact-set classification beyond the result representation;
- automatic trimming/healing or topology repair;
- tolerance widening as a solver substitute.
