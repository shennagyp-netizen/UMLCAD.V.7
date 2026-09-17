# UMLCAD.V.7 Continuation Handoff

## Snapshot
- Repository: `shennagyp-netizen/UMLCAD.V.7`
- Default branch: `main`
- Current milestone branch: `milestone/solver-status-authority`
- Previous completed milestone: `milestone/solver-analytic-relation-jacobian`

## Completed mathematical-authority state
The hardened math station and comprehensive cross-layer E2E gate are merged to `main` and post-merge green.

The V7 math program has strong implemented/tested foundations across M1-M7, M11 spatial/tessellation, and M12 GPU abstraction groundwork. The full M0-M16 roadmap is not complete.

## Completed M10 relation-Jacobian station
`kernel/math/relation_jacobian.rs` is the analytic authority for the current relation residual equations. Central differences are used only as an independent verification oracle. Nondifferentiable configurations fail closed as `Indeterminate`.

`kernel/math/jacobian.rs` emits the production Jacobian in exact residual order: analytic constraint rows followed by analytic relation rows. The production solver therefore uses analytic relation derivatives for the currently supported relation set.

A regression in `kernel/native/tests/defect_regressions.rs` sets `finite_difference_step = f64::MAX` in a relation-only radius solve and still converges, proving the supported production relation solve is independent of finite-difference probing.

## Current M10 status-authority station
`kernel/math/solver_status.rs` adds a conservative typed `SolverStatus` authority around the existing immutable `ConstraintSolveResult` evidence. It distinguishes currently provable states such as `Converged`, `ConvergedWithWarning`, `Diverged`, `Singular`, `IllConditioned`, `MaxIterations`, and `Indeterminate` without fabricating unsupported `Inconsistent`, `InvalidInput`, or `Cancelled` states from insufficient evidence.

The classifier is diagnostic-only in this station: numerical solver control flow is unchanged. Four unit tests cover healthy convergence, singular result evidence, no-progress max-iteration behavior, and non-finite/indeterminate evidence.

## Verification gate
Do not merge the status-authority milestone until both GitHub Actions gates are green on its exact final head:
- `.github/workflows/rust-kernel.yml`
- `.github/workflows/e2e.yml`, running `python3 tests/e2e/run.py --release --verbose`

The comprehensive E2E gate runs full Rust debug/release suites, typed Rust API E2E, .NET framework discovery/full suite, production black-box HTTP E2E, Demo E2E, raw HTTP red-team checks, and rejects ignored Rust tests.

## Immediate continuation after this station
1. Merge the fully validated status-authority station to `main` and verify post-merge Rust + E2E.
2. Continue M10 with explicit inconsistent/rank-revealing analysis where the mathematics can prove it.
3. Strengthen condition estimates, residual/step monotonicity, stagnation and convergence verification.
4. Remove or strictly type the remaining generic finite-difference fallback once all intended Jacobian equation families have authoritative derivatives.
5. Continue broader analytic relation/constraint adversarial coverage.
6. Then continue M8/M9 construction and B-Rep mathematical authority, followed later by M13-M16 GPU/conformance/performance/final-red-team work.

## Non-negotiable authority rules
- CPU math is normative.
- OCCT is an optional conformance oracle/backend, never semantic authority.
- GPU must not silently downgrade precision or semantics.
- Unsupported, ambiguous, singular, degenerate and indeterminate cases fail closed.
- Semantic state and inputs remain immutable/stateless.
- Never claim hardware validation unless it actually ran.
- A green milestone is not completion of M0-M16.
