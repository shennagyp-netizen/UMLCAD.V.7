//! IEEE-754 binary64 interval arithmetic for robustness escalation.
//!
//! Bounds are expanded with adjacent representable floating-point values after
//! each arithmetic operation. Non-finite result bounds are rejected rather
//! than silently widening a finite interval into an unbounded one.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IntervalError {
    NonFinite,
    Empty,
    ContainsZero,
    InvalidDomain,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Interval {
    pub lo: f64,
    pub hi: f64,
}

impl Interval {
    pub fn new(lo: f64, hi: f64) -> Result<Self, IntervalError> {
        if !lo.is_finite() || !hi.is_finite() {
            return Err(IntervalError::NonFinite);
        }
        if lo > hi {
            return Err(IntervalError::Empty);
        }
        Ok(Self { lo, hi })
    }

    /// Construct an exact finite interval without permitting NaN/∞.
    pub fn exact(value: f64) -> Result<Self, IntervalError> {
        Self::new(value, value)
    }

    pub fn width(self) -> Result<f64, IntervalError> {
        let width = self.hi - self.lo;
        if width.is_finite() {
            Ok(width)
        } else {
            Err(IntervalError::NonFinite)
        }
    }

    pub fn midpoint(self) -> Result<f64, IntervalError> {
        let midpoint = self.lo + (self.hi - self.lo) * 0.5;
        if midpoint.is_finite() {
            Ok(midpoint)
        } else {
            Err(IntervalError::NonFinite)
        }
    }

    pub fn contains(self, value: f64) -> bool {
        value.is_finite() && value >= self.lo && value <= self.hi
    }

    pub fn contains_zero(self) -> bool {
        self.lo <= 0.0 && self.hi >= 0.0
    }

    pub fn intersects(self, other: Self) -> bool {
        self.lo <= other.hi && other.lo <= self.hi
    }

    pub fn separated(self, other: Self) -> bool {
        self.hi < other.lo || other.hi < self.lo
    }

    pub fn add(self, other: Self) -> Result<Self, IntervalError> {
        make_interval(next_down(self.lo + other.lo), next_up(self.hi + other.hi))
    }

    pub fn sub(self, other: Self) -> Result<Self, IntervalError> {
        make_interval(next_down(self.lo - other.hi), next_up(self.hi - other.lo))
    }

    pub fn mul(self, other: Self) -> Result<Self, IntervalError> {
        let p0 = self.lo * other.lo;
        let p1 = self.lo * other.hi;
        let p2 = self.hi * other.lo;
        let p3 = self.hi * other.hi;
        if [p0, p1, p2, p3].iter().any(|value| value.is_nan()) {
            return Err(IntervalError::NonFinite);
        }
        let lo = p0.min(p1).min(p2).min(p3);
        let hi = p0.max(p1).max(p2).max(p3);
        make_interval(next_down(lo), next_up(hi))
    }

    pub fn div(self, other: Self) -> Result<Self, IntervalError> {
        if other.contains_zero() {
            return Err(IntervalError::ContainsZero);
        }
        let p0 = self.lo / other.lo;
        let p1 = self.lo / other.hi;
        let p2 = self.hi / other.lo;
        let p3 = self.hi / other.hi;
        if [p0, p1, p2, p3].iter().any(|value| value.is_nan()) {
            return Err(IntervalError::NonFinite);
        }
        make_interval(
            next_down(p0.min(p1).min(p2).min(p3)),
            next_up(p0.max(p1).max(p2).max(p3)),
        )
    }

    pub fn sqrt(self) -> Result<Self, IntervalError> {
        if self.hi < 0.0 {
            return Err(IntervalError::InvalidDomain);
        }
        let lo = self.lo.max(0.0).sqrt();
        let hi = self.hi.sqrt();
        make_interval(next_down(lo), next_up(hi))
    }

    pub fn abs(self) -> Result<Self, IntervalError> {
        if self.lo >= 0.0 {
            return Ok(self);
        }
        if self.hi <= 0.0 {
            return make_interval(next_down(-self.hi), next_up(-self.lo));
        }
        make_interval(0.0, next_up((-self.lo).max(self.hi)))
    }

    pub fn intersect(self, other: Self) -> Result<Self, IntervalError> {
        make_interval(self.lo.max(other.lo), self.hi.min(other.hi))
    }
}

fn make_interval(lo: f64, hi: f64) -> Result<Interval, IntervalError> {
    if !lo.is_finite() || !hi.is_finite() {
        return Err(IntervalError::NonFinite);
    }
    if lo > hi {
        return Err(IntervalError::Empty);
    }
    Ok(Interval { lo, hi })
}

fn next_up(x: f64) -> f64 {
    if x.is_nan() || x == f64::INFINITY {
        return x;
    }
    if x == 0.0 {
        return f64::from_bits(1);
    }
    let bits = x.to_bits();
    if x > 0.0 {
        f64::from_bits(bits + 1)
    } else {
        f64::from_bits(bits - 1)
    }
}

fn next_down(x: f64) -> f64 {
    if x.is_nan() || x == f64::NEG_INFINITY {
        return x;
    }
    if x == 0.0 {
        return -f64::from_bits(1);
    }
    let bits = x.to_bits();
    if x > 0.0 {
        f64::from_bits(bits - 1)
    } else {
        f64::from_bits(bits + 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_interval_arithmetic_contains_exact_results() {
        let a = Interval::new(1.0, 2.0).unwrap();
        let b = Interval::new(3.0, 4.0).unwrap();
        assert!(a.add(b).unwrap().contains(5.0));
        assert!(a.mul(b).unwrap().contains(8.0));
        assert!(b.sub(a).unwrap().contains(2.0));
    }

    #[test]
    fn exact_rejects_nonfinite_values() {
        assert_eq!(Interval::exact(f64::NAN), Err(IntervalError::NonFinite));
        assert_eq!(Interval::exact(f64::INFINITY), Err(IntervalError::NonFinite));
    }

    #[test]
    fn division_rejects_zero_crossing_denominator() {
        let a = Interval::exact(1.0).unwrap();
        let b = Interval::new(-1.0, 1.0).unwrap();
        assert_eq!(a.div(b), Err(IntervalError::ContainsZero));
    }

    #[test]
    fn sqrt_clips_negative_lower_bound() {
        let a = Interval::new(-1.0, 4.0).unwrap();
        let r = a.sqrt().unwrap();
        assert!(r.contains(0.0) && r.contains(2.0));
    }

    #[test]
    fn interval_intersection_and_separation_are_consistent() {
        let a = Interval::new(0.0, 1.0).unwrap();
        let b = Interval::new(0.5, 2.0).unwrap();
        let c = Interval::new(2.0, 3.0).unwrap();
        assert!(a.intersects(b) && a.intersect(b).is_ok());
        assert!(a.separated(c));
    }

    #[test]
    fn outward_rounding_expands_nonzero_exact_endpoint() {
        let r = Interval::exact(1.0).unwrap().add(Interval::exact(2.0).unwrap()).unwrap();
        assert!(r.lo <= 3.0 && r.hi >= 3.0);
        assert!(r.lo < r.hi);
    }

    #[test]
    fn finite_overflow_is_rejected() {
        let a = Interval::exact(f64::MAX).unwrap();
        assert_eq!(a.add(a), Err(IntervalError::NonFinite));
    }
}
