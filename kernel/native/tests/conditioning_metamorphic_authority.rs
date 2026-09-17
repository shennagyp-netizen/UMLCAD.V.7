use nalgebra::DMatrix;
use umlcad_kernel_rust::math::conditioning::{analyze, ConditioningStatus};

#[test]
fn uniform_positive_and_negative_scaling_preserves_conditioning() {
    let base = DMatrix::from_row_slice(2, 2, &[4.0, 1.0, 0.0, 2.0]);
    let positive = &base * 37.0;
    let negative = &base * -11.0;

    let a = analyze(&base, 1.0e-12, 1.0e4).unwrap();
    let b = analyze(&positive, 1.0e-12, 1.0e4).unwrap();
    let c = analyze(&negative, 1.0e-12, 1.0e4).unwrap();

    assert_eq!(a.status, b.status);
    assert_eq!(a.status, c.status);
    assert_eq!(a.rank, b.rank);
    assert_eq!(a.rank, c.rank);
    assert!((a.condition_number - b.condition_number).abs() < 1.0e-12);
    assert!((a.condition_number - c.condition_number).abs() < 1.0e-12);
}

#[test]
fn orthogonal_row_rotation_preserves_singular_spectrum_and_conditioning() {
    let base = DMatrix::from_row_slice(2, 2, &[4.0, 1.0, 0.0, 2.0]);
    let theta = 0.731_f64;
    let rotation = DMatrix::from_row_slice(
        2,
        2,
        &[theta.cos(), -theta.sin(), theta.sin(), theta.cos()],
    );
    let rotated = &rotation * &base;

    let a = analyze(&base, 1.0e-12, 1.0e4).unwrap();
    let b = analyze(&rotated, 1.0e-12, 1.0e4).unwrap();

    assert_eq!(a.status, b.status);
    assert_eq!(a.rank, b.rank);
    assert!((a.condition_number - b.condition_number).abs() < 1.0e-12);
    assert!((a.nonzero_spectral_spread - b.nonzero_spectral_spread).abs() < 1.0e-12);
}

#[test]
fn row_and_column_permutations_preserve_rank_and_conditioning() {
    let base = DMatrix::from_row_slice(2, 2, &[5.0, 1.0, 2.0, 3.0]);
    let row_swap = DMatrix::from_row_slice(2, 2, &[0.0, 1.0, 1.0, 0.0]);
    let col_swap = row_swap.clone();
    let permuted = &row_swap * &base * &col_swap;

    let a = analyze(&base, 1.0e-12, 1.0e4).unwrap();
    let b = analyze(&permuted, 1.0e-12, 1.0e4).unwrap();

    assert_eq!(a.status, b.status);
    assert_eq!(a.rank, b.rank);
    assert!((a.condition_number - b.condition_number).abs() < 1.0e-12);
}

#[test]
fn rank_transition_is_determined_by_relative_singular_value_threshold() {
    let tol = 1.0e-8;
    let above = DMatrix::from_row_slice(2, 2, &[1.0, 0.0, 0.0, 2.0 * tol]);
    let below = DMatrix::from_row_slice(2, 2, &[1.0, 0.0, 0.0, 0.5 * tol]);

    let a = analyze(&above, tol, 1.0e4).unwrap();
    let b = analyze(&below, tol, 1.0e4).unwrap();

    assert_eq!(a.rank, 2);
    assert_eq!(a.status, ConditioningStatus::IllConditioned);
    assert_eq!(b.rank, 1);
    assert_eq!(b.status, ConditioningStatus::Singular);
    assert_eq!(b.condition_number, f64::INFINITY);
}

#[test]
fn wide_full_row_rank_system_has_finite_conventional_condition_number() {
    let matrix = DMatrix::from_row_slice(2, 3, &[1.0, 0.0, 0.0, 0.0, 2.0, 0.0]);
    let evidence = analyze(&matrix, 1.0e-12, 1.0e4).unwrap();

    assert_eq!(evidence.rank, 2);
    assert_eq!(evidence.status, ConditioningStatus::WellConditioned);
    assert!(evidence.condition_number.is_finite());
    assert!((evidence.condition_number - 2.0).abs() < 1.0e-12);
}
