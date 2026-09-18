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
| Trim mathematics | Analytic parameter-space line/arc trims with nesting, boundary, orientation, and self-intersection predicates | Focused trim regressions, including line/arc and arc/arc intersections plus translation stability | General freeform 3D trim correspondence and broader touch/overlap classifications remain future contracts | Curves / predicates | Medium | P0 | Implemented / Tested (certified line/arc domain) |
| General distance/projection | `distance.rs`, existing spatial | Focused primitive tests | General curve/surface closest-point engine with verification | Curves / surfaces | High | P0 | Partial |
| General intersections | Existing bounded V6 families + `intersections.rs` | Existing V7 tests | General curve-curve, curve-surface, surface-surface candidate isolation/refinement/verification | Polynomial / interval / surfaces | Medium | P0 | Partial |
| Offset / sweep / loft / blend | `construction.rs` plus existing `offsets.rs`/`sweeps.rs` | Independent construction authority tests | Future extensions require new explicit certified domains | Surface differential / intersection | Medium | P0 | Implemented / Tested (certified domains) |
| Fillet / chamfer | `construction.rs` bounded analytic box families + existing backend contracts | Independent closed-form tests + backend contracts | General arbitrary-edge fillet/chamfer remains a future extension | Intersections / curvature | Medium | P1 | Implemented / Tested (bounded families) |
| B-Rep supporting math | Explicit edge/coedge/wire/face/shell incidence, ownership, orientation, closure, shell connectivity, and vertex-link validation | B-Rep structural regressions and scale/boundary tests | General curved-face sewing and unrestricted topology construction remain future contracts | Predicates / intersections | Medium | P0 | Implemented / Tested (certified explicit-topology planar domain) |
| Solid mathematics | Explicit closed/oriented planar-face solid mathematics with point classification, volume, centroid, surface area, and inertia | B-Rep/solid regression tests | General curved-face and holed-face exact decomposition remains outside the certified solid domain and is rejected fail-closed | B-Rep | Low/Medium | P0 | Implemented / Tested (certified domain) |
| Boolean support | Exact bounded AABB intersection/union/difference plus split/imprint partition mathematics | Existing AABB Boolean/split/imprint regressions | General topology-aware B-Rep Boolean construction from surface intersections remains outside the certified domain | B-Rep + intersections | Low/Medium | P0 | Implemented / Tested (bounded AABB domain) |
| Constraint equations | `constraints.rs`, `relations.rs` | Exhaustive constraint + relation authority tests | Supported semantic constraint/relation enums are complete; new families require new contracts | Differential geometry | Medium | P0 | Implemented / Tested |
| Analytic Jacobians | `jacobian.rs`, `relation_jacobian.rs` | Exhaustive independent finite-difference verification for every supported family | Finite differences remain verification-only; future new equations require analytic rows | Constraints + derivatives | High | P0 | Implemented / Tested |
| Nonlinear solver | Damped SVD/least-squares solve with adaptive damping, accepted-history convergence, terminal/status authority | Extensive solver, scale, mixed-unit, rank/conditioning, convergence and red-team tests | No separate named Newton/TR backend is required while the current damped least-squares acceptance model remains the supported equivalent | Linalg + Jacobians | Medium | P0 | Implemented / Tested |
| Tessellation math | Adaptive 3D curve tessellation, adaptive parametric-surface tessellation, and certified convex line/arc trim-aware outer-loop tessellation with explicit UV/position/normal/error metadata | Curve tests; planar/curved/depth-failure surface tests; trim boundary, convex-fill, concave-rejection, boundary-preservation, and translation/determinism regressions | General concave/holed/freeform trim filling remains outside the certified M11 domain and requires a new mathematical contract; sampled chord/angular metrics are approximation certificates, not exact curvature bounds | Curves / surfaces / trims | High | P1 | Implemented / Tested (certified M11 domain) |
| Spatial acceleration | AABB, conservative bounding spheres, deterministic 8-way spatial subdivision, BVH query/candidate traversal, and parameter-space bounds | Focused AABB/sphere/parameter/subdivision/BVH determinism, overflow, resident-item, and brute-force-equivalence tests | OBB remains intentionally deferred because no current measured workload justifies another numerical authority; a broader unified bounds interface can be added under a future contract | Bounds / predicates | High | P1 | Implemented / Tested (certified current acceleration domain) |
| GPU abstraction | `gpu.rs` backend-neutral capability/selection, batch memory, dispatch, executor, and f64 conformance contracts | Focused capability/selection, precision, determinism, overflow, dispatch, fallback, and conformance regressions | No hardware backend is implemented in M12; measured crossover thresholds remain future work; selection uses explicit documented policy inputs | CPU math stable | High | P0 | Implemented / Tested (no hardware backend) |
| Metal | None authoritative yet | None | Real Metal backend + conformance on Apple Silicon | GPU abstraction | High | P0 | Missing |
| CUDA | None authoritative yet | None | Real CUDA backend + conformance on NVIDIA hardware | GPU abstraction | High | P0 | Missing |
| Cross-backend conformance | None authoritative yet | None | CPU/Metal/CUDA comparison harness | All GPU backends | High | P0 | Missing |
| Final scale/red-team matrix | Existing scattered tests | Good but incomplete | Standardized 1e-12 … 1e12 and pathological fixture families across all P0 operations | All P0 math | Medium | P0 | Partial |

## M11 closure record

