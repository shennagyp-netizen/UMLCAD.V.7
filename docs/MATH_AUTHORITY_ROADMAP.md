# UMLCAD V7 — Mathematical Authority Roadmap

## Status

Authoritative implementation roadmap for `kernel/math/`.

This document records the complete mathematical-authority mission supplied for V7 development. It is subordinate to the repository architecture in `README.md` only where it conflicts with existing architectural boundaries; otherwise it is the active implementation roadmap for mathematical work.

## Mission

Finish the UMLCAD mathematical authority layer as a professional mechanical-CAD mathematical foundation. The scope is the mathematical layer only. CPU is the normative mathematical reference; Metal and CUDA are accelerating implementations; OCCT is an optional conformance oracle/backend and never defines UMLCAD semantics.

## Authority principles

- Mathematics defines UMLCAD.
- Semantic contracts define backend-neutral meaning.
- Native UMLCAD implementation realizes the contracts.
- OCCT may realize or challenge the implementation but is not semantic authority.
- GPU implementations conform to the CPU mathematical contract.
- Renderer, viewer, application, HTTP, database, Git, and AI are outside mathematical authority.
- Exact/analytic mathematics is preferred over sampled display geometry.
- Tolerance is explicit input, never hidden global state.
- Unsupported, ambiguous, singular, degenerate, and indeterminate cases fail closed.
- Immutable inputs produce immutable results.
- No silent NaN/default/partial-success behavior.

## Definition of done

The mathematical layer is complete only when it provides the mathematical primitives and algorithms required for serious parametric mechanical CAD, with explicit contracts, numerical evidence, adversarial tests, deterministic behavior, and verified CPU/GPU conformance where hardware permits.

## Capability families

### M0 — Audit and baseline

Establish repository understanding, math inventory, gap matrix, existing test baseline, dependency map, GPU capability map, current branch/commit/build status, known failures, and coverage. Do not fix unrelated baseline failures merely to improve the baseline.

### M1 — Scalar + constants + tolerance

Complete finite/NaN/Inf-safe scalar operations, mathematical constants, and immutable tolerance/numerical policies. Distinguish linear, angular, parametric, normal, curvature, intersection, constraint, solver-residual, and topological tolerances as applicable.

### M2 — Vector / matrix / quaternion / transform

Complete Vec2/Vec3/Vec4, Mat2/Mat3/Mat4, quaternion, 2D/3D/rigid/affine transforms, correct point/vector/normal transformation, determinant/inverse/solve/rank/norm/conditioning, and robust linear algebra including LU, QR, Cholesky, least squares, SVD or equivalent rank analysis, pseudo-inverse, and null space.

### M3 — Analytic geometry + predicates

Complete points, lines, rays, segments, planes, circles, arcs, ellipses, spheres, cylinders, cones, tori, robust geometric predicates, and analytic intersections with explicit classifications such as unique, multiple, tangent, parallel, coincident, overlap, skew, degenerate, and indeterminate.

### M4 — Polynomial / root / interval mathematics

Complete polynomial operations, derivatives, division/remainder, normalization, GCD where required, real/complex roots where required, root isolation, multiplicity and clustered-root handling, plus interval/error-bound mathematics for robust pruning and classification.

### M5 — Curves

Complete Curve2/Curve3 authority for line, arc, circle, ellipse, Bezier, B-spline, NURBS, and composite/trimmed curves. Provide evaluation, analytic derivatives, tangent/normal/curvature, arc length where applicable, domain operations, splitting/reversal/restriction, projection, closest point, and mathematically supported offsets.

### M6 — Surfaces + NURBS

Complete analytic surfaces, Bezier/B-spline/NURBS surfaces, tensor-product evaluation, homogeneous/projective mathematics, analytic first/second derivatives, normals, fundamental forms, curvature, continuity, singular-parameterization handling, and trimmed-surface parameter-domain mathematics.

NURBS validation must cover degree, knot-vector structure/order/multiplicity, control-point count, positive finite weights, domain, finite values, parameter validity, and homogeneous denominator safety. Support knot insertion/removal where safe, degree elevation where required, splitting, reversal, domain restriction, continuity analysis, and trim curves.

