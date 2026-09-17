# UMLCAD V6 — Roadmap Stations

This document is the live station map for the V6 geometry-kernel program. The mathematical/semantic layer is developed ahead of native realization; stations therefore distinguish semantic authority, backend-neutral contracts, and native conformance.

## Completed stations

### S8 — Interchange + visualization boundary hardening — COMPLETE

Analytic primitives, transforms, Booleans, extrusion, revolution, loft, fillet, chamfer, rational NURBS curves/surfaces, circular-arc sweep, mesh validation/tessellation, STEP/IGES exchange, and the opaque OCCT boundary are established. OCCT DataExchange remains serialized because release validation exposed unsafe concurrent lifetime behavior.

### S9 — Native tensor-product NURBS surface backend — COMPLETE

The validated backend-neutral `NurbsSurface3DDefinition` is realized by OCCT `Geom_BSplineSurface`, including complete knot-vector conversion, 2D rational weights, preserved `u * count_v + v` control-net ordering, opaque native results, and deterministic construction tests.

### S10 — NURBS surface differential/backend conformance — COMPLETE

Independent first/second differential mathematics, rational homogeneous-to-Euclidean quotient rules, normals, continuity policy, and deterministic edge cases are validated against native OCCT `Geom_Surface::D2` behavior. Full debug/release/format/Clippy/kernel gates passed.

### S11 — Freeform face construction and trimming — COMPLETE

Backend-neutral trimmed NURBS surface definitions now validate UV-domain containment, loop closure/orientation, simplicity, holes, and loop intersections. OCCT creates native p-curves and 3D boundary curves behind the opaque boundary, with deterministic topology/area/validity tests. S11 passed its full authoritative CI gate.

### S12 — Surface-surface / curve-surface operations — COMPLETE

S12 establishes the representative geometric operations needed to derive and modify freeform boundaries without promoting numerical guesses into topology.

Completed capabilities:

1. Backend-neutral line-segment / NURBS-surface intersection with deterministic multi-seed Newton isolation and exact affine-planar classification.
2. Explicit handling for unique roots, no intersection, tangent contact, and coplanar/coincident or underdetermined relations in the supported cases.
3. General NURBS-curve / NURBS-surface isolated-root semantics using an independent rational curve evaluator, explicit parameter domains, deterministic three-variable Newton isolation, root ordering, ambiguity reporting, and conservative tangent classification.
4. Exact degree-1 linear-curve handling through the established line/surface semantic authority, preserving the curve's real parameter interval.
5. Backend-neutral point/surface closest-point and distance semantics for the supported affine-planar NURBS family, including arbitrary UV domains and boundary projection.
6. Exact affine-planar NURBS surface/surface intersection through an independent plane-plane oracle with bounded UV-domain clipping and explicit parallel/coincident behavior.
7. Intersection-driven deterministic line splitting; ambiguous intersection evidence fails closed and cannot become arbitrary split topology.
8. A certified general NURBS surface-pair broad phase using the positive-weight control-net convex-hull property. Strictly separated control-net AABBs certify disjointness; overlapping bounds are only `PotentialContact`.
9. `intersect_nurbs_surfaces` consumes that broad phase first. Certified disjoint pairs produce deterministic `NoIntersection`; potential-contact pairs are delegated only to a proven exact solver family, otherwise returning `UnsupportedSurfaceFamily`.
10. Independent semantic expectations are checked against opaque OCCT realizations using `GeomAPI_IntCS`, `GeomAPI_IntSS`, and `GeomAPI_ProjectPointOnSurf`; OCCT does not define semantic meaning.
11. Deterministic ordering, explicit tolerance/domain policies, validation-before-FFI, and fail-closed unsupported/error paths are covered by the authoritative test matrix.
12. The S12 surface-intersection boundary and continuation requirements are documented in `docs/S12_SURFACE_INTERSECTION_CONTRACT.md` and `docs/S12_CONTINUATION_HANDOFF.md`.

S12 scope boundary:

- The implemented exact surface/surface intersection curve construction is intentionally limited to the affine-planar patch family.
- The general surface-pair dispatcher can certify disjointness for arbitrary valid positive-weight NURBS surfaces, but it does not claim exhaustive arbitrary NURBS/NURBS intersection-curve isolation.
- General potential contact outside the proven solver families remains explicitly unsupported.
- General freeform intersection-curve tracing and generalized trimming remain later work; no renderer approximation is promoted into topology.

