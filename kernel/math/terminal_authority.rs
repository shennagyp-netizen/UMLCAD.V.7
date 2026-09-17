//! Authoritative production wrapper for terminal solver certification.
//!
//! The existing numerical solver remains the implementation engine. This module
//! makes its terminal certificate part of the semantic solve result and enforces
//! consistency between that certificate and the reported convergence fields.
//! No iteration history is manufactured and no numerical backend is changed.

use super::{
    convergence::{verify_terminal, TerminalConvergenceEvidence, TerminalConvergenceStatus},
    solver_legacy,
    geometry::Geometry,
    snapshot::SemanticSnapshot,
    solver_status::{classify_with_terminal_convergence, SolverStatus, SolverStatusEvidence},
};

pub use solver_legacy::{
    scaled_damped_qr, ConstraintAnalysis, ConstraintResidualReport, LinearSolveReport,
    SolveOptions, SolveReason,
};
pub use super::solver_status::{SolverStatus, SolverStatusEvidence};

#[derive(Clone, Debug, PartialEq)]
pub struct ConstraintSolveResult {
    pub converged: bool,
    pub reason: SolveReason,
    pub iterations: usize,
    pub initial_residual_norm: f64,
    pub final_residual_norm: f64,
    pub initial_scaled_residual_norm: f64,
    pub final_scaled_residual_norm: f64,
    pub final_step_norm: f64,
    /// Authoritative terminal certificate for the returned immutable result.
    ///
    /// Its residual, step, and iteration fields are required to match the result
    /// exactly. It is diagnostic evidence made part of the production result,
    /// not a second iteration-control mechanism.
    pub terminal_convergence: TerminalConvergenceEvidence,
    /// Authoritative semantic classification of the returned solve.
    pub status: SolverStatus,
    /// Evidence backing the status, including terminal/rank/conditioning facts.
    pub status_evidence: SolverStatusEvidence,
    pub analysis: ConstraintAnalysis,
    pub geometry: Vec<(String, Geometry)>,
}

fn certify(
    result: solver_legacy::ConstraintSolveResult,
    options: &SolveOptions,
) -> Result<ConstraintSolveResult, String> {
    let terminal = verify_terminal(
        result.final_scaled_residual_norm,
        result.final_step_norm,
        options.residual_tolerance,
        options.step_tolerance,
        result.iterations,
    );

    if terminal.status == TerminalConvergenceStatus::Indeterminate {
        return Err("solver produced indeterminate terminal convergence evidence".into());
    }

    let converged_by_terminal = terminal.status == TerminalConvergenceStatus::Converged;
    if result.converged != converged_by_terminal {
        return Err("solver convergence flag disagrees with terminal authority".into());
    }
    if result.converged && result.reason != SolveReason::Converged {
        return Err("converged solver result has non-converged reason".into());
    }
    if !result.converged && result.reason == SolveReason::Converged {
        return Err("non-converged solver result has converged reason".into());
    }

    let status_evidence = classify_with_terminal_convergence(&result, &terminal);
    if status_evidence.status == SolverStatus::Indeterminate {
        return Err("solver status authority rejected terminal result evidence".into());
    }
    let status = status_evidence.status;

    Ok(ConstraintSolveResult {
        converged: result.converged,
        reason: result.reason,
        iterations: result.iterations,
        initial_residual_norm: result.initial_residual_norm,
        final_residual_norm: result.final_residual_norm,
        initial_scaled_residual_norm: result.initial_scaled_residual_norm,
        final_scaled_residual_norm: result.final_scaled_residual_norm,
        final_step_norm: result.final_step_norm,
        terminal_convergence: terminal,
        status,
        status_evidence,
        analysis: result.analysis,
        geometry: result.geometry,
    })
}

pub fn solve_snapshot(
    snapshot: &SemanticSnapshot,
    options: SolveOptions,
) -> Result<ConstraintSolveResult, String> {
    let result = solver_legacy::solve_snapshot(snapshot, options.clone())?;
    certify(result, &options)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::{
        geometry::{Geometry, Line, Point},
        snapshot::{Constraint, GeometryItem, SemanticSnapshot},
    };

    fn snapshot() -> SemanticSnapshot {
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
            constraints: vec![(
                "horizontal".into(),
                Constraint::Horizontal { entity_id: "l".into() },
            )],
            relations: vec![],
        }
        .deterministic()
    }

    #[test]
    fn returned_result_contains_lossless_terminal_certificate() {
        let result = solve_snapshot(&snapshot(), SolveOptions::default()).unwrap();
        let terminal = result.terminal_convergence;
        assert_eq!(terminal.iterations, result.iterations);
        assert_eq!(
            terminal.final_residual.to_bits(),
            result.final_scaled_residual_norm.to_bits()
        );
        assert_eq!(
            terminal.final_step_norm.to_bits(),
            result.final_step_norm.to_bits()
        );
        assert_eq!(terminal.status, TerminalConvergenceStatus::Converged);
        assert!(result.converged);
        assert_eq!(result.reason, SolveReason::Converged);
        assert_eq!(result.status, SolverStatus::Converged);
        assert_eq!(
            result.status_evidence.terminal_status,
            Some(TerminalConvergenceStatus::Converged)
        );
    }

    #[test]
    fn zero_iteration_result_is_certified_without_history_invention() {
        let mut satisfied = snapshot();
        if let Geometry::Line(line) = &mut satisfied.geometry[0].geometry {
            line.end.y = line.start.y;
        }
        let result = solve_snapshot(&satisfied, SolveOptions::default()).unwrap();
        assert_eq!(result.iterations, 0);
        assert_eq!(result.terminal_convergence.iterations, 0);
        assert_eq!(
            result.terminal_convergence.status,
            TerminalConvergenceStatus::Converged
        );
        assert_eq!(result.final_step_norm, 0.0);
    }

    #[test]
    fn max_iteration_result_carries_nonconverged_terminal_certificate() {
        let mut options = SolveOptions::default();
        options.max_iterations = 1;
        options.step_tolerance = 1.0e-20;
        let result = solve_snapshot(&snapshot(), options).unwrap();
        assert_eq!(result.reason, SolveReason::MaxIterations);
        assert!(!result.converged);
        assert_eq!(result.iterations, 1);
        assert_eq!(result.terminal_convergence.iterations, 1);
        assert_ne!(
            result.terminal_convergence.status,
            TerminalConvergenceStatus::Converged
        );
        assert_eq!(result.status, SolverStatus::MaxIterations);
        assert_eq!(result.status_evidence.iterations, result.iterations);
    }
}
