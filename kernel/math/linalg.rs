//! Numerical linear algebra owned by the UMLCAD mathematical authority.
//!
//! `nalgebra` is an implementation dependency of this module; its types are
//! not part of the semantic kernel contract. Callers provide tolerances
//! explicitly. Rank and symmetry decisions fail closed when the requested
//! numerical classification cannot be represented safely.

use nalgebra::{DMatrix, DVector};

use super::predicates::Tri;

pub const CANONICAL_RANK_TOL: f64 = 1.0e-10;
pub const CANONICAL_ILL_COND_THRESHOLD: f64 = 1.0e10;

#[derive(Clone, Debug, PartialEq)]
pub enum LinAlgError {
    NonFinite,
    EmptyMatrix,
    DimensionMismatch {
        lhs: (usize, usize),
        rhs: (usize, usize),
    },
    Singular,
    NotPositiveDefinite,
    InvalidTolerance,
    DecompositionFailed,
    Unsolvable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RankClassification {
    FullRank,
    RankDeficient,
    Singular,
    IllConditioned,
    Indeterminate,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RankEvidence {
    pub rank: usize,
    /// Dimension of the right null space: `n_columns - rank`.
    pub nullity: usize,
    pub min_dim: usize,
    /// `sigma_max / sigma_min_nonzero`, or `+∞` when rank is zero.
    pub condition_number: f64,
    pub largest_singular: f64,
    pub smallest_nonzero_singular: f64,
    pub classification: RankClassification,
}

#[derive(Clone, Debug)]
pub struct SvdResult {
    pub u: DMatrix<f64>,
    pub singular_values: DVector<f64>,
    pub v_t: DMatrix<f64>,
    pub evidence: RankEvidence,
}

fn valid_tolerances(rank_tol: f64, ill_cond_threshold: f64) -> bool {
    rank_tol.is_finite()
        && rank_tol >= 0.0
        && ill_cond_threshold.is_finite()
        && ill_cond_threshold >= 0.0
}

fn indeterminate_evidence(
    rows: usize,
    cols: usize,
    largest: f64,
    smallest_nonzero: f64,
    rank: usize,
) -> RankEvidence {
    RankEvidence {
        rank,
        nullity: cols.saturating_sub(rank),
        min_dim: rows.min(cols),
        condition_number: f64::INFINITY,
        largest_singular: largest,
        smallest_nonzero_singular: smallest_nonzero,
        classification: RankClassification::Indeterminate,
    }
}

fn classify_rank(
    singular_values: &[f64],
    rows: usize,
    cols: usize,
    rank_tol: f64,
    ill_cond_threshold: f64,
) -> RankEvidence {
    let min_dim = rows.min(cols);
    if singular_values.is_empty() {
        return RankEvidence {
            rank: 0,
            nullity: cols,
            min_dim,
            condition_number: f64::INFINITY,
            largest_singular: 0.0,
            smallest_nonzero_singular: 0.0,
            classification: RankClassification::Singular,
        };
    }

    let largest = singular_values[0];
    if !largest.is_finite() {
        return indeterminate_evidence(rows, cols, largest, 0.0, 0);
    }
    if largest <= 0.0 {
        return RankEvidence {
            rank: 0,
            nullity: cols,
            min_dim,
            condition_number: f64::INFINITY,
            largest_singular: 0.0,
            smallest_nonzero_singular: 0.0,
            classification: RankClassification::Singular,
        };
    }

    let threshold = rank_tol * largest;
    if !threshold.is_finite() {
        return indeterminate_evidence(rows, cols, largest, 0.0, 0);
    }

    let mut rank = 0usize;
    let mut smallest_nonzero = f64::INFINITY;
    for &sigma in singular_values.iter().take(min_dim) {
        if !sigma.is_finite() {
            return indeterminate_evidence(
                rows,
                cols,
                largest,
                if smallest_nonzero.is_finite() {
                    smallest_nonzero
                } else {
                    00.0
                },
                rank,
            );
        }
        if sigma > threshold {
            rank += 1;
            smallest_nonzero = smallest_nonzero.min(sigma);
        }
    }

    if rank == 0 {
        return RankEvidence {
            rank: 0,
            nullity: cols,
            min_dim,
            condition_number: f64::INFINITY,
            largest_singular: largest,
            smallest_nonzero_singular: 0.0,
            classification: RankClassification::Singular,
        };
    }

    let condition_number = largest / smallest_nonzero;
    let classification = if rank < min_dim {
        RankClassification::RankDeficient
    } else if !condition_number.is_finite() {
        RankClassification::Indeterminate
    } else if condition_number > ill_cond_threshold {
        RankClassification::IllConditioned
    } else {
        RankClassification::FullRank
    };

    RankEvidence {
        rank,
        nullity: cols - rank,
        min_dim,
        condition_number,
        largest_singular: largest,
        smallest_nonzero_singular: smallest_nonzero,
        classification,
    }
}

pub fn svd(
    matrix: &DMatrix<f64>,
    rank_tol: f64,
    ill_cond_threshold: f64,
) -> Result<SvdResult, LinAlgError> {
    if matrix.nrows() == 0 || matrix.ncols() == 0 {
        return Err(LinAlgError::EmptyMatrix);
    }
    if matrix.iter().any(|value| !value.is_finite()) {
        return Err(LinAlgError::NonFinite);
    }
    if !valid_tolerances(rank_tol, ill_cond_threshold) {
        return Err(LinAlgError::InvalidTolerance);
    }

    let decomposition = matrix.clone().svd(true, true);
    let u = decomposition.u.ok_or(LinAlgError::DecompositionFailed)?;
    let v_t = decomposition.v_t.ok_or(LinAlgError::DecompositionFailed)?;
    let singular_values = decomposition.singular_values;
    let evidence = classify_rank(
        singular_values.as_slice(),
        matrix.nrows(),
        matrix.ncols(),
        rank_tol,
        ill_cond_threshold,
    );

    Ok(SvdResult {
        u,
        singular_values,
        v_t,
        evidence,
    })
}

pub fn rank_evidence(
    matrix: &DMatrix<f64>,
    rank_tol: f64,
    ill_cond_threshold: f64,
) -> Result<RankEvidence, LinAlgError> {
    Ok(svd(matrix, rank_tol, ill_cond_threshold)?.evidence)
}

/// Moore-Penrose pseudo-inverse using the same rank threshold as [`svd`].
pub fn pseudo_inverse(
    matrix: &DMatrix<f64>,
    rank_tol: f64,
    ill_cond_threshold: f64,
) -> Result<DMatrix<f64>, LinAlgError> {
    let decomposition = svd(matrix, rank_tol, ill_cond_threshold)?;
    let singular_values = decomposition.singular_values.as_slice();
    let u = &decomposition.u;
    let v_t = &decomposition.v_t;
    let rows = matrix.nrows();
    let cols = matrix.ncols();
    let rank = decomposition.evidence.rank;

    let mut result = DMatrix::<f64>::zeros(cols, rows);
    for k in 0..rank {
        let sigma = singular_values[k];
        if sigma <= 0.0 || !sigma.is_finite() {
            continue;
        }
        let inverse = 1.0 / sigma;
        if !inverse.is_finite() {
            return Err(LinAlgError::Unsolvable);
        }
        for column in 0..cols {
            let v = v_t[(k, column)] * inverse;
            for row in 0..rows {
                result[(column, row)] += v * u[(row, k)];
            }
        }
    }
    if result.iter().any(|value| !value.is_finite()) {
        return Err(LinAlgError::Unsolvable);
    }
    Ok(result)
}

/// Return an orthonormal basis of the right null space as `n × (n-rank)`.
pub fn null_space(
    matrix: &DMatrix<f64>,
    rank_tol: f64,
    ill_cond_threshold: f64,
) -> Result<DMatrix<f64>, LinAlgError> {
    let decomposition = svd(matrix, rank_tol, ill_cond_threshold)?;
    let cols = matrix.ncols();
    let rank = decomposition.evidence.rank;
    let nullity = cols - rank;
    if nullity == 0 {
        return Ok(DMatrix::<f64>::zeros(cols, 0));
    }

    let mut orthonormal: Vec<Vec<f64>> = Vec::with_capacity(cols);
    for row in 0..rank {
        let basis = (0..cols)
            .map(|column| decomposition.v_t[(row, column)])
            .collect::<Vec<_>>();
        if basis.iter().any(|value| !value.is_finite()) {
            return Err(LinAlgError::DecompositionFailed);
        }
        orthonormal.push(basis);
    }

    let completion_tol = 32.0 * f64::EPSILON * (cols.max(1) as f64).sqrt();
    if !completion_tol.is_finite() {
        return Err(LinAlgError::DecompositionFailed);
    }

    let mut result = DMatrix::<f64>::zeros(cols, nullity);
    let mut count = 0usize;

    for canonical_index in 0..cols {
        if count == nullity {
            break;
        }
        let mut candidate = vec![0.0; cols];
        candidate[canonical_index] = 1.0;

        for existing in &orthonormal {
            let projection = candidate
                .iter()
                .zip(existing.iter())
                .map(|(a, b)| a * b)
                .sum::<f64>();
            if !projection.is_finite() {
                return Err(LinAlgError::DecompositionFailed);
            }
            for index in 0..cols {
                candidate[index] -= projection * existing[index];
            }
        }

        let norm = candidate
            .iter()
            .map(|value| value * value)
            .sum::<f64>()
            .sqrt();
        if !norm.is_finite() {
            return Err(LinAlgError::DecompositionFailed);
        }
        if norm <= completion_tol {
            continue;
        }
        for value in &mut candidate {
            *value /= norm;
        }
        if candidate.iter().any(|value| !value.is_finite()) {
            return Err(LinAlgError::DecompositionFailed);
        }
        for row in 0..cols {
            result[(row, count)] = candidate[row];
        }
        orthonormal.push(candidate);
        count += 1;
    }

    if count != nullity || result.iter().any(|value| !value.is_finite()) {
        return Err(LinAlgError::DecompositionFailed);
    }
    Ok(result)
}

pub fn solve_lu(a: &DMatrix<f64>, b: &DVector<f64>) -> Result<DVector<f64>, LinAlgError> {
    if a.nrows() == 0 || a.ncols() == 0 {
        return Err(LinAlgError::EmptyMatrix);
    }
    if a.nrows() != a.ncols() || b.len() != a.nrows() {
        return Err(LinAlgError::DimensionMismatch {
            lhs: (a.nrows(), a.ncols()),
            rhs: (b.len(), 1),
        });
    }
    if a.iter().any(|value| !value.is_finite()) || b.iter().any(|value| !value.is_finite()) {
        return Err(LinAlgError::NonFinite);
    }
    let solution = a.clone().lu().solve(b).ok_or(LinAlgError::Singular)?;
    if solution.iter().all(|value| value.is_finite()) {
        Ok(solution)
    } else {
        Err(LinAlgError::Unsolvable)
    }
}

pub fn solve_qr(a: &DMatrix<f64>, b: &DVector<f64>) -> Result<DVector<f64>, LinAlgError> {
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
    let solution = a.clone().qr().solve(b).ok_or(LinAlgError::Unsolvable)?;
    if solution.iter().all(|value| value.is_finite()) {
        Ok(solution)
    } else {
        Err(LinAlgError::Unsolvable)
    }
}

pub fn solve_svd(
    a: &DMatrix<f64>,
    b: &DVector<f64>,
    rank_tol: f64,
    ill_cond_threshold: f64,
) -> Result<DVector<f64>, LinAlgError> {
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
    let result = pseudo_inverse(a, rank_tol, ill_cond_threshold)? * b;
    if result.iter().all(|value| value.is_finite()) {
        Ok(result)
    } else {
        Err(LinAlgError::Unsolvable)
    }
}

/// Positive definiteness is meaningful only for a finite square symmetric
/// matrix. Symmetry is checked with a dimensionless relative tolerance.
pub fn is_positive_definite(a: &DMatrix<f64>, tol: f64) -> Tri {
    if a.nrows() == 0 || a.nrows() != a.ncols() || !tol.is_finite() || tol < 0.0 {
        return Tri::Indeterminate;
    }
    if a.iter().any(|value| !value.is_finite()) {
        return Tri::Indeterminate;
    }
    let scale = a.iter().map(|value| value.abs()).fold(0.0, f64::max);
    if scale == 0.0 {
        return Tri::False;
    }
    let symmetry_band = tol * scale;
    if !symmetry_band.is_finite() {
        return Tri::Indeterminate;
    }
    for row in 0..a.nrows() {
        for column in 0..row {
            let difference = (a[(row, column)] - a[(column, row)]).abs();
            if !difference.is_finite() {
                return Tri::Indeterminate;
            }
            if difference > symmetry_band {
                return Tri::Indeterminate;
            }
        }
    }
    if a.clone().cholesky().is_some() {
        Tri::True
    } else {
        Tri::False
    }
}

pub fn from_rows(data: &[Vec<f64>]) -> Result<DMatrix<f64>, LinAlgError> {
    if data.is_empty() || data[0].is_empty() {
        return Err(LinAlgError::EmptyMatrix);
    }
    let cols = data[0].len();
    if data.iter().any(|row| row.len() != cols) {
        return Err(LinAlgError::DimensionMismatch {
            lhs: (data.len(), cols),
            rhs: (0, 0),
        });
    }
    if data.iter().flatten().any(|value| !value.is_finite()) {
        return Err(LinAlgError::NonFinite);
    }
    let mut matrix = DMatrix::<f64>::zeros(data.len(), cols);
    for (row, values) in data.iter().enumerate() {
        for (column, value) in values.iter().copied().enumerate() {
            matrix[(row, column)] = value;
        }
    }
    Ok(matrix)
}

#[cfg(test)]
mod tests {
    use super::*;

    const RTOL: f64 = CANONICAL_RANK_TOL;
    const ICT: f64 = CANONICAL_ILL_COND_THRESHOLD;

    fn matrix(rows: &[&[f64]]) -> DMatrix<f64> {
        from_rows(&rows.iter().map(|row| row.to_vec()).collect::<Vec<_>>()).unwrap()
    }

    fn vector(values: &[f64]) -> DVector<f64> {
        DVector::from_vec(values.to_vec())
    }

    fn assert_matrix_close(a: &DMatrix<f64>, b: &DMatrix<f64>, tolerance: f64) {
        assert_eq!(a.shape(), b.shape());
        for row in 0..a.nrows() {
            for column in 0..a.ncols() {
                assert!(
                    (a[(row, column)] - b[(row, column)]).abs() <= tolerance,
                    "({},{}): {} vs {}",
                    row,
                    column,
                    a[(row, column)],
                    b[(row, column)]
                );
            }
        }
    }

    #[test]
    fn svd_classifies_full_rank_and_rank_deficiency() {
        let full = svd(&matrix(&[&[1.0, 0.0], &[0.0, 2.0]]), RTOL, ICT).unwrap();
        assert_eq!(full.evidence.rank, 2);
        assert_eq!(full.evidence.classification, RankClassification::FullRank);

        let singular = svd(&matrix(&[&[1.0, 2.0], &[2.0, 4.0]]), RTOL, ICT).unwrap();
        assert_eq!(singular.evidence.rank, 1);
        assert_eq!(singular.evidence.classification, RankClassification::RankDeficient);
    }

    #[test]
    fn rank_is_invariant_under_uniform_small_and_large_scaling() {
        let base = matrix(&[&[1.0, 2.0], &[3.0, 4.0]]);
        for factor in [1.0e-12, 1.0e12] {
            let scaled = &base * factor;
            let a = rank_evidence(&base, RTOL, ICT).unwrap();
            let b = rank_evidence(&scaled, RTOL, ICT).unwrap();
            assert_eq!(a.rank, b.rank);
            assert_eq!(a.classification, b.classification);
        }
    }

    #[test]
    fn pseudo_inverse_satisfies_reflexive_identity_for_rank_deficiency() {
        let a = matrix(&[&[1.0, 2.0], &[2.0, 4.0]]);
        let ap = pseudo_inverse(&a, RTOL, ICT).unwrap();
        assert_matrix_close(&(&a * &ap * &a), &a, 1.0e-10);
    }

    #[test]
    fn null_space_has_expected_dimension_and_is_orthogonal_to_rows() {
        let a = matrix(&[&[1.0, 1.0, 1.0]]);
        let ns = null_space(&a, RTOL, ICT).unwrap();
        assert_eq!(ns.nrows(), 3);
        assert_eq!(ns.ncols(), 2);
        let residual = &a * &ns;
        assert!(residual.iter().all(|value| value.abs() < 1.0e-10));
    }

    #[test]
    fn null_space_handles_a_wide_rank_deficient_matrix() {
        let a = matrix(&[&[1.0, 0.0, 0.0], &[0.0, 0.0, 0.0]]);
        let ns = null_space(&a, RTOL, ICT).unwrap();
        assert_eq!(ns.nrows(), 3);
        assert_eq!(ns.ncols(), 2);
        let residual = &a * &ns;
        assert!(residual.iter().all(|value| value.abs() < 1.0e-10));
    }

    #[test]
    fn lu_and_svd_agree_on_a_well_conditioned_square_system() {
        let a = matrix(&[&[3.0, 1.0], &[1.0, 2.0]]);
        let b = vector(&[5.0, 5.0]);
        let lu = solve_lu(&a, &b).unwrap();
        let svd = solve_svd(&a, &b, RTOL, ICT).unwrap();
        for index in 0..lu.len() {
            assert!((lu[index] - svd[index]).abs() < 1.0e-10);
        }
    }

    #[test]
    fn qr_solves_an_overdetermined_exact_system() {
        let a = matrix(&[&[0.0, 1.0], &[1.0, 1.0], &[2.0, 1.0]]);
        let b = vector(&[1.0, 3.0, 5.0]);
        let x = solve_qr(&a, &b).unwrap();
        assert!((x[0] - 2.0).abs() < 1.0e-10);
        assert!((x[1] - 1.0).abs() < 1.0e-10);
    }

    #[test]
    fn positive_definiteness_has_explicit_indeterminate_state_for_nonsymmetry() {
        let spd = matrix(&[&[2.0, 1.0], &[1.0, 2.0]]);
        assert_eq!(is_positive_definite(&spd, 1.0e-12), Tri::True);

        let nonsymmetric = matrix(&[&[2.0, 2.0], &[1.0, 2.0]]);
        assert_eq!(is_positive_definite(&nonsymmetric, 1.0e-12), Tri::Indeterminate);
    }

    #[test]
    fn nonfinite_inputs_are_rejected() {
        let a = matrix(&[&[1.0, 2.0], &[3.0, 4.0]]);
        let bad = DMatrix::from_row_slice(2, 2, &[1.0, f64::NAN, 3.0, 4.0]);
        assert!(matches!(svd(&bad, RTOL, ICT), Err(LinAlgError::NonFinite)));
        assert_eq!(solve_lu(&bad, &vector(&[1.0, 2.0])), Err(LinAlgError::NonFinite));
        assert!(is_positive_definite(&bad, RTOL).is_indeterminate());
        assert_eq!(rank_evidence(&a, -1.0, ICT), Err(LinAlgError::InvalidTolerance));
    }

    #[test]
    fn overflowed_rank_threshold_is_indeterminate() {
        let a = matrix(&[&[1.0e308, 0.0], &[0.0, 1.0e308]]);
        let evidence = rank_evidence(&a, 1.0e308, ICT).unwrap();
        assert_eq!(evidence.classification, RankClassification::Indeterminate);
    }

    #[test]
    fn overflowed_symmetry_band_is_indeterminate() {
        let a = matrix(&[&[1.0e308, 1.0], &[0.0, 1.0e308]]);
        assert_eq!(is_positive_definite(&a, 1.0e308), Tri::Indeterminate);
    }
}
