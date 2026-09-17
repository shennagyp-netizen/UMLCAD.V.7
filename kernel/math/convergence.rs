//! Explicit convergence evidence for iterative numerical methods.
//!
//! The authority is independent of the iteration implementation. It evaluates
//! finite residual/step evidence under caller-supplied tolerances and refuses
//! to claim convergence when the residual and step criteria are not jointly
//! satisfied. Accepted residual history is expected to be monotone
//! non-increasing; a violation is treated as divergence evidence.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConvergenceStatus {
    Converged,
    Progressing,
    Stagnated,
    MaxIterations,
    Diverged,
    Indeterminate,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ConvergenceEvidence {
    pub status: ConvergenceStatus,
    pub initial_residual: f64,
    pub final_residual: f64,
    pub final_step_norm: f64,
    pub iterations: usize,
    pub residual_satisfied: bool,
    pub step_satisfied: bool,
    pub residual_decreased: bool,
    pub monotone_nonincreasing: bool,
    pub progress_ratio: f64,
}

fn finite_tolerances(residual_tolerance: f64, step_tolerance: f64) -> bool {
    residual_tolerance.is_finite()
        && residual_tolerance >= 0.0
        && step_tolerance.is_finite()
        && step_tolerance >= 0.0
}

fn invalid_evidence(
    residual_history: &[f64],
    final_step_norm: f64,
    iterations: usize,
) -> ConvergenceEvidence {
    ConvergenceEvidence {
        status: ConvergenceStatus::Indeterminate,
        initial_residual: residual_history.first().copied().unwrap_or(f64::NAN),
        final_residual: residual_history.last().copied().unwrap_or(f64::NAN),
        final_step_norm,
        iterations,
        residual_satisfied: false,
        step_satisfied: false,
        residual_decreased: false,
        monotone_nonincreasing: false,
        progress_ratio: f64::NAN,
    }
}

