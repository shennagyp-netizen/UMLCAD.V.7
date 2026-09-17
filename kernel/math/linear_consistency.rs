//! Tolerance-explicit consistency classification for linear systems.
//!
//! For a system `A x = b`, consistency is established from the numerical ranks
//! of `A` and the augmented matrix `[A | b]` under the caller-provided rank
//! tolerance. No heuristic residual threshold is used to claim inconsistency.

use nalgebra::{DMatrix, DVector};

use super::linalg::{rank_evidence, LinAlgError, RankClassification};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LinearSystemStatus {
    Unique,
    UnderdeterminedConsistent,
    Inconsistent,
    Indeterminate,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LinearConsistencyEvidence {
    pub status: LinearSystemStatus,
    pub coefficient_rank: usize,
    pub augmented_rank: usize,
    pub variable_count: usize,
    pub equation_count: usize,
    pub coefficient_condition_number: f64,
    pub coefficient_classification: RankClassification,
}

/// Classify `A x = b` using rank(A) and rank([A|b]).
///
/// The rank tolerance is explicit and identical for both matrices. Therefore
/// `Inconsistent` means the augmented system has strictly larger numerical rank
/// than the coefficient matrix at that declared tolerance.
pub fn classify_linear_system(
    a: &DMatrix<f64>,
    b: &DVector<f64>,
    rank_tol: f64,
    ill_cond_threshold: f64,
) -> Result<LinearConsistencyEvidence, LinAlgError> {
    if a.nrows() == 0 || a.ncols() == 0 {
        return Err(LinAlgError::EmptyMatrix);
    }
    if b.len() != a.nrows() {
        return Err(LinAlgError::DimensionMismatch {
            lhs: (a.nrows(), a.ncols()),
            rhs: (b.len(), 1),
        });
    }
    if a.iter().any(|value| !value.is_finite()) || b.iter().any(|value| !value.is_finite()) {
        return Err(LinAlgError::NonFinite);
    }

    let coefficient = rank_evidence(a, rank_tol, ill_cond_threshold)?;
    let mut augmented = DMatrix::<f64>::zeros(a.nrows(), a.ncols() + 1);
    for row in 0..a.nrows() {
        for column in 0..a.ncols() {
            augmented[(row, column)] = a[(row, column)];
        }
        augmented[(row, a.ncols())] = b[row];
    }
    let augmented_rank = rank_evidence(&augmented, rank_tol, ill_cond_threshold)?.rank;

    let status = if augmented_rank > coefficient.rank {
        LinearSystemStatus::Inconsistent
    } else if coefficient.rank == a.ncols() {
        LinearSystemStatus::Unique
    } else {
        LinearSystemStatus::UnderdeterminedConsistent
    };

    Ok(LinearConsistencyEvidence {
        status,
        coefficient_rank: coefficient.rank,
        augmented_rank,
        variable_count: a.ncols(),
        equation_count: a.nrows(),
        coefficient_condition_number: coefficient.condition_number,
        coefficient_classification: coefficient.classification,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn classify(a: &[&[f64]], b: &[f64]) -> LinearConsistencyEvidence {
        let rows = a.len();
        let cols = a[0].len();
        let matrix = DMatrix::from_fn(rows, cols, |r, c| a[r][c]);
        let rhs = DVector::from_column_slice(b);
        classify_linear_system(&matrix, &rhs, 1.0e-10, 1.0e10).unwrap()
    }

    #[test]
    fn full_column_rank_system_is_unique() {
        let evidence = classify(&[&[1.0, 0.0], &[0.0, 1.0]], &[3.0, -2.0]);
        assert_eq!(evidence.status, LinearSystemStatus::Unique);
        assert_eq!(evidence.coefficient_rank, 2);
        assert_eq!(evidence.augmented_rank, 2);
    }

    #[test]
    fn rank_deficient_consistent_system_is_underdetermined() {
        let evidence = classify(&[&[1.0, 1.0], &[2.0, 2.0]], &[2.0, 4.0]);
        assert_eq!(evidence.status, LinearSystemStatus::UnderdeterminedConsistent);
        assert_eq!(evidence.coefficient_rank, 1);
        assert_eq!(evidence.augmented_rank, 1);
    }

    #[test]
    fn augmented_rank_increase_proves_inconsistency() {
        let evidence = classify(&[&[1.0, 1.0], &[2.0, 2.0]], &[2.0, 5.0]);
        assert_eq!(evidence.status, LinearSystemStatus::Inconsistent);
        assert_eq!(evidence.coefficient_rank, 1);
        assert_eq!(evidence.augmented_rank, 2);
    }

    #[test]
    fn consistency_is_preserved_under_uniform_system_scaling() {
        let base = classify(&[&[1.0, 1.0], &[2.0, 2.0]], &[2.0, 4.0]);
        let scaled = classify(&[&[1.0e9, 1.0e9], &[2.0e9, 2.0e9]], &[2.0e9, 4.0e9]);
        assert_eq!(base.status, scaled.status);
        assert_eq!(base.coefficient_rank, scaled.coefficient_rank);
        assert_eq!(base.augmented_rank, scaled.augmented_rank);
    }

    #[test]
    fn nonfinite_system_is_rejected() {
        let a = DMatrix::from_row_slice(1, 1, &[f64::NAN]);
        let b = DVector::from_column_slice(&[1.0]);
        assert_eq!(
            classify_linear_system(&a, &b, 1.0e-10, 1.0e10),
            Err(LinAlgError::NonFinite)
        );
    }
}
