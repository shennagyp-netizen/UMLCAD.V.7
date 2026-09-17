//! Regression coverage for authoritative terminal convergence evidence.
//!
//! The production solver owns the final state, while `verify_terminal` provides
//! the mathematical certificate carried with that state. Tests here require the
//! certificate to remain lossless: the evidence must describe exactly the result's
//! terminal residual, terminal step, and iteration count.

use super::{convergence::TerminalConvergenceStatus, geometry::{Geometry, Line, Point}, snapshot::{Constraint, GeometryItem, SemanticSnapshot}, solver::{solve_snapshot, SolveOptions}};

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
fn production_result_carries_an_exact_terminal_certificate() {
    let result = solve_snapshot(&snapshot(), SolveOptions::default()).unwrap();
    let certificate = result.terminal_convergence;
    assert_eq!(certificate.iterations, result.iterations);
    assert_eq!(
        certificate.final_residual.to_bits(),
        result.final_scaled_residual_norm.to_bits()
    );
    assert_eq!(
        certificate.final_step_norm.to_bits(),
        result.final_step_norm.to_bits()
    );
    assert!(certificate.residual_satisfied);
    assert!(certificate.step_satisfied);
    assert_eq!(certificate.status, TerminalConvergenceStatus::Converged);
    assert!(result.converged);
}

#[test]
fn terminal_certificate_is_zero_iteration_authority_for_already_satisfied_systems() {
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
fn terminal_certificate_preserves_nonconverged_max_iteration_state() {
    let mut options = SolveOptions::default();
    options.max_iterations = 1;
    options.step_tolerance = 1.0e-20;
    let result = solve_snapshot(&snapshot(), options).unwrap();
    assert_eq!(result.reason, super::solver::SolveReason::MaxIterations);
    assert_eq!(result.iterations, 1);
    assert_eq!(result.terminal_convergence.iterations, 1);
    assert_eq!(
        result.terminal_convergence.final_residual.to_bits(),
        result.final_scaled_residual_norm.to_bits()
    );
    assert_eq!(
        result.terminal_convergence.final_step_norm.to_bits(),
        result.final_step_norm.to_bits()
    );
    assert_ne!(
        result.terminal_convergence.status,
        TerminalConvergenceStatus::Converged
    );
}
