//! Dimensionally explicit scaling of nonlinear least-squares linearizations.
//!
//! A residual row and its Jacobian row represent the same equation. When a
//! semantic residual scale is applied, both must be divided by the same positive
//! finite scale. This keeps the linearized equation dimensionally consistent
//! with the residual quantity used by acceptance and convergence authorities.
//!
//! This module is backend-independent mathematical infrastructure. It does not
//! choose tolerances, solve the system, or redefine semantic equations.

#[derive(Clone, Debug, PartialEq)]
pub struct RowScaledLinearSystem {
    pub jacobian: Vec<Vec<f64>>,
    pub residual: Vec<f64>,
    pub row_scales: Vec<f64>,
}

/// Apply explicit positive row scales to both a Jacobian and residual vector.
///
/// For every row `i`, the returned equation is
/// `(J_i / s_i) * delta = -(r_i / s_i)`.
///
/// Invalid dimensions, non-finite values, or non-positive scales fail closed.
pub fn scale_linearization(
    jacobian: &[Vec<f64>],
    residual: &[f64],
    row_scales: &[f64],
) -> Result<RowScaledLinearSystem, String> {
    if jacobian.len() != residual.len() || residual.len() != row_scales.len() {
        return Err("linearization row/scale length mismatch".into());
    }

    let columns = jacobian.first().map_or(0, Vec::len);
    if jacobian.iter().any(|row| row.len() != columns) {
        return Err("ragged linearization Jacobian".into());
    }

    if residual.iter().any(|value| !value.is_finite())
        || jacobian
            .iter()
            .flat_map(|row| row.iter())
            .any(|value| !value.is_finite())
        || row_scales
            .iter()
            .any(|scale| !scale.is_finite() || *scale <= 0.0)
    {
        return Err("non-finite or non-positive linearization evidence".into());
    }

    let mut scaled_jacobian = Vec::with_capacity(jacobian.len());
    let mut scaled_residual = Vec::with_capacity(residual.len());
    for ((row, value), scale) in jacobian.iter().zip(residual).zip(row_scales) {
        let scaled_row = row.iter().map(|entry| *entry / *scale).collect::<Vec<_>>();
        let scaled_value = *value / *scale;
        if scaled_row.iter().any(|entry| !entry.is_finite()) || !scaled_value.is_finite() {
            return Err("non-finite scaled linearization".into());
        }
        scaled_jacobian.push(scaled_row);
        scaled_residual.push(scaled_value);
    }

    Ok(RowScaledLinearSystem {
        jacobian: scaled_jacobian,
        residual: scaled_residual,
        row_scales: row_scales.to_vec(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn residual_norm(values: &[f64]) -> f64 {
        values.iter().map(|value| value * value).sum::<f64>().sqrt()
    }

    #[test]
    fn row_scaling_applies_identically_to_equation_and_jacobian_row() {
        let system = scale_linearization(
            &[vec![2.0, -4.0], vec![3.0, 6.0]],
            &[8.0, -9.0],
            &[4.0, 3.0],
        )
        .unwrap();

        assert_eq!(system.jacobian, vec![vec![0.5, -1.0], vec![1.0, 2.0]]);
        assert_eq!(system.residual, vec![2.0, -3.0]);
        assert_eq!(system.row_scales, vec![4.0, 3.0]);
    }

    #[test]
    fn positive_row_scaling_preserves_exact_square_solution() {
        // J * delta = -r has exact solution delta = [2, -1].
        let jacobian = vec![vec![2.0, 1.0], vec![1.0, -1.0]];
        let residual = vec![-3.0, -3.0];
        let unscaled = scale_linearization(&jacobian, &residual, &[1.0, 1.0]).unwrap();
        let scaled = scale_linearization(&jacobian, &residual, &[7.0, 0.25]).unwrap();

        // Closed-form 2x2 solve, kept local and independent of nalgebra.
        fn solve_2x2(j: &[Vec<f64>], r: &[f64]) -> [f64; 2] {
            let determinant = j[0][0] * j[1][1] - j[0][1] * j[1][0];
            [
                (-r[0] * j[1][1] + j[0][1] * r[1]) / determinant,
                (j[0][0] * -r[1] + r[0] * j[1][0]) / determinant,
            ]
        }

        let a = solve_2x2(&unscaled.jacobian, &unscaled.residual);
        let b = solve_2x2(&scaled.jacobian, &scaled.residual);
        assert!((a[0] - 2.0).abs() <= 1.0e-15);
        assert!((a[1] + 1.0).abs() <= 1.0e-15);
        assert!((a[0] - b[0]).abs() <= 1.0e-14);
        assert!((a[1] - b[1]).abs() <= 1.0e-14);
    }

    #[test]
    fn scaling_reduces_mixed_units_to_dimensionless_row_evidence() {
        let system = scale_linearization(
            &[vec![10.0, 2.0], vec![3.0, 4.0], vec![0.0, 1.0]],
            &[5.0, 0.6, -0.25],
            &[10.0, 2.0, 0.5],
        )
        .unwrap();
        assert_eq!(system.residual, vec![0.5, 0.3, -0.5]);
        assert_eq!(system.jacobian[0], vec![1.0, 0.2]);
        assert_eq!(system.jacobian[1], vec![1.5, 2.0]);
        assert_eq!(system.jacobian[2], vec![0.0, 2.0]);
        assert!(residual_norm(&system.residual).is_finite());
    }

    #[test]
    fn invalid_dimensions_and_scales_fail_closed() {
        assert!(scale_linearization(&[vec![1.0, 2.0]], &[1.0, 2.0], &[1.0]).is_err());
        assert!(scale_linearization(&[vec![1.0], vec![2.0]], &[1.0, 2.0], &[1.0, 0.0]).is_err());
        assert!(scale_linearization(&[vec![1.0]], &[f64::NAN], &[1.0]).is_err());
        assert!(scale_linearization(&[vec![f64::INFINITY]], &[1.0], &[1.0]).is_err());
        assert!(scale_linearization(&[vec![1.0]], &[1.0], &[-1.0]).is_err());
    }
}
