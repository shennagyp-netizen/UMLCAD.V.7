# V6 Blind Review Disposition

This document records the engineering disposition of the external blind-review observations against the actual V6 contracts and implementation. The review is treated as input, not as authority.

## Fixed findings

### Independent symmetric residuals
`Symmetric` previously emitted four residual rows, while the second pair was an exact scalar multiple of the first pair. This introduced algebraic redundancy into the Jacobian without adding information. V6 now emits the two independent midpoint-coordinate equations only.

### Self-loop incidence
Topology degree accounting previously counted an edge only once when its start and end vertices were the same. A self-loop contributes two incidences to a vertex. V6 now counts start and end incidences separately, so a loop plus one tail is correctly rejected as degree three.

### Arc spatial intersection completeness
The previous line/arc, arc/circle, and arc/arc paths relied on sampled arc points for important cases while reporting `analytic-2d`. V6 now uses analytic line-circle and circle-circle intersection candidates, filters candidates through the authoritative arc membership contract, and evaluates the finite endpoint/extremum candidate set for the distance fallback. Multi-wrap arc semantics remain unchanged.

### Spatial validation tautology
The engineering validation path previously checked `intersects == (distance <= tolerance)` even though `intersects` was constructed from that exact predicate. The equality was removed; validation now checks the meaningful integrity condition that computed distances are finite.

## Findings deliberately not changed

### Spatial analysis broad phase
`spatial_analysis` intentionally reports only AABB-overlapping pairs. This is the current V6 broad-phase contract, not a defect in the geometry distance implementation.

### Multi-wrap arcs
V6 intentionally permits spans greater than one revolution and treats a full-or-greater angular span as containing the whole geometric circle for point-set membership while preserving parameterized length. The implementation and tests are required to remain internally consistent with that semantic choice.

### Fixed residual behavior
A fixed geometry is evaluated against its base snapshot separately from ordinary pure residual functions. The empty public residual in that conceptual path is deliberate architecture, not evidence of a missing equation.

### Residual scaling
Constraint scaling is performed by the solver's column normalization / numerical solve path. Raw residuals are intentionally preserved for diagnostics and are not rewritten merely to satisfy the review's proposed formulation.

### Zero damping
The solver already uses the SVD pseudoinverse formula `s / (s*s + d)`, so `d = 0` is a valid minimum-norm pseudoinverse case. A regression test protects this behavior; no artificial damping floor was introduced.

### Rank threshold
The rank calculation is applied after column normalization in `scaled_damped_qr`. A scale-invariance regression now protects that contract. No semantic change to the solver was justified solely by the review's threshold concern.

### Topology / manifold scope
The topology check remains a structural validator, not a proof of manifoldness for arbitrary pathological B-Reps. Native OCCT validation and V6's existing incidence checks remain separate responsibilities.

## Performance note

The snapshot's linear geometry lookup inside the solver is a legitimate optimization opportunity for large models, but it is not a correctness defect. It remains outside this hardening change so semantic behavior stays isolated from an algorithmic cache/index redesign.

## Verification principle

Each fixed finding is protected by a regression or adversarial test. The CI workflow now gates both the existing OCCT Rust workspace and the `kernel_rust` workspace with format, debug tests, release tests, and clippy.
