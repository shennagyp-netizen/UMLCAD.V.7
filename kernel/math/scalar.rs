//! Finite-safe scalar helpers for the UMLCAD mathematical authority.
//!
//! Scalar operations never hide non-finite input. Operations that can fail
//! return a structured result instead of inventing a default numerical value.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScalarError {
    NonFinite,
    Overflow,
    DivisionByZero,
    InvalidTolerance,
}

#[inline]
pub fn finite(value: f64) -> Result<f64, ScalarError> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(ScalarError::NonFinite)
    }
}

#[inline]
pub fn add(a: f64, b: f64) -> Result<f64, ScalarError> {
    finite(a)?;
    finite(b)?;
    finite(a + b).map_err(|_| ScalarError::Overflow)
}

#[inline]
pub fn sub(a: f64, b: f64) -> Result<f64, ScalarError> {
    finite(a)?;
    finite(b)?;
    finite(a - b).map_err(|_| ScalarError::Overflow)
}

#[inline]
pub fn mul(a: f64, b: f64) -> Result<f64, ScalarError> {
    finite(a)?;
    finite(b)?;
    finite(a * b).map_err(|_| ScalarError::Overflow)
}

#[inline]
pub fn div(a: f64, b: f64) -> Result<f64, ScalarError> {
    finite(a)?;
    finite(b)?;
    if b == 0.0 {
        return Err(ScalarError::DivisionByZero);
    }
    finite(a / b).map_err(|_| ScalarError::Overflow)
}

#[inline]
pub fn square(a: f64) -> Result<f64, ScalarError> {
    mul(a, a)
}

#[inline]
pub fn safe_sqrt(a: f64) -> Result<f64, ScalarError> {
    finite(a)?;
    if a < 0.0 {
        return Err(ScalarError::DivisionByZero);
    }
    finite(a.sqrt()).map_err(|_| ScalarError::Overflow)
}

#[inline]
pub fn approximately_equal(a: f64, b: f64, absolute: f64, relative: f64) -> Result<bool, ScalarError> {
    finite(a)?;
    finite(b)?;
    if !absolute.is_finite() || !relative.is_finite() || absolute < 0.0 || relative < 0.0 {
        return Err(ScalarError::InvalidTolerance);
    }
    let scale = a.abs().max(b.abs());
    let threshold = absolute + relative * scale;
    if !threshold.is_finite() {
        return Err(ScalarError::Overflow);
    }
    Ok((a - b).abs() <= threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arithmetic_rejects_nonfinite_inputs() {
        assert_eq!(add(f64::NAN, 1.0), Err(ScalarError::NonFinite));
        assert_eq!(mul(f64::INFINITY, 1.0), Err(ScalarError::NonFinite));
        assert_eq!(div(1.0, 0.0), Err(ScalarError::DivisionByZero));
    }

    #[test]
    fn finite_overflow_is_rejected() {
        assert_eq!(add(f64::MAX, f64::MAX), Err(ScalarError::Overflow));
        assert_eq!(mul(f64::MAX, 2.0), Err(ScalarError::Overflow));
    }

    #[test]
    fn approximate_comparison_uses_explicit_tolerance() {
        assert!(approximately_equal(1.0, 1.0 + 5.0e-7, 0.0, 1.0e-6).unwrap());
        assert!(!approximately_equal(1.0, 1.0 + 2.0e-6, 0.0, 1.0e-6).unwrap());
    }
}
