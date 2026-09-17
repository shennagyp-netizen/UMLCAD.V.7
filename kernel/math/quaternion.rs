//! Quaternion mathematics for 3D rotations.
//!
//! A quaternion used as a rotation is required to be finite and non-zero.
//! Operations normalize explicitly; no global tolerance or hidden state is used.

use super::vec::Vec3;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Quaternion {
    pub w: f64,
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QuaternionError {
    NonFinite,
    Degenerate,
    InvalidTolerance,
}

impl Quaternion {
    pub const IDENTITY: Self = Self { w: 1.0, x: 0.0, y: 0.0, z: 0.0 };

    #[inline]
    pub const fn new(w: f64, x: f64, y: f64, z: f64) -> Self { Self { w, x, y, z } }

    #[inline]
    pub fn is_finite(self) -> bool {
        self.w.is_finite() && self.x.is_finite() && self.y.is_finite() && self.z.is_finite()
    }

    #[inline]
    pub fn norm_squared(self) -> f64 { self.w*self.w + self.x*self.x + self.y*self.y + self.z*self.z }

    pub fn normalized(self) -> Result<Self, QuaternionError> {
        if !self.is_finite() { return Err(QuaternionError::NonFinite); }
        let n = self.norm_squared().sqrt();
        if !n.is_finite() { return Err(QuaternionError::NonFinite); }
        if n == 0.0 { return Err(QuaternionError::Degenerate); }
        Ok(Self::new(self.w/n, self.x/n, self.y/n, self.z/n))
    }

    #[inline]
    pub fn conjugate(self) -> Self { Self::new(self.w, -self.x, -self.y, -self.z) }

    pub fn inverse(self) -> Result<Self, QuaternionError> {
        if !self.is_finite() { return Err(QuaternionError::NonFinite); }
        let n2 = self.norm_squared();
        if !n2.is_finite() { return Err(QuaternionError::NonFinite); }
        if n2 == 0.0 { return Err(QuaternionError::Degenerate); }
        let c = self.conjugate();
        Ok(Self::new(c.w/n2, c.x/n2, c.y/n2, c.z/n2))
    }

    #[inline]
    pub fn mul(self, other: Self) -> Self {
        Self::new(
            self.w*other.w - self.x*other.x - self.y*other.y - self.z*other.z,
            self.w*other.x + self.x*other.w + self.y*other.z - self.z*other.y,
            self.w*other.y - self.x*other.z + self.y*other.w + self.z*other.x,
            self.w*other.z + self.x*other.y - self.y*other.x + self.z*other.w,
        )
    }

    pub fn from_axis_angle(axis: Vec3, angle: f64) -> Result<Self, QuaternionError> {
        if !axis.is_finite() || !angle.is_finite() { return Err(QuaternionError::NonFinite); }
        let unit = axis.normalized().map_err(|_| QuaternionError::Degenerate)?;
        let half = 0.5 * angle;
        let s = half.sin();
        Ok(Self::new(half.cos(), unit.x*s, unit.y*s, unit.z*s))
    }

    pub fn rotate_vector(self, vector: Vec3) -> Result<Vec3, QuaternionError> {
        if !vector.is_finite() { return Err(QuaternionError::NonFinite); }
        let q = self.normalized()?;
        let p = Self::new(0.0, vector.x, vector.y, vector.z);
        let r = q.mul(p).mul(q.conjugate());
        let result = Vec3::new(r.x, r.y, r.z);
        if result.is_finite() { Ok(result) } else { Err(QuaternionError::NonFinite) }
    }

    /// Active right-handed rotation matrix associated with the normalized quaternion.
    pub fn to_mat3(self) -> Result<super::mat::Mat3, QuaternionError> {
        let q = self.normalized()?;
        let (w,x,y,z) = (q.w,q.x,q.y,q.z);
        Ok(super::mat::Mat3::new([
            [1.0-2.0*(y*y+z*z), 2.0*(x*y-z*w), 2.0*(x*z+y*w)],
            [2.0*(x*y+z*w), 1.0-2.0*(x*x+z*z), 2.0*(y*z-x*w)],
            [2.0*(x*z-y*w), 2.0*(y*z+x*w), 1.0-2.0*(x*x+y*y)],
        ]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::constants::HALF_PI;

    #[test]
    fn axis_angle_rotation_matches_known_quarter_turn() {
        let q = Quaternion::from_axis_angle(Vec3::new(0.0,0.0,1.0), HALF_PI).unwrap();
        let rotated = q.rotate_vector(Vec3::new(1.0,0.0,0.0)).unwrap();
        assert!(rotated.x.abs() < 1.0e-14);
        assert!((rotated.y - 1.0).abs() < 1.0e-14);
        assert!(rotated.z.abs() < 1.0e-14);
    }

    #[test]
    fn quaternion_inverse_round_trips() {
        let q = Quaternion::new(2.0, 3.0, -1.0, 4.0);
        let p = q.mul(q.inverse().unwrap()).normalized().unwrap();
        assert_eq!(p, Quaternion::IDENTITY);
    }

    #[test]
    fn zero_axis_is_rejected() {
        assert_eq!(
            Quaternion::from_axis_angle(Vec3::new(0.0,0.0,0.0), 1.0),
            Err(QuaternionError::Degenerate)
        );
    }
}
