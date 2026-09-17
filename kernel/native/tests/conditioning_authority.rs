use nalgebra::DMatrix;
use umlcad_kernel_rust::math::linalg::{
    rank_evidence, svd, RankClassification, CANONICAL_ILL_COND_THRESHOLD, CANONICAL_RANK_TOL,
};

fn diagonal(a: f64, b: f64) -> DMatrix<f64> {
    DMatrix::from_row_slice(2, 2, &[a, 0.0, 0.0, b])
}

#[test]
fn condition_below_explicit_threshold_is_full_rank() {
    let evidence = rank_evidence(&diagonal(1.0, 2.0e-4), 1.0e-12, 1.0e4).unwrap();
    assert_eq!(evidence.rank, 2);
    assert_eq!(evidence.classification, RankClassification::FullRank);
    assert!((evidence.condition_number - 5.0e3).abs() < 1.0e-9);
}

#[test]
fn condition_above_explicit_threshold_is_ill_conditioned_while_rank_remains_full() {
    let evidence = rank_evidence(&diagonal(1.0, 1.0e-5), 1.0e-12, 1.0e4).unwrap();
    assert_eq!(evidence.rank, 2);
    assert_eq!(evidence.classification, RankClassification::IllConditioned);
    assert!((evidence.condition_number - 1.0e5).abs() < 1.0e-8);
}

#[test]
fn condition_exactly_on_explicit_threshold_is_not_ill_conditioned() {
    let evidence = rank_evidence(&diagonal(1.0, 1.0e-4), 1.0e-12, 1.0e4).unwrap();
    assert_eq!(evidence.rank, 2);
    assert_eq!(evidence.classification, RankClassification::FullRank);
    assert!((evidence.condition_number - 1.0e4).abs() < 1.0e-9);
}

#[test]
fn canonical_rank_and_condition_contracts_are_finite_and_nonnegative() {
    assert!(CANONICAL_RANK_TOL.is_finite());
    assert!(CANONICAL_RANK_TOL >= 0.0);
    assert!(CANONICAL_ILL_COND_THRESHOLD.is_finite());
    assert!(CANONICAL_ILL_COND_THRESHOLD >= 0.0);
}

#[test]
fn svd_and_rank_evidence_expose_identical_condition_diagnostics() {
    let matrix = diagonal(3.0, 0.25);
    let decomposition = svd(&matrix, 1.0e-12, 1.0e4).unwrap();
    let evidence = rank_evidence(&matrix, 1.0e-12, 1.0e4).unwrap();
    assert_eq!(decomposition.evidence.rank, evidence.rank);
    assert_eq!(decomposition.evidence.classification, evidence.classification);
    assert!((decomposition.evidence.condition_number - evidence.condition_number).abs() < 1.0e-12);
}