M11 is closed for the declared certified mathematical domain represented by the tessellation and spatial modules:
- adaptive 3D curve tessellation with explicit chord/angular policy and fail-closed depth handling;
- adaptive parametric-surface tessellation with UV, position, normal, chord/angular error metadata, deterministic subdivision, and explicit depth failure;
- trim-aware tessellation for convex parameter-space outer loops composed of certified line/arc trim curves, including exact UV boundary-sample preservation, convexity certification, explicit interior classification, adaptive surface error control, and deterministic translation-invariant output structure;
- conservative AABB and bounding-sphere broad-phase volumes;
- deterministic parameter-space bounds and 8-way spatial subdivision with parent-resident item filtering;
- deterministic BVH AABB query and node-pair candidate traversal with brute-force overlap equivalence.

M11 deliberately does not claim general concave/holed/freeform trimmed-surface triangulation, exact trim-surface curvature bounds, or an OBB semantic contract. Those remain future extensions when their mathematical contracts and workload justification are explicit.

The exact-head Rust and comprehensive E2E/red-team gates are green on the pre-closure head `f585dddc83471556fc8950e9138cd53f05bb6261`. The next documentation commit records the final branch head and merge status. M11 status is therefore `Implemented / Tested` for the declared certified domain.

## M10 closure record

As of main commit `8fea60633c4f95f0175d18faa54878e7af9f44a6`, the current supported M10 family is closed as an implemented/tested mathematical authority:
- all currently supported semantic constraint and relation families have analytic residual/Jacobian coverage;
- the production solver uses the complete analytic Jacobian, explicit dimensionless row scaling, nalgebra-backed damped SVD least-squares steps, and adaptive damping/acceptance;
- rank, nullity/DOF, conditioning, linear consistency, and dependent-equation evidence are explicit and fail-closed;
- accepted-step convergence history and terminal convergence certification are authoritative;
- solve results carry explicit `SolverStatus` and supporting evidence;
- mixed-unit and large-scale nonlinear regression coverage exercises the production path;
- exact-head and post-merge Rust + comprehensive E2E/red-team gates are green for the M10 closure PR.

M10 status is therefore `Implemented / Tested`. This does not imply completion of M8, M9, or M0-M16 as a whole.

## M8 closure record

The construction authority is certified only for the declared bounded analytic families. The current hardening also certifies that polygon-loft affine interpolation remains strictly convex through its full parameter family rather than assuming endpoint convexity alone.


As of main commit `fdb34d5a1e03ed399f900848b6dbe96da71d1a13`, M8 is closed for the currently supported certified construction domains:
- convex planar offsets with deterministic miter joins;
- linear extrusion and off-axis-safe polygon revolution;
- corresponding convex polygon lofts;
- circular, variable-radius, and circular-arc pipe mathematics;
- analytic Hermite blends with G0/G1/G2 continuity verification;
- deterministic frame and twist construction with explicit singularity handling;
- bounded all-edge box fillet/chamfer mathematics;
- closed-box shell/thickening mathematics;
- fail-closed non-finite, degenerate, self-intersection, parameter, singular-frame, unsupported-domain, and overflow handling;
- exact-head and post-merge Rust + comprehensive E2E/red-team gates are green.

M8 status is therefore `Implemented / Tested` for the declared certified domains. Arbitrary freeform offset/fillet/chamfer/topology construction remains outside this milestone and requires separate mathematical contracts.

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

## M9 closure record

As of branch head `5d8b77f1c00ae31a9b22abb1187c1710f267923e`, the M9 mathematical authority is closed for its declared certified domains:
- explicit B-Rep vertex/edge/coedge/wire/face/shell incidence and reference validation;
- opposite coedge orientation and face ownership checks;
- closed-wire endpoint continuity;
- shell face-connectivity validation;
- connected two-manifold vertex-link validation;
- planar-region outer-loop and convex-hole validation, nesting/disjointness checks, and explicit boundary classification;
- analytic line/arc trim self-intersection predicates in the certified parameter-space domain;
- exact closed, single-shell, planar-face solid volume, centroid, surface area, and inertia mathematics;
- fail-closed rejection of holed solid moments/point classification until a certified polygon-with-hole decomposition exists;
- exact bounded AABB intersection/union/difference, split, and imprint partition mathematics;
- translation/scale stability regressions for planar regions, trims, solid moments, and bounded AABB operations.
- bounded AABB tolerance scales use actual geometric extent, including sub-unit and large-translation regression coverage.

The CPU mathematical layer remains authoritative. OCCT and future GPU backends are not used to define M9 semantics. General curved-face B-Rep sewing, unrestricted topology-aware Boolean construction, and arbitrary curved/holed exact solid decomposition remain outside the certified M9 domain.

M9 status is therefore `Implemented / Tested` for the declared certified domains. Exact-head Rust and comprehensive E2E/red-team gates are green on `9d04c05f...`. Post-merge main verification is still required before this closure becomes final.

## M12 closure record

M12 is closed for the declared backend-neutral GPU abstraction domain. The CPU mathematical layer remains normative and no Metal/CUDA execution is claimed.
- explicit conceptual backend model for Auto/CPU/Metal/CUDA;
- capability discovery covering f64 support, operation support, maximum batch size, workgroup limits, and determinism class;
- explicit automatic/preferred/explicit selection with recorded CPU fallback and no accelerator eligibility when f64 or required determinism is unavailable;
- checked batched-memory sizing and host/device transfer accounting with overflow rejection;
- deterministic workgroup/dispatch planning with capability validation;
- backend-neutral executor submission trait with backend-local submission identity explicitly excluded from semantic state;
- CPU-versus-accelerator f64 conformance comparison with explicit absolute/relative tolerances and fail-closed length, non-finite, numerical, and bitwise mismatches;
- legacy GPU API retained only as a deprecated compatibility wrapper; authority decisions use the diagnostic selection result.

No fake GPU execution, f32 semantic downgrade, fast-math authority, Metal/CUDA device resource, or backend-defined mathematical meaning was introduced. M13 (Metal) and M14 (CUDA) remain responsible for real hardware execution.