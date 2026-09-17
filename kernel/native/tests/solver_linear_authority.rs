use umlcad_kernel_rust::math::{
    linear_consistency::{classify_linear_system, LinearSystemStatus},
    solver::scaled_damped_qr,
};

fn residual_norm(values: &[f64]) -> f64 {
    values.iter().map(|v| v * v).sum::<f64>().sqrt()
}

fn mat_vec(matrix: &[Vec<f64>], x: &[f64]) -> Vec<f64> {
    matrix
        .iter()
        .map(|row| row.iter().zip(x.iter()).map(|(a, b)| a * b).sum())
        .collect()
}

#[test]
fn identity_system_produces_the_exact_undamped_step() {
    let j = vec![vec![1.0, 0.0], vec![0.0, 1.0]];
    let r = vec![3.0, -2.0];
    let report = scaled_damped_qr(&j, &r, 0.0, 1.0e-10).unwrap();
    assert_eq!(report.delta, vec![-3.0, 2.0]);
    assert_eq!(report.rank, 2);
    assert_eq!(report.degrees_of_freedom, 0);
    assert!((report.condition_number - 1.0).abs() < 1.0e-12);
}

#[test]
fn exact_solver_step_satisfies_the_linearized_equation() {
    let j = vec![vec![2.0, 1.0], vec![1.0, 3.0]];
    let r = vec![4.0, -5.0];
    let report = scaled_damped_qr(&j, &r, 0.0, 1.0e-12).unwrap();
    let j_delta = mat_vec(&j, &report.delta);
    let residual = j_delta
        .iter()
        .zip(r.iter())
        .map(|(lhs, rhs)| lhs + rhs)
        .collect::<Vec<_>>();
    assert!(residual_norm(&residual) < 1.0e-12);
}

#[test]
fn rank_and_degrees_of_freedom_match_independent_consistency_authority() {
    let j = vec![vec![1.0, 1.0], vec![2.0, 2.0]];
    let r = vec![2.0, 4.0];
    let report = scaled_damped_qr(&j, &r, 0.0, 1.0e-10).unwrap();
    let evidence = classify_linear_system(
        &nalgebra::DMatrix::from_row_slice(2, 2, &[1.0, 1.0, 2.0, 2.0]),
        &nalgebra::DVector::from_column_slice(&[-2.0, -4.0]),
        1.0e-10,
        1.0e10,
    )
    .unwrap();
    assert_eq!(evidence.status, LinearSystemStatus::UnderdeterminedConsistent);
    assert_eq!(report.rank, evidence.coefficient_rank);
    assert_eq!(report.degrees_of_freedom, 2 - evidence.coefficient_rank);
}

#[test]
fn an_inconsistent_linearization_is_proved_by_augmented_rank() {
    let j = nalgebra::DMatrix::from_row_slice(2, 2, &[1.0, 1.0, 2.0, 2.0]);
    let r = nalgebra::DVector::from_column_slice(&[-2.0, -5.0]);
    let evidence = classify_linear_system(&j, &r, 1.0e-10, 1.0e10).unwrap();
    assert_eq!(evidence.status, LinearSystemStatus::Inconsistent);
    assert_eq!(evidence.coefficient_rank, 1);
    assert_eq!(evidence.augmented_rank, 2);
}

#[test]
fn column_unit_changes_do_not_change_the_undamped_physical_solution() {
    let base = vec![vec![2.0, 1.0], vec![1.0, 3.0]];
    let scaled = vec![vec![2.0e-9, 1.0e9], vec![1.0e-9, 3.0e9]];
    let r = vec![4.0, -5.0];
    let base_report = scaled_damped_qr(&base, &r, 0.0, 1.0e-12).unwrap();
    let scaled_report = scaled_damped_qr(&scaled, &r, 0.0, 1.0e-12).unwrap();
    for (lhs, rhs) in base_report.delta.iter().zip(scaled_report.delta.iter()) {
        assert!((lhs - rhs).abs() < 1.0e-9);
    }
    assert_eq!(base_report.rank, scaled_report.rank);
}

#[test]
fn ragged_jacobian_and_nonfinite_inputs_fail_closed() {
    assert!(scaled_damped_qr(&[vec![1.0], vec![2.0, 3.0]], &[1.0, 2.0], 0.0, 1.0e-10).is_err());
    assert!(scaled_damped_qr(&[vec![f64::NAN]], &[1.0], 0.0, 1.0e-10).is_err());
    assert!(scaled_damped_qr(&[vec![1.0]], &[f64::INFINITY], 0.0, 1.0e-10).is_err());
}

#[test]
fn invalid_damping_and_rank_tolerance_fail_closed() {
    let j = vec![vec![1.0]];
    let r = vec![1.0];
    assert!(scaled_damped_qr(&j, &r, f64::NAN, 1.0e-10).is_err());
    assert!(scaled_damped_qr(&j, &r, -1.0, 1.0e-10).is_err());
    assert!(scaled_damped_qr(&j, &r, 0.0, f64::NAN).is_err());
    assert!(scaled_damped_qr(&j, &r, 0.0, -1.0).is_err());
}

#[test]
fn zero_column_reduces_rank_and_dof_without_nonfinite_step() {
    let j = vec![vec![1.0, 0.0], vec![0.0, 0.0]];
    let r = vec![2.0, 0.0];
    let report = scaled_damped_qr(&j, &r, 0.0, 1.0e-10).unwrap();
    assert_eq!(report.rank, 1);
    assert_eq!(report.degrees_of_freedom, 1);
    assert!(report.delta.iter().all(|v| v.is_finite()));
}
