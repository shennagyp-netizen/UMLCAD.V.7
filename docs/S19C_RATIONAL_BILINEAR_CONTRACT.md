# V6 S19C — Rational Bilinear Freeform/Planar Intersection Contract

## Scope

S19C extends the S19A/S19B bounded bilinear freeform/planar intersection family to rational bilinear freeform patches with strictly positive finite weights.

For a rational bilinear surface

`S(s,t) = N(s,t) / W(s,t)`,

where `W(s,t) > 0` over the parameter rectangle, intersection with a plane `n·x-h=0` is equivalent to the bilinear numerator equation

`sum_ij N_ij(s,t) * w_ij * (n·P_ij-h) = 0`.

Therefore the plane restriction remains exactly a bilinear polynomial in `(s,t)`; no iterative surface tracing is used for this bounded family.

## Authority

UMLCAD owns the rational evaluation, positive-weight validity, homogeneous plane restriction, root classification, deterministic ordering, and endpoint parameter mapping. OCCT is only a realization/conformance oracle.

Native handles, native topology identity, renderer behavior, tolerance widening, and heuristic candidate selection are not semantic authority.

## Certified outcomes

- zero isolated boundary roots: `NoIntersection`;
- two isolated roots: one deterministic segment;
- four isolated roots: two deterministic segments;
- one or three isolated roots, coincident restrictions, degenerate patches, or numerically indeterminate cases: fail closed as `Ambiguous`, `UnsupportedSurfaceFamily`, or structured numerical error as appropriate.

The plane patch itself remains an unweighted planar bilinear patch in this slice so its inverse parameter map is affine and deterministic.

## Required gate

The slice must pass independent semantic tests, deterministic repeat tests, native conformance where added, workspace and kernel debug/release tests, and both kernel Clippy gates before merge.

This contract does not claim unrestricted rational NURBS/NURBS intersection tracing or general freeform intersection completeness.
