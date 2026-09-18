# UMLCAD.V.7 Continuation Handoff

## Snapshot
- Repository: `shennagyp-netizen/UMLCAD.V.7`
- Default branch: `main`
- Main head before PR #30 merge: `d267d9cb5d277da6d655fefd9d122c1c165e9a53`
- Latest completed roadmap family after merge: M15 — Cross-backend conformance (declared AABB domain; CPU/Metal hardware-validated; CUDA hardware-unvalidated)
- Latest merged M10 closure PR: #27, `math: complete solver result status authority`
- PR #27 merge commit: `8fea60633c4f95f0175d18faa54878e7af9f44a6`
- Latest merged M8 closure PR: #28, `math: complete M8 construction authority`
- PR #28 merge commit: `fdb34d5a1e03ed399f900848b6dbe96da71d1a13`
- M9 closure PR: #30, `math: complete M8/M9 authority hardening`
- PR #30 validated implementation head: `9d04c05f67f4e5542df56b8612806fe8296f624b`

## Completed mathematical-authority state
PR #30 completes M9 for the declared certified domains. M0-M16 is now closed after the M16 post-merge verification.

Completed and validated stations now include:
- analytic constraint Jacobian authority;
- exhaustive analytic Jacobian relation coverage for the supported relation/constraint families;
- production solver use of the complete analytic Jacobian for the supported equation set;
- terminal convergence certification as an authoritative production result contract;
- exact terminal residual/step evidence checks with fail-closed contradictory-state handling;
- rank/nullity and augmented-rank consistency authority;
- wide null-space and pseudoinverse authority tests;
- conditioning authority with explicit conventional singular-system semantics;
- solver-status authority with fail-closed rank/conditioning semantics;
- linear-solver conformance tests against the independent consistency authority;
- zero-damping truncated pseudoinverse handling for numerically null singular directions;
- backend-neutral row-scaling contract for linearized systems;
- production row scaling of each analytic Jacobian row and matching residual row by the same explicit semantic scale before the existing nalgebra-backed damped SVD solve.
- truthful accepted-step convergence history authority that excludes rejected trials while preserving the authoritative attempted-iteration count;
- adversarial mixed-unit nonlinear solver regression coverage combining fixed geometry, coincident endpoints, model-unit distance, and angular relation equations across scales through `1e9`;

PR #23 specifically closed the previous mathematical mismatch in which convergence/acceptance used dimensionless scaled residuals while the nonlinear linearization step consumed raw mixed-unit residual rows. The production solve now applies the same semantic row normalization to both the Jacobian and residual before the existing solver backend.

## Analytic Jacobian policy
`kernel/math/jacobian.rs` is the production Jacobian authority in exact residual order: analytic constraint rows followed by analytic relation rows. The production solver does not fall back to generic finite differences for the supported equation set.

`SolveOptions::finite_difference_step` is retained for API compatibility, but it is not used for production Jacobian generation. Central/finite differences remain independent verification techniques only.

## Linear and conditioning authority
`kernel/math/linear_consistency.rs` distinguishes unique, underdetermined-consistent, inconsistent, and indeterminate systems using coefficient rank versus augmented rank under the declared numerical tolerance.

`kernel/math/conditioning.rs` provides explicit conventional conditioning evidence. Rank-deficient systems are not treated as conventionally finite-conditioned merely because a reduced nonzero singular spectrum has finite spread.

`kernel/math/solver_status.rs` consumes these proofs conservatively: `Inconsistent` is only claimed from explicit dimension-matched inconsistency evidence, and rank-deficient systems cannot be promoted to healthy conditioning.

## Accepted-step convergence, convergence and terminal authority
`kernel/math/convergence.rs` contains two complementary contracts:
- `evaluate` verifies accepted residual history, monotonicity, stagnation, progress, iteration cap, and joint residual/step convergence;
- `verify_terminal` verifies final residual and final step evidence without manufacturing an iteration history when an implementation rejects trial steps.

`kernel/math/terminal_authority.rs` is the public terminal-convergence authority used by the solver-facing module. It validates the terminal evidence returned by the wrapped solver and rejects contradictory convergence state.

The production iteration-control algorithm now stores the initial scaled residual and only residuals at actually accepted iterates. Rejected trial steps are not inserted into the history. The accepted-history authority therefore evaluates monotonicity and convergence truthfully while the solver's attempted-iteration count remains the authoritative count for iteration caps and result reporting.

## Production linearization scaling
`kernel/math/linearization.rs` defines the backend-neutral `RowScaledLinearSystem` contract and `scale_linearization` operation. A positive finite semantic row scale is applied identically to each Jacobian row and its corresponding residual component.

