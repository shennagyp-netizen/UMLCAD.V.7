# UMLCAD.V.7 Continuation Handoff

## Snapshot
- Repository: `shennagyp-netizen/UMLCAD.V.7`
- Default branch: `main`
- Completed milestone branch: `milestone/solver-analytic-relation-jacobian`
- Completed milestone head: `b0561bfabff6e6d4b60fae7fecfead00bf8e6b05`
- This handoff records the state immediately before advancing `main`.

## Completed mathematical-authority state
The hardened math station and comprehensive cross-layer E2E gate are merged to `main`.

The V7 math program has strong implemented/tested foundations across M1-M7, M11 spatial/tessellation, and M12 GPU abstraction groundwork. The full M0-M16 roadmap is not complete.

## Completed M10 relation-Jacobian station
`kernel/math/relation_jacobian.rs` is the analytic authority for the current relation residual equations. Central differences are used only as an independent verification oracle. Nondifferentiable configurations fail closed as `Indeterminate`.

`kernel/math/jacobian.rs` now emits the complete production Jacobian in exact residual order:
1. analytic constraint rows;
2. analytic relation rows.

The existing production solver therefore consumes analytic relation derivatives directly for the supported relation set. The former production finite-difference relation fallback is no longer used for these current equations.

`kernel/native/tests/defect_regressions.rs` contains a relation-only radius solve that deliberately sets `finite_difference_step = f64::MAX`; it converges, providing regression evidence that the production relation path is independent of finite-difference probing.

## Validation achieved
For milestone head `b0561bfabff6e6d4b60fae7fecfead00bf8e6b05`:
- Rust kernel validation: green, including `137 passed; 0 failed; 0 ignored` in the library tests and `23 passed; 0 failed; 0 ignored` in aggressive integration coverage.
- Comprehensive E2E: green, including full Rust debug/release, typed Rust API E2E, .NET framework discovery/full suite, production black-box HTTP E2E, Demo E2E, and raw HTTP red-team checks.
- The E2E run used Python 3.13, .NET 10.0.401, and Rust 1.98.1 on GitHub Actions.

## Immediate continuation
1. Fast-forward/merge the validated milestone to `main` and verify post-merge Rust + E2E.
2. Continue M10 with explicit solver status semantics for singular/indeterminate cases.
3. Strengthen rank, conditioning, convergence and step-acceptance verification.
4. Expand analytic relation/constraint coverage and adversarial cases.
5. Continue remaining M8/M9 construction and B-Rep mathematical authority.
6. Then advance GPU conformance/performance/final-red-team M13-M16.

## Non-negotiable authority rules
- CPU math is normative.
- OCCT is an optional conformance oracle/backend, never semantic authority.
- GPU must not silently downgrade precision or semantics.
- Unsupported, ambiguous, singular, degenerate and indeterminate cases fail closed.
- Semantic state and inputs remain immutable/stateless.
- Never claim hardware validation unless it actually ran.
- A green milestone is not completion of M0-M16.
