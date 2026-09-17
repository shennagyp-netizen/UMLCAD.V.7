# UMLCAD.V.7 Continuation Handoff

## Snapshot
- Repository: `shennagyp-netizen/UMLCAD.V.7`
- Default branch: `main`
- Current completed baseline: hardened math + comprehensive Python E2E gate, merged on `main` before the analytic relation-Jacobian work.
- Current development branch: `milestone/solver-analytic-relation-jacobian`
- Current branch head: `f24d72825c3546d88884ff73fc8555c2c5704196`

## Mathematical authority state
The current V7 work is a mathematical-authority hardening program, not a feature-expansion phase.

Completed/tested foundations include:
- M1 scalar/constants/tolerance hardening.
- M2 vectors, matrices, quaternions, transforms, LU/QR/SVD/pseudoinverse/null-space and rank/conditioning diagnostics.
- M3 analytic geometry and predicates.
- M4 polynomial/root and interval foundations.
- M5 curve evaluation and differential foundations.
- M6 NURBS curve/surface evaluation, differential geometry, knot operations and conservative geometric contracts in the currently supported scope.
- M7 analytic distance/projection/intersection foundations.
- M11 spatial acceleration/tessellation foundations.
- M12 CPU/GPU execution abstraction groundwork.

M10 solver hardening is actively being completed. The analytic constraint Jacobian was already authoritative; the new relation Jacobian is now integrated into the same production Jacobian hook.

## This milestone
### Analytic relation Jacobian
`kernel/math/relation_jacobian.rs` is the mathematical authority for the current relation residual equations. It uses explicit fail-closed `Indeterminate` handling at nondifferentiable configurations. Its rows are independently checked against central-difference verification tests; finite differences are test evidence only.

### Solver integration
`kernel/math/jacobian.rs` now emits:
1. analytic constraint Jacobian rows;
2. analytic relation Jacobian rows;
3. one combined row set matching solver residual ordering.

This means the production solver no longer needs finite-difference relation rows for the currently supported relation set. The existing solver Jacobian hook consumes the combined authority without changing solver control flow.

### Regression coverage
`kernel/native/tests/defect_regressions.rs` now verifies a relation-only radius solve while deliberately setting `finite_difference_step = f64::MAX`. The solve must still converge, proving that the production path is independent of finite-difference probing.

## Verification gate
The authoritative CI gate is:
- Rust kernel validation (`cargo test` through `.github/workflows/rust-kernel.yml`).
- Comprehensive Python E2E (`python3 tests/e2e/run.py --release --verbose`) through `.github/workflows/e2e.yml`.

The comprehensive E2E gate runs full Rust debug and release suites, typed Rust API E2E, .NET framework discovery/full suite, production black-box Rust HTTP E2E, Demo E2E, raw HTTP red-team checks, and rejects ignored Rust tests.

Do not merge this milestone until both required GitHub Actions gates are green on the actual PR merge context.

## Immediate next step
1. Validate PR for `milestone/solver-analytic-relation-jacobian` against `main`.
2. If green, merge the milestone to `main`.
3. Verify post-merge `main` Rust + comprehensive E2E.
4. Continue M10 with explicit solver-status semantics, rank/conditioning verification, convergence correctness, and broader relation/constraint adversarial cases.
5. Then address remaining M8/M9 mathematical construction/B-Rep coverage before GPU/conformance M13-M16.

## Important constraints
- CPU mathematical implementation remains normative.
- OCCT is an optional conformance oracle/backend, never semantic authority.
- GPU must not silently change precision/semantics.
- Unsupported, ambiguous, singular, degenerate and indeterminate cases must fail closed.
- Inputs and semantic snapshots remain immutable/stateless.
- Never claim a hardware validation that has not actually run.
- Do not treat green CI as completion of the full M0-M16 math roadmap.
