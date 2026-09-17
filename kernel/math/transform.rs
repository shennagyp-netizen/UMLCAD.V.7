//! Affine and rigid transformation mathematics.
//!
//! Points, vectors, and normals deliberately have different transformation
//! semantics. A normal under a general affine transform is transformed by the
//! inverse-transpose of the linear part, then renormalized.

use super::{
    mat::{Mat2, Mat3, MatrixError},
    quaternion::{Quaternion, QuaternionError},
    vec::{Vec2, Vec3},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransformError {
    NonFinite,
    Singular,
    InvalidTolerance,
    Degenerate,
    Overflow,
}

impl From<MatrixError> for TransformError {
    fn from(value: MatrixError) -> Self {
        match value {
            MatrixError::NonFinite => Self::NonFinite,
            MatrixError::Overflow => Self::Overflow,
            MatrixError::Singular => Self::Singular,
            MatrixError::InvalidTolerance => Self::InvalidTolerance,
            MatrixError::DimensionMismatch => Self::Degenerate,
        }
    }
}

impl From<QuaternionError> for TransformError {
    fn from(value: QuaternionError) -> Self {
        match value {
            QuaternionError::NonFinite => Self::NonFinite,
            QuaternionError::Degenerate => Self::Degenerate,
            QuaternionError::InvalidTolerance => Self::InvalidTolerance,
        }
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
        if result.is_finite() { Ok(result) } else { Err(TransformError::Overflow) }
    }

    pub fn vector(self, v: Vec2) -> Result<Vec2, TransformError> {
        if !self.is_finite() || !v.is_finite() {
            return Err(TransformError::NonFinite);
        }
        let result = self.linear.mul_vec(v).map_err(TransformError::from)?;
        Ok(result)
    }

    pub fn normal(self, n: Vec2, tolerance: f64) -> Result<Vec2, TransformError> {
        if !self.is_finite() || !n.is_finite() {
            return Err(TransformError::NonFinite);
        }
        let inverse_transpose = self.linear.inverse(tolerance)?.transpose();
        inverse_transpose.mul_vec(n)
            .map_err(TransformError::from)?
            .normalized()
            .map_err(|_| TransformError::Degenerate)
    }

    pub fn compose(self, other: Self) -> Result<Self, TransformError> {
        if !self.is_finite() || !other.is_finite() {
            return Err(TransformError::NonFinite);
        }
        let linear = self.linear.mul(other.linear);
        let translation = self.point(other.translation)?;
        if !linear.m.iter().flatten().all(|v| v.is_finite()) {
            return Err(TransformError::Overflow);
        }
        Ok(Self { linear, translation })
    }

    pub fn inverse(self, tolerance: f64) -> Result<Self, TransformError> {
        if !self.is_finite() { return Err(TransformError::NonFinite); }
        let inv = self.linear.inverse(tolerance)?;
        let translation = inv.mul_vec(self.translation.scale(-1.0))?;
        Ok(Self { linear: inv, translation })
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
        if !self.is_finite() || !p.is_finite() { return Err(TransformError::NonFinite); }
        let result = self.apply_linear(p).add(self.translation);
        if result.is_finite() { Ok(result) } else { Err(TransformError::Overflow) }
    }

    pub fn vector(self, v: Vec3) -> Result<Vec3, TransformError> {
        if !self.is_finite() || !v.is_finite() { return Err(TransformError::NonFinite); }
        let result = self.apply_linear(v);
        if result.is_finite() { Ok(result) } else { Err(TransformError::Overflow) }
    }

    pub fn normal(self, n: Vec3, tolerance: f64) -> Result<Vec3, TransformError> {
        if !self.is_finite() || !n.is_finite() { return Err(TransformError::NonFinite); }
        let inverse_transpose = self.linear.inverse(tolerance)?.transpose();
        let transformed = inverse_transpose.mul_vec(n)?;
        transformed.normalized().map_err(|_| TransformError::Degenerate)
    }

    pub fn compose(self, other: Self) -> Result<Self, TransformError> {
        if !self.is_finite() || !other.is_finite() { return Err(TransformError::NonFinite); }
        let linear = self.linear.mul(other.linear);
        let translation = self.point(other.translation)?;
        if !linear.m.iter().flatten().all(|v| v.is_finite()) { return Err(TransformError::Overflow); }
        Ok(Self { linear, translation })
    }

    pub fn inverse(self, tolerance: f64) -> Result<Self, TransformError> {
        if !self.is_finite() { return Err(TransformError::NonFinite); }
        let inverse = self.linear.inverse(tolerance)?;
        let translation = inverse.mul_vec(self.translation.scale(-1.0))?;
        Ok(Self { linear: inverse, translation })
    }

    fn apply_linear(self, v: Vec3) -> Vec3 {
        Vec3::new(
            self.linear.m[0][0] * v.x + self.linear.m[0][1] * v.y + self.linear.m[0][2] * v.z,
            self.linear.m[1][0] * v.x + self.linear.m[1][1] * v.y + self.linear.m[1][2] * v.z,
            self.linear.m[2][0] * v.x + self.linear.m[2][1] * v.y + self.linear.m[2][2] * v.z,
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RigidTransform3 {
    pub rotation: Quaternion,
    pub translation: Vec3,
}

impl RigidTransform3 {
    pub const IDENTITY: Self = Self { rotation: Quaternion::IDENTITY, translation: Vec3::new(0.0, 0.0, 0.0) };

    pub fn validate(self) -> Result<(), TransformError> {
        if !self.translation.is_finite() || !self.rotation.is_finite() { return Err(TransformError::NonFinite); }
        self.rotation.normalized().map(|_| ()).map_err(TransformError::from)
    }

    pub fn point(self, p: Vec3) -> Result<Vec3, TransformError> {
        self.validate()?;
        if !p.is_finite() { return Err(TransformError::NonFinite); }
        let result = self.rotation.rotate_vector(p)?.add(self.translation);
        if result.is_finite() { Ok(result) } else { Err(TransformError::Overflow) }
    }

    pub fn vector(self, v: Vec3) -> Result<Vec3, TransformError> {
        self.validate()?;
        if !v.is_finite() { return Err(TransformError::NonFinite); }
        self.rotation.rotate_vector(v).map_err(TransformError::from)
    }

    pub fn normal(self, n: Vec3) -> Result<Vec3, TransformError> {
        self.vector(n)?.normalized().map_err(|_| TransformError::Degenerate)
    }

    pub fn compose(self, other: Self) -> Result<Self, TransformError> {
        self.validate()?; other.validate()?;
        let rotation = self.rotation.mul(other.rotation).normalized().map_err(TransformError::from)?;
        let translation = self.point(other.translation)?;
        Ok(Self { rotation, translation })
    }

    pub fn inverse(self) -> Result<Self, TransformError> {
        self.validate()?;
        let rotation = self.rotation.normalized()?.conjugate();
        let translation = rotation.rotate_vector(self.translation.scale(-1.0))?;
        Ok(Self { rotation, translation })
    }

    pub fn to_affine(self) -> Result<Transform3, TransformError> {
        self.validate()?;
        Ok(Transform3 { linear: self.rotation.to_mat3()?, translation: self.translation })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::constants::HALF_PI;

    #[test]
    fn affine_point_vector_and_normal_have_distinct_semantics() {
        let transform = Transform3 { linear: Mat3::new([[2.0,0.0,0.0],[0.0,1.0,0.0],[0.0,0.0,1.0]]), translation: Vec3::new(5.0,0.0,0.0) };
        assert_eq!(transform.point(Vec3::new(1.0,2.0,3.0)).unwrap(), Vec3::new(7.0,2.0,3.0));
        assert_eq!(transform.vector(Vec3::new(1.0,0.0,0.0)).unwrap(), Vec3::new(2.0,0.0,0.0));
        let normal = transform.normal(Vec3::new(1.0,0.0,0.0), 1.0e-12).unwrap();
        assert!((normal.length()-1.0).abs()<1.0e-14);
    }

    #[test]
    fn affine_inverse_round_trips_point() {
        let transform = Transform3 { linear: Mat3::new([[2.0,0.0,0.0],[0.0,3.0,0.0],[0.0,0.0,4.0]]), translation: Vec3::new(5.0,-2.0,1.0) };
        let point=Vec3::new(1.0,2.0,3.0);
        let mapped=transform.point(point).unwrap();
        let recovered=transform.inverse(1.0e-12).unwrap().point(mapped).unwrap();
        assert!((recovered.x-point.x).abs()<1.0e-12 && (recovered.y-point.y).abs()<1.0e-12 && (recovered.z-point.z).abs()<1.0e-12);
    }

    #[test]
    fn rigid_quarter_turn_preserves_length() {
        let q=Quaternion::from_axis_angle(Vec3::new(0.0,0.0,1.0),HALF_PI).unwrap();
        let transform=RigidTransform3{rotation:q,translation:Vec3::new(4.0,5.0,6.0)};
        let vector=transform.vector(Vec3::new(3.0,4.0,0.0)).unwrap();
        assert!((vector.length()-5.0).abs()<1.0e-14);
    }

    #[test]
    fn nonfinite_composition_fails_closed() {
        let invalid=Transform3{linear:Mat3::IDENTITY,translation:Vec3::new(f64::NAN,0.0,0.0)};
        assert_eq!(Transform3::IDENTITY.compose(invalid),Err(TransformError::NonFinite));
    }
}
