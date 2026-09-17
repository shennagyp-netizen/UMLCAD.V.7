//! Fixed-size matrix mathematics for the UMLCAD authority.
//!
//! Matrices are immutable value types. Inversion and solution operations take
//! explicit tolerances and fail rather than silently accepting singularity.

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MatrixError {
    NonFinite,
    Singular,
    InvalidTolerance,
    DimensionMismatch,
    Overflow,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mat2 {
    pub m: [[f64; 2]; 2],
}

impl Mat2 {
    pub const IDENTITY: Self = Self {
        m: [[1.0, 0.0], [0.0, 1.0]],
    };

    #[inline]
    pub const fn new(m: [[f64; 2]; 2]) -> Self {
        Self { m }
    }

    #[inline]
    pub fn transpose(self) -> Self {
        Self::new([
            [self.m[0][0], self.m[1][0]],
            [self.m[0][1], self.m[1][1]],
        ])
    }

    #[inline]
    pub fn determinant(self) -> f64 {
        self.m[0][0] * self.m[1][1] - self.m[0][1] * self.m[1][0]
    }

    pub fn inverse(self, tolerance: f64) -> Result<Self, MatrixError> {
        validate_matrix2(self, tolerance)?;
        let scale = self.max_abs();
        if scale == 0.0 {
            return Err(MatrixError::Singular);
        }
        let a = [
            [self.m[0][0] / scale, self.m[0][1] / scale],
            [self.m[1][0] / scale, self.m[1][1] / scale],
        ];
        let det = a[0][0] * a[1][1] - a[0][1] * a[1][0];
        if !det.is_finite() {
            return Err(MatrixError::Overflow);
        }
        if det.abs() <= tolerance {
            return Err(MatrixError::Singular);
        }
        let inv_det = 1.0 / det;
        let result = Self::new([
            [a[1][1] * inv_det / scale, -a[0][1] * inv_det / scale],
            [-a[1][0] * inv_det / scale, a[0][0] * inv_det / scale],
        ]);
        if result.is_finite() {
            Ok(result)
        } else {
            Err(MatrixError::Overflow)
        }
    }

    pub fn solve(self, b: [f64; 2], tolerance: f64) -> Result<[f64; 2], MatrixError> {
        if b.iter().any(|v| !v.is_finite()) {
            return Err(MatrixError::NonFinite);
        }
        let inv = self.inverse(tolerance)?;
        let result = [
            inv.m[0][0] * b[0] + inv.m[0][1] * b[1],
            inv.m[1][0] * b[0] + inv.m[1][1] * b[1],
        ];
        if result.iter().all(|v| v.is_finite()) {
            Ok(result)
        } else {
            Err(MatrixError::Overflow)
        }
    }

    pub fn mul(self, other: Self) -> Self {
        let mut out = [[0.0; 2]; 2];
        for i in 0..2 {
            for j in 0..2 {
                for k in 0..2 {
                    out[i][j] += self.m[i][k] * other.m[k][j];
                }
            }
        }
        Self::new(out)
    }

    pub fn max_abs(self) -> f64 {
        self.m.iter().flatten().map(|v| v.abs()).fold(0.0, f64::max)
    }

    pub fn norm_inf(self) -> f64 {
        (0..2)
            .map(|i| self.m[i].iter().map(|v| v.abs()).sum())
            .fold(0.0, f64::max)
    }

    pub fn condition_estimate(self, tolerance: f64) -> Result<f64, MatrixError> {
        let inverse = self.inverse(tolerance)?;
        let condition = self.norm_inf() * inverse.norm_inf();
        if condition.is_finite() {
            Ok(condition)
        } else {
            Err(MatrixError::Overflow)
        }
    }

    fn is_finite(self) -> bool {
        self.m.iter().flatten().all(|v| v.is_finite())
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mat3 {
    pub m: [[f64; 3]; 3],
}

impl Mat3 {
    pub const IDENTITY: Self = Self {
        m: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
    };

    #[inline]
    pub const fn new(m: [[f64; 3]; 3]) -> Self {
        Self { m }
    }

    pub fn transpose(self) -> Self {
        let mut out = [[0.0; 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                out[i][j] = self.m[j][i];
            }
        }
        Self::new(out)
    }

    pub fn determinant(self) -> f64 {
        let a = self.m;
        a[0][0] * (a[1][1] * a[2][2] - a[1][2] * a[2][1])
            - a[0][1] * (a[1][0] * a[2][2] - a[1][2] * a[2][0])
            + a[0][2] * (a[1][0] * a[2][1] - a[1][1] * a[2][0])
    }

    pub fn inverse(self, tolerance: f64) -> Result<Self, MatrixError> {
        validate_matrix3(self, tolerance)?;
        let scale = self.max_abs();
        if scale == 0.0 {
            return Err(MatrixError::Singular);
        }
        let a: [[f64; 3]; 3] = std::array::from_fn(|i| std::array::from_fn(|j| self.m[i][j] / scale));
        let det = a[0][0] * (a[1][1] * a[2][2] - a[1][2] * a[2][1])
            - a[0][1] * (a[1][0] * a[2][2] - a[1][2] * a[2][0])
            + a[0][2] * (a[1][0] * a[2][1] - a[1][1] * a[2][0]);
        if !det.is_finite() {
            return Err(MatrixError::Overflow);
        }
        if det.abs() <= tolerance {
            return Err(MatrixError::Singular);
        }
        let c = [
            [
                a[1][1] * a[2][2] - a[1][2] * a[2][1],
                a[0][2] * a[2][1] - a[0][1] * a[2][2],
                a[0][1] * a[1][2] - a[0][2] * a[1][1],
            ],
            [
                a[1][2] * a[2][0] - a[1][0] * a[2][2],
                a[0][0] * a[2][2] - a[0][2] * a[2][0],
                a[0][2] * a[1][0] - a[0][0] * a[1][2],
            ],
            [
                a[1][0] * a[2][1] - a[1][1] * a[2][0],
                a[0][1] * a[2][0] - a[0][0] * a[2][1],
                a[0][0] * a[1][1] - a[0][1] * a[1][0],
            ],
        ];
        let mut out = [[0.0; 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                out[i][j] = c[i][j] / det / scale;
            }
        }
        let result = Self::new(out);
        if result.is_finite() {
            Ok(result)
        } else {
            Err(MatrixError::Overflow)
        }
    }

    pub fn solve(self, b: [f64; 3], tolerance: f64) -> Result<[f64; 3], MatrixError> {
        if b.iter().any(|v| !v.is_finite()) {
            return Err(MatrixError::NonFinite);
        }
        let inv = self.inverse(tolerance)?;
        let result: [f64; 3] = std::array::from_fn(|i| (0..3).map(|j| inv.m[i][j] * b[j]).sum());
        if result.iter().all(|v| v.is_finite()) {
            Ok(result)
        } else {
            Err(MatrixError::Overflow)
        }
    }

    pub fn mul(self, other: Self) -> Self {
        let mut out = [[0.0; 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                for k in 0..3 {
                    out[i][j] += self.m[i][k] * other.m[k][j];
                }
            }
        }
        Self::new(out)
    }

    pub fn max_abs(self) -> f64 {
        self.m.iter().flatten().map(|v| v.abs()).fold(0.0, f64::max)
    }

    pub fn norm_inf(self) -> f64 {
        (0..3)
            .map(|i| self.m[i].iter().map(|v| v.abs()).sum())
            .fold(0.0, f64::max)
    }

    pub fn condition_estimate(self, tolerance: f64) -> Result<f64, MatrixError> {
        let inverse = self.inverse(tolerance)?;
        let condition = self.norm_inf() * inverse.norm_inf();
        if condition.is_finite() {
            Ok(condition)
        } else {
            Err(MatrixError::Overflow)
        }
    }

    fn is_finite(self) -> bool {
        self.m.iter().flatten().all(|v| v.is_finite())
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mat4 {
    pub m: [[f64; 4]; 4],
}

impl Mat4 {
    pub const IDENTITY: Self = Self {
        m: [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ],
    };

    #[inline]
    pub const fn new(m: [[f64; 4]; 4]) -> Self {
        Self { m }
    }

    pub fn transpose(self) -> Self {
        let mut out = [[0.0; 4]; 4];
        for i in 0..4 {
            for j in 0..4 {
                out[i][j] = self.m[j][i];
            }
        }
        Self::new(out)
    }

    pub fn determinant(self) -> f64 {
        let mut a = self.m;
        let mut det = 1.0;
        for col in 0..4 {
            let mut pivot = col;
            for row in (col + 1)..4 {
                if a[row][col].abs() > a[pivot][col].abs() {
                    pivot = row;
                }
            }
            if a[pivot][col] == 0.0 {
                return 000.0;
            }
            if pivot != col {
                a.swap(pivot, col);
                det = -det;
            }
            let p = a[col][col];
            det *= p;
            for row in (col + 1)..4 {
                let factor = a[row][col] / p;
                for k in (col + 1)..4 {
                    a[row][k] -= factor * a[col][k];
                }
            }
        }
        det
    }

    pub fn inverse(self, tolerance: f64) -> Result<Self, MatrixError> {
        validate_matrix4(self, tolerance)?;
        let scale = self.max_abs();
        if scale == 0.0 {
            return Err(MatrixError::Singular);
        }
        let mut a = [[0.0; 8]; 4];
        for i in 0..4 {
            for j in 0..4 {
                a[i][j] = self.m[i][j] / scale;
            }
            a[i][4 + i] = 1.0;
        }
        for col in 0..4 {
            let mut pivot = col;
            for row in (col + 1)..4 {
                if a[row][col].abs() > a[pivot][col].abs() {
                    pivot = row;
                }
            }
            if a[pivot][col].abs() <= tolerance {
                return Err(MatrixError::Singular);
            }
            if pivot != col {
                a.swap(pivot, col);
            }
            let p = a[col][col];
            for j in 0..8 {
                a[col][j] /= p;
            }
            for row in 0..4 {
                if row == col {
                    continue;
                }
                let factor = a[row][col];
                for j in 0..8 {
                    a[row][j] -= factor * a[col][j];
                }
            }
        }
        let mut out = [[0.0; 4]; 4];
        for i in 0..4 {
            for j in 0..4 {
                out[i][j] = a[i][4 + j] / scale;
            }
        }
        let result = Self::new(out);
        if result.is_finite() {
            Ok(result)
        } else {
            Err(MatrixError::Overflow)
        }
    }

    pub fn solve(self, b: [f64; 4], tolerance: f64) -> Result<[f64; 4], MatrixError> {
        if b.iter().any(|v| !v.is_finite()) {
            return Err(MatrixError::NonFinite);
        }
        let inverse = self.inverse(tolerance)?;
        let result: [f64; 4] = std::array::from_fn(|i| (0..4).map(|j| inverse.m[i][j] * b[j]).sum());
        if result.iter().all(|v| v.is_finite()) {
            Ok(result)
        } else {
            Err(MatrixError::Overflow)
        }
    }

    pub fn mul(self, other: Self) -> Self {
        let mut out = [[0.0; 4]; 4];
        for i in 0..4 {
            for j in 0..4 {
                for k in 0..4 {
                    out[i][j] += self.m[i][k] * other.m[k][j];
                }
            }
        }
        Self::new(out)
    }

    pub fn max_abs(self) -> f64 {
        self.m.iter().flatten().map(|v| v.abs()).fold(0.0, f64::max)
    }

    pub fn norm_inf(self) -> f64 {
        (0..4)
            .map(|i| self.m[i].iter().map(|v| v.abs()).sum())
            .fold(0.0, f64::max)
    }

    pub fn condition_estimate(self, tolerance: f64) -> Result<f64, MatrixError> {
        let inverse = self.inverse(tolerance)?;
        let condition = self.norm_inf() * inverse.norm_inf();
        if condition.is_finite() {
            Ok(condition)
        } else {
            Err(MatrixError::Overflow)
        }
    }

    fn is_finite(self) -> bool {
        self.m.iter().flatten().all(|v| v.is_finite())
    }
}

fn validate_matrix2(m: Mat2, tolerance: f64) -> Result<(), MatrixError> {
    if !m.is_finite() {
        return Err(MatrixError::NonFinite);
    }
    validate_tolerance(tolerance)
}

fn validate_matrix3(m: Mat3, tolerance: f64) -> Result<(), MatrixError> {
    if !m.is_finite() {
        return Err(MatrixError::NonFinite);
    }
    validate_tolerance(tolerance)
}

fn validate_matrix4(m: Mat4, tolerance: f64) -> Result<(), MatrixError> {
    if !m.is_finite() {
        return Err(MatrixError::NonFinite);
    }
    validate_tolerance(tolerance)
}

fn validate_tolerance(tolerance: f64) -> Result<(), MatrixError> {
    if tolerance.is_finite() && tolerance >= 0.0 {
        Ok(())
    } else {
        Err(MatrixError::InvalidTolerance)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const TOL: f64 = 1.0e-12;

    #[test]
    fn mat2_inverse_round_trip() {
        let a = Mat2::new([[2.0, 1.0], [1.0, 3.0]]);
        let product = a.mul(a.inverse(TOL).unwrap());
        assert!((product.m[0][0] - 1.0).abs() < 1.0e-12);
        assert!(product.m[0][1].abs() < 1.0e-12);
        assert!(product.m[1][0].abs() < 1.0e-12);
        assert!((product.m[1][1] - 1.0).abs() < 1.0e-12);
    }

    #[test]
    fn mat3_determinant_and_solve_are_exact_for_small_system() {
        let a = Mat3::new([[1.0, 2.0, 3.0], [0.0, 1.0, 4.0], [5.0, 6.0, 0.0]]);
        assert!((a.determinant() - 1.0).abs() < 1.0e-12);
        let x = a.solve([14.0, 14.0, 23.0], TOL).unwrap();
        assert!((x[0] - 1.0).abs() < 1.0e-12);
        assert!((x[1] - 2.0).abs() < 1.0e-12);
        assert!((x[2] - 3.0).abs() < 1.0e-12);
    }

    #[test]
    fn mat4_identity_inverse_and_condition() {
        let a = Mat4::IDENTITY;
        assert_eq!(a.inverse(TOL).unwrap(), a);
        assert!((a.condition_estimate(TOL).unwrap() - 1.0).abs() < 1.0e-12);
    }

    #[test]
    fn inverse_classification_is_invariant_under_uniform_scaling() {
        let a2 = Mat2::new([[2.0, 1.0], [1.0, 3.0]]);
        let a3 = Mat3::new([[2.0, 1.0, 0.5], [1.0, 3.0, 1.0], [0.5, 1.0, 4.0]]);
        let a4 = Mat4::new([
            [2.0, 1.0, 0.0, 0.0],
            [1.0, 3.0, 1.0, 0.0],
            [0.0, 1.0, 4.0, 1.0],
            [0.0, 0.0, 1.0, 2.0],
        ]);
        for scale in [1.0e-12, 1.0e12] {
            let inv2 = a2.inverse(TOL).unwrap();
            let scaled2 = Mat2::new(a2.m.map(|row| row.map(|v| v * scale)));
            let scaled_inv2 = scaled2.inverse(TOL).unwrap();
            for i in 0..2 {
                for j in 0..2 {
                    let recovered = scaled_inv2.m[i][j] * scale;
                    let reference = inv2.m[i][j];
                    let error = (recovered - reference).abs();
                    let scale = reference.abs().max(1.0);
                    assert!(error <= 1.0e-10 * scale);
                }
            }

            let scaled3 = Mat3::new(std::array::from_fn(|i| {
                std::array::from_fn(|j| a3.m[i][j] * scale)
            }));
            assert!(scaled3.inverse(TOL).is_ok());

            let scaled4 = Mat4::new(std::array::from_fn(|i| {
                std::array::from_fn(|j| a4.m[i][j] * scale)
            }));
            assert!(scaled4.inverse(TOL).is_ok());
        }
    }

    #[test]
    fn singular_matrices_fail_closed() {
        assert_eq!(
            Mat2::new([[1.0, 2.0], [2.0, 4.0]]).inverse(TOL),
            Err(MatrixError::Singular)
        );
        assert_eq!(
            Mat3::new([[1.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 1.0]]).inverse(TOL),
            Err(MatrixError::Singular)
        );
        assert_eq!(
            Mat4::new([
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 0.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ])
            .inverse(TOL),
            Err(MatrixError::Singular)
        );
    }

    #[test]
    fn nonfinite_matrix_is_rejected() {
        assert_eq!(
            Mat2::new([[f64::NAN, 0.0], [0.0, 1.0]]).inverse(TOL),
            Err(MatrixError::NonFinite)
        );
    }
}
