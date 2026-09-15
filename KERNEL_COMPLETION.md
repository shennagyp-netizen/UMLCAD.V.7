# UMLCAD V5 Kernel Completion Scope

The native V5 kernel is **implementation-complete and engineering-green for its declared current 2D capability surface**.

## Declared V5 kernel surface

V5 covers the implemented 2D engineering model of:

- line segments, circles, and circular arcs;
- immutable semantic snapshots and deterministic dependencies;
- native constraints and first-class geometric relations;
- scaled damped QR least-squares solving with explicit convergence and candidate validation;
- residual, scaling, rank, degrees-of-freedom, and conditioning analysis;
- contradiction/redundancy diagnostics and fail-closed invalid domains;
- semantic references with stale-reference and explicit migration diagnostics;
- semantic topology and ambiguity/non-manifold validation;
- exact 2D spatial analysis using AABB broad phase plus analytic narrow phase;
- semantic dimensions and measurements;
- engineering acceptance as an explicit solve → materialize → validate → accept/reject transaction;
- deterministic DXF export;
- structured engineering evidence for AI/client consumers;
- transport-neutral, stateless kernel evaluation with immutable results.

## Scientific hardening completed

The hardening pass resolved the critical defects identified in the declared surface, including:

- circles treated as closed curves without fabricated endpoints;
- true analytic circle circumference and circular-arc length;
- exact curve predicates after broad-phase rejection;
- explicit rejection of non-manifold/ambiguous topological continuation;
- fail-closed invalid constraint/relation domains;
- scaled damped QR least-squares instead of authoritative normal equations;
- geometry validation before accepting a solver candidate;
- correct rank/conditioning analysis for rectangular Jacobians by preserving the nonzero singular-value structure through transpose-based row-space analysis;
- acceptance logic that does not reject an already-satisfied underconstrained system merely because legitimate free degrees of freedom remain;
- explicit semantic relation dependencies and immutable relation snapshots;
- deterministic export and regression coverage for boundary, degeneracy, scale, topology, reference, solver, and acceptance cases.

## Completion gate

For the current V5 kernel generation, completion requires:

1. the complete declared V5 2D surface to have native semantic authority;
2. scientific regression/property coverage to remain green;
3. invalid or unsupported operations to fail closed rather than be approximated;
4. the kernel to remain stateless, transport-neutral, immutable in its results, and independent of client/UI/rendering concerns;
5. the full native test gate to execute all `test/native-*.test.ts` suites rather than a single smoke test.

All five conditions are satisfied on `main` and are directly verified by the native CI workflow.

## Explicit boundary

This completion does **not** pull V6-class geometry/modeling into V5. The following remain future V6 scope: Bézier/B-spline/NURBS freeform geometry, 3D curves and surfaces, full 3D B-Rep solids, robust booleans and trimming, shells, fillets, chamfers, drafts, sweeps, lofts, and full 3D assembly solving.

V5 is complete for its declared 2D kernel surface without pretending that V6 functionality already exists.
