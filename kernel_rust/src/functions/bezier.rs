use serde::{Deserialize, Serialize};
use thiserror::Error;

const EPSILON: f64 = 1.0e-9;
const ROOT_EPSILON: f64 = 1.0e-14;
const PARAM_EPSILON: f64 = 1.0e-12;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Point2 {
    pub x: f64,
    pub y: f64,
}

impl Point2 {
    fn distance(self, other: Self) -> f64 {
        (self.x - other.x).hypot(self.y - other.y)
    }

    fn scale(self, factor: f64) -> Self {
        Self { x: self.x * factor, y: self.y * factor }
    }

    fn add(self, other: Self) -> Self {
        Self { x: self.x + other.x, y: self.y + other.y }
    }

    fn sub(self, other: Self) -> Self {
        Self { x: self.x - other.x, y: self.y - other.y }
    }

    fn norm(self) -> f64 {
        self.x.hypot(self.y)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoundingBox2 {
    pub min: Point2,
    pub max: Point2,
}

#[derive(Error, Clone, Copy, Debug, PartialEq)]
pub enum BezierError {
    #[error("Bezier control point contains a non-finite value")]
    NonFinite,
    #[error("Bezier curve is degenerate")]
    Degenerate,
    #[error("Bezier parameter is outside the unit domain")]
    OutOfDomain,
    #[error("Bezier tangent is zero at the requested parameter")]
    ZeroTangent,
    #[error("Bezier derived geometry is not representable as finite values")]
    Overflow,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct CubicBezier {
    pub p0: Point2,
    pub p1: Point2,
    pub p2: Point2,
    pub p3: Point2,
}

impl CubicBezier {
    pub fn new(p0: Point2, p1: Point2, p2: Point2, p3: Point2) -> Self {
        Self { p0, p1, p2, p3 }
    }

    pub fn validate(&self) -> Result<(), BezierError> {
        let points = [self.p0, self.p1, self.p2, self.p3];
        if points.iter().any(|p| !p.x.is_finite() || !p.y.is_finite()) {
            return Err(BezierError::NonFinite);
        }
        let max_distance = points
            .iter()
            .flat_map(|a| points.iter().map(move |b| a.distance(*b)))
            .fold(0.0, f64::max);
        if !max_distance.is_finite() {
            return Err(BezierError::Overflow);
        }
        if max_distance <= EPSILON {
            return Err(BezierError::Degenerate);
        }
        Ok(())
    }

    pub fn point_at(&self, parameter: f64) -> Result<Point2, BezierError> {
        self.validate()?;
        if !parameter.is_finite() {
            return Err(BezierError::NonFinite);
        }
        if !(0.0..=1.0).contains(&parameter) {
            return Err(BezierError::OutOfDomain);
        }
        let t = parameter;
        let u = 1.0 - t;
        let a = self.p0.scale(u).add(self.p1.scale(t));
        let b = self.p1.scale(u).add(self.p2.scale(t));
        let c = self.p2.scale(u).add(self.p3.scale(t));
        let d = a.scale(u).add(b.scale(t));
        let e = b.scale(u).add(c.scale(t));
        let result = d.scale(u).add(e.scale(t));
        if !result.x.is_finite() || !result.y.is_finite() {
            return Err(BezierError::Overflow);
        }
        Ok(result)
    }

    pub fn derivative_at(&self, parameter: f64) -> Result<Point2, BezierError> {
        self.validate()?;
        if !parameter.is_finite() {
            return Err(BezierError::NonFinite);
        }
        if !(0.0..=1.0).contains(&parameter) {
            return Err(BezierError::OutOfDomain);
        }
        let t = parameter;
        let u = 1.0 - t;
        let d0 = self.p1.sub(self.p0).scale(u * u);
        let d1 = self.p2.sub(self.p1).scale(2.0 * u * t);
        let d2 = self.p3.sub(self.p2).scale(t * t);
        let result = d0.add(d1).add(d2).scale(3.0);
        if !result.x.is_finite() || !result.y.is_finite() {
            return Err(BezierError::Overflow);
        }
        Ok(result)
    }

    pub fn tangent_at(&self, parameter: f64) -> Result<Point2, BezierError> {
        let derivative = self.derivative_at(parameter)?;
        let norm = derivative.norm();
        if !norm.is_finite() {
            return Err(BezierError::Overflow);
        }
        if norm <= EPSILON {
            return Err(BezierError::ZeroTangent);
        }
        Ok(derivative.scale(1.0 / norm))
    }

    pub fn start_point(&self) -> Result<Point2, BezierError> {
        self.point_at(0.0)
    }

    pub fn end_point(&self) -> Result<Point2, BezierError> {
        self.point_at(1.0)
    }

    pub fn bounding_box(&self) -> Result<BoundingBox2, BezierError> {
        self.validate()?;
        let mut candidates = vec![self.p0, self.p3];
        for parameter in self.coordinate_extrema(self.p0.x, self.p1.x, self.p2.x, self.p3.x) {
            candidates.push(self.point_at(parameter)?);
        }
        for parameter in self.coordinate_extrema(self.p0.y, self.p1.y, self.p2.y, self.p3.y) {
            candidates.push(self.point_at(parameter)?);
        }
        let min = Point2 {
            x: candidates.iter().map(|p| p.x).fold(f64::INFINITY, f64::min),
            y: candidates.iter().map(|p| p.y).fold(f64::INFINITY, f64::min),
        };
        let max = Point2 {
            x: candidates.iter().map(|p| p.x).fold(f64::NEG_INFINITY, f64::max),
            y: candidates.iter().map(|p| p.y).fold(f64::NEG_INFINITY, f64::max),
        };
        if !min.x.is_finite() || !min.y.is_finite() || !max.x.is_finite() || !max.y.is_finite() {
            return Err(BezierError::Overflow);
        }
        Ok(BoundingBox2 { min, max })
    }

    pub fn translated(&self, dx: f64, dy: f64) -> Result<Self, BezierError> {
        self.validate()?;
        if !dx.is_finite() || !dy.is_finite() {
            return Err(BezierError::NonFinite);
        }
        let shift = Point2 { x: dx, y: dy };
        let points = [
            self.p0.add(shift),
            self.p1.add(shift),
            self.p2.add(shift),
            self.p3.add(shift),
        ];
        if points.iter().any(|p| !p.x.is_finite() || !p.y.is_finite()) {
            return Err(BezierError::Overflow);
        }
        Ok(Self::new(points[0], points[1], points[2], points[3]))
    }

    fn coordinate_extrema(&self, p0: f64, p1: f64, p2: f64, p3: f64) -> Vec<f64> {
        let scale = p0.abs().max(p1.abs()).max(p2.abs()).max(p3.abs());
        if scale == 0.0 || !scale.is_finite() {
            return Vec::new();
        }
        let p0 = p0 / scale;
        let p1 = p1 / scale;
        let p2 = p2 / scale;
        let p3 = p3 / scale;
        let a = -p0 + 3.0 * p1 - 3.0 * p2 + p3;
        let b = 2.0 * (p0 - 2.0 * p1 + p2);
        let c = p1 - p0;
        let mut roots = Vec::with_capacity(2);
        if a.abs() <= ROOT_EPSILON {
            if b.abs() > ROOT_EPSILON {
                push_parameter(&mut roots, -c / b);
            }
            return roots;
        }
        let discriminant = b * b - 4.0 * a * c;
        if discriminant < -ROOT_EPSILON {
            return roots;
        }
        if discriminant.abs() <= ROOT_EPSILON {
            push_parameter(&mut roots, -b / (2.0 * a));
            return roots;
        }
        let sqrt = discriminant.sqrt();
        push_parameter(&mut roots, (-b - sqrt) / (2.0 * a));
        push_parameter(&mut roots, (-b + sqrt) / (2.0 * a));
        roots
    }
}

fn push_parameter(values: &mut Vec<f64>, value: f64) {
    if !value.is_finite() || !(-PARAM_EPSILON..=1.0 + PARAM_EPSILON).contains(&value) {
        return;
    }
    let clamped = value.clamp(0.0, 1.0);
    if !values.iter().any(|existing| (existing - clamped).abs() <= PARAM_EPSILON) {
        values.push(clamped);
    }
}
