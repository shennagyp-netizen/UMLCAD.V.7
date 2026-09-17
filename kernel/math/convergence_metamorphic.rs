use super::convergence::{verify_terminal, TerminalConvergenceStatus};

#[test]
fn terminal_residual_scaling_preserves_classification_when_tolerance_scales() {
    let residual = 2.5e-7;
    let residual_tolerance = 1.0e-8;
    let step = 5.0e-11;
    let step_tolerance = 1.0e-10;
    let base = verify_terminal(residual, step, residual_tolerance, step_tolerance, 9);

    for factor in [1.0e-12, 1.0e-6, 1.0, 1.0e6, 1.0e12] {
        let transformed = verify_terminal(
            residual * factor,
            step,
            residual_tolerance * factor,
            step_tolerance,
            9,
        );
        assert_eq!(transformed.status, base.status);
        assert_eq!(transformed.residual_satisfied, base.residual_satisfied);
        assert_eq!(transformed.step_satisfied, base.step_satisfied);
    }
}

#[test]
fn terminal_step_scaling_preserves_classification_when_tolerance_scales() {
    let residual = 2.5e-12;
    let residual_tolerance = 1.0e-8;
    let step = 5.0e-5;
    let step_tolerance = 1.0e-6;
    let base = verify_terminal(residual, step, residual_tolerance, step_tolerance, 9);

    for factor in [1.0e-12, 1.0e-6, 1.0, 1.0e6, 1.0e12] {
        let transformed = verify_terminal(
            residual,
            step * factor,
            residual_tolerance,
            step_tolerance * factor,
            9,
        );
        assert_eq!(transformed.status, base.status);
        assert_eq!(transformed.residual_satisfied, base.residual_satisfied);
        assert_eq!(transformed.step_satisfied, base.step_satisfied);
    }
}

#[test]
fn terminal_boundary_is_inclusive_for_both_criteria() {
    let evidence = verify_terminal(1.0e-8, 1.0e-10, 1.0e-8, 1.0e-10, 3);
    assert_eq!(evidence.status, TerminalConvergenceStatus::Converged);
    assert!(evidence.residual_satisfied);
    assert!(evidence.step_satisfied);
}

#[test]
fn terminal_status_regions_are_exhaustive_and_mutually_consistent() {
    let cases = [
        (
            1.0e-9,
            1.0e-11,
            TerminalConvergenceStatus::Converged,
            true,
            true,
        ),
        (
            1.0,
            1.0e-11,
            TerminalConvergenceStatus::Stagnated,
            false,
            true,
        ),
        (
            1.0e-9,
            1.0,
            TerminalConvergenceStatus::NotConverged,
            true,
            false,
        ),
        (
            1.0,
            1.0,
            TerminalConvergenceStatus::NotConverged,
            false,
            false,
        ),
    ];

    for (residual, step, expected_status, residual_ok, step_ok) in cases {
        let evidence = verify_terminal(residual, step, 1.0e-8, 1.0e-10, 4);
        assert_eq!(evidence.status, expected_status);
        assert_eq!(evidence.residual_satisfied, residual_ok);
        assert_eq!(evidence.step_satisfied, step_ok);
    }
}

#[test]
fn terminal_iteration_count_is_semantically_orthogonal_to_threshold_classification() {
    let values = (1.0e-7, 1.0e-12, 1.0e-8, 1.0e-10);
    let early = verify_terminal(values.0, values.1, values.2, values.3, 0);
    let late = verify_terminal(values.0, values.1, values.2, values.3, 1000);
    assert_eq!(early.status, late.status);
    assert_eq!(early.residual_satisfied, late.residual_satisfied);
    assert_eq!(early.step_satisfied, late.step_satisfied);
    assert_eq!(early.iterations, 0);
    assert_eq!(late.iterations, 1000);
}
