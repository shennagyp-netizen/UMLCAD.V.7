# UMLCAD.V.7 Continuation Handoff

## Snapshot
- Repository: `shennagyp-netizen/UMLCAD.V.7`
- Default branch: `main`
- Main head at handoff update: `f014d8fdc7263bc0bb1fe704efeba857086e40fa` before this documentation commit
- Latest completed implementation station: M10/P0 authoritative accepted-step convergence history
- Latest merged implementation PR: #24, `math: make accepted-step history authoritative`
- PR #24 merge commit: `f014d8fdc7263bc0bb1fe704efeba857086e40fa`

## Completed mathematical-authority state
Main is post-merge green through the current M10 solver-authority stations. The V7 mathematical-authority program is not complete; the M0-M16 roadmap remains active.

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

The comprehensive E2E gate exercises the repository's Rust, typed API, .NET, black-box HTTP, Demo, raw HTTP red-team, release-path, and ignored-test checks.

## Immediate continuation
1. Add adversarial mixed-unit nonlinear fixtures that exercise row scaling together with coupled distance/angle/relation equations across the declared geometry-scale range.
2. Expand numerical-boundary tests for rank transitions, conditioning, damping, stagnation, accepted-history behavior, and deterministic repeated solves.
3. Audit every remaining supported constraint/relation family for explicit analytic Jacobian coverage; unsupported equations must fail explicitly rather than re-enter finite differences.
4. Review the solver's non-convergence classification so rejected-trial, stagnation, singular, invalid-domain, and max-iteration states remain explicit and mathematically distinguishable.
5. Finish the remaining M10 solver foundation, then advance to the M8/M9 construction and B-Rep mathematical-authority families according to `docs/MATH_AUTHORITY_ROADMAP.md`.
6. Evaluate GPU acceleration only later as a conformance-tested implementation backend; CPU `f64` remains the semantic reference and no precision/fast-math shortcut may redefine authority.

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
