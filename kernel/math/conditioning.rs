//! Conventional condition-number interpretation for the mathematical authority.
//!
//! `RankEvidence::condition_number` intentionally reports the spread of the
//! nonzero singular spectrum. This module adds the conventional interpretation
//! needed by solver diagnostics: a rank-deficient operator has infinite
//! condition number even when its nonzero singular values have finite spread.

use nalgebra::DMatrix;

use super::linalg::{rank_evidence, LinAlgError, RankClassification, RankEvidence};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConditioningStatus {
    WellConditioned,
    IllConditioned,
    Singular,
    Indeterminate,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ConditioningEvidence {
    pub status: ConditioningStatus,
    /// Conventional condition number of the represented linear operator.
    /// This is +∞ whenever the numerical rank is below min(m, n).
    pub condition_number: f64,
    /// Spread of the retained nonzero singular spectrum. For a full-rank
    /// matrix this equals `condition_number`; for rank-deficient matrices it
    /// remains finite when the retained spectrum is finite.
    pub nonzero_spectral_spread: f64,
    pub rank: usize,
    pub min_dim: usize,
    pub threshold: f64,
}

fn from_rank_evidence(
    evidence: RankEvidence,
    ill_cond_threshold: f64,
) -> Result<ConditioningEvidence, LinAlgError> {
    if !ill_cond_threshold.is_finite() || ill_cond_threshold < 0.0 {
        return Err(LinAlgError::InvalidTolerance);
    }
    let threshold = ill_cond_threshold;
    let full_rank = evidence.rank == evidence.min_dim;
    let condition_number = if full_rank {
        evidence.condition_number
    } else {
        f64::INFINITY
    };
    let status = match evidence.classification {
        RankClassification::Indeterminate => ConditioningStatus::Indeterminate,
        RankClassification::Singular if evidence.rank == 0 => ConditioningStatus::Singular,
        RankClassification::RankDeficient => ConditioningStatus::Singular,
        RankClassification::IllConditioned => ConditioningStatus::IllConditioned,
        RankClassification::FullRank => {
            if !condition_number.is_finite() {
                ConditioningStatus::Indeterminate
            } else if condition_number > threshold {
                ConditioningStatus::IllConditioned
            } else {
                ConditioningStatus::WellConditioned
            }
        }
        RankClassification::Singular => ConditioningStatus::Singular,
    };

    Ok(ConditioningEvidence {
        status,
        condition_number,
        nonzero_spectral_spread: evidence.condition_number,
        rank: evidence.rank,
        min_dim: evidence.min_dim,
        threshold,
    })
}

pub fn from_rank_evidence_with_threshold(
    evidence: RankEvidence,
    ill_cond_threshold: f64,
) -> Result<ConditioningEvidence, LinAlgError> {
    from_rank_evidence(evidence, ill_cond_threshold)
}

pub fn analyze(
    matrix: &DMatrix<f64>,
    rank_tol: f64,
    ill_cond_threshold: f64,
) -> Result<ConditioningEvidence, LinAlgError> {
    let evidence = rank_evidence(matrix, rank_tol, ill_cond_threshold)?;
    from_rank_evidence(evidence, ill_cond_threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn diagonal(a: f64, b: f64) -> DMatrix<f64> {
        DMatrix::from_row_slice(2, 2, &[a, 0.0, 0.0, b])
    }

    #[test]
    fn full_rank_matrix_uses_the_finite_spectral_condition_number() {
        let evidence = analyze(&diagonal(1.0, 2.0e-4), 1.0e-12, 1.0e4).unwrap();
        assert_eq!(evidence.status, ConditioningStatus::WellConditioned);
        assert_eq!(evidence.rank, 2);
        assert!((evidence.condition_number - 5.0e3).abs() < 1.0e-9);
        assert!((evidence.nonzero_spectral_spread - 5.0e3).abs() < 1.0e-9);
    }

    #[test]
    fn full_rank_matrix_above_threshold_is_ill_conditioned() {
        let evidence = analyze(&diagonal(1.0, 1.0e-5), 1.0e-12, 1.0e4).unwrap();
        assert_eq!(evidence.status, ConditioningStatus::IllConditioned);
        assert_eq!(evidence.rank, 2);
        assert!((evidence.condition_number - 1.0e5).abs() < 1.0e-8);
    }

    #[test]
    fn rank_deficient_matrix_has_infinite_conventional_condition_number() {
        let evidence = analyze(&diagonal(1.0, 0.0), 1.0e-12, 1.0e4).unwrap();
        assert_eq!(evidence.status, ConditioningStatus::Singular);
        assert_eq!(evidence.rank, 1);
        assert_eq!(evidence.condition_number, f64::INFINITY);
        assert!(evidence.nonzero_spectral_spread.is_finite());
    }

    #[test]
    fn zero_matrix_is_singular() {
        let evidence = analyze(&diagonal(0.0, 0.0), 1.0e-12, 1.0e4).unwrap();
        assert_eq!(evidence.status, ConditioningStatus::Singular);
        assert_eq!(evidence.rank, 0);
        assert_eq!(evidence.condition_number, f64::INFINITY);
    }

    #[test]
    fn exact_threshold_is_not_classified_as_ill_conditioned() {
        let evidence = analyze(&diagonal(1.0, 1.0e-4), 1.0e-12, 1.0e4).unwrap();
        assert_eq!(evidence.status, ConditioningStatus::WellConditioned);
    }

    #[test]
    fn invalid_condition_threshold_fails_closed() {
        let matrix = diagonal(1.0, 1.0);
        for threshold in [f64::NAN, f64::INFINITY, -1.0] {
            assert_eq!(
                analyze(&matrix, 1.0e-12, threshold),
                Err(LinAlgError::InvalidTolerance)
            );
        }
    }

    #[test]
    fn explicit_rank_evidence_and_matrix_analysis_are_identical() {
        let matrix = diagonal(4.0, 0.5);
        let rank = rank_evidence(&matrix, 1.0e-12, 1.0e4).unwrap();
        let from_rank = from_rank_evidence_with_threshold(rank, 1.0e4).unwrap();
        let analyzed = analyze(&matrix, 1.0e-12, 1.0e4).unwrap();
        assert_eq!(from_rank, analyzed);
    }
}
