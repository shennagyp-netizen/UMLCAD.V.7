# UMLCAD.V.7 Continuation Handoff

## Snapshot
- Repository: `shennagyp-netizen/UMLCAD.V.7`
- Default branch: `main`
- Main head: `a9922e8ad64affe2a43781e98e84f2e0f6421132`
- Current milestone branch: `milestone/solver-evidence-authority`
- Previous completed station: `milestone/analytic-jacobian-complete`

## Completed mathematical-authority state
Main is post-merge green through the current M10 mathematical-authority stations. The V7 program is not complete: the full M0-M16 roadmap remains active.

Completed and validated M10 foundations now include:
- analytic constraint Jacobian authority;
- analytic relation Jacobian authority;
- production solver use of the complete analytic Jacobian for the supported equation set;
- rank/nullity and augmented-rank consistency authority;
- conditioning authority with explicit conventional singular-system semantics;
- convergence/stagnation authority for explicit residual and step evidence;
- terminal convergence verification without manufacturing rejected-step history;
- solver-status authority with fail-closed rank/conditioning semantics;
- linear-solver conformance tests against the independent consistency authority;
- zero-damping truncated pseudoinverse handling for numerically null singular directions.

## Analytic Jacobian policy
`kernel/math/jacobian.rs` is the production Jacobian authority in exact residual order: analytic constraint rows followed by analytic relation rows. The production solver no longer falls back to generic finite differences for the supported equation set.

`SolveOptions::finite_difference_step` is retained for API compatibility, but it is not used for production Jacobian generation. Central/finite differences remain independent verification techniques only.

## Linear and conditioning authority
`kernel/math/linear_consistency.rs` distinguishes unique, underdetermined-consistent, inconsistent, and indeterminate systems using coefficient rank versus augmented rank under the declared numerical tolerance.

`kernel/math/conditioning.rs` provides explicit conventional conditioning evidence. Rank-deficient systems are not treated as conventionally finite-conditioned merely because a reduced nonzero singular spectrum has finite spread.

`kernel/math/solver_status.rs` consumes these proofs conservatively: `Inconsistent` is only claimed from explicit dimension-matched inconsistency evidence, and rank-deficient systems cannot be promoted to healthy conditioning.

## Convergence authority
`kernel/math/convergence.rs` contains two complementary contracts:
- `evaluate` verifies accepted residual history, monotonicity, stagnation, progress, iteration cap, and joint residual/step convergence;
- `verify_terminal` verifies final residual and final step evidence without inventing an iteration history when an implementation rejects trial steps.

The production solver has not yet changed its iteration-control algorithm to store/replay full accepted-step history. That integration is the next controlled station.

## Verification gate
Every mathematical station must pass the exact-head Rust kernel gate and comprehensive E2E/red-team gate before merge. Post-merge Rust + E2E must also be green on the resulting `main` commit.

The comprehensive E2E gate exercises the repository's Rust, typed API, .NET, black-box HTTP, Demo, raw HTTP red-team, release-path, and ignored-test checks.

## Immediate continuation
1. Integrate truthful solver-result step evidence with `ConstraintSolveResult` without redefining attempted-iteration counts or silently mixing physical units.
2. Use that evidence to make terminal convergence certification authoritative in the production solver, while preserving solver-state immutability and fail-closed behavior.
3. Extend analytic Jacobian coverage for every remaining supported relation/constraint family; any unsupported equation must fail explicitly rather than re-enter finite differences.
4. Expand adversarial rank/conditioning/convergence tests around numerical boundaries and deterministic behavior.
5. Complete the M10 solver foundation, then advance to M8/M9 construction and B-Rep mathematical authority.
6. Later evaluate GPU acceleration only as a conformance-tested implementation backend; CPU `f64` remains the semantic reference and no precision/fast-math shortcut may silently redefine authority.

## Non-negotiable authority rules
- CPU math is normative.
- Existing `nalgebra` remains numerical implementation infrastructure; do not introduce a second library without a demonstrated architectural need and a compatible proof/verification plan.
- GPU is acceleration only, never semantic authority.
- Unsupported, ambiguous, singular, degenerate and indeterminate cases fail closed.
- Semantic state and inputs remain immutable/stateless.
- Never claim hardware validation unless it actually ran.
- A green milestone is not completion of M0-M16.
