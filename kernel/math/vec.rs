//! Backend-independent Euclidean vector primitives for the UMLCAD math authority.
//!
//! These types intentionally contain no serialization, transport, renderer, or
//! backend state. Higher-level geometry types may wrap or migrate to them later.

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

impl Vec2 {
    #[inline]
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    #[inline]
    pub fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite()
    }

    #[inline]
    pub fn dot(self, other: Self) -> f64 {
        self.x * other.x + self.y * other.y
    }

    #[inline]
    pub fn cross(self, other: Self) -> f64 {
        self.x * other.y - self.y * other.x
    }

    #[inline]
    pub fn length(self) -> f64 {
        self.x.hypot(self.y)
    }

    #[inline]
    pub fn normalized(self) -> Result<Self, VectorError> {
        if !self.is_finite() {
            return Err(VectorError::NonFinite);
        }
        let length = self.length();
        if !length.is_finite() {
            return Err(VectorError::Overflow);
        }
        if length == 0.0 {
            return Err(VectorError::Degenerate);
        }
        Ok(self.scale(1.0 / length))
    }

    #[inline]
    pub fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y)
    }

    #[inline]
    pub fn sub(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y)
    }

    #[inline]
    pub fn scale(self, factor: f64) -> Self {
        Self::new(self.x * factor, self.y * factor)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    #[inline]
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    #[inline]
    pub fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite() && self.z.is_finite()
    }

    #[inline]
    pub fn dot(self, other: Self) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    #[inline]
    pub fn cross(self, other: Self) -> Self {
        Self::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    #[inline]
    pub fn length(self) -> f64 {
        self.x.hypot(self.y).hypot(self.z)
    }

    #[inline]
    pub fn normalized(self) -> Result<Self, VectorError> {
        if !self.is_finite() {
            return Err(VectorError::NonFinite);
        }
        let length = self.length();
        if !length.is_finite() {
            return Err(VectorError::Overflow);
        }
        if length == 0.0 {
            return Err(VectorError::Degenerate);
        }
        Ok(self.scale(1.0 / length))
    }

    #[inline]
    pub fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }

    #[inline]
    pub fn sub(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }

    #[inline]
    pub fn scale(self, factor: f64) -> Self {
        Self::new(self.x * factor, self.y * factor, self.z * factor)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VectorError {
    NonFinite,
    Degenerate,
    Overflow,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vec2_length_and_normalization_are_consistent() {
        let v = Vec2::new(3.0, 4.0);
        assert_eq!(v.length(), 5.0);
        let unit = v.normalized().unwrap();
        assert!((unit.length() - 1.0).abs() < 1.0e-15);
        assert!((unit.x - 0.6).abs() < 1.0e-15);
        assert!((unit.y - 0.8).abs() < 1.0e-15);
    }

    #[test]
    fn vec3_cross_is_orthogonal_to_both_inputs() {
        let a = Vec3::new(1.0, 2.0, 3.0);
        let b = Vec3::new(-2.0, 5.0, 4.0);
        let c = a.cross(b);
        assert!(c.dot(a).abs() < 1.0e-14);
        assert!(c.dot(b).abs() < 1.0e-14);
    }

    #[test]
    fn normalization_rejects_nonfinite_and_zero_vectors() {
        assert_eq!(Vec2::new(0.0, 0.0).normalized(), Err(VectorError::Degenerate));
        assert_eq!(Vec3::new(f64::NAN, 0.0, 0.0).normalized(), Err(VectorError::NonFinite));
    }

    #[test]
    fn hypot_based_length_survives_extreme_finite_components() {
        let v = Vec2::new(1.0e308, 1.0e308);
        assert!(v.length().is_finite());
        let u = v.normalized().unwrap();
        assert!(u.is_finite());
        assert!((u.length() - 1.0).abs() < 1.0e-15);
    }
}
