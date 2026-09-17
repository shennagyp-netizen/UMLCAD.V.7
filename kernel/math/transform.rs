//! Affine and rigid transformation mathematics.
//!
//! Points, vectors, and normals deliberately have different transformation
//! semantics. A normal under a general affine transform is transformed by the
//! inverse-transpose of the linear part, then renormalized.

use super::{
    mat::{Mat2, Mat3, MatrixError},
    quaternion::{Quaternion, QuaternionError},
    vec::{Vec2, Vec3, VectorError},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransformError {
    NonFinite,
    Singular,
    InvalidTolerance,
    Degenerate,
}

impl From<MatrixError> for TransformError {
    fn from(value: MatrixError) -> Self {
        match value {
            MatrixError::NonFinite | MatrixError::Overflow => TransformError::NonFinite,
            MatrixError::Singular => TransformError::Singular,
            MatrixError::InvalidTolerance => TransformError::InvalidTolerance,
            MatrixError::DimensionMismatch => TransformError::Degenerate,
        }
    }
}

impl From<QuaternionError> for TransformError {
    fn from(value: QuaternionError) -> Self {
        match value {
            QuaternionError::NonFinite | QuaternionError::Overflow => TransformError::NonFinite,
            QuaternionError::Degenerate => TransformError::Degenerate,
        }
    }
}

fn map_vector_error(error: VectorError) -> TransformError {
    match error {
        VectorError::NonFinite | VectorError::Overflow => TransformError::NonFinite,
        VectorError::Degenerate => TransformError::Degenerate,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform2 {
    pub linear: Mat2,
    pub translation: Vec2,
}

impl Transform2 {
    pub const IDENTITY: Self = Self {
        linear: Mat2::IDENTITY,
        translation: Vec2::new(0.0, 0.0),
    };

    pub fn is_finite(self) -> bool {
        self.linear.m.iter().flatten().all(|v| v.is_finite()) && self.translation.is_finite()
    }

    pub fn point(self, p: Vec2) -> Result<Vec2, TransformError> {
        if !self.is_finite() || !p.is_finite() {
            return Err(TransformError::NonFinite);
        }
        let result = Vec2::new(
            self.linear.m[0][0] * p.x + self.linear.m[0][1] * p.y + self.translation.x,
            self.linear.m[1][0] * p.x + self.linear.m[1][1] * p.y + self.translation.y,
        );
        if result.is_finite() {
            Ok(result)
        } else {
            Err(TransformError::NonFinite)
        }
    }

    pub fn vector(self, v: Vec2) -> Result<Vec2, TransformError> {
        if !self.is_finite() || !v.is_finite() {
            return Err(TransformError::NonFinite);
        }
        let result = Vec2::new(
            self.linear.m[0][0] * v.x + self.linear.m[0][1] * v.y,
            self.linear.m[1][0] * v.x + self.linear.m[1][1] * v.y,
        );
        if result.is_finite() {
            Ok(result)
        } else {
            Err(TransformError::NonFinite)
        }
    }

    pub fn normal(self, n: Vec2, tolerance: f64) -> Result<Vec2, TransformError> {
        if !self.is_finite() || !n.is_finite() {
            return Err(TransformError::NonFinite);
        }
        let inverse_transpose = self
            .linear
            .inverse(tolerance)
            .map_err(TransformError::from)?
            .transpose();
        let transformed = Vec2::new(
            inverse_transpose.m[0][0] * n.x + inverse_transpose.m[0][1] * n.y,
            inverse_transpose.m[1][0] * n.x + inverse_transpose.m[1][1] * n.y,
        );
        if !transformed.is_finite() {
            return Err(TransformError::NonFinite);
        }
        transformed.normalized().map_err(map_vector_error)
    }

    pub fn compose(self, other: Self) -> Result<Self, TransformError> {
        if !self.is_finite() || !other.is_finite() {
            return Err(TransformError::NonFinite);
        }
        let linear = self.linear.mul(other.linear);
        let translation = self
            .point(other.translation)
            .map_err(|error| error)?;
        let result = Self { linear, translation };
        if result.is_finite() {
            Ok(result)
        } else {
            Err(TransformError::NonFinite)
        }
    }

    pub fn inverse(self, tolerance: f64) -> Result<Self, TransformError> {
        if !self.is_finite() {
            return Err(TransformError::NonFinite);
        }
        let inv = self.linear.inverse(tolerance).map_err(TransformError::from)?;
        let neg = Vec2::new(-self.translation.x, -self.translation.y);
        let translation = Vec2::new(
            inv.m[0][0] * neg.x + inv.m[0][1] * neg.y,
            inv.m[1][0] * neg.x + inv.m[1][1] * neg.y,
        );
        let result = Self {
            linear: inv,
            translation,
        };
        if result.is_finite() {
            Ok(result)
        } else {
            Err(TransformError::NonFinite)
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform3 {
    pub linear: Mat3,
    pub translation: Vec3,
}

impl Transform3 {
    pub const IDENTITY: Self = Self {
        linear: Mat3::IDENTITY,
        translation: Vec3::new(0.0, 0.0, 0.0),
    };

    pub fn is_finite(self) -> bool {
        self.linear.m.iter().flatten().all(|v| v.is_finite()) && self.translation.is_finite()
    }

    pub fn point(self, p: Vec3) -> Result<Vec3, TransformError> {
        if !self.is_finite() || !p.is_finite() {
            return Err(TransformError::NonFinite);
        }
        let result = self.apply_linear(p).add(self.translation);
        if result.is_finite() {
            Ok(result)
        } else {
            Err(TransformError::NonFinite)
        }
    }

    pub fn vector(self, v: Vec3) -> Result<Vec3, TransformError> {
        if !self.is_finite() || !v.is_finite() {
            return Err(TransformError::NonFinite);
        }
        let result = self.apply_linear(v);
        if result.is_finite() {
            Ok(result)
        } else {
            Err(TransformError::NonFinite)
        }
    }

    pub fn normal(self, n: Vec3, tolerance: f64) -> Result<Vec3, TransformError> {
        if !self.is_finite() || !n.is_finite() {
            return Err(TransformError::NonFinite);
        }
        let inverse_transpose = self
            .linear
            .inverse(tolerance)
            .map_err(TransformError::from)?
            .transpose();
        let transformed = Vec3::new(
            inverse_transpose.m[0][0] * n.x
                + inverse_transpose.m[0][1] * n.y
                + inverse_transpose.m[0][2] * n.z,
            inverse_transpose.m[1][0] * n.x
                + inverse_transpose.m[1][1] * n.y
                + inverse_transpose.m[1][2] * n.z,
            inverse_transpose.m[2][0] * n.x
                + inverse_transpose.m[2][1] * n.y
                + inverse_transpose.m[2][2] * n.z,
        );
        if !transformed.is_finite() {
            return Err(TransformError::NonFinite);
        }
        transformed.normalized().map_err(map_vector_error)
    }

    pub fn compose(self, other: Self) -> Result<Self, TransformError> {
        if !self.is_finite() || !other.is_finite() {
            return Err(TransformError::NonFinite);
        }
        let result = Self {
            linear: self.linear.mul(other.linear),
            translation: self.apply_linear(other.translation).add(self.translation),
        };
        if result.is_finite() {
            Ok(result)
        } else {
            Err(TransformError::NonFinite)
        }
    }

    pub fn inverse(self, tolerance: f64) -> Result<Self, TransformError> {
        if !self.is_finite() {
            return Err(TransformError::NonFinite);
        }
        let inverse = self.linear.inverse(tolerance).map_err(TransformError::from)?;
        let neg = Vec3::new(-self.translation.x, -self.translation.y, -self.translation.z);
        let translation = Vec3::new(
            inverse.m[0][0] * neg.x + inverse.m[0][1] * neg.y + inverse.m[0][2] * neg.z,
            inverse.m[1][0] * neg.x + inverse.m[1][1] * neg.y + inverse.m[1][2] * neg.z,
            inverse.m[2][0] * neg.x + inverse.m[2][1] * neg.y + inverse.m[2][2] * neg.z,
        );
        let result = Self {
            linear: inverse,
            translation,
        };
        if result.is_finite() {
            Ok(result)
        } else {
            Err(TransformError::NonFinite)
        }
    }

    fn apply_linear(self, v: Vec3) -> Vec3 {
        Vec3::new(
            self.linear.m[0][0] * v.x
                + self.linear.m[0][1] * v.y
                + self.linear.m[0][2] * v.z,
            self.linear.m[1][0] * v.x
                + self.linear.m[1][1] * v.y
                + self.linear.m[1][2] * v.z,
            self.linear.m[2][0] * v.x
                + self.linear.m[2][1] * v.y
                + self.linear.m[2][2] * v.z,
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RigidTransform3 {
    pub rotation: Quaternion,
    pub translation: Vec3,
}

impl RigidTransform3 {
    pub const IDENTITY: Self = Self {
        rotation: Quaternion::IDENTITY,
        translation: Vec3::new(0.0, 0.0, 0.0),
    };

    pub fn point(self, p: Vec3) -> Result<Vec3, TransformError> {
        if !self.rotation.is_finite() || !self.translation.is_finite() || !p.is_finite() {
            return Err(TransformError::NonFinite);
        }
        let result = self.rotation.rotate_vector(p)?.add(self.translation);
        if result.is_finite() {
            Ok(result)
        } else {
            Err(TransformError::NonFinite)
        }
    }

    pub fn vector(self, v: Vec3) -> Result<Vec3, TransformError> {
        if !self.rotation.is_finite() || !self.translation.is_finite() || !v.is_finite() {
            return Err(TransformError::NonFinite);
        }
        Ok(self.rotation.rotate_vector(v)?)
    }

    pub fn normal(self, n: Vec3) -> Result<Vec3, TransformError> {
        self.vector(n)?.normalized().map_err(map_vector_error)
    }

    pub fn compose(self, other: Self) -> Result<Self, TransformError> {
        if !self.rotation.is_finite()
            || !self.translation.is_finite()
            || !other.rotation.is_finite()
            || !other.translation.is_finite()
        {
            return Err(TransformError::NonFinite);
        }
        let rotation = self.rotation.mul(other.rotation).normalized()?;
        let translation = self.point(other.translation)?;
        let result = Self {
            rotation,
            translation,
        };
        if result.rotation.is_finite() && result.translation.is_finite() {
            Ok(result)
        } else {
            Err(TransformError::NonFinite)
        }
    }

    pub fn inverse(self) -> Result<Self, TransformError> {
        if !self.rotation.is_finite() || !self.translation.is_finite() {
            return Err(TransformError::NonFinite);
        }
        let rotation = self.rotation.inverse()?.normalized()?;
        let neg = self.translation.scale(-1.0);
        let translation = rotation.rotate_vector(neg)?;
        let result = Self {
            rotation,
            translation,
        };
        if result.rotation.is_finite() && result.translation.is_finite() {
            Ok(result)
        } else {
            Err(TransformError::NonFinite)
        }
    }

    pub fn to_affine(self) -> Result<Transform3, TransformError> {
        if !self.rotation.is_finite() || !self.translation.is_finite() {
            return Err(TransformError::NonFinite);
        }
        let result = Transform3 {
            linear: self.rotation.to_mat3()?,
            translation: self.translation,
        };
        if result.is_finite() {
            Ok(result)
        } else {
            Err(TransformError::NonFinite)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::constants::HALF_PI;

    #[test]
    fn affine_point_vector_and_normal_have_distinct_semantics() {
        let transform = Transform3 {
            linear: Mat3::new([[2.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]),
            translation: Vec3::new(5.0, 0.0, 0.0),
        };
        assert_eq!(
            transform.point(Vec3::new(1.0, 2.0, 3.0)).unwrap(),
            Vec3::new(7.0, 2.0, 3.0)
        );
        assert_eq!(
            transform.vector(Vec3::new(1.0, 0.0, 0.0)).unwrap(),
            Vec3::new(2.0, 0.0, 0.0)
        );
        let normal = transform.normal(Vec3::new(1.0, 0.0, 0.0), 1.0e-12).unwrap();
        assert!((normal.length() - 1.0).abs() < 1.0e-14);
    }

    #[test]
    fn affine_inverse_round_trips_point() {
        let transform = Transform3 {
            linear: Mat3::new([[2.0, 0.0, 0.0], [0.0, 3.0, 0.0], [0.0, 0.0, 4.0]]),
            translation: Vec3::new(5.0, -2.0, 1.0),
        };
        let point = Vec3::new(1.0, 2.0, 3.0);
        let recovered = transform
            .inverse(1.0e-12)
            .unwrap()
            .point(transform.point(point).unwrap())
            .unwrap();
        assert!((recovered.x - point.x).abs() < 1.0e-12);
        assert!((recovered.y - point.y).abs() < 1.0e-12);
        assert!((recovered.z - point.z).abs() < 1.0e-12);
    }

    #[test]
    fn rigid_quarter_turn_preserves_length() {
        let q = Quaternion::from_axis_angle(Vec3::new(0.0, 0.0, 1.0), HALF_PI).unwrap();
        let transform = RigidTransform3 {
            rotation: q,
            translation: Vec3::new(4.0, 5.0, 6.0),
        };
        let vector = transform.vector(Vec3::new(3.0, 4.0, 0.0)).unwrap();
        assert!((vector.length() - 5.0).abs() < 1.0e-14);
    }

    #[test]
    fn affine_composition_never_silently_falls_back_on_overflow() {
        let huge = Transform2 {
            linear: Mat2::new([[f64::MAX, 0.0], [0.0, f64::MAX]]),
            translation: Vec2::new(0.0, 0.0),
        };
        assert_eq!(huge.compose(huge), Err(TransformError::NonFinite));
    }
}
