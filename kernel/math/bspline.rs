#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point2 {
    pub x: f64,
    pub y: f64,
}

impl Point2 {
    fn add(self, other: Self) -> Self {
        Self { x: self.x + other.x, y: self.y + other.y }
    }

    fn scale(self, factor: f64) -> Self {
        Self { x: self.x * factor, y: self.y * factor }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoundingBox2 {
    pub min: Point2,
    pub max: Point2,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BSplineError {
    NonFinite,
    InvalidDegree,
    InvalidControlPointCount,
    InvalidKnotCount,
    KnotsMustBeNondecreasing,
    NotClamped,
    InvalidDomain,
    OutOfDomain,
    Degenerate,
    Overflow,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BSplineCurve2D {
    pub degree: usize,
    pub control_points: Vec<Point2>,
    pub knots: Vec<f64>,
}

impl BSplineCurve2D {
    pub fn new(degree: usize, control_points: Vec<Point2>, knots: Vec<f64>) -> Self {
        Self { degree, control_points, knots }
    }

    pub fn validate(&self) -> Result<(), BSplineError> {
        if self.degree == 0 {
            return Err(BSplineError::InvalidDegree);
        }
        if self.control_points.len() < self.degree + 1 {
            return Err(BSplineError::InvalidControlPointCount);
        }
        let expected_knots = self.control_points.len() + self.degree + 1;
        if self.knots.len() != expected_knots {
            return Err(BSplineError::InvalidKnotCount);
        }
        if self.control_points.iter().any(|p| !p.x.is_finite() || !p.y.is_finite()) {
            return Err(BSplineError::NonFinite);
        }
        if self.knots.iter().any(|k| !k.is_finite()) {
            return Err(BSplineError::NonFinite);
        }
        if self.knots.windows(2).any(|pair| pair[1] < pair[0]) {
            return Err(BSplineError::KnotsMustBeNondecreasing);
        }

        let start = self.knots[self.degree];
        let end = self.knots[self.control_points.len()];
        if !start.is_finite() || !end.is_finite() || end <= start {
            return Err(BSplineError::InvalidDomain);
        }

        if !self.knots[..=self.degree].iter().all(|k| *k == start)
            || !self.knots[self.control_points.len()..].iter().all(|k| *k == end)
        {
            return Err(BSplineError::NotClamped);
        }

        if self.control_points.windows(2).all(|pair| pair[0] == pair[1]) {
            return Err(BSplineError::Degenerate);
        }

        Ok(())
    }

    pub fn parameter_domain(&self) -> Result<(f64, f64), BSplineError> {
        self.validate()?;
        Ok((self.knots[self.degree], self.knots[self.control_points.len()]))
    }

    pub fn point_at(&self, parameter: f64) -> Result<Point2, BSplineError> {
        self.validate()?;
        if !parameter.is_finite() {
            return Err(BSplineError::NonFinite);
        }
        let (start, end) = self.parameter_domain_unchecked();
        if parameter < start || parameter > end {
            return Err(BSplineError::OutOfDomain);
        }

        let span = self.find_span(parameter);
        let mut work = self.control_points[span - self.degree..=span].to_vec();

        for level in 1..=self.degree {
            for j in (level..=self.degree).rev() {
                let i = span - self.degree + j;
                let left = self.knots[i];
                let right = self.knots[i + self.degree + 1 - level];
                let denominator = right - left;
                let alpha = if denominator == 0.0 { 0.0 } else { (parameter - left) / denominator };
                work[j] = work[j - 1].scale(1.0 - alpha).add(work[j].scale(alpha));
            }
        }

        let result = work[self.degree];
        if !result.x.is_finite() || !result.y.is_finite() {
            return Err(BSplineError::Overflow);
        }
        Ok(result)
    }

    pub fn control_hull_bounds(&self) -> Result<BoundingBox2, BSplineError> {
        self.validate()?;
        let min = Point2 {
            x: self.control_points.iter().map(|p| p.x).fold(f64::INFINITY, f64::min),
            y: self.control_points.iter().map(|p| p.y).fold(f64::INFINITY, f64::min),
        };
        let max = Point2 {
            x: self.control_points.iter().map(|p| p.x).fold(f64::NEG_INFINITY, f64::max),
            y: self.control_points.iter().map(|p| p.y).fold(f64::NEG_INFINITY, f64::max),
        };
        Ok(BoundingBox2 { min, max })
    }

    pub fn translated(&self, dx: f64, dy: f64) -> Result<Self, BSplineError> {
        self.validate()?;
        if !dx.is_finite() || !dy.is_finite() {
            return Err(BSplineError::NonFinite);
        }
        let shift = Point2 { x: dx, y: dy };
        let control_points = self
            .control_points
            .iter()
            .map(|p| p.add(shift))
            .collect::<Vec<_>>();
        if control_points.iter().any(|p| !p.x.is_finite() || !p.y.is_finite()) {
            return Err(BSplineError::Overflow);
        }
        Ok(Self::new(self.degree, control_points, self.knots.clone()))
    }

    fn parameter_domain_unchecked(&self) -> (f64, f64) {
        (self.knots[self.degree], self.knots[self.control_points.len()])
    }

    fn find_span(&self, parameter: f64) -> usize {
        let n = self.control_points.len() - 1;
        let degree = self.degree;
        let end = self.knots[n + 1];
        if parameter >= end {
            return n;
        }
        if parameter <= self.knots[degree] {
            return degree;
        }

        let mut low = degree;
        let mut high = n + 1;
        let mut mid = (low + high) / 2;
        while parameter < self.knots[mid] || parameter >= self.knots[mid + 1] {
            if parameter < self.knots[mid] {
                high = mid;
            } else {
                low = mid;
            }
            mid = (low + high) / 2;
        }
        mid
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn de_boor_quadratic_matches_known_value() {
        let curve = BSplineCurve2D::new(
            2,
            vec![Point2 { x: 0.0, y: 0.0 }, Point2 { x: 1.0, y: 1.0 }, Point2 { x: 2.0, y: 0.0 }],
            vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
        );
        assert!((curve.point_at(0.5).unwrap().x - 1.0).abs() < 1e-12);
        assert!((curve.point_at(0.5).unwrap().y - 0.5).abs() < 1e-12);
    }
}