### M7 — Distance / projection / intersection

Complete reliable point/curve/surface distance and projection, curve-curve, curve-surface, and surface-surface intersection frameworks. Use broad phase, subdivision/candidate isolation, refinement, verification, and classification. Convergence alone is never proof. Require domain validity, residual validity, parameter validity, geometric verification, and classification.

### M8 — CAD construction mathematics

Complete or robustly support offsets, extrusions, revolutions, sweeps/pipes, lofts, blends, fillets, chamfers, shell/thickening mathematics, frame construction, twist handling, singularity detection, self-intersection candidates, and explicit G0/G1/G2 continuity verification where relevant.

### M9 — B-Rep / solid mathematics

Provide mathematical support for vertices, edges, coedges, wires, faces, trims, shells, solids, orientation, closure, manifoldness-related validation, inside/outside classification, surface-region mathematics, volume, area, centroid, inertia where appropriate, and Boolean-support operations such as intersection, union, difference, split, and imprint. Never derive exact topology from display meshes.

### M10 — Constraints / Jacobian / solver

Complete semantic constraint equations, analytic Jacobians, controlled automatic differentiation where compatible, finite-difference fallback only with explicit step strategy and limitations, rank/nullity/conditioning/DOF analysis, redundancy/dependency/inconsistency classification, and robust Newton/damped Newton/Gauss-Newton/Levenberg-Marquardt/trust-region or equivalent solver foundations.

Solver results must report solution, iterations, residual norm, step norm, rank, condition estimate, and explicit status such as Converged, ConvergedWithWarning, Diverged, Singular, IllConditioned, Inconsistent, MaxIterations, Indeterminate, InvalidInput, or Cancelled. Convergence is accepted only after explicit mathematical verification.

### M11 — Tessellation / spatial acceleration

Complete authoritative approximation support for curve/surface/trim-aware adaptive tessellation with chord and angular error controls, boundary preservation, normals, and tolerance metadata. Complete AABB, bounding sphere, OBB where justified, BVH, spatial subdivision, and parameter-space bounds. Broad phase may produce false positives but never false negatives.

### M12 — GPU abstraction

Create backend-neutral GPU capability/execution abstractions, batch memory and dispatch models, and CPU/GPU conformance harnesses. Keep GPU resources as implementation state, never mathematical semantic state. Support Auto/CPU/Metal/CUDA conceptually with measured or documented selection criteria.

### M13 — Apple Silicon Metal

Implement high-value Metal batch acceleration for vectors, matrices, transforms, predicates, curve/surface/NURBS evaluation, Jacobian/residual batches, candidate sets, tessellation, and other genuinely parallel workloads. Preserve f64 authority; fallback to CPU or report Unsupported when the backend cannot meet the contract. Do not use unsafe fast-math for authoritative geometry.

### M14 — NVIDIA CUDA

Implement corresponding CUDA acceleration and use mature CUDA numerical libraries where appropriate for high-throughput linear algebra, factorization, solver subproblems, and batched kernels. CUDA is acceleration only and must conform to CPU semantics.

### M15 — Cross-backend conformance

Run identical vectors through CPU, Metal, and CUDA where hardware is available. Compare classifications, geometric results, residuals, and error bounds under the same tolerance policy. Fix implementation discrepancies rather than weakening the contract. Never claim hardware validation where hardware execution did not occur.

### M16 — Performance + final red team

Benchmark latency, throughput, CPU/GPU crossover, transfer overhead, and batch efficiency. Then attack the entire authority layer with near-parallel/coincident/tangent geometry, repeated roots, degenerate/extreme NURBS, extreme coordinate scales, rank-deficient/inconsistent/near-singular constraints, solver stagnation, GPU ordering differences, NaN/Inf propagation, invalid domains, self-intersecting trims, zero-area faces, open solids, and orientation errors. Fix real defects; never suppress failures merely to obtain green CI.

## Required numerical foundations

### Scalars and constants

Authoritative geometry uses `f64` unless an operation explicitly documents another precision. Centralize PI, TAU, E, SQRT_2, SQRT_3, HALF_PI, QUARTER_PI, and machine epsilon. No scattered literal approximations.