/// Evaluate convergence from an explicit residual history and final step.
///
/// The accepted history contains residual norms at accepted iterates, including
/// the initial value and the final accepted value. Its length must therefore be
/// exactly `iterations + 1`. Convergence requires both a satisfied residual
/// criterion and, after at least one iteration, a satisfied step criterion.
/// An initially satisfied system is allowed to converge at zero iterations.
pub fn evaluate(
    residual_history: &[f64],
    final_step_norm: f64,
    residual_tolerance: f64,
    step_tolerance: f64,
    iterations: usize,
    max_iterations: usize,
) -> ConvergenceEvidence {
    let expected_history_len = iterations.checked_add(1);
    let history_shape_valid = expected_history_len == Some(residual_history.len());
    let finite_history = !residual_history.is_empty()
        && residual_history.iter().all(|value| value.is_finite() && *value >= 0.0);
    let finite_inputs = history_shape_valid
        && finite_history
        && final_step_norm.is_finite()
        && final_step_norm >= 0.0
        && finite_tolerances(residual_tolerance, step_tolerance)
        && max_iterations > 0;
    if !finite_inputs {
        return invalid_evidence(residual_history, final_step_norm, iterations);
    }

    let initial_residual = residual_history[0];
    let final_residual = *residual_history.last().unwrap();
    let residual_satisfied = final_residual <= residual_tolerance;
    let step_satisfied = final_step_norm <= step_tolerance;
    let residual_decreased = final_residual < initial_residual;
    let monotone_nonincreasing = residual_history
        .windows(2)
        .all(|pair| pair[1] <= pair[0]);
    let progress_ratio = if initial_residual > 0.0 {
        final_residual / initial_residual
    } else if final_residual == 0.0 {
        0.0
    } else {
        f64::INFINITY
    };

    let status = if !monotone_nonincreasing {
        ConvergenceStatus::Diverged
    } else if residual_satisfied && (iterations == 0 || step_satisfied) {
        ConvergenceStatus::Converged
    } else if step_satisfied && !residual_satisfied {
        ConvergenceStatus::Stagnated
    } else if iterations >= max_iterations {
        ConvergenceStatus::MaxIterations
    } else {
        ConvergenceStatus::Progressing
    };

    ConvergenceEvidence {
        status,
        initial_residual,
        final_residual,
        final_step_norm,
        iterations,
        residual_satisfied,
        step_satisfied,
        residual_decreased,
        monotone_nonincreasing,
        progress_ratio,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn already_satisfied_zero_iteration_system_converges() {
        let evidence = evaluate(&[0.0], 0.0, 1.0e-8, 1.0e-10, 0, 100);
        assert_eq!(evidence.status, ConvergenceStatus::Converged);
        assert!(evidence.residual_satisfied);
        assert!(evidence.step_satisfied);
    }

    #[test]
    fn residual_and_step_must_both_be_satisfied_after_iteration() {
        let residual_only = evaluate(&[1.0, 1.0e-12], 1.0, 1.0e-8, 1.0e-10, 1, 100);
        assert_eq!(residual_only.status, ConvergenceStatus::Progressing);

        let both = evaluate(&[1.0, 1.0e-12], 1.0e-12, 1.0e-8, 1.0e-10, 1, 100);
        assert_eq!(both.status, ConvergenceStatus::Converged);
    }

    #[test]
    fn mismatched_history_shape_fails_closed() {
        let evidence = evaluate(&[10.0, 9.0], 1.0e-12, 1.0e-8, 1.0e-10, 2, 100);
        assert_eq!(evidence.status, ConvergenceStatus::Indeterminate);
    }

    #[test]
    fn tiny_step_with_unsatisfied_residual_is_stagnation_with_matching_history() {
        let mut history = vec![10.0; 9];
        history[8] = 9.999999;
        let evidence = evaluate(&history, 1.0e-12, 1.0e-8, 1.0e-10, 8, 100);
        assert_eq!(evidence.status, ConvergenceStatus::Stagnated);
        assert!(!evidence.residual_satisfied);
        assert!(evidence.step_satisfied);
    }

    #[test]
    fn iteration_cap_is_reported_when_progress_exists_but_convergence_is_not_reached() {
        let evidence = evaluate(&[10.0, 9.0, 8.0, 7.0], 1.0e-2, 1.0e-8, 1.0e-10, 3, 3);
        assert_eq!(evidence.status, ConvergenceStatus::MaxIterations);
        assert!(evidence.residual_decreased);
    }

    #[test]
    fn broken_residual_monotonicity_is_divergence_evidence() {
        let evidence = evaluate(&[10.0, 5.0, 6.0], 1.0, 1.0e-8, 1.0e-10, 2, 100);
        assert_eq!(evidence.status, ConvergenceStatus::Diverged);
        assert!(!evidence.monotone_nonincreasing);
    }

    #[test]
    fn strict_residual_decrease_is_progress() {
        let evidence = evaluate(&[10.0, 8.0], 1.0, 1.0e-8, 1.0e-10, 1, 100);
        assert_eq!(evidence.status, ConvergenceStatus::Progressing);
        assert!(evidence.residual_decreased);
        assert!(evidence.progress_ratio < 1.0);
    }

    #[test]
    fn constant_nonzero_residual_with_large_step_is_not_stagnation() {
        let evidence = evaluate(&[10.0, 10.0, 10.0], 1.0, 1.0e-8, 1.0e-10, 2, 100);
        assert_eq!(evidence.status, ConvergenceStatus::Progressing);
        assert!(!evidence.residual_decreased);
    }

    #[test]
    fn invalid_tolerance_empty_history_or_history_shape_fails_closed() {
        for (residual_tolerance, step_tolerance) in [
            (f64::NAN, 1.0e-10),
            (f64::INFINITY, 1.0e-10),
            (-1.0, 1.0e-10),
            (1.0e-8, f64::NAN),
            (1.0e-8, f64::INFINITY),
            (1.0e-8, -1.0),
        ] {
            let evidence = evaluate(&[1.0], 0.0, residual_tolerance, step_tolerance, 0, 100);
            assert_eq!(evidence.status, ConvergenceStatus::Indeterminate);
        }
        assert_eq!(
            evaluate(&[], 0.0, 1.0e-8, 1.0e-10, 0, 100).status,
            ConvergenceStatus::Indeterminate
        );
        assert_eq!(
            evaluate(&[1.0, 0.5], 0.0, 1.0e-8, 1.0e-10, 2, 100).status,
            ConvergenceStatus::Indeterminate
        );
    }

    #[test]
    fn nonfinite_or_negative_norm_evidence_fails_closed() {
        for history in [vec![1.0, f64::NAN], vec![1.0, f64::INFINITY], vec![1.0, -1.0]] {
            assert_eq!(
                evaluate(&history, 0.0, 1.0e-8, 1.0e-10, 1, 100).status,
                ConvergenceStatus::Indeterminate
            );
        }
        for step in [f64::NAN, f64::INFINITY, -1.0] {
            assert_eq!(
                evaluate(&[1.0], step, 1.0e-8, 1.0e-10, 0, 100).status,
                ConvergenceStatus::Indeterminate
            );
        }
    }

    #[test]
    fn progress_ratio_reports_complete_reduction_to_zero() {
        let evidence = evaluate(&[5.0, 0.0], 0.0, 1.0e-8, 1.0e-10, 1, 100);
        assert_eq!(evidence.status, ConvergenceStatus::Converged);
        assert_eq!(evidence.progress_ratio, 0.0);
    }

    #[test]
    fn positive_scaling_of_residuals_and_residual_tolerance_preserves_status() {
        let base = evaluate(&[100.0, 50.0, 10.0], 1.0e-8, 1.0, 1.0e-7, 2, 100);
        let scaled = evaluate(&[1.0e8, 5.0e7, 1.0e7], 1.0e-8, 1.0e6, 1.0e-7, 2, 100);
        assert_eq!(base.status, scaled.status);
        assert!((base.progress_ratio - scaled.progress_ratio).abs() < 1.0e-15);
    }

    #[test]
    fn tightening_residual_tolerance_cannot_turn_nonconvergence_into_convergence() {
        let loose = evaluate(&[1.0, 1.0e-6], 1.0e-12, 1.0e-5, 1.0e-10, 1, 100);
        let tight = evaluate(&[1.0, 1.0e-6], 1.0e-12, 1.0e-7, 1.0e-10, 1, 100);
        assert_eq!(loose.status, ConvergenceStatus::Converged);
        assert_ne!(tight.status, ConvergenceStatus::Converged);
    }

    #[test]
    fn adding_a_smaller_accepted_residual_preserves_monotonicity() {
        let base = evaluate(&[10.0, 5.0], 1.0e-2, 1.0e-8, 1.0e-10, 1, 100);
        let extended = evaluate(&[10.0, 5.0, 2.0], 1.0e-2, 1.0e-8, 1.0e-10, 2, 100);
        assert!(base.monotone_nonincreasing);
        assert!(extended.monotone_nonincreasing);
        assert_eq!(base.residual_satisfied, extended.residual_satisfied);
        assert!(extended.progress_ratio < base.progress_ratio);
    }
}
