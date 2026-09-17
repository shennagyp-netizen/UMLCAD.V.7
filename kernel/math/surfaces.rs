use std::f64::consts::PI;

use thiserror::Error;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Point3 {
    pub fn distance(self, other: Self) -> f64 { (self.x - other.x).hypot((self.y - other.y).hypot(self.z - other.z)) }
    pub fn vector_to(self, other: Self) -> Self { Self { x: other.x - self.x, y: other.y - self.y, z: other.z - self.z } }
    pub fn dot(self, other: Self) -> f64 { self.x * other.x + self.y * other.y + self.z * other.z }
    pub fn norm(self) -> f64 { self.x.hypot(self.y.hypot(self.z)) }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoundingBox3 { pub min: Point3, pub max: Point3 }

#[derive(Error, Clone, Copy, Debug, PartialEq)]
pub enum SurfaceError {
    #[error("surface contains non-finite values")] NonFinite,
    #[error("surface width and depth must be positive")] InvalidExtent,
    #[error("surface radius must be positive")] InvalidRadius,
    #[error("surface parameter is outside the unit domain")] OutOfDomain,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlanarSurface { pub center: Point3, pub width: f64, pub depth: f64 }

impl PlanarSurface {
    pub fn new(center: Point3, width: f64, depth: f64) -> Self { Self { center, width, depth } }
    pub fn validate(&self) -> Result<(), SurfaceError> {
        if !self.center.x.is_finite() || !self.center.y.is_finite() || !self.center.z.is_finite() || !self.width.is_finite() || !self.depth.is_finite() { return Err(SurfaceError::NonFinite); }
        if self.width <= 0.0 || self.depth <= 0.0 { return Err(SurfaceError::InvalidExtent); }
        if !self.area().is_finite() { return Err(SurfaceError::NonFinite); }
        let values = [self.center.x - self.width * 0.5, self.center.x + self.width * 0.5, self.center.y - self.depth * 0.5, self.center.y + self.depth * 0.5];
        if values.iter().any(|value| !value.is_finite()) { return Err(SurfaceError::NonFinite); }
        Ok(())
    }
    pub fn area(&self) -> f64 { self.width * self.depth }
    pub fn normal(&self) -> Point3 { Point3 { x: 0.0, y: 0.0, z: 1.0 } }
    pub fn point_at(&self, u: f64, v: f64) -> Result<Point3, SurfaceError> {
        self.validate()?; validate_parameters(u, v)?;
        Ok(Point3 { x: self.center.x + (u - 0.5) * self.width, y: self.center.y + (v - 0.5) * self.depth, z: self.center.z })
    }
    pub fn bounding_box(&self) -> Result<BoundingBox3, SurfaceError> {
        self.validate()?;
        Ok(BoundingBox3 { min: Point3 { x: self.center.x - self.width * 0.5, y: self.center.y - self.depth * 0.5, z: self.center.z }, max: Point3 { x: self.center.x + self.width * 0.5, y: self.center.y + self.depth * 0.5, z: self.center.z } })
    }
    pub fn distance_to_point(&self, point: Point3) -> Result<f64, SurfaceError> {
        self.validate()?; validate_point(point)?;
        let xmin = self.center.x - self.width * 0.5; let xmax = self.center.x + self.width * 0.5;
        let ymin = self.center.y - self.depth * 0.5; let ymax = self.center.y + self.depth * 0.5;
        let dx = if point.x < xmin { xmin - point.x } else if point.x > xmax { point.x - xmax } else { 0.0 };
        let dy = if point.y < ymin { ymin - point.y } else if point.y > ymax { point.y - ymax } else { 0.0 };
        Ok(dx.hypot(dy).hypot(point.z - self.center.z))
    }
    pub fn translated(&self, dx: f64, dy: f64, dz: f64) -> Result<Self, SurfaceError> {
        self.validate()?; validate_finite_delta(dx, dy, dz)?;
        let center = Point3 { x: self.center.x + dx, y: self.center.y + dy, z: self.center.z + dz }; validate_point(center)?;
        Ok(Self { center, ..*self })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SphereSurface { pub center_point: Point3, pub radius_value: f64 }

impl SphereSurface {
    pub fn new(center: Point3, radius: f64) -> Self { Self { center_point: center, radius_value: radius } }
    pub fn center(&self) -> Point3 { self.center_point }
    pub fn radius(&self) -> f64 { self.radius_value }
    pub fn validate(&self) -> Result<(), SurfaceError> {
        validate_point(self.center_point)?;
        if !self.radius_value.is_finite() { return Err(SurfaceError::NonFinite); }
        if self.radius_value <= 0.0 { return Err(SurfaceError::InvalidRadius); }
        if !self.area().is_finite() { return Err(SurfaceError::NonFinite); }
        let r = self.radius_value;
        let values = [self.center_point.x - r, self.center_point.x + r, self.center_point.y - r, self.center_point.y + r, self.center_point.z - r, self.center_point.z + r];
        if values.iter().any(|value| !value.is_finite()) { return Err(SurfaceError::NonFinite); }
        Ok(())
    }
    pub fn area(&self) -> f64 { 4.0 * PI * self.radius_value * self.radius_value }
    pub fn point_at(&self, u: f64, v: f64) -> Result<Point3, SurfaceError> {
        self.validate()?; validate_parameters(u, v)?;
        let azimuth = 2.0 * PI * u; let polar = PI * v; let sin_polar = polar.sin();
        let result = Point3 { x: self.center_point.x + self.radius_value * sin_polar * azimuth.sin(), y: self.center_point.y + self.radius_value * sin_polar * azimuth.cos(), z: self.center_point.z + self.radius_value * polar.cos() };
        validate_point(result)?; Ok(result)
    }
    pub fn normal_at(&self, u: f64, v: f64) -> Result<Point3, SurfaceError> {
        let radial = self.center_point.vector_to(self.point_at(u, v)?); let norm = radial.norm();
        if !norm.is_finite() || norm <= 0.0 { return Err(SurfaceError::NonFinite); }
        Ok(Point3 { x: radial.x / norm, y: radial.y / norm, z: radial.z / norm })
    }
    pub fn bounding_box(&self) -> Result<BoundingBox3, SurfaceError> {
        self.validate()?; let r = self.radius_value;
        Ok(BoundingBox3 { min: Point3 { x: self.center_point.x - r, y: self.center_point.y - r, z: self.center_point.z - r }, max: Point3 { x: self.center_point.x + r, y: self.center_point.y + r, z: self.center_point.z + r } })
    }
    pub fn distance_to_point(&self, point: Point3) -> Result<f64, SurfaceError> {
        self.validate()?; validate_point(point)?; let d = self.center_point.distance(point);
        if !d.is_finite() { return Err(SurfaceError::NonFinite); }
        Ok((d - self.radius_value).abs())
    }
    pub fn translated(&self, dx: f64, dy: f64, dz: f64) -> Result<Self, SurfaceError> {
        self.validate()?; validate_finite_delta(dx, dy, dz)?;
        let center = Point3 { x: self.center_point.x + dx, y: self.center_point.y + dy, z: self.center_point.z + dz }; validate_point(center)?;
        Ok(Self { center_point: center, radius_value: self.radius_value })
    }
}

fn validate_point(point: Point3) -> Result<(), SurfaceError> {
    if point.x.is_finite() && point.y.is_finite() && point.z.is_finite() { Ok(()) } else { Err(SurfaceError::NonFinite) }
}

fn validate_parameters(u: f64, v: f64) -> Result<(), SurfaceError> {
    if !u.is_finite() || !v.is_finite() { return Err(SurfaceError::NonFinite); }
    if !(0.0..=1.0).contains(&u) || !(0.0..=1.0).contains(&v) { return Err(SurfaceError::OutOfDomain); }
    Ok(())
}

fn validate_finite_delta(dx: f64, dy: f64, dz: f64) -> Result<(), SurfaceError> {
    if dx.is_finite() && dy.is_finite() && dz.is_finite() { Ok(()) } else { Err(SurfaceError::NonFinite) }
}
