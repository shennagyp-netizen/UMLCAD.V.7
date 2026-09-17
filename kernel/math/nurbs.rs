#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point2 {
    pub x: f64,
    pub y: f64,
}

impl Point2 {
    fn add(self, other: Self) -> Self {
        Self { x: self.x + other.x, y: self.y + other.y }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoundingBox2 {
    pub min: Point2,
    pub max: Point2,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NurbsError {
    NonFinite,
    InvalidDegree,
    InvalidControlPointCount,
    InvalidWeightCount,
    InvalidWeight,
    InvalidKnotCount,
    KnotsMustBeNondecreasing,
    NotClamped,
    InvalidDomain,
    OutOfDomain,
    Degenerate,
    ZeroProjectiveWeight,
    Overflow,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NurbsCurve2D {
    pub degree: usize,
    pub control_points: Vec<Point2>,
    pub weights: Vec<f64>,
    pub knots: Vec<f64>,
}

impl NurbsCurve2D {
    pub fn new(degree: usize, control_points: Vec<Point2>, weights: Vec<f64>, knots: Vec<f64>) -> Self {
        Self { degree, control_points, weights, knots }
    }

    pub fn validate(&self) -> Result<(), NurbsError> {
        if self.degree == 0 { return Err(NurbsError::InvalidDegree); }
        if self.control_points.len() < self.degree + 1 { return Err(NurbsError::InvalidControlPointCount); }
        if self.weights.len() != self.control_points.len() { return Err(NurbsError::InvalidWeightCount); }
        if self.knots.len() != self.control_points.len() + self.degree + 1 { return Err(NurbsError::InvalidKnotCount); }
        if self.control_points.iter().any(|p| !p.x.is_finite() || !p.y.is_finite()) { return Err(NurbsError::NonFinite); }
        if self.weights.iter().any(|w| !w.is_finite()) { return Err(NurbsError::NonFinite); }
        if self.weights.iter().any(|w| *w <= 0.0) { return Err(NurbsError::InvalidWeight); }
        if self.knots.iter().any(|k| !k.is_finite()) { return Err(NurbsError::NonFinite); }
        if self.knots.windows(2).any(|pair| pair[1] < pair[0]) { return Err(NurbsError::KnotsMustBeNondecreasing); }

        let start = self.knots[self.degree];
        let end = self.knots[self.control_points.len()];
        if end <= start { return Err(NurbsError::InvalidDomain); }
        if !self.knots[..=self.degree].iter().all(|k| *k == start)
            || !self.knots[self.control_points.len()..].iter().all(|k| *k == end)
        { return Err(NurbsError::NotClamped); }
        if self.control_points.windows(2).all(|pair| pair[0] == pair[1]) { return Err(NurbsError::Degenerate); }
        Ok(())
    }

    pub fn parameter_domain(&self) -> Result<(f64, f64), NurbsError> {
        self.validate()?;
        Ok((self.knots[self.degree], self.knots[self.control_points.len()]))
    }

    pub fn point_at(&self, parameter: f64) -> Result<Point2, NurbsError> {
        self.validate()?;
        if !parameter.is_finite() { return Err(NurbsError::NonFinite); }
        let (start, end) = self.parameter_domain_unchecked();
        if parameter < start || parameter > end { return Err(NurbsError::OutOfDomain); }

        let span = self.find_span(parameter);
        let mut work: Vec<HomogeneousPoint> = (span - self.degree..=span).map(|index| {
            let p = self.control_points[index];
            let w = self.weights[index];
            HomogeneousPoint { xw: p.x * w, yw: p.y * w, w }
        }).collect();

        for level in 1..=self.degree {
            for j in (level..=self.degree).rev() {
                let i = span - self.degree + j;
                let left = self.knots[i];
                let right = self.knots[i + self.degree + 1 - level];
                let denominator = right - left;
                let alpha = if denominator == 0.0 { 0.0 } else { (parameter - left) / denominator };
                work[j] = work[j - 1].lerp(work[j], alpha);
            }
        }

        let result = work[self.degree];
        if !result.xw.is_finite() || !result.yw.is_finite() || !result.w.is_finite() { return Err(NurbsError::Overflow); }
        if result.w <= 0.0 { return Err(NurbsError::ZeroProjectiveWeight); }
        let point = Point2 { x: result.xw / result.w, y: result.yw / result.w };
        if !point.x.is_finite() || !point.y.is_finite() { return Err(NurbsError::Overflow); }
        Ok(point)
    }

    pub fn control_hull_bounds(&self) -> Result<BoundingBox2, NurbsError> {
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

    pub fn translated(&self, dx: f64, dy: f64) -> Result<Self, NurbsError> {
        self.validate()?;
        if !dx.is_finite() || !dy.is_finite() { return Err(NurbsError::NonFinite); }
        let shift = Point2 { x: dx, y: dy };
        let control_points = self.control_points.iter().map(|p| p.add(shift)).collect::<Vec<_>>();
        if control_points.iter().any(|p| !p.x.is_finite() || !p.y.is_finite()) { return Err(NurbsError::Overflow); }
        Ok(Self::new(self.degree, control_points, self.weights.clone(), self.knots.clone()))
    }

    fn parameter_domain_unchecked(&self) -> (f64, f64) {
        (self.knots[self.degree], self.knots[self.control_points.len()])
    }

    fn find_span(&self, parameter: f64) -> usize {
        let n = self.control_points.len() - 1;
        let degree = self.degree;
        let end = self.knots[n + 1];
        if parameter >= end { return n; }
        if parameter <= self.knots[degree] { return degree; }
        let mut low = degree;
        let mut high = n + 1;
        let mut mid = (low + high) / 2;
        while parameter < self.knots[mid] || parameter >= self.knots[mid + 1] {
            if parameter < self.knots[mid] { high = mid; } else { low = mid; }
            mid = (low + high) / 2;
        }
        mid
    }
}

#[derive(Clone, Copy)]
struct HomogeneousPoint {
    xw: f64,
    yw: f64,
    w: f64,
}

impl HomogeneousPoint {
    fn lerp(self, other: Self, alpha: f64) -> Self {
        Self {
            xw: self.xw * (1.0 - alpha) + other.xw * alpha,
            yw: self.yw * (1.0 - alpha) + other.yw * alpha,
            w: self.w * (1.0 - alpha) + other.w * alpha,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quarter_circle_projective_evaluation_is_unit_radius_at_midpoint() {
        let curve = NurbsCurve2D::new(
            2,
            vec![Point2 { x: 1.0, y: 0.0 }, Point2 { x: 1.0, y: 1.0 }, Point2 { x: 0.0, y: 1.0 }],
            vec![1.0, 2.0_f64.sqrt() / 2.0, 1.0],
            vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
        );
        let midpoint = curve.point_at(0.5).unwrap();
        let expected = 2.0_f64.sqrt() / 2.0;
        assert!((midpoint.x - expected).abs() < 1e-12);
        assert!((midpoint.y - expected).abs() < 1e-12);
    }
}
