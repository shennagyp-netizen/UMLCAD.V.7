use umlcad_kernel_rust::math::convergence::{verify_terminal, TerminalConvergenceStatus};

#[test]
fn terminal_convergence_requires_both_residual_and_step() {
    let residual_only = verify_terminal(1.0e-12, 1.0, 1.0e-8, 1.0e-10, 5);
    assert_eq!(residual_only.status, TerminalConvergenceStatus::NotConverged);
    assert!(residual_only.residual_satisfied);
    assert!(!residual_only.step_satisfied);

    let both = verify_terminal(1.0e-12, 1.0e-12, 1.0e-8, 1.0e-10, 5);
    assert_eq!(both.status, TerminalConvergenceStatus::Converged);
    assert!(both.residual_satisfied);
    assert!(both.step_satisfied);
}

#[test]
fn terminal_small_step_with_unsatisfied_residual_is_stagnation() {
    let evidence = verify_terminal(1.0, 1.0e-12, 1.0e-8, 1.0e-10, 5);
    assert_eq!(evidence.status, TerminalConvergenceStatus::Stagnated);
}

#[test]
fn terminal_invalid_evidence_fails_closed() {
    for (residual, step, residual_tol, step_tol) in [
        (f64::NAN, 0.0, 1.0e-8, 1.0e-10),
        (1.0, f64::INFINITY, 1.0e-8, 1.0e-10),
        (1.0, -1.0, 1.0e-8, 1.0e-10),
        (1.0, 0.0, f64::NAN, 1.0e-10),
        (1.0, 0.0, 1.0e-8, -1.0),
    ] {
        assert_eq!(
            verify_terminal(residual, step, residual_tol, step_tol, 5).status,
            TerminalConvergenceStatus::Indeterminate
        );
    }
}

#[test]
fn terminal_iteration_count_is_reported_without_redefining_solver_iterations() {
    let evidence = verify_terminal(1.0e-12, 1.0e-12, 1.0e-8, 1.0e-10, 17);
    assert_eq!(evidence.iterations, 17);
}
