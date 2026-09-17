//! Conservative semantic classification for solver results.
//!
//! This module does not alter the numerical solver. It converts the existing
//! immutable `ConstraintSolveResult` evidence into an explicit authority status.
//! Classification is deliberately fail-closed: a status is only claimed when
//! the available evidence is sufficient to justify it.

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::{
        geometry::{Geometry, Line, Point},
        snapshot::{Constraint, GeometryItem, SemanticSnapshot},
        solver::{solve_snapshot, SolveOptions},
    };

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
}
