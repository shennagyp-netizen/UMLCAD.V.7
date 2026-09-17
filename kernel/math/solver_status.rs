//! Conservative semantic classification for solver results.
//!
//! This module does not alter the numerical solver. It converts the existing
//! immutable solver evidence into an explicit authority status. The classifier
//! consumes the private implementation result so the public terminal-authority
//! wrapper can embed the verified status without a module dependency cycle.
//! Classification is deliberately fail-closed: a status is only claimed when
//! the available evidence is sufficient to justify it.

use super::convergence::{TerminalConvergenceEvidence, TerminalConvergenceStatus};
use super::linear_consistency::{LinearConsistencyEvidence, LinearSystemStatus};
use super::solver_legacy::SolveReason;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SolverStatus {
    Converged,
    ConvergedWithWarning,
    Stagnated,
    Diverged,
    Singular,
    IllConditioned,
    Inconsistent,
    MaxIterations,
    Indeterminate,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SolverClassificationInput {
    pub converged: bool,
    pub reason: SolveReason,
    pub final_residual_norm: f64,
    pub final_scaled_residual_norm: f64,
    pub initial_scaled_residual_norm: f64,
    pub final_step_norm: f64,
    pub analysis_valid: bool,
    pub well_conditioned: bool,
    pub rank: usize,
    pub equation_count: usize,
    pub variable_count: usize,
    pub condition_estimate: f64,
    pub iterations: usize,
}

/// Convert the private numerical result into the backend-neutral classification contract.
pub(crate) fn input_from_legacy(
    result: &super::solver_legacy::ConstraintSolveResult,
) -> SolverClassificationInput {
    SolverClassificationInput {
        converged: input.converged,
        reason: input.reason.clone(),
        final_residual_norm: result.final_residual_norm,
        final_scaled_residual_norm: input.final_scaled_residual_norm,
        initial_scaled_residual_norm: result.initial_scaled_residual_norm,
        final_step_norm: input.final_step_norm,
        analysis_valid: result.analysis.valid,
        well_conditioned: result.analysis.well_conditioned,
        rank: result.analysis.rank,
        equation_count: input.equation_count,
        variable_count: input.variable_count,
        condition_estimate: result.analysis.condition_estimate,
        iterations: input.iterations,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SolverStatusEvidence {
    pub status: SolverStatus,
    pub terminal_status: Option<TerminalConvergenceStatus>,
    pub residual_reduced: bool,
    pub final_residual_finite: bool,
    pub final_step_finite: bool,
    pub final_step_norm: f64,
    pub condition_finite: bool,
    pub well_conditioned: bool,
    pub rank: usize,
    pub equation_count: usize,
    pub iterations: usize,
}

/// Classify a solver result using only evidence already present in the result.
///
/// The classifier is intentionally conservative:
/// - converged + healthy conditioning => `Converged`;
/// - converged + unhealthy conditioning => `ConvergedWithWarning`;
/// - singular reason => `Singular`;
/// - max iterations with a reduced finite residual => `MaxIterations`;
/// - max iterations with no residual reduction => `Diverged`;
/// - non-finite residual/step analysis => `Indeterminate`.
///
/// A finite nonzero singular-spectrum spread is not sufficient to claim that a
/// rank-deficient system is well-conditioned. Numerical rank must span the
/// smaller matrix dimension before the condition evidence is considered finite
/// for status classification.
///
/// `Inconsistent`, `InvalidInput`, and `Cancelled` need richer upstream evidence
/// than the current result type carries and are therefore not fabricated here.
pub fn classify(input: &SolverClassificationInput) -> SolverStatusEvidence {
    let final_residual_finite = input.final_residual_norm.is_finite()
        && input.final_scaled_residual_norm.is_finite()
        && input.analysis_valid;
    let final_step_finite = input.final_step_norm.is_finite() && input.final_step_norm >= 0.0;
    let residual_reduced = input.final_scaled_residual_norm < input.initial_scaled_residual_norm;
    let rank_complete = input.rank >= input.equation_count.min(input.variable_count);
    let condition_finite = input.condition_estimate.is_finite() && rank_complete;
    let well_conditioned = input.well_conditioned && rank_complete;

    let status = if !final_residual_finite || !final_step_finite {
        SolverStatus::Indeterminate
    } else {
        match input.reason {
            SolveReason::Converged if input.converged => {
                if well_conditioned {
                    SolverStatus::Converged
                } else {
                    SolverStatus::ConvergedWithWarning
                }
            }
            SolveReason::Singular => SolverStatus::Singular,
            SolveReason::MaxIterations => {
                if residual_reduced {
                    if condition_finite && !well_conditioned {
                        SolverStatus::IllConditioned
                    } else {
                        SolverStatus::MaxIterations
                    }
                } else {
                    SolverStatus::Diverged
                }
            }
            SolveReason::InvalidDomain => SolverStatus::Indeterminate,
            SolveReason::Converged => SolverStatus::Indeterminate,
        }
    };

    SolverStatusEvidence {
        status,
        terminal_status: None,
        residual_reduced,
        final_residual_finite,
        final_step_finite,
        final_step_norm: input.final_step_norm,
        condition_finite,
        well_conditioned,
        rank: input.rank,
        equation_count: input.equation_count,
        iterations: input.iterations,
    }
}

/// Refine the status using an independently verified terminal convergence
/// certificate.
///
/// The certificate is required to identify the same terminal residual, step,
/// and iteration count as the solver result. A mismatched certificate is treated
/// as indeterminate instead of being silently combined with the result.
///
/// This function is diagnostic only: it does not change solver iteration control
/// or introduce a new `SolveReason`. In particular, `Stagnated` means the
/// terminal certificate proved a small step while the residual remained above
/// tolerance; the production solver may still have continued beyond that state.
pub fn classify_with_terminal_convergence(
    input: &SolverClassificationInput,
    terminal: &TerminalConvergenceEvidence,
) -> SolverStatusEvidence {
    let mut evidence = classify(input);
    evidence.terminal_status = Some(terminal.status);

    let terminal_matches = terminal.iterations == input.iterations
        && terminal.final_residual.to_bits() == input.final_scaled_residual_norm.to_bits()
        && terminal.final_step_norm.to_bits() == input.final_step_norm.to_bits();
    if !terminal_matches {
        evidence.status = SolverStatus::Indeterminate;
        return evidence;
    }

    evidence.status = match terminal.status {
        TerminalConvergenceStatus::Converged => {
            if input.converged && input.reason == SolveReason::Converged {
                evidence.status
            } else {
                SolverStatus::Indeterminate
            }
        }
        TerminalConvergenceStatus::Stagnated => {
            if !input.converged && input.reason == SolveReason::MaxIterations {
                SolverStatus::Stagnated
            } else {
                SolverStatus::Indeterminate
            }
        }
        TerminalConvergenceStatus::NotConverged => {
            if input.converged || input.reason == SolveReason::Converged {
                SolverStatus::Indeterminate
            } else {
                evidence.status
            }
        }
        TerminalConvergenceStatus::Indeterminate => SolverStatus::Indeterminate,
    };

    evidence
}

/// Refine the diagnostic classification when an independent linear-consistency
/// authority has proved the current linearized system inconsistent.
///
/// The extra evidence is accepted only when its matrix dimensions agree with
/// the solver analysis and the solver has not reported convergence. This keeps
/// the API fail-closed: mismatched or insufficient evidence leaves the original
/// status untouched.
pub fn classify_with_linear_consistency(
    input: &SolverClassificationInput,
    consistency: &LinearConsistencyEvidence,
) -> SolverStatusEvidence {
    let mut evidence = classify(input);
    let dimensions_match = consistency.variable_count == input.variable_count
        && consistency.equation_count == input.equation_count;
    if dimensions_match
        && !input.converged
        && evidence.final_residual_finite
        && consistency.status == LinearSystemStatus::Inconsistent
    {
        evidence.status = SolverStatus::Inconsistent;
    }
    evidence
}

/// Compute the linear-consistency witness from an explicit linearization and
/// feed it into the solver-status authority.
///
/// This is intentionally separate from solver iteration control. Callers must
/// supply the exact Jacobian/residual linearization they want classified, and
/// the result is rejected rather than guessed when dimensions or numerical
/// inputs are invalid.
pub fn classify_with_linear_system(
    input: &SolverClassificationInput,
    jacobian: &[Vec<f64>],
    residual: &[f64],
    rank_tol: f64,
    ill_cond_threshold: f64,
) -> Result<SolverStatusEvidence, super::linalg::LinAlgError> {
    let matrix = super::linalg::from_rows(jacobian)?;
    let rhs = nalgebra::DVector::from_iterator(residual.len(), residual.iter().copied().map(|value| -value));
    let consistency = super::linear_consistency::classify_linear_system(
        &matrix,
        &rhs,
        rank_tol,
        ill_cond_threshold,
    )?;
    Ok(classify_with_linear_consistency(input, &consistency))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::{
        geometry::{Geometry, Line, Point},
        snapshot::{Constraint, GeometryItem, SemanticSnapshot},
    };
    use super::super::solver_legacy::{solve_snapshot, ConstraintSolveResult, SolveOptions};
    use nalgebra::{DMatrix, DVector};

    fn base_snapshot() -> SemanticSnapshot {
        SemanticSnapshot {
            parameters: vec![],
            geometry: vec![GeometryItem {
                id: "l".into(),
                geometry: Geometry::Line(Line {
                    start: Point { x: 0.0, y: 1.0 },
                    end: Point { x: 2.0, y: 2.0 },
                }),
                parameter_dependencies: vec![],
            }],
            constraints: vec![
                (
                    "horizontal".into(),
                    Constraint::Horizontal { entity_id: "l".into() },
                ),
            ],
            relations: vec![],
        }
        .deterministic()
    }

    fn inconsistent_evidence(result: &ConstraintSolveResult) -> LinearConsistencyEvidence {
        LinearConsistencyEvidence {
            status: LinearSystemStatus::Inconsistent,
            coefficient_rank: result.analysis.rank,
            augmented_rank: result.analysis.rank + 1,
            variable_count: result.variable_count,
            equation_count: result.equation_count,
            coefficient_condition_number: 1.0,
            coefficient_classification: super::super::linalg::RankClassification::FullRank,
        }
    }

    #[test]
    fn converged_well_conditioned_result_is_classified_as_converged() {
        let result = solve_snapshot(&base_snapshot(), SolveOptions::default()).unwrap();
        assert!(result.converged);
        let evidence = classify(&input_from_legacy(&result));
        assert_eq!(evidence.status, SolverStatus::Converged);
        assert_eq!(evidence.terminal_status, None);
        assert!(evidence.final_residual_finite);
        assert!(evidence.final_step_finite);
        assert!(evidence.final_step_norm >= 0.0);
    }

    #[test]
    fn converged_rank_deficient_result_is_warning_even_when_spread_is_finite() {
        let mut result = solve_snapshot(&base_snapshot(), SolveOptions::default()).unwrap();
        result.converged = true;
        result.reason = SolveReason::Converged;
        result.variable_count = 4;
        result.equation_count = 3;
        result.analysis.rank = 2;
        result.analysis.condition_estimate = 25.0;
        result.analysis.well_conditioned = true;
        let evidence = classify(&input_from_legacy(&result));
        assert_eq!(evidence.status, SolverStatus::ConvergedWithWarning);
        assert!(!evidence.condition_finite);
        assert!(!evidence.well_conditioned);
    }

    #[test]
    fn converged_full_row_rank_wide_result_can_remain_conditioned() {
        let mut result = solve_snapshot(&base_snapshot(), SolveOptions::default()).unwrap();
        result.converged = true;
        result.reason = SolveReason::Converged;
        result.variable_count = 4;
        result.equation_count = 3;
        result.analysis.rank = 3;
        result.analysis.condition_estimate = 25.0;
        result.analysis.well_conditioned = true;
        let evidence = classify(&input_from_legacy(&result));
        assert_eq!(evidence.status, SolverStatus::Converged);
        assert!(evidence.condition_finite);
        assert!(evidence.well_conditioned);
    }

    #[test]
    fn finite_residual_with_singular_reason_is_not_reported_as_converged() {
        let mut result = solve_snapshot(&base_snapshot(), SolveOptions::default()).unwrap();
        result.converged = false;
        result.reason = SolveReason::Singular;
        let evidence = classify(&input_from_legacy(&result));
        assert_eq!(evidence.status, SolverStatus::Singular);
    }

    #[test]
    fn max_iterations_without_progress_is_classified_as_diverged() {
        let mut result = solve_snapshot(&base_snapshot(), SolveOptions::default()).unwrap();
        result.converged = false;
        result.reason = SolveReason::MaxIterations;
        result.final_scaled_residual_norm = result.initial_scaled_residual_norm;
        let evidence = classify(&input_from_legacy(&result));
        assert_eq!(evidence.status, SolverStatus::Diverged);
    }

    #[test]
    fn nonfinite_analysis_is_indeterminate() {
        let mut result = solve_snapshot(&base_snapshot(), SolveOptions::default()).unwrap();
        result.analysis.valid = false;
        result.final_scaled_residual_norm = f64::NAN;
        let evidence = classify(&input_from_legacy(&result));
        assert_eq!(evidence.status, SolverStatus::Indeterminate);
        assert!(!evidence.final_residual_finite);
    }

    #[test]
    fn nonfinite_terminal_step_is_indeterminate() {
        let mut result = solve_snapshot(&base_snapshot(), SolveOptions::default()).unwrap();
        result.final_step_norm = f64::NAN;
        let evidence = classify(&input_from_legacy(&result));
        assert_eq!(evidence.status, SolverStatus::Indeterminate);
        assert!(!evidence.final_step_finite);
    }

    #[test]
    fn negative_terminal_step_is_indeterminate() {
        let mut result = solve_snapshot(&base_snapshot(), SolveOptions::default()).unwrap();
        result.final_step_norm = -1.0;
        let evidence = classify(&input_from_legacy(&result));
        assert_eq!(evidence.status, SolverStatus::Indeterminate);
        assert!(!evidence.final_step_finite);
    }

    #[test]
    fn terminal_convergence_certificate_can_confirm_convergence() {
        let result = solve_snapshot(&base_snapshot(), SolveOptions::default()).unwrap();
        let terminal = super::super::convergence::verify_terminal(
            result.final_scaled_residual_norm,
            result.final_step_norm,
            1.0e-8,
            1.0e-10,
            result.iterations,
        );
        assert_eq!(terminal.status, TerminalConvergenceStatus::Converged);
        let evidence = classify_with_terminal_convergence(&input_from_legacy(&result), &terminal);
        assert_eq!(evidence.status, SolverStatus::Converged);
        assert_eq!(evidence.terminal_status, Some(TerminalConvergenceStatus::Converged));
    }

    #[test]
    fn terminal_stagnation_certificate_refines_max_iterations() {
        let mut result = solve_snapshot(&base_snapshot(), SolveOptions::default()).unwrap();
        result.converged = false;
        result.reason = SolveReason::MaxIterations;
        result.final_scaled_residual_norm = result.initial_scaled_residual_norm.max(1.0);
        result.final_step_norm = 0.0;
        result.analysis.valid = true;
        let terminal = super::super::convergence::verify_terminal(
            result.final_scaled_residual_norm,
            result.final_step_norm,
            1.0e-8,
            1.0e-10,
            result.iterations,
        );
        assert_eq!(terminal.status, TerminalConvergenceStatus::Stagnated);
        let evidence = classify_with_terminal_convergence(&input_from_legacy(&result), &terminal);
        assert_eq!(evidence.status, SolverStatus::Stagnated);
    }

    #[test]
    fn mismatched_terminal_certificate_fails_closed() {
        let result = solve_snapshot(&base_snapshot(), SolveOptions::default()).unwrap();
        let mut terminal = super::super::convergence::verify_terminal(
            result.final_scaled_residual_norm,
            result.final_step_norm,
            1.0e-8,
            1.0e-10,
            result.iterations,
        );
        terminal.final_step_norm = terminal.final_step_norm + 1.0;
        terminal.status = TerminalConvergenceStatus::NotConverged;
        let evidence = classify_with_terminal_convergence(&input_from_legacy(&result), &terminal);
        assert_eq!(evidence.status, SolverStatus::Indeterminate);
        assert_eq!(evidence.terminal_status, Some(TerminalConvergenceStatus::NotConverged));
    }

    #[test]
    fn terminal_not_converged_cannot_override_solver_convergence() {
        let result = solve_snapshot(&base_snapshot(), SolveOptions::default()).unwrap();
        let terminal = super::super::convergence::verify_terminal(
            result.final_scaled_residual_norm,
            result.final_step_norm,
            0.0,
            0.0,
            result.iterations,
        );
        let evidence = classify_with_terminal_convergence(&input_from_legacy(&result), &terminal);
        assert_eq!(terminal.status, TerminalConvergenceStatus::NotConverged);
        assert_eq!(evidence.status, SolverStatus::Indeterminate);
    }

    #[test]
    fn explicit_inconsistency_evidence_refines_nonconverged_status() {
        let mut result = solve_snapshot(&base_snapshot(), SolveOptions::default()).unwrap();
        result.converged = false;
        result.reason = SolveReason::MaxIterations;
        let consistency = inconsistent_evidence(&result);
        let evidence = classify_with_linear_consistency(&input_from_legacy(&result), &consistency);
        assert_eq!(evidence.status, SolverStatus::Inconsistent);
    }

    #[test]
    fn inconsistent_evidence_cannot_override_a_converged_result() {
        let result = solve_snapshot(&base_snapshot(), SolveOptions::default()).unwrap();
        assert!(result.converged);
        let consistency = inconsistent_evidence(&result);
        let evidence = classify_with_linear_consistency(&input_from_legacy(&result), &consistency);
        assert_eq!(evidence.status, SolverStatus::Converged);
    }

    #[test]
    fn mismatched_inconsistency_evidence_is_ignored() {
        let mut result = solve_snapshot(&base_snapshot(), SolveOptions::default()).unwrap();
        result.converged = false;
        result.reason = SolveReason::MaxIterations;
        result.final_scaled_residual_norm = result.initial_scaled_residual_norm;
        let mut consistency = inconsistent_evidence(&result);
        consistency.equation_count += 1;
        let evidence = classify_with_linear_consistency(&input_from_legacy(&result), &consistency);
        assert_eq!(evidence.status, SolverStatus::Diverged);
    }

    #[test]
    fn inconsistent_linear_rank_witness_is_explicitly_representable() {
        let a = DMatrix::from_row_slice(2, 2, &[1.0, 1.0, 2.0, 2.0]);
        let b = DVector::from_column_slice(&[2.0, 5.0]);
        let evidence = super::super::linear_consistency::classify_linear_system(
            &a, &b, 1.0e-10, 1.0e10,
        )
        .unwrap();
        assert_eq!(evidence.status, LinearSystemStatus::Inconsistent);
        assert!(evidence.augmented_rank > evidence.coefficient_rank);
    }

    #[test]
    fn explicit_linear_system_api_proves_inconsistency_without_solver_guessing() {
        let mut result = solve_snapshot(&base_snapshot(), SolveOptions::default()).unwrap();
        result.converged = false;
        result.reason = SolveReason::MaxIterations;
        result.final_scaled_residual_norm = result.initial_scaled_residual_norm;
        result.variable_count = 2;
        result.equation_count = 2;
        result.analysis.rank = 1;
        let evidence = classify_with_linear_system(
            &input_from_legacy(&result),
            &[vec![1.0, 1.0], vec![2.0, 2.0]],
            &[2.0, 5.0],
            1.0e-10,
            1.0e10,
        )
        .unwrap();
        assert_eq!(evidence.status, SolverStatus::Inconsistent);
    }

    #[test]
    fn explicit_linear_system_api_fails_closed_on_invalid_jacobian_dimensions() {
        let result = solve_snapshot(&base_snapshot(), SolveOptions::default()).unwrap();
        let error = classify_with_linear_system(
            &input_from_legacy(&result),
            &[vec![1.0], vec![2.0, 2.0]],
            &[2.0, 5.0],
            1.0e-10,
            1.0e10,
        );
        assert_eq!(
            error,
            Err(super::super::linalg::LinAlgError::DimensionMismatch {
                lhs: (2, 1),
                rhs: (0, 0)
            })
        );
    }
}