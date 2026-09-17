//! Explicit tolerance primitives for the UMLCAD mathematical authority.
//!
//! A tolerance is a value-domain contract. No global tolerance is applied by
//! this module. Callers choose the tolerance instance appropriate to the
//! operation (modeling, validation, comparison, or another explicit domain).

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tolerance {
    pub absolute: f64,
    pub relative: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToleranceError {
    NonFinite,
    Negative,
}

impl Tolerance {
    pub fn new(absolute: f64, relative: f64) -> Result<Self, ToleranceError> {
        if !absolute.is_finite() || !relative.is_finite() {
            return Err(ToleranceError::NonFinite);
        }
        if absolute < 0.0 || relative < 0.0 {
            return Err(ToleranceError::Negative);
        }
        Ok(Self { absolute, relative })
    }

    /// Effective tolerance for a quantity whose natural scale is `scale`.
    ///
    /// The absolute component intentionally remains absolute; therefore this
    /// function does not claim scale invariance when `absolute != 0`.
    pub fn threshold(self, scale: f64) -> Result<f64, ToleranceError> {
        if !scale.is_finite() {
            return Err(ToleranceError::NonFinite);
        }
        let scale = scale.abs();
        let relative = self.relative * scale;
        let threshold = self.absolute + relative;
        if !threshold.is_finite() {
            return Err(ToleranceError::NonFinite);
        }
        Ok(threshold)
    }

    #[inline]
    pub fn approximately_equal(self, a: f64, b: f64, scale: f64) -> Result<bool, ToleranceError> {
        if !a.is_finite() || !b.is_finite() {
            return Err(ToleranceError::NonFinite);
        }
        Ok((a - b).abs() <= self.threshold(scale)?)
    }

    #[inline]
    pub fn approximately_zero(self, value: f64, scale: f64) -> Result<bool, ToleranceError> {
        if !value.is_finite() {
            return Err(ToleranceError::NonFinite);
        }
        Ok(value.abs() <= self.threshold(scale)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absolute_and_relative_components_are_combined_explicitly() {
        let tolerance = Tolerance::new(1.0e-6, 1.0e-4).unwrap();
        assert!((tolerance.threshold(2.0).unwrap() - 1.0002e-6).abs() < 1.0e-18);
    }

    #[test]
    fn pure_relative_tolerance_scales_with_the_quantity() {
        let tolerance = Tolerance::new(0.0, 1.0e-6).unwrap();
        assert_eq!(tolerance.threshold(1.0), Ok(1.0e-6));
        assert_eq!(tolerance.threshold(1.0e6), Ok(1.0));
    }

    #[test]
    fn invalid_tolerance_and_scale_are_rejected() {
        assert_eq!(Tolerance::new(-1.0, 0.0), Err(ToleranceError::Negative));
        assert_eq!(Tolerance::new(f64::NAN, 0.0), Err(ToleranceError::NonFinite));
        let tolerance = Tolerance::new(0.0, 1.0e-6).unwrap();
        assert_eq!(tolerance.threshold(f64::INFINITY), Err(ToleranceError::NonFinite));
    }

    #[test]
    fn absolute_component_remains_absolute_across_scales() {
        let tolerance = Tolerance::new(1.0e-3, 0.0).unwrap();
        assert_eq!(tolerance.threshold(1.0), Ok(1.0e-3));
        assert_eq!(tolerance.threshold(1.0e6), Ok(1.0e-3));
    }
}
