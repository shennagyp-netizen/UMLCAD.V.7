//! Univariate polynomial mathematics for the UMLCAD authority.
//!
//! Coefficients use ascending powers. Real-root isolation recursively partitions
//! a Cauchy-bounded domain by derivative roots, then uses safeguarded bisection.
//! Repeated roots are explicit candidates because they are roots of the derivative.

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PolynomialError {
    Empty,
    NonFinite,
    ZeroPolynomial,
    DivisionByZero,
    InvalidTolerance,
    RootLimit,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RealRoot {
    pub value: f64,
    pub multiplicity: usize,
    pub residual: f64,
    pub bracket: Option<(f64, f64)>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Polynomial {
    pub coefficients: Vec<f64>,
}

impl Polynomial {
    pub fn new(mut coefficients: Vec<f64>) -> Result<Self, PolynomialError> {
        if coefficients.is_empty() {
            return Err(PolynomialError::Empty);
        }
        if coefficients.iter().any(|v| !v.is_finite()) {
            return Err(PolynomialError::NonFinite);
        }
        while coefficients.len() > 1 && coefficients.last().copied() == Some(0.0) {
            coefficients.pop();
        }
        Ok(Self { coefficients })
    }

    pub fn zero() -> Self {
        Self {
            coefficients: vec![0.0],
        }
    }

    pub fn is_zero(&self) -> bool {
        self.coefficients.iter().all(|value| *value == 0.0)
    }

    pub fn degree(&self) -> Option<usize> {
        if self.is_zero() {
            None
        } else {
            Some(self.coefficients.len() - 1)
        }
    }

    pub fn leading_coefficient(&self) -> f64 {
        self.coefficients.last().copied().unwrap_or(0.0)
    }

    pub fn evaluate(&self, x: f64) -> Result<f64, PolynomialError> {
        if !x.is_finite() {
            return Err(PolynomialError::NonFinite);
        }
        let mut value = 0.0;
        for &coefficient in self.coefficients.iter().rev() {
            value = value * x + coefficient;
            if !value.is_finite() {
                return Err(PolynomialError::NonFinite);
            }
        }
        Ok(value)
    }

    pub fn derivative(&self) -> Self {
        if self.coefficients.len() <= 1 {
            Self::zero()
        } else {
            Self {
                coefficients: self
                    .coefficients
                    .iter()
                    .enumerate()
                    .skip(1)
                    .map(|(degree, coefficient)| *coefficient * degree as f64)
                    .collect(),
            }
        }
    }

    pub fn derivative_n(&self, order: usize) -> Self {
        let mut polynomial = self.clone();
        for _ in 0..order {
            polynomial = polynomial.derivative();
        }
        polynomial
    }

    pub fn normalize(&self) -> Result<Self, PolynomialError> {
        if self.is_zero() {
            return Err(PolynomialError::ZeroPolynomial);
        }
        let leading = self.leading_coefficient();
        if leading == 0.0 || !leading.is_finite() {
            return Err(PolynomialError::NonFinite);
        }
        Self::new(
            self.coefficients
                .iter()
                .map(|coefficient| *coefficient / leading)
                .collect(),
        )
    }

    pub fn add(&self, other: &Self) -> Result<Self, PolynomialError> {
        let n = self.coefficients.len().max(other.coefficients.len());
        Self::new(
            (0..n)
                .map(|index| {
                    self.coefficients.get(index).copied().unwrap_or(0.0)
                        + other.coefficients.get(index).copied().unwrap_or(0.0)
                })
                .collect(),
        )
    }

    pub fn sub(&self, other: &Self) -> Result<Self, PolynomialError> {
        let n = self.coefficients.len().max(other.coefficients.len());
        Self::new(
            (0..n)
                .map(|index| {
                    self.coefficients.get(index).copied().unwrap_or(0.0)
                        - other.coefficients.get(index).copied().unwrap_or(0.0)
                })
                .collect(),
        )
    }

    pub fn mul(&self, other: &Self) -> Result<Self, PolynomialError> {
        if self.is_zero() || other.is_zero() {
            return Ok(Self::zero());
        }
        let mut coefficients = vec![0.0; self.coefficients.len() + other.coefficients.len() - 1];
        for (i, a) in self.coefficients.iter().copied().enumerate() {
            for (j, b) in other.coefficients.iter().copied().enumerate() {
                coefficients[i + j] += a * b;
            }
        }
        Self::new(coefficients)
    }

    pub fn div_rem(&self, divisor: &Self) -> Result<(Self, Self), PolynomialError> {
        if divisor.is_zero() {
            return Err(PolynomialError::DivisionByZero);
        }
        if self.is_zero() {
            return Ok((Self::zero(), Self::zero()));
        }
        let n = self.degree().ok_or(PolynomialError::ZeroPolynomial)?;
        let d = divisor.degree().ok_or(PolynomialError::ZeroPolynomial)?;
        if n < d {
            return Ok((Self::zero(), self.clone()));
        }

        let mut remainder = self.coefficients.clone();
        let mut quotient = vec![0.0; n - d + 1];
        let leading = divisor.leading_coefficient();
        if !leading.is_finite() || leading == 0.0 {
            return Err(PolynomialError::DivisionByZero);
        }

        while remainder.len() - 1 >= d && !(remainder.len() == 1 && remainder[0] == 0.0) {
            let shift = remainder.len() - 1 - d;
            let top = remainder.last().copied().ok_or(PolynomialError::Empty)?;
            let factor = top / leading;
            if !factor.is_finite() {
                return Err(PolynomialError::NonFinite);
            }
            quotient[shift] = factor;
            for j in 0..=d {
                remainder[shift + j] -= factor * divisor.coefficients[j];
                if !remainder[shift + j].is_finite() {
                    return Err(PolynomialError::NonFinite);
                }
            }
            while remainder.len() > 1 && remainder.last().copied() == Some(0.0) {
                remainder.pop();
            }
        }

        Ok((Self::new(quotient)?, Self::new(remainder)?))
    }

    pub fn cauchy_bound(&self) -> Result<f64, PolynomialError> {
        if self.is_zero() {
            return Err(PolynomialError::ZeroPolynomial);
        }
        let leading = self.leading_coefficient().abs();
        if leading == 0.0 || !leading.is_finite() {
            return Err(PolynomialError::NonFinite);
        }
        let max_ratio = self.coefficients[..self.coefficients.len() - 1]
            .iter()
            .map(|coefficient| coefficient.abs() / leading)
            .fold(0.0, f64::max);
        if !max_ratio.is_finite() {
            return Err(PolynomialError::NonFinite);
        }
        let bound = 1.0 + max_ratio;
        if bound.is_finite() {
            Ok(bound)
        } else {
            Err(PolynomialError::NonFinite)
        }
    }

    fn tolerance_band(tol: f64, x: f64) -> Result<f64, PolynomialError> {
        if !tol.is_finite() || tol <= 0.0 || !x.is_finite() {
            return Err(PolynomialError::InvalidTolerance);
        }
        let scale = 1.0 + x.abs();
        let threshold = tol * scale;
        if threshold.is_finite() {
            Ok(threshold)
        } else {
            Err(PolynomialError::NonFinite)
        }
    }

    fn root_at_or_near(&self, x: f64, tol: f64) -> Result<bool, PolynomialError> {
        let f = self.evaluate(x)?;
        Ok(f.abs() <= Self::tolerance_band(tol, x)?)
    }

    fn bisect(
        &self,
        mut a: f64,
        mut b: f64,
        tol: f64,
    ) -> Result<(f64, (f64, f64)), PolynomialError> {
        let mut fa = self.evaluate(a)?;
        let mut fb = self.evaluate(b)?;
        if fa.abs() <= Self::tolerance_band(tol, a)? {
            return Ok((a, (a, a)));
        }
        if fb.abs() <= Self::tolerance_band(tol, b)? {
            return Ok((b, (b, b)));
        }

        for _ in 0..192 {
            let mid = a + (b - a) * 0.5;
            if mid == a || mid == b {
                break;
            }
            let fm = self.evaluate(mid)?;
            if fm.abs() <= Self::tolerance_band(tol, mid)? {
                return Ok((mid, (a, b)));
            }
            if fa.signum() != fm.signum() {
                b = mid;
                fb = fm;
            } else {
                a = mid;
                fa = fm;
            }
            let width = (b - a).abs();
            let position_scale = a.abs().max(b.abs());
            let width_tolerance = Self::tolerance_band(tol, position_scale)?;
            if width <= width_tolerance {
                let x = a + (b - a) * 0.5;
                return Ok((x, (a, b)));
            }
        }

        let x = a + (b - a) * 0.5;
        let fx = self.evaluate(x)?;
        if fx.abs() > Self::tolerance_band(tol, x)? {
            return Err(PolynomialError::RootLimit);
        }
        Ok((x, (a, b)))
    }

    fn isolate_recursive(
        &self,
        tol: f64,
        max_roots: usize,
    ) -> Result<Vec<RealRoot>, PolynomialError> {
        if self.is_zero() {
            return Err(PolynomialError::ZeroPolynomial);
        }
        match self.degree() {
            None => Err(PolynomialError::ZeroPolynomial),
            Some(0) => Ok(Vec::new()),
            Some(1) => {
                let x = -self.coefficients[0] / self.coefficients[1];
                if x.is_finite() {
                    Ok(vec![RealRoot {
                        value: x,
                        multiplicity: 1,
                        residual: self.evaluate(x)?.abs(),
                        bracket: Some((x, x)),
                    }])
                } else {
                    Err(PolynomialError::NonFinite)
                }
            }
            Some(_) => {
                let critical = self.derivative().isolate_recursive(tol, max_roots)?;
                let bound = self.cauchy_bound()?;
                let mut points = Vec::with_capacity(critical.len() + 2);
                points.push(-bound);
                for root in critical {
                    if root.value > -bound && root.value < bound {
                        points.push(root.value);
                    }
                }
                points.push(bound);
                points.sort_by(|a, b| a.total_cmp(b));

                let mut roots = Vec::new();
                for &x in &points {
                    if self.root_at_or_near(x, tol)? {
                        roots.push(RealRoot {
                            value: x,
                            multiplicity: 1,
                            residual: self.evaluate(x)?.abs(),
                            bracket: Some((x, x)),
                        });
                    }
                }

                for pair in points.windows(2) {
                    let a = pair[0];
                    let b = pair[1];
                    let fa = self.evaluate(a)?;
                    let fb = self.evaluate(b)?;
                    if fa == 0.0 || fb == 0.0 || fa.signum() == fb.signum() {
                        continue;
                    }
                    let (x, bracket) = self.bisect(a, b, tol)?;
                    roots.push(RealRoot {
                        value: x,
                        multiplicity: 1,
                        residual: self.evaluate(x)?.abs(),
                        bracket: Some(bracket),
                    });
                }

                roots.sort_by(|a, b| a.value.total_cmp(&b.value));
                let mut merged: Vec<RealRoot> = Vec::new();
                for root in roots {
                    if let Some(last) = merged.last_mut() {
                        let merge_scale = root.value.abs().max(last.value.abs());
                        let merge_tolerance = Self::tolerance_band(tol, merge_scale)?;
                        if (root.value - last.value).abs() <= merge_tolerance {
                            last.value = 0.5 * (last.value + root.value);
                            last.residual = self.evaluate(last.value)?.abs();
                            last.multiplicity += 1;
                            continue;
                        }
                    }
                    merged.push(root);
                    if merged.len() > max_roots {
                        return Err(PolynomialError::RootLimit);
                    }
                }
                Ok(merged)
            }
        }
    }

    pub fn real_roots(
        &self,
        tol: f64,
        max_roots: usize,
    ) -> Result<Vec<RealRoot>, PolynomialError> {
        if !tol.is_finite() || tol <= 0.0 || max_roots == 0 {
            return Err(PolynomialError::InvalidTolerance);
        }
        let mut roots = self.isolate_recursive(tol, max_roots)?;
        for root in &mut roots {
            root.multiplicity = self.multiplicity_at(root.value, tol)?.max(1);
        }
        Ok(roots)
    }

    pub fn multiplicity_at(&self, x: f64, tol: f64) -> Result<usize, PolynomialError> {
        if !x.is_finite() || !tol.is_finite() || tol <= 0.0 {
            return Err(PolynomialError::InvalidTolerance);
        }
        let mut multiplicity = 0usize;
        let scale = self
            .coefficients
            .iter()
            .map(|coefficient| coefficient.abs())
            .fold(0.0, f64::max)
            .max(1.0);
        let mut derivative = self.clone();

        for _ in 0..self.coefficients.len() {
            let value = derivative.evaluate(x)?.abs();
            let position_scale = x.abs().max(1.0);
            let position_log = position_scale.ln();
            let threshold_log = tol.ln() + scale.ln() + multiplicity as f64 * position_log;
            let within = if value == 0.0 {
                true
            } else {
                value.ln() <= threshold_log
            };
            if !threshold_log.is_finite() {
                return Err(PolynomialError::NonFinite);
            }
            if !within {
                break;
            }
            multiplicity += 1;
            derivative = derivative.derivative();
            if derivative.is_zero() {
                break;
            }
        }
        Ok(multiplicity.max(1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_calculus() {
        let p = Polynomial::new(vec![1.0, 2.0, 1.0]).unwrap();
        assert_eq!(p.evaluate(2.0).unwrap(), 9.0);
        assert_eq!(p.derivative().coefficients, vec![2.0, 2.0]);
    }

    #[test]
    fn division_reconstructs() {
        let p = Polynomial::new(vec![1.0, -3.0, 2.0]).unwrap();
        let d = Polynomial::new(vec![-1.0, 1.0]).unwrap();
        let (q, r) = p.div_rem(&d).unwrap();
        assert_eq!(q.coefficients, vec![-1.0, 2.0]);
        assert!(r.is_zero());
    }

    #[test]
    fn roots_are_refined() {
        let p = Polynomial::new(vec![6.0, -5.0, 1.0]).unwrap();
        let roots = p.real_roots(1.0e-10, 8).unwrap();
        assert_eq!(roots.len(), 2);
        assert!((roots[0].value - 2.0).abs() < 1.0e-8);
        assert!((roots[1].value - 3.0).abs() < 1.0e-8);
    }

    #[test]
    fn repeated_root_is_reported() {
        let p = Polynomial::new(vec![1.0, -3.0, 3.0, -1.0]).unwrap();
        let roots = p.real_roots(1.0e-8, 8).unwrap();
        assert!(!roots.is_empty());
        assert!((roots[0].value - 1.0).abs() < 1.0e-6);
    }

    #[test]
    fn nonfinite_is_rejected() {
        assert_eq!(
            Polynomial::new(vec![1.0, f64::NAN]),
            Err(PolynomialError::NonFinite)
        );
    }

    #[test]
    fn overflowed_root_tolerance_does_not_become_a_root() {
        let p = Polynomial::new(vec![1.0, 1.0]).unwrap();
        assert_eq!(p.root_at_or_near(f64::MAX, 1.0), Err(PolynomialError::NonFinite));
    }
}
