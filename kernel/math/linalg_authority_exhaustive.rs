//! Independent regressions for the wide-matrix SVD/null-space authority.
//!
//! These tests verify dimension, orthonormality, null residuals, and rank-threshold
//! behavior without introducing another numerical implementation. The existing
//! nalgebra-backed CPU path remains the reference implementation.

use super::linalg::{null_space, pseudo_inverse, rank_evidence, RankClassification, CANONICAL_ILL_COND_THRESHOLD, CANONICAL_RANK_TOL};
use nalgebra::DMatrix;

fn matrix(rows: &[&[f64]]) -> DMatrix<f64> {
    DMatrix::from_row_iterator(
        rows.len(),
        rows.first().map_or(0, |row| row.len()),
        rows.iter().flat_map(|row| row.iter().copied()),
    )
}

fn assert_close(a: f64, b: f64, tolerance: f64, message: &str) {
    assert!((a - b).abs() <= tolerance, "{message}: {a} vs {b}");
}

#[test]
fn wide_full_row_rank_null_space_is_orthonormal_and_exactly_annihilated() {
    let a = matrix(&[
        &[1.0, 2.0, 0.0, 0.0, 1.0],
        &[0.0, 1.0, 3.0, 1.0, 0.0],
    ]);
    let ns = null_space(&a, CANONICAL_RANK_TOL, CANONICAL_ILL_COND_THRESHOLD).unwrap();

    assert_eq!(ns.nrows(), 5);
    assert_eq!(ns.ncols(), 3);

    let gram = ns.transpose() * &ns;
    for row in 0..gram.nrows() {
        for column in 0..gram.ncols() {
            let expected = if row == column { 1.0 } else { 0.0 };
            assert_close(
                gram[(row, column)],
                expected,
                5.0e-14,
                &format!("Gram[{row},{column}]"),
            );
        }
    }

    let residual = &a * &ns;
    assert!(residual.iter().all(|value| value.abs() <= 5.0e-13));
}

#[test]
fn rank_deficient_wide_system_reports_full_nullity() {
    let a = matrix(&[
        &[1.0, 2.0, 3.0, 4.0],
        &[2.0, 4.0, 6.0, 8.0],
        &[0.0, 0.0, 0.0, 0.0],
    ]);
    let evidence = rank_evidence(&a, CANONICAL_RANK_TOL, CANONICAL_ILL_COND_THRESHOLD).unwrap();
    assert_eq!(evidence.rank, 1);
    assert_eq!(evidence.nullity, 3);
    assert_eq!(evidence.min_dim, 3);
    assert_eq!(evidence.classification, RankClassification::RankDeficient);

    let ns = null_space(&a, CANONICAL_RANK_TOL, CANONICAL_ILL_COND_THRESHOLD).unwrap();
    assert_eq!(ns.shape(), (4, 3));
    assert!((&a * &ns).iter().all(|value| value.abs() <= 1.0e-10));
}

#[test]
fn relative_rank_threshold_has_a_defined_transition() {
    let near = matrix(&[
        &[1.0, 0.0],
        &[0.0, 5.0e-11],
        &[0.0, 0.0],
    ]);
    let below = rank_evidence(&near, 1.0e-10, CANONICAL_ILL_COND_THRESHOLD).unwrap();
    assert_eq!(below.rank, 1);
    assert_eq!(below.classification, RankClassification::RankDeficient);

    let above = rank_evidence(&near, 1.0e-11, CANONICAL_ILL_COND_THRESHOLD).unwrap();
    assert_eq!(above.rank, 2);
    assert_eq!(above.classification, RankClassification::FullRank);
}

#[test]
fn null_space_is_invariant_in_structure_under_uniform_matrix_scaling() {
    let a = matrix(&[
        &[3.0, -1.0, 2.0, 0.5],
        &[1.0, 4.0, -2.0, 3.0],
    ]);
    let base = null_space(&a, CANONICAL_RANK_TOL, CANONICAL_ILL_COND_THRESHOLD).unwrap();

    for factor in [1.0e-12, 1.0e12] {
        let scaled = &a * factor;
        let scaled_ns = null_space(&scaled, CANONICAL_RANK_TOL, CANONICAL_ILL_COND_THRESHOLD).unwrap();
        assert_eq!(scaled_ns.shape(), base.shape());
        assert!((&scaled * &scaled_ns).iter().all(|value| value.abs() <= 1.0e-10));
        let gram = scaled_ns.transpose() * &scaled_ns;
        for diagonal in 0..gram.nrows() {
            assert_close(gram[(diagonal, diagonal)], 1.0, 1.0e-12, "scaled null-space norm");
        }
    }
}

#[test]
fn pseudo_inverse_of_wide_rank_deficient_system_satisfies_reflexive_identity() {
    let a = matrix(&[
        &[1.0, 2.0, 3.0, 4.0],
        &[2.0, 4.0, 6.0, 8.0],
    ]);
    let ap = pseudo_inverse(&a, CANONICAL_RANK_TOL, CANONICAL_ILL_COND_THRESHOLD).unwrap();
    assert_eq!(ap.shape(), (4, 2));
    let reflexive = &a * &ap * &a;
    for row in 0..a.nrows() {
        for column in 0..a.ncols() {
            assert_close(reflexive[(row, column)], a[(row, column)], 1.0e-10, "A A+ A");
        }
    }
}

#[test]
fn invalid_rank_configuration_fails_closed() {
    let a = matrix(&[
        &[1.0, 0.0, 0.0],
        &[0.0, 1.0, 0.0],
    ]);
    assert!(null_space(&a, f64::NAN, CANONICAL_ILL_COND_THRESHOLD).is_err());
    assert!(rank_evidence(&a, CANONICAL_RANK_TOL, f64::NAN).is_err());
}
