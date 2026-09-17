# UMLCAD V7 Mathematical Authority — Gap Matrix

This matrix follows the V7 master implementation prompt. `Implemented` means code exists in the CPU mathematical layer; `Tested` means focused unit/regression coverage exists; hardware validation is tracked separately.

| Capability | Existing / added implementation | Tests | Missing / risk | Dependency | GPU suitability | Priority | Status |
|---|---|---|---|---|---|---|---|
| Scalars / finite arithmetic | `scalar.rs`, existing finite checks | Unit tests | Existing geometry still contains direct arithmetic outside scalar helpers | None | High | P0 | Implemented / partial migration |
| Constants | `constants.rs` | Unit tests | None material to P0 | None | High | P0 | Implemented |
| Explicit tolerance | `tolerance.rs` | Unit tests | Existing legacy modules still use local epsilons; migration required | Scalar | High | P0 | Implemented / migration pending |
| Vec2 / Vec3 / Vec4 | `vec.rs`, `vec4.rs` | Unit + extreme-scale tests | Existing geometry has duplicate Point types | None | High | P0 | Implemented |
| Mat2 / Mat3 / Mat4 | `mat.rs` | Inverse, solve, scale, singular tests | Broader robust condition/rank API should route through `linalg.rs` | Vec | High | P0 | Implemented / integration pending |
| Quaternion | `quaternion.rs` | Rotation + inverse + extreme tests | Intermediate overflow in very large vector rotations needs further hardening | Vec3 | High | P0 | Implemented / hardening pending |
| Affine / rigid transforms | `transform.rs` | Inverse/covariance/normal tests | More factory constructors and property tests | Matrix + quaternion | High | P0 | Implemented / extension pending |
| Robust LU / QR / SVD | `linalg.rs` | Rank, scale, pseudo-inverse tests | Wide-matrix null-space and deeper ill-conditioning matrix | nalgebra | High | P0 | Implemented / hardening pending |
| Predicates | `predicates.rs` | Sign, degeneracy, scale tests | Exact/error-bound escalation and surface/solid predicates | Interval | High | P0 | Implemented / extension pending |
| Analytic primitives | `analytic.rs`, `conics3d.rs`, `analytic_surfaces.rs` | Primitive-focused tests | More containment/projection/distance and shared framework | Vec / transforms | High | P0 | Implemented / extension pending |
| Analytic intersections | `intersections.rs` | Line/circle/plane/sphere tests | Complete primitive cross-product matrix, structured overlap-circle representation | Polynomial / predicates | Medium | P0 | Partial |
| Segments / rays | `segments.rs`, `analytic.rs` | Domain/projection tests | 2D/3D pairwise intersection family | Predicates | High | P0 | Partial |
| Polynomial / roots | `polynomial.rs` | Basic/repeated root tests | Stable multiplicity, clustered roots, complex roots, independent validation | Scalar / interval | High | P0 | Partial |
| Interval arithmetic | `interval.rs` | Arithmetic/separation tests | Full interval transcendentals and polynomial interval evaluation | Scalar | High | P0 | Partial |
| Curve differentials | `curve_differential.rs`, existing Bezier/B-spline/NURBS | First/second derivative tests | Complete Curve2/Curve3 framework, curvature/torsion/Frenet | Linalg / NURBS | High | P0 | Partial |
| NURBS exact operations | `nurbs_ops.rs` + existing NURBS | Knot insertion/reversal/continuity tests | Knot removal, degree elevation, splitting, rational curve generalization | Polynomial / homogeneous math | High | P0 | Partial |
| Surface differentials | `surface_differential.rs`, existing NURBS surface differential | Second-order/curvature planar tests | Independent oracle coverage, singular-point classification, more general surfaces | Curve/NURBS | High | P0 | Partial |
| Trim mathematics | Existing historical implementation plus current math foundation | Existing project tests | Unified parameter-space trims, nesting/self-intersection/touch classification, 3D correspondence | Curves / predicates | Medium | P0 | Missing as unified V7 layer |
| General distance/projection | `distance.rs`, existing spatial | Focused primitive tests | General curve/surface closest-point engine with verification | Curves / surfaces | High | P0 | Partial |
| General intersections | Existing bounded V6 families + `intersections.rs` | Existing V7 tests | General curve-curve, curve-surface, surface-surface candidate isolation/refinement/verification | Polynomial / interval / surfaces | Medium | P0 | Partial |
| Offset / sweep / loft / blend | Existing `offsets.rs`, `sweeps.rs` | Existing focused tests | Unified general mathematical construction layer | Surface differential / intersection | Medium | P0 | Existing bounded families; extension pending |
| Fillet / chamfer | Existing bounded implementation | Existing tests | General analytic construction framework | Intersections / curvature | Medium | P1 | Partial |
| B-Rep supporting math | Existing topology plus bounded V6 mathematics | Existing topology tests | Unified edge/face/wire/shell incidence and geometric consistency validators | Predicates / intersections | Medium | P0 | Partial |
| Solid mathematics | Existing bounded volume/engineering evidence | Existing tests | General point-in-solid, centroid/inertia, orientation/closure mathematics | B-Rep | Low/Medium | P0 | Partial |
| Boolean support | Existing bounded/native pathways | Existing tests | General topology-aware region construction from intersections | B-Rep + intersections | Low/Medium | P0 | Partial |
| Constraint equations | `constraints.rs`, `relations.rs` | Existing solver tests | Complete analytic constraint family | Differential geometry | Medium | P0 | Partial |
| Analytic Jacobians | Existing finite-difference solver | Existing solver tests | Analytic Jacobian authority for every supported constraint/relation | Constraints + derivatives | High | P0 | Missing |
| Nonlinear solver | Existing damped QR solver | Extensive existing tests | Newton/Gauss-Newton/LM/trust-region classification and analytic Jacobians | Linalg + Jacobians | Medium | P0 | Partial |
| Tessellation math | Existing mesh-related capabilities | Existing project tests | Unified adaptive/trim-aware tessellator with explicit error metadata | Curves / surfaces / trims | High | P1 | Partial |
| Spatial acceleration | Existing AABB/spatial module | Existing spatial tests | Unified BVH/OBB/parameter-space pruning contracts | Bounds / predicates | High | P1 | Partial |
| GPU abstraction | None authoritative yet | None | Backend-neutral batch/execution contract | CPU math stable | N/A | P0 | Missing |
| Metal | None authoritative yet | None | Real Metal backend + conformance on Apple Silicon | GPU abstraction | High | P0 | Missing |
| CUDA | None authoritative yet | None | Real CUDA backend + conformance on NVIDIA hardware | GPU abstraction | High | P0 | Missing |
| Cross-backend conformance | None authoritative yet | None | CPU/Metal/CUDA comparison harness | All GPU backends | High | P0 | Missing |
| Final scale/red-team matrix | Existing scattered tests | Good but incomplete | Standardized 1e-12 … 1e12 and pathological fixture families across all P0 operations | All P0 math | Medium | P0 | Partial |

## Execution order

The high-leverage path remains:

```text
core scalar/tolerance
→ linear algebra
→ predicates
→ analytic geometry
→ polynomial + interval
→ NURBS exact operations
→ differential geometry
→ intersections
→ constraints/Jacobians/solver
→ B-Rep/solid mathematics
→ tessellation/spatial
→ GPU abstraction
→ Metal/CUDA
→ cross-backend conformance
```

The CPU implementation remains normative. GPU implementations are accelerators and cannot redefine semantics.