The native production solver integration is source-anchored and deterministic:
- `kernel/math/solver.rs` remains the sole solver implementation corpus;
- `kernel/native/build.rs` generates the native compile-time solver variant with the controlled row-scaling transformation;
- the generated solver calls the shared backend-neutral row-scaling contract, then the existing nalgebra-based damped SVD solver;
- a production regression checks uniform geometry scaling invariance of the terminal result evidence.

No second numerical library, duplicate solver implementation, GPU semantic path, or silent precision downgrade was introduced.

## Verification gate
Every mathematical station must pass the exact-head Rust kernel gate and comprehensive E2E/red-team gate before merge. Post-merge Rust + E2E must also be green on the resulting `main` commit.

For PR #23:
- exact-head Rust kernel validation: PASS;
- exact-head comprehensive E2E/red-team gate: PASS;
- post-merge Rust kernel validation on `201637b7...`: PASS;
- post-merge comprehensive E2E/red-team gate on `201637b7...`: PASS.

For PR #24:
- exact-head Rust kernel validation: PASS;
- exact-head comprehensive E2E/red-team gate: PASS;
- post-merge Rust kernel validation on `f014d8fd...`: PASS;
- post-merge comprehensive E2E/red-team gate on `f014d8fd...`: PASS.

For PR #25:
- exact-head Rust kernel validation: PASS;
- exact-head comprehensive E2E/red-team gate: PASS;
- post-merge Rust kernel validation on `ceea20ba...`: PASS;
- post-merge comprehensive E2E/red-team gate on `ceea20ba...`: PASS.

For PR #27:
- exact-head Rust kernel validation: PASS;
- exact-head comprehensive E2E/red-team gate: PASS;
- post-merge Rust kernel validation on `8fea6063...`: PASS;
- post-merge comprehensive E2E/red-team gate on `8fea6063...`: PASS.

M10 family status: **Implemented / Tested / CPU-validated by authoritative CI**.

M8 family status: **Implemented / Tested / CPU-validated by authoritative CI** for its declared certified construction domains.

M9 family status at PR #30 head: **Implemented / Tested** for its declared certified B-Rep/solid domains. The implementation head `9d04c05f67f4e5542df56b8612806fe8296f624b` includes bounded AABB tolerance hardening; its post-change Rust + E2E gates are now mandatory before merge.

The comprehensive E2E gate exercises the repository's Rust, typed API, .NET, black-box HTTP, Demo, raw HTTP red-team, release-path, and ignored-test checks.

## Immediate continuation
1. Verify PR #30's final exact-head Rust and comprehensive E2E/red-team gates on the documentation head, then merge it.
2. Verify post-merge main Rust and comprehensive E2E/red-team gates on the resulting main commit.
3. After post-merge green, begin M12 GPU abstraction from the updated main head.
4. M15 is merged and post-merge green on `b88862004debfda016d11c3d7da4dced0501065a`; begin M16 from that verified `main` head. CPU `f64` remains the semantic reference and no precision/fast-math shortcut may redefine authority.

## M11 completion boundary

M11 is now documented as `Implemented / Tested` for its declared certified domain. Current branch head `0db7d42694465b9d7146c90b99ba1c62c51fdc11` records the closure in `kernel/math/GAP_MATRIX.md`.

The certified scope is adaptive curve/surface tessellation; convex line/arc trim-aware outer-loop tessellation with preserved UV boundary samples, explicit interior classification, deterministic refinement, and chord/angular/parameter metadata; conservative AABB/bounding-sphere volumes; deterministic parameter-space bounds and 8-way spatial subdivision; and deterministic BVH query/candidate traversal with brute-force equivalence coverage.

The closure deliberately excludes general concave/holed/freeform trim filling, exact curvature-bound certification, and OBB as a semantic requirement. OBB is deferred because the current certified acceleration workload has no measured need that justifies another numerical authority.

Exact-head Rust and comprehensive E2E/red-team validation are green on `c9e542bb99122fa15fb86e8e86049de1b8ee5ca8`. The immediately preceding implementation head `f585dddc83471556fc8950e9138cd53f05bb6261` was also green in both authoritative gates.

## M9 completion boundary

M9 is closed for the certified explicit-topology planar-face/solid domains recorded in `kernel/math/GAP_MATRIX.md`. General curved-face sewing, unrestricted topology-aware Boolean construction, and exact holed-solid decomposition require separate mathematical contracts and are fail-closed where not certified. Bounded AABB Boolean/split/imprint math is certified, including sub-unit and translated-scale tolerance behavior.