### Functional programming discipline

Prefer pure functions, immutable values, explicit inputs/outputs, algebraic result types, pattern matching, deterministic composition, and no hidden global numerical state. Caches are permitted only as semantics-preserving optimizations. Cancellation is explicit and returns `Cancelled` rather than a partial authoritative result.

### Failure model

Use structured outcomes such as InvalidInput, Unsupported, Ambiguous, Indeterminate, Degenerate, Singular, IllConditioned, NonConvergent, BackendFailure, Cancelled, and Internal where the repository's contracts support them. Attach residuals, error estimates, condition estimates, iterations, parameters, and tolerance context where useful.

### Robustness escalation

Prefer:

```text
standard arithmetic
→ stable/scaled formulation
→ error-bound/interval test
→ stronger numerical algorithm
→ CPU reference fallback
→ Indeterminate
```

Do not fabricate certainty and do not introduce arbitrary precision everywhere without mathematical justification.

## NURBS authority requirements

NURBS is P0. Use positive finite weights, validated nondecreasing full knot vectors, explicit domains, homogeneous evaluation, safe dehomogenization, differentiated control nets, quotient-rule derivatives, tensor-product derivatives, continuity/multiplicity analysis, and conservative control-net bounds. Never allow a zero/invalid homogeneous denominator to become NaN. Overlapping bounds are candidate contact only, not an intersection certificate.

## Trimmed geometry

Support outer/inner loops, parameter-space curves, 3D/UV correspondence, orientation, containment, nested holes, touching/near-touching boundaries, self-intersection detection, and degenerate trim classification. Preserve semantic distinction between geometric mathematics and topology interpretation.

## Differential geometry

Support tangent, normal, binormal, curvature, torsion, Frenet frames where defined, surface normals, principal directions/curvatures, Gaussian curvature, mean curvature, and shape-operator-equivalent mathematics. Explicitly handle zero-speed curves, umbilics, and parameterization singularities.

## Testing and verification gates

Every new capability follows:

```text
semantic contract
→ independent mathematical tests
→ backend-neutral contract tests
→ native implementation
→ native conformance
→ adversarial/red-team tests
→ determinism tests
→ cross-layer integration
→ release/performance validation
→ authoritative CI
```

Use property-based tests where useful for translation, rotation, scale, symmetry, reversibility, inverse relationships, parameter/domain invariants, and continuity. Repeat important numerical tests at scales `1e-12`, `1e-9`, `1e-6`, `1e-3`, `1`, `1e3`, `1e6`, `1e9`, and `1e12`.

Golden tests originate from analytic solutions, the independent CPU reference, or independent verification—not Metal, CUDA, viewer rendering, or OCCT alone.

## Red-team minimum set

Every relevant milestone attacks zero/near-zero, parallel/near-parallel, coincident/near-coincident, tangent/near-tangent, degenerate, singular, rank-deficient, ill-conditioned, huge/tiny coordinates, mixed scales, NaN, +Inf, -Inf, repeated polynomial roots, repeated knots, high knot multiplicity, extreme NURBS weights, nearly singular NURBS denominators, false-positive/false-negative classification, silent divergence, incorrect tangent/normal/rank, false convergence, and CPU/GPU semantic divergence.

## GPU rules

CPU defines reference behavior. GPU never defines a new mathematical meaning. Authoritative geometry remains f64. Do not silently downgrade to f32. Do not use unsafe fast-math for authority. Define per-operation determinism as bitwise, numerically deterministic within tolerance, or explicitly non-deterministic acceleration. Use deterministic reduction order, candidate ordering, and tie-breaking when required. Batch GPU workloads rather than dispatching one scalar operation per submission. Hybrid CPU/GPU solving is preferred where iteration control and final validation are inherently sequential.

If CUDA hardware is unavailable, complete the backend and conformance tests that can be executed, compile/check what is possible, and explicitly record hardware-unverified status. Likewise use Apple Silicon Metal hardware when available.

## Architecture boundaries

`kernel/math/` must remain backend-neutral and must not depend on OCCT, HTTP, WebSocket, browser APIs, React, SVG, database, Git, application state, or AI. Metal/CUDA adapters may exist privately as implementation infrastructure but must not leak MTL/CUDA handles into mathematical semantic types. Mathematical values remain immutable and thread-safe.

