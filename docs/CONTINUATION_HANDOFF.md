# UMLCAD.V.7 Continuation Handoff

## Snapshot
- Repository: `shennagyp-netizen/UMLCAD.V.7`
- Default branch: `main`
- Current development branch: `milestone/solver-analytic-relation-jacobian`
- Branch head after the handoff update: this commit is the latest branch head.

## Current mathematical-authority state
The previous hardened math station is already merged to `main` and verified by both Rust validation and the comprehensive Python/.NET/HTTP/Demo/red-team gate.

The current V7 mathematical work has strong implemented/tested foundations across M1-M7, M11 spatial/tessellation, and M12 GPU abstraction groundwork. The full M0-M16 roadmap is not complete.

## M10 station in progress
`kernel/math/relation_jacobian.rs` is the analytic mathematical authority for the current relation residual equations. Its derivatives are independently checked with central finite differences, but finite differences are verification evidence only and are not used by the production implementation.

`kernel/math/jacobian.rs` now composes the complete production Jacobian in the exact residual order:
1. analytic constraint rows;
2. analytic relation rows.

The existing solver Jacobian hook therefore consumes analytic relation derivatives directly. The former production finite-difference relation fallback is removed for the currently supported relation set without changing the solver control-flow algorithm.

`kernel/native/tests/defect_regressions.rs` now contains a relation-only radius solve that sets `finite_difference_step = f64::MAX`; convergence must still succeed. This is a regression proving the production solve is independent of finite-difference probing.

## Verification gate
The milestone must not be merged until both GitHub Actions gates are green on the PR merge context:
- `.github/workflows/rust-kernel.yml`
- `.github/workflows/e2e.yml`, running `python3 tests/e2e/run.py --release --verbose`

The Python E2E gate runs full Rust debug/release suites, typed Rust API E2E, .NET framework discovery and full suite, production black-box Rust HTTP E2E, Demo E2E, and raw HTTP red-team checks. The runner rejects ignored Rust tests.

## Immediate continuation
After this solver-integration milestone is green and merged:
- continue M10 with explicit solver status semantics for singular/indeterminate cases;
- strengthen rank, conditioning, convergence and step-acceptance verification;
- expand analytic relation/constraint coverage and adversarial cases;
- then continue remaining M8/M9 construction and B-Rep mathematical authority;
- only afterward advance GPU conformance/performance/final-red-team M13-M16.

## Non-negotiable authority rules
- CPU math remains normative.
- OCCT is an optional conformance oracle/backend, never semantic authority.
- GPU must not silently downgrade precision or semantics.
- Unsupported, ambiguous, singular, degenerate and indeterminate cases fail closed.
- Semantic state and inputs remain immutable/stateless.
- Never claim hardware validation that did not run.
- Do not equate a green milestone with completion of the whole M0-M16 roadmap.