## M8 completion boundary

M8 is closed for the certified domains recorded in `kernel/math/GAP_MATRIX.md`. Arbitrary freeform offset/fillet/chamfer/topology construction requires new explicit mathematical contracts and is not silently included.

## M10 completion boundary

M10 is now treated as a closed roadmap family for the current supported equation/solver scope. Any future new constraint/relation or solver mode requires a new mathematical contract and its own authority tests; it is not silently added under the closed M10 status.

## Non-negotiable authority rules
- CPU math is normative.
- Existing `nalgebra` remains numerical implementation infrastructure; do not introduce a second library without a demonstrated architectural need and a compatible proof/verification plan.
- GPU is acceleration only, never semantic authority.
- Unsupported, ambiguous, singular, degenerate and indeterminate cases fail closed.
- Semantic state and inputs remain immutable/stateless.
- Never claim hardware validation unless it actually ran.
- A green milestone is not completion of M0-M16.
- Never weaken a mathematical contract merely to obtain green CI.

## Continuation protocol
Before starting the next station:
1. Read `README.md`, `docs/MATH_AUTHORITY_ROADMAP.md`, this handoff, the current math-module sources, and the relevant tests.
2. Establish the exact `main` head and current authoritative CI state.
3. Change only the mathematical capability required by the next station, plus technically required native build/test exposure.
4. Test independently, then through native and repository-wide gates.
5. Merge only the exact validated head.
6. Validate both authoritative gates again on the resulting `main` commit.
7. Update this handoff with the exact commit, capability state, tests, and remaining work.

## Final M0-M16 acceptance reminder
Mathematical-authority completion still requires explicit reporting of:
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

## M12 completion boundary

M12 is now documented as `Implemented / Tested` for its declared backend-neutral abstraction domain. The validated implementation head is `8a3043b5c8e216845d281dfe6c6f620d30d868cb`.

The M12 abstraction defines conceptual Auto/CPU/Metal/CUDA backends; explicit capability discovery; f64 precision and determinism requirements; automatic/preferred/explicit backend selection with diagnostic CPU fallback; checked batch-memory sizing and transfer accounting; immutable workgroup/dispatch planning; a backend-neutral executor submission interface; and CPU-versus-accelerator f64 conformance comparison with explicit tolerances and fail-closed mismatch/non-finite handling.

No Metal or CUDA hardware backend is implemented or hardware-validated by M12. No GPU handle/resource enters mathematical semantic state. The deprecated legacy selector remains only for compatibility; authority code uses the diagnostic selection result. M13 adds real Apple Silicon Metal execution; M14 adds real NVIDIA CUDA execution; M15 adds cross-backend hardware conformance.

The source implementation was green in the authoritative Rust kernel and comprehensive E2E/red-team gates before this documentation closure. The documentation-inclusive branch head must pass both gates again before PR #32 is merged.

## M13 completion boundary

M13 is now documented as `Implemented / Tested / Hardware-validated` for its declared Apple Silicon Metal domain. The validated implementation head was `625fae34f858c940085712ba315cff8a57b26abb`; M13 post-merge `main` validation passed on `7b55ee99a3ae122dbbff398df5287c8e020bb726`.

The Metal backend lives at `kernel/native/src/gpu/metal.rs` and uses the modern `objc2-metal` binding. It implements a real compute pipeline for conservative AABB candidate generation. Authoritative CPU `f64` AABBs are converted with outward-rounded `f32` bounds only for this broad-phase workload; the GPU output is required to contain every exact CPU `f64` overlap, and the adapter fails closed on a false negative. Candidate pairs are reconstructed deterministically on the CPU, and the hardware test repeats the same workload to verify stable output.

Apple Metal's lack of native hardware `f64` is represented explicitly in the backend capability. M13 therefore does not claim Metal execution of authoritative f64 vector/matrix/transform/NURBS geometry and does not silently downgrade those semantics. Those remain CPU-authoritative.

Exact-head validation on `b32e8c9...`: Rust kernel workflow #492 PASS; comprehensive E2E/red-team workflow #414 PASS; Metal hardware workflow #11 PASS on the `macos-14` Apple Silicon runner. Earlier failures on the same milestone were corrected before closure and are not part of the validated head.

M14 is the current CUDA backend milestone; M15 remains cross-backend conformance; M16 remains final performance/crossover and red-team closure.

## M14 completion boundary

