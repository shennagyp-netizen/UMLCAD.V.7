use std::f64::consts::PI;

use thiserror::Error;

const EPSILON: f64 = 1.0e-9;
const ORTHOGONAL_TOLERANCE: f64 = 1.0e-9;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point3 { pub x: f64, pub y: f64, pub z: f64 }

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec3 { pub x: f64, pub y: f64, pub z: f64 }

impl Vec3 {
    pub fn dot(self, other: Self) -> f64 { self.x * other.x + self.y * other.y + self.z * other.z }
    pub fn cross(self, other: Self) -> Self { Self { x: self.y * other.z - self.z * other.y, y: self.z * other.x - self.x * other.z, z: self.x * other.y - self.y * other.x } }
    pub fn norm(self) -> f64 { self.dot(self).sqrt() }
    pub fn scale(self, factor: f64) -> Self { Self { x: self.x * factor, y: self.y * factor, z: self.z * factor } }
    pub fn normalized(self) -> Result<Self, Curve3DError> {
        if !self.x.is_finite() || !self.y.is_finite() || !self.z.is_finite() { return Err(Curve3DError::NonFinite); }
        let norm = self.norm();
        if norm <= EPSILON { return Err(Curve3DError::Degenerate); }
        Ok(self.scale(1.0 / norm))
    }
    fn add(self, other: Self) -> Self { Self { x: self.x + other.x, y: self.y + other.y, z: self.z + other.z } }
    fn sub(self, other: Self) -> Self { Self { x: self.x - other.x, y: self.y - other.y, z: self.z - other.z } }
}

impl Point3 {
    pub fn vector_to(self, other: Self) -> Vec3 { Vec3 { x: other.x - self.x, y: other.y - self.y, z: other.z - self.z } }
    pub fn offset(self, v: Vec3) -> Self { Self { x: self.x + v.x, y: self.y + v.y, z: self.z + v.z } }
    pub fn distance(self, other: Self) -> f64 { self.vector_to(other).norm() }
    fn finite(self) -> bool { self.x.is_finite() && self.y.is_finite() && self.z.is_finite() }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Line3D { pub start: Point3, pub end: Point3 }

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Circle3D { pub center: Point3, pub radius: f64, pub normal: Vec3 }

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Curve3D { Line(Line3D), Circle(Circle3D) }

#[derive(Error, Clone, Copy, Debug, PartialEq)]
pub enum Curve3DError {
    #[error("curve contains non-finite values")]
    NonFinite,
    #[error("curve is degenerate")]
    Degenerate,
    #[error("curve radius is invalid")]
    InvalidRadius,
    #[error("parameter is outside the unit domain")]
    OutOfDomain,
}

impl Line3D {
    pub fn validate(&self) -> Result<(), Curve3DError> {
        if !self.start.finite() || !self.end.finite() { return Err(Curve3DError::NonFinite); }
        if self.start.distance(self.end) <= EPSILON { return Err(Curve3DError::Degenerate); }
        Ok(())
    }
    pub fn length(&self) -> f64 { self.start.distance(self.end) }
    pub fn point_at(&self, t: f64) -> Result<Point3, Curve3DError> {
        self.validate()?;
        if !t.is_finite() || !(0.0..=1.0).contains(&t) { return Err(Curve3DError::OutOfDomain); }
        Ok(Point3 { x: self.start.x + (self.end.x - self.start.x) * t, y: self.start.y + (self.end.y - self.start.y) * t, z: self.start.z + (self.end.z - self.start.z) * t })
    }
    pub fn tangent(&self) -> Result<Vec3, Curve3DError> { self.start.vector_to(self.end).normalized() }
    pub fn distance_to_point(&self, p: Point3) -> Result<f64, Curve3DError> {
        self.validate()?;
        if !p.finite() { return Err(Curve3DError::NonFinite); }
        let d = self.start.vector_to(self.end);
        let t = self.start.vector_to(p).dot(d) / d.dot(d);
        Ok(p.distance(self.start.offset(d.scale(t.clamp(0.0, 1.0)))))
    }
}

impl Circle3D {
    pub fn validate(&self) -> Result<(), Curve3DError> {
        if !self.center.finite() || !self.radius.is_finite() || !self.normal.x.is_finite() || !self.normal.y.is_finite() || !self.normal.z.is_finite() { return Err(Curve3DError::NonFinite); }
        if self.radius <= EPSILON { return Err(Curve3DError::InvalidRadius); }
        let norm = self.normal.norm();
        if norm <= EPSILON || (norm - 1.0).abs() > ORTHOGONAL_TOLERANCE { return Err(Curve3DError::Degenerate); }
        Ok(())
    }
    pub fn circumference(&self) -> f64 { 2.0 * PI * self.radius }
    pub fn point_at(&self, t: f64) -> Result<Point3, Curve3DError> {
        self.validate()?;
        if !t.is_finite() || !(0.0..=1.0).contains(&t) { return Err(Curve3DError::OutOfDomain); }
        let (u, v) = basis(self.normal)?;
        let angle = 2.0 * PI * t;
        Ok(self.center.offset(u.scale(self.radius * angle.cos())).offset(v.scale(self.radius * angle.sin())))
    }
    pub fn tangent_at(&self, t: f64) -> Result<Vec3, Curve3DError> {
        self.validate()?;
        if !t.is_finite() || !(0.0..=1.0).contains(&t) { return Err(Curve3DError::OutOfDomain); }
        let (u, v) = basis(self.normal)?;
        let angle = 2.0 * PI * t;
        Ok(u.scale(-angle.sin()).add(v.scale(angle.cos())))
    }
    pub fn distance_to_point(&self, p: Point3) -> Result<f64, Curve3DError> {
        self.validate()?;
        if !p.finite() { return Err(Curve3DError::NonFinite); }
        let displacement = self.center.vector_to(p);
        let normal = self.normal.normalized()?;
        let axial = displacement.dot(normal);
        let radial = displacement.sub(normal.scale(axial)).norm();
        Ok((radial - self.radius).hypot(axial))
    }
}

impl Curve3D {
    pub fn validate(&self) -> Result<(), Curve3DError> { match self { Self::Line(v) => v.validate(), Self::Circle(v) => v.validate() } }
    pub fn point_at(&self, t: f64) -> Result<Point3, Curve3DError> { match self { Self::Line(v) => v.point_at(t), Self::Circle(v) => v.point_at(t) } }
    pub fn distance_to_point(&self, p: Point3) -> Result<f64, Curve3DError> { match self { Self::Line(v) => v.distance_to_point(p), Self::Circle(v) => v.distance_to_point(p) } }
}

fn basis(normal: Vec3) -> Result<(Vec3, Vec3), Curve3DError> {
    let n = normal.normalized()?;
    let seed = if n.x.abs() > 0.9 { Vec3 { x: 0.0, y: 1.0, z: 0.0 } } else { Vec3 { x: 1.0, y: 0.0, z: 0.0 } };
    let u = seed.sub(n.scale(seed.dot(n))).normalized()?;
    let v = n.cross(u).normalized()?;
    Ok((u, v))
}