**S12 exit gate:** satisfied by the authoritative full CI matrix on PR #44: workspace debug/release tests, workspace Clippy, kernel format/tests, and kernel Clippy gates all green, with semantic/native conformance tests passing.

### S13 — Offsets and healing — COMPLETE

S13 established the first bounded offset/healing slice without widening the semantic boundary.

Completed capabilities and scope remain as documented in `docs/S13_OFFSET_HEALING_CONTRACT.md`.

### S14 — Freeform feature generation — COMPLETE

S14 established bounded sweep, variable-radius sweep, fillet, shell/thickness, and drafting contracts while preserving the authority order and explicit unsupported cases.

### S15 — Robust B-Rep topology kernel completion — BOUNDED SLICES COMPLETE; GENERALIZED WORK CONTINUES IN S18

The previously completed S15 bounded slices remain authoritative: shell orientation propagation, boundary sewing, native sewing conformance, containment evidence, pathology evidence, immutable orientation repair, degenerated-edge evidence, combined pathology snapshots, deterministic imported-pathology evidence, and controlled explicit face-removal repair.

The original S15 station is not reopened as a separate completion phase. Remaining generalized arbitrary-B-Rep work is part of the final S18 completion gate.

### S16 — Deterministic kernel integration — BOUNDED SLICES COMPLETE; GENERALIZED WORK CONTINUES IN S18

The previously completed S16 bounded slices remain authoritative: deterministic topology snapshot identity, canonicalization, immutable operation provenance, cancellation query semantics, structured operation errors, immutable integration snapshots, and atomic root transitions.

The original S16 station is not reopened as a separate completion phase. Remaining integration expansion is part of the final S18 completion gate.

### S17 — Kernel performance and stress gate — COMPLETE

S17 completed the deterministic stress/scale validation slices across integration, topology, large-topology behavior, freeform validation, numeric contracts, native geometry, and the final completion handoff. Optimization remains subordinate to semantic correctness and deterministic behavior.

## Current station

### S18 — V6 geometry-kernel and system completion gate — FINAL STATION

S18 is the final numbered V6 station and the single authoritative completion gate for the **integrated V6 kernel system**.

S18 completion requires the applicable geometry, B-Rep, freeform, intersection, sweep/pipe, offset/healing, tessellation, exchange, topology, reference-evolution/migration, and kernel-integration capabilities required by the declared V6 boundary to be implemented, integrated, and green.

S18 is not complete from isolated feature tests alone. The exact `main` completion candidate must pass the full authoritative system test matrix, including all applicable workspace and kernel tests, native conformance, adversarial/determinism coverage, and dedicated cross-crate/cross-layer system-integration scenarios.

S18 is completed only when each applicable contract has:

1. mathematical / semantic authority;
2. a backend-neutral contract;
3. independent semantic tests;
4. native OCCT realization where applicable;
5. independent native conformance;
6. cross-system integration coverage;
7. adversarial and determinism coverage;
8. debug/release validation and authoritative CI green on `main`.

The already-merged S19A/S19B/S19C changes are treated as S18.x implementation slices, not additional V6 stations. Future completion work must use S18.x identifiers and remains under the single S18 system-completion gate.

S18 does **not** claim that every theoretically possible industrial CAD operation is implemented. Unsupported cases remain explicit and fail closed. Assemblies, kinematics, drawings, FEA, and machine-design application behavior remain subsequent system layers outside the V6 part-geometry-kernel boundary unless explicitly brought into the V6 scope by a separate roadmap revision.

See `docs/S18_SYSTEM_INTEGRATION_GATE.md` for the mandatory full-system completion rule.

## Scope discipline

The station pipeline is:

```text
mathematical / semantic authority
        ↓
backend-neutral contract
        ↓
native realization (OCCT reference backend)
        ↓
independent conformance against the mathematics
        ↓
B-Rep topology / advanced freeform operations
        ↓
cross-system integration
        ↓
adversarial + determinism + release + authoritative CI
```

A backend never defines semantics. A station is never complete merely because code exists or a renderer displays a shape; the applicable TDD, adversarial, determinism, integration, release, and CI gates must pass on `main`.

See `docs/S18_FINAL_STATION_POLICY.md` for the station-numbering freeze and S18.x rule.