## Dependency policy

Audit existing numerical crates first. Reuse trusted dependencies where they reduce risk, behind UMLCAD-owned interfaces. Do not reinvent mature BLAS/LAPACK-class algorithms unnecessarily, and do not make an external library the semantic authority.

## Documentation requirements

Every major mathematical module documents definition, formula, domain, preconditions, postconditions, algorithm, numerical method, tolerances, degeneracies, failure states, complexity, tests, GPU support, and known limitations. Subtle algorithms should cite an established mathematical source or derivation, including NURBS/CAGD and numerical-analysis literature as appropriate.

## Change and commit discipline

Keep changes focused on mathematical authority. Outside-`kernel/math` changes are allowed only when technically required to build, test, link, expose internally, or enable GPU compilation. Prefer milestone-sized commits such as:

```text
math: complete scalar and tolerance authority
math: complete linear algebra authority
math: complete analytic geometry predicates
math: complete nurbs authority
math: complete intersection authority
math: complete constraint solver mathematics
math: add metal acceleration
math: add cuda acceleration
math: add cross-backend conformance
```

Never rewrite protected history. Never create placeholder implementations, fake GPU execution, CPU masquerading as GPU, mesh sampling masquerading as exact geometry, unchecked NaN, or silent fallback without diagnostic.

## Final acceptance report

At mathematical-authority completion, report exactly:

1. mathematical capabilities implemented;
2. existing capabilities reused;
3. new algorithms added;
4. CPU reference status;
5. Metal status;
6. CUDA status;
7. TDD coverage;
8. red-team coverage;
9. determinism status;
10. known limitations;
11. hardware validation status;
12. performance measurements;
13. exact files changed;
14. tests executed;
15. remaining work.

Every capability must be distinguished as `Implemented`, `Tested`, `Hardware-validated`, `Not yet hardware-validated`, or `Unsupported`.

## Governing principles

> Mathematics first. Verification second. Optimization third.

> CPU defines the reference behavior. Metal and CUDA accelerate it. OCCT can challenge it. Nothing downstream defines it.

> Do not chase feature count. Build strong mathematical primitives from which professional CAD features can be constructed.


## M14 closure record

M14 is implemented and repository-tested for the declared NVIDIA CUDA acceleration domain, but is not hardware-validated in the current environment. The real backend is implemented outside `kernel/math` using `cudarc` as the CUDA driver/NVRTC boundary; `nalgebra` remains the existing CPU numerical infrastructure and CPU f64 remains semantic authority.
- real CUDA compute kernel for batched AABB candidate generation using native device f64;
- explicit invalid/non-finite/batch-size handling;
- CUDA driver/NVRTC absence fails closed instead of panicking during capability discovery;
- GPU candidate output is reconstructed in deterministic CPU order;
- every exact CPU f64 overlap must be present or the adapter returns a false-negative error;
- repeated hardware execution is compared for deterministic candidate output;
- hardware tests are feature-gated with `cuda-hardware` so ordinary authoritative CI contains zero ignored tests;
- CUDA hardware validation remains **Not yet hardware-validated** because no NVIDIA runner has executed the feature-gated hardware suite.

The M14 certified domain does not include CUDA execution of the full f64 vector/matrix/transform/NURBS solver stack. Those remain CPU-authoritative pending additional contracts and M15 cross-backend conformance.

## M15 closure record

M15 is closed for the declared common GPU workload supported by both accelerator implementations: conservative AABB candidate generation.

The conformance layer is backend-neutral and keeps CPU f64 exact overlap classification authoritative. Accelerator candidate sets must contain every reference overlap; false positives are measured separately because broad-phase conservatism is permitted. Repeat candidate execution must be deterministic. The identical adversarial fixture is executed by CPU reference and Metal hardware; the CUDA path is wired to the same suite but remains hardware-unvalidated because no NVIDIA runner execution record is available.

M15 does not promote Metal or CUDA to authority for the broader f64 vector/matrix/transform/NURBS solver stack. Those capabilities remain CPU-authoritative pending a common hardware contract.