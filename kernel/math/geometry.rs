#![allow(clippy::should_implement_trait)]

use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

pub const EPSILON: f64 = 1.0e-9;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}
impl Point {
    pub fn distance(self, other: Self) -> f64 {
        (self.x - other.x).hypot(self.y - other.y)
    }
    pub fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
    pub fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
    pub fn scale(self, factor: f64) -> Self {
        Self {
            x: self.x * factor,
            y: self.y * factor,
        }
    }
    pub fn dot(self, other: Self) -> f64 {
        self.x * other.x + self.y * other.y
    }
    pub fn norm(self) -> f64 {
        self.x.hypot(self.y)
    }
    pub fn length(self) -> f64 {
        self.norm()
    }
    pub fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite()
    }
    pub fn cross(self, other: Self) -> f64 {
        self.x * other.y - self.y * other.x
    }
    pub fn normalized(self) -> Result<Self, GeometryError> {
        let n = self.norm();
        if n <= EPSILON {
            return Err(GeometryError::DegenerateVector);
        }
        Ok(self.scale(1.0 / n))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Line {
    pub start: Point,
    pub end: Point,
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Circle {
    pub center: Point,
    pub radius: f64,
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Arc {
    pub center: Point,
    pub radius: f64,
    pub start_angle: f64,
    pub end_angle: f64,
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Geometry {
    Line(Line),
    Circle(Circle),
    Arc(Arc),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Aabb {
    pub min: Point,
    pub max: Point,
}
#[derive(Clone, Debug, PartialEq)]
pub struct GeometryQuery {
    pub start: Point,
    pub end: Point,
    pub parameter_domain: (f64, f64),
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum GeometryError {
    #[error("non-finite coordinate")]
    NonFinite,
    #[error("degenerate geometry")]
    Degenerate,
    #[error("invalid radius")]
    InvalidRadius,
    #[error("degenerate vector")]
    DegenerateVector,
}

impl Line {
    pub fn validate(&self) -> Result<(), GeometryError> {
        validate_point(self.start)?;
        validate_point(self.end)?;
        if self.start.distance(self.end) <= EPSILON {
            return Err(GeometryError::Degenerate);
        }
        Ok(())
    }
    pub fn length(&self) -> f64 {
        self.start.distance(self.end)
    }
}
impl Circle {
    pub fn validate(&self) -> Result<(), GeometryError> {
        validate_point(self.center)?;
        if !self.radius.is_finite() {
            return Err(GeometryError::NonFinite);
        }
        if self.radius <= EPSILON {
            return Err(GeometryError::InvalidRadius);
        }
        Ok(())
    }
    pub fn circumference(&self) -> f64 {
        2.0 * PI * self.radius
    }
}
impl Arc {
    pub fn validate(&self) -> Result<(), GeometryError> {
        validate_point(self.center)?;
        if !self.radius.is_finite() || !self.start_angle.is_finite() || !self.end_angle.is_finite()
        {
            return Err(GeometryError::NonFinite);
        }
        if self.radius <= EPSILON {
            return Err(GeometryError::InvalidRadius);
        }
        if (self.end_angle - self.start_angle).abs() <= EPSILON {
            return Err(GeometryError::Degenerate);
        }
        Ok(())
    }
    pub fn length(&self) -> f64 {
        self.radius * (self.end_angle - self.start_angle).abs()
    }
    pub fn point_at(&self, parameter: f64) -> Point {
        point_on_circle(
            Circle {
                center: self.center,
                radius: self.radius,
            },
            self.start_angle + (self.end_angle - self.start_angle) * parameter,
        )
    }
    pub fn start_point(&self) -> Point {
        self.point_at(0.0)
    }
    pub fn end_point(&self) -> Point {
        self.point_at(1.0)
    }
    pub fn contains_angle(&self, angle: f64) -> bool {
        angle_in_arc(angle, self.start_angle, self.end_angle)
    }
    pub fn contains_point(&self, point: Point) -> bool {
        let radial = point.sub(self.center);
        let n = radial.norm();
        if (n - self.radius).abs() > EPSILON * self.radius.max(1.0) {
            return false;
        }
        self.contains_angle(radial.y.atan2(radial.x))
    }
}

impl Geometry {
    pub fn validate(&self) -> Result<(), GeometryError> {
        match self {
            Self::Line(v) => v.validate(),
            Self::Circle(v) => v.validate(),
            Self::Arc(v) => v.validate(),
        }
    }
    pub fn aabb(&self) -> Aabb {
        match self {
            Self::Line(v) => Aabb {
                min: Point {
                    x: v.start.x.min(v.end.x),
                    y: v.start.y.min(v.end.y),
                },
                max: Point {
                    x: v.start.x.max(v.end.x),
                    y: v.start.y.max(v.end.y),
                },
            },
            Self::Circle(v) => Aabb {
                min: Point {
                    x: v.center.x - v.radius,
                    y: v.center.y - v.radius,
                },
                max: Point {
                    x: v.center.x + v.radius,
                    y: v.center.y + v.radius,
                },
            },
            Self::Arc(v) => arc_aabb(*v),
        }
    }
    pub fn query(&self) -> GeometryQuery {
        match self {
            Self::Line(v) => GeometryQuery {
                start: v.start,
                end: v.end,
                parameter_domain: (0.0, 1.0),
            },
            Self::Circle(v) => {
                let p = Point {
                    x: v.center.x + v.radius,
                    y: v.center.y,
                };
                GeometryQuery {
                    start: p,
                    end: p,
                    parameter_domain: (0.0, 1.0),
                }
            }
            Self::Arc(v) => GeometryQuery {
                start: v.start_point(),
                end: v.end_point(),
                parameter_domain: (0.0, 1.0),
            },
        }
    }
    pub fn point_at(&self, parameter: f64) -> Point {
        match self {
            Self::Line(v) => Point {
                x: v.start.x + (v.end.x - v.start.x) * parameter,
                y: v.start.y + (v.end.y - v.start.y) * parameter,
            },
            Self::Circle(v) => point_on_circle(*v, parameter * 2.0 * PI),
            Self::Arc(v) => v.point_at(parameter),
        }
    }
    pub fn tangent_at(&self, parameter: f64) -> Result<Point, GeometryError> {
        match self {
            Self::Line(v) => v.end.sub(v.start).normalized(),
            Self::Circle(v) => tangent_circle(*v, parameter),
            Self::Arc(v) => tangent_arc(*v, parameter),
        }
    }
    pub fn distance_to_point(&self, point: Point) -> f64 {
        match self {
            Self::Line(v) => point.distance(closest_line_point(*v, point)),
            Self::Circle(v) => (point.distance(v.center) - v.radius).abs(),
            Self::Arc(v) => point.distance(closest_arc_point(*v, point)),
        }
    }
}

fn validate_point(p: Point) -> Result<(), GeometryError> {
    if p.x.is_finite() && p.y.is_finite() {
        Ok(())
    } else {
        Err(GeometryError::NonFinite)
    }
}

fn point_on_circle(c: Circle, angle: f64) -> Point {
    Point {
        x: c.center.x + c.radius * angle.cos(),
        y: c.center.y + c.radius * angle.sin(),
    }
}

fn tangent_circle(_c: Circle, t: f64) -> Result<Point, GeometryError> {
    let angle = t * 2.0 * PI;
    Point {
        x: -angle.sin(),
        y: angle.cos(),
    }
    .normalized()
}

fn tangent_arc(a: Arc, t: f64) -> Result<Point, GeometryError> {
    let angle = a.start_angle + (a.end_angle - a.start_angle) * t;
    let sign = if a.end_angle >= a.start_angle {
        1.0
    } else {
        -1000.0
    };
    Point {
        x: -angle.sin() * sign,
        y: angle.cos() * sign,
    }
    .normalized()
}

fn closest_line_point(l: Line, p: Point) -> Point {
    let d = l.end.sub(l.start);
    let t = ((p.sub(l.start)).dot(d)) / d.dot(d);
    l.start.add(d.scale(t.clamp(0.0, 1.0)))
}

fn closest_arc_point(a: Arc, p: Point) -> Point {
    let radial = p.sub(a.center);
    if radial.norm() <= EPSILON {
        return a.start_point();
    }
    let q = Point {
        x: a.center.x + radial.x * a.radius / radial.norm(),
        y: a.center.y + radial.y * a.radius / radial.norm(),
    };
    if a.contains_point(q) {
        q
    } else if a.start_point().distance(p) <= a.end_point().distance(p) {
        a.start_point()
    } else {
        a.end_point()
    }
}

fn angle_in_arc(angle: f64, start: f64, end: f64) -> bool {
    let delta = end - start;
    if delta.abs() <= EPSILON {
        return false;
    }
    if delta > 0.0 {
        if delta >= 2.0 * PI - EPSILON {
            return true;
        }
        (angle - start).rem_euclid(2.0 * PI) <= delta + EPSILON
    } else {
        if -delta >= 2.0 * PI - EPSILON {
            return true;
        }
        (start - angle).rem_euclid(2.0 * PI) <= -delta + EPSILON
    }
}

fn arc_aabb(a: Arc) -> Aabb {
    if (a.end_angle - a.start_angle).abs() >= 2.0 * PI - EPSILON {
        return Aabb {
            min: Point {
                x: a.center.x - a.radius,
                y: a.center.y - a.radius,
            },
            max: Point {
                x: a.center.x + a.radius,
                y: a.center.y + a.radius,
            },
        };
    }
    let mut pts = vec![a.start_point(), a.end_point()];
    for angle in [0.0, PI / 2.0, PI, 3.0 * PI / 2.0] {
        if a.contains_angle(angle) {
            pts.push(point_on_circle(
                Circle {
                    center: a.center,
                    radius: a.radius,
                },
                angle,
            ));
        }
    }
    Aabb {
        min: Point {
            x: pts.iter().map(|p| p.x).fold(f64::INFINITY, f64::min),
            y: pts.iter().map(|p| p.y).fold(f64::INFINITY, f64::min),
        },
        max: Point {
            x: pts.iter().map(|p| p.x).fold(f64::NEG_INFINITY, f64::max),
            y: pts.iter().map(|p| p.y).fold(f64::NEG_INFINITY, f64::max),
        },
    }
}

pub fn aabb_intersects(a: Aabb, b: Aabb, tol: f64) -> bool {
    a.min.x <= b.max.x + tol
        && a.max.x + tol >= b.min.x
        && a.min.y <= b.max.y + tol
        && a.max.y + tol >= b.min.y
}