M14 is implemented and tested for the declared CUDA AABB candidate-generation domain. The CPU f64 implementation remains authoritative. CUDA uses device-native f64 for the certified predicate; candidate ordering is reconstructed deterministically on the CPU; exact CPU overlap omission fails closed. The CUDA driver/NVRTC absence path is explicitly handled as unavailable rather than panic-prone. Hardware validation is **Not yet hardware-validated** because the repository has no confirmed NVIDIA runner execution record for this milestone.

## M14 post-merge closure record

M14 is closed on `main` for the declared NVIDIA CUDA acceleration domain at merge commit `f5095cdf53e5db1ef610c8e2fd3620920456e2da`.

- PR #34 exact documentation-inclusive head: `f4009a4abd17ad4bc78de11e216a7fcac70f1e8a`.
- PR #34 exact-head authoritative gates: Rust kernel PASS, comprehensive E2E/red-team PASS, Metal hardware PASS.
- Post-merge `main` authoritative gates on `f5095cdf53e5db1ef610c8e2fd3620920456e2da`: Rust kernel PASS, comprehensive E2E/red-team PASS, Metal hardware PASS.
- CUDA implementation: real device-native-f64 AABB candidate generation via `cudarc`, CPU f64 acceptance/reference authority, deterministic CPU reconstruction, fail-closed false-negative detection, and explicit no-driver handling.
- Normal repository gates contain zero ignored CUDA tests; hardware cases are feature-gated behind `cuda-hardware` and must be explicitly enabled on an NVIDIA runner.
- CUDA hardware status: **Not yet hardware-validated**. No confirmed NVIDIA hardware execution record was available during M14 closure.
- M15 cross-backend conformance is now closed for the declared common AABB candidate-generation workload; M16 remains final performance/crossover and full red-team closure.

## M15 post-merge closure record

M15 is closed on `main` for the declared common CPU/Metal/CUDA AABB candidate-generation workload at merge commit `b88862004debfda016d11c3d7da4dced0501065a`.

- PR #35 validated implementation head: `a6eded2c7de66b6708e819e17916d82798c26a8a`.
- PR #35 exact-head authoritative gates: Rust kernel PASS; comprehensive E2E/red-team PASS; Metal hardware PASS.
- Post-merge `main` authoritative gates on `b88862004debfda016d11c3d7da4dced0501065a`: Rust kernel PASS; comprehensive E2E/red-team PASS; Metal hardware PASS.
- The backend-neutral conformance contract reports false negatives separately from permitted broad-phase false positives and verifies repeat determinism.
- One canonical adversarial fixture is reused for CPU reference, Metal, and CUDA paths.
- Metal hardware executes and validates the shared fixture against the CPU f64 reference on Apple Silicon CI.
- CUDA is wired to the same conformance suite behind the explicit `cuda-hardware` feature, but no NVIDIA hardware execution record is currently available; CUDA remains **Not yet hardware-validated**.
- The closure does not certify cross-backend equivalence for the broader vector/matrix/transform/NURBS stack; those operations remain CPU-authoritative until additional common hardware contracts are established.

## M16 completion boundary

M16 is now the final roadmap family and is closed on the M16 branch after exact-head Rust, comprehensive E2E/red-team, Metal hardware, and release-performance validation all passed.

The final validated branch head is `9257d508676c1b20dde7dc7528e3b8a64ed05825`.

M16 performance evidence on Apple Silicon (`macos-14`) measured the declared AABB candidate-generation workload at batches 32, 64, 128, 256, and 512. Median CPU/Metal latencies were respectively 1.25/745.79 µs, 4.79/1260.67 µs, 17.33/1136.58 µs, 61.88/2035.33 µs, and 229.75/10602.17 µs. No CPU/Metal crossover occurred in that tested range. The report also captured host preparation, buffer setup/upload, device execution, readback, CPU post-processing, transfer/setup overhead, throughput, and batch efficiency.

The final M16 red-team matrix covers:
- scale invariance across 1e-12 through 1e12;
- near-parallel, tangent, and coincident intersection behavior;
- non-finite intersection inputs;
- extreme positive finite NURBS weights and invalid parameter/weight domains;
- rank-deficient linear systems and deterministic GPU candidate ordering;
- malformed/duplicate GPU candidate sets.

M16 CUDA status remains **hardware-unverified** because no NVIDIA runner was available for execution. No claim of CUDA hardware performance or CUDA hardware conformance is made.

M0-M16 is now closed as a roadmap of certified mathematical capabilities, not as a claim that every conceivable CAD operation is complete. Future changes to the mathematical authority layer require new explicit capability contracts and fresh gates.