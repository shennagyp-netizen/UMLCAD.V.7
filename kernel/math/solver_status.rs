//! Conservative semantic classification for solver results.
//!
//! This module does not alter the numerical solver. It converts the existing
//! immutable `ConstraintSolveResult` evidence into an explicit authority status.
//! Classification is deliberately fail-closed: a status is only claimed when
//! the available evidence is sufficient to justify it.

use super::linear_consistency::{LinearConsistencyEvidence, LinearSystemStatus};
use super::solver::{ConstraintSolveResult, SolveReason};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SolverStatus {
    Converged,
    ConvergedWithWarning,
    Diverged,
    Singular,
    IllConditioned,
    Inconsistent,
    MaxIterations,
    Indeterminate,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SolverStatusEvidence {
    pub status: SolverStatus,
    pub residual_reduced: bool,
    pub final_residual_finite: bool,
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
/// - non-finite analysis => `Indeterminate`.
///
/// `Inconsistent`, `InvalidInput`, and `Cancelled` need richer upstream evidence
/// than the current result type carries and are therefore not fabricated here.
pub fn classify(result: &ConstraintSolveResult) -> SolverStatusEvidence {
    let final_residual_finite = result.final_residual_norm.is_finite()
        && result.final_scaled_residual_norm.is_finite()
        && result.analysis.valid;
    let residual_reduced = result.final_scaled_residual_norm < result.initial_scaled_residual_norm;
    let condition_finite = result.analysis.condition_estimate.is_finite();
    let well_conditioned = result.analysis.well_conditioned;

    let status = if !final_residual_finite {
        SolverStatus::Indeterminate
    } else {
        match result.reason {
            SolveReason::Converged if result.converged => {
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
        residual_reduced,
        final_residual_finite,
        condition_finite,
        well_conditioned,
        rank: result.analysis.rank,
        equation_count: result.analysis.equation_count,
        iterations: result.iterations,
    }
}

/// Refine the diagnostic classification when an independent linear-consistency
/// authority has proved the current linearized system inconsistent.
///
/// The extra evidence is accepted only when its matrix dimensions agree with
/// the solver analysis and the solver has not reported convergence. This keeps
/// the API fail-closed: mismatched or insufficient evidence leaves the original
/// status untouched.
pub fn classify_with_linear_consistency(
    result: &ConstraintSolveResult,
    consistency: &LinearConsistencyEvidence,
) -> SolverStatusEvidence {
    let mut evidence = classify(result);
    let dimensions_match = consistency.variable_count == result.analysis.variable_count
        && consistency.equation_count == result.analysis.equation_count;
    if dimensions_match
        && !result.converged
        && evidence.final_residual_finite
        && consistency.status == LinearSystemStatus::Inconsistent
    {
        evidence.status = SolverStatus::Inconsistent;
    }
    evidence
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::{
        geometry::{Geometry, Line, Point},
        snapshot::{Constraint, GeometryItem, SemanticSnapshot},
        solver::{solve_snapshot, SolveOptions},
    };
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
            variable_count: result.analysis.variable_count,
            equation_count: result.analysis.equation_count,
            coefficient_condition_number: 1.0,
            coefficient_classification: super::super::linalg::RankClassification::FullRank,
        }
    }

    #[test]
    fn converged_well_conditioned_result_is_classified_as_converged() {
        let result = solve_snapshot(&base_snapshot(), SolveOptions::default()).unwrap();
        assert!(result.converged);
        let evidence = classify(&result);
        assert_eq!(evidence.status, SolverStatus::Converged);
        assert!(evidence.final_residual_finite);
    }

    #[test]
    fn finite_residual_with_singular_reason_is_not_reported_as_converged() {
        let mut result = solve_snapshot(&base_snapshot(), SolveOptions::default()).unwrap();
        result.converged = false;
        result.reason = SolveReason::Singular;
        let evidence = classify(&result);
        assert_eq!(evidence.status, SolverStatus::Singular);
    }

    #[test]
    fn max_iterations_without_progress_is_classified_as_diverged() {
        let mut result = solve_snapshot(&base_snapshot(), SolveOptions::default()).unwrap();
        result.converged = false;
        result.reason = SolveReason::MaxIterations;
        result.final_scaled_residual_norm = result.initial_scaled_residual_norm;
        let evidence = classify(&result);
        assert_eq!(evidence.status, SolverStatus::Diverged);
    }

    #[test]
    fn nonfinite_analysis_is_indeterminate() {
        let mut result = solve_snapshot(&base_snapshot(), SolveOptions::default()).unwrap();
        result.analysis.valid = false;
        result.final_scaled_residual_norm = f64::NAN;
        let evidence = classify(&result);
        assert_eq!(evidence.status, SolverStatus::Indeterminate);
        assert!(!evidence.final_residual_finite);
    }

    #[test]
    fn explicit_inconsistency_evidence_refines_nonconverged_status() {
        let mut result = solve_snapshot(&base_snapshot(), SolveOptions::default()).unwrap();
        result.converged = false;
        result.reason = SolveReason::MaxIterations;
        let consistency = inconsistent_evidence(&result);
        let evidence = classify_with_linear_consistency(&result, &consistency);
        assert_eq!(evidence.status, SolverStatus::Inconsistent);
    }

    #[test]
    fn inconsistent_evidence_cannot_override_a_converged_result() {
        let result = solve_snapshot(&base_snapshot(), SolveOptions::default()).unwrap();
        assert!(result.converged);
        let consistency = inconsistent_evidence(&result);
        let evidence = classify_with_linear_consistency(&result, &consistency);
        assert_eq!(evidence.status, SolverStatus::Converged);
    }

    #[test]
    fn mismatched_inconsistency_evidence_is_ignored() {
        let mut result = solve_snapshot(&base_snapshot(), SolveOptions::default()).unwrap();
        result.converged = false;
        result.reason = SolveReason::MaxIterations;
        let mut consistency = inconsistent_evidence(&result);
        consistency.equation_count += 1;
        let evidence = classify_with_linear_consistency(&result, &consistency);
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
}
