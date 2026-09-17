# S12 — General NURBS Surface Intersection Boundary

## Purpose

S12 must distinguish what is mathematically certified from what merely appears numerically plausible. The general surface-intersection entry point therefore uses a conservative two-stage contract:

```text
valid NURBS surfaces
       |
       v
control-net convex-hull broad phase
       |
       +--> separated AABBs --> DisjointCertified --> NoIntersection
       |
       +--> possible contact --> exact supported solver / Unsupported
```

## Certified broad phase

For a tensor-product NURBS surface with strictly positive weights, each evaluated point is a convex combination of the control points. Therefore the surface image is contained in the convex hull of its control net. An axis-aligned bounding box around that control net is consequently an outer bound on the represented surface.

For two surfaces, strict separation of these outer AABBs by more than the declared tolerance is a certificate of geometric disjointness. The semantic API exposes this as `NurbsSurfacePairRelation::DisjointCertified`.

Failure to separate the boxes is deliberately weaker evidence. It means only `PotentialContact`; it is not an intersection certificate.

## General dispatcher

`intersect_nurbs_surfaces` consumes the broad-phase relation first.

- `DisjointCertified` returns an ordinary `SurfaceSurfaceIntersectionResult` with `NoIntersection` and no segments.
- `PotentialContact` is delegated only to the currently proven exact affine-planar patch solver.
- A potential-contact pair outside that solver's supported family returns `UnsupportedSurfaceFamily` rather than an approximate or renderer-derived curve.
- Invalid input, invalid tolerance, and numerical failures remain typed errors.

## Scope intentionally not claimed

This contract is **not** a general NURBS/NURBS intersection-curve solver. In particular, it does not claim:

- exhaustive isolation of all intersection components;
- detection of tangential or coincident intersection manifolds for arbitrary NURBS pairs;
- construction of continuous intersection curves for arbitrary NURBS surfaces;
- topology/trimming from an unverified numerical trace.

Those capabilities remain outstanding S12 work and require independent mathematical authority before native OCCT realization is promoted.

## Determinism and fail-closed behavior

The broad phase is a pure function of the supplied surfaces and tolerance. It uses no native geometry state and performs no mutation. Potential contact never falls through to guessed topology.

The S12 exit condition remains unchanged: representative curve-surface and surface-surface operations must be independently defined, numerically validated, natively realized, deterministic, and unable to silently convert ambiguous geometric relations into arbitrary topology.
