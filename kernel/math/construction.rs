//! CPU/f64 authority for bounded CAD construction mathematics.
//!
//! Supported families are explicit and fail closed outside their certified
//! domains. No renderer, OCCT object, or display mesh participates.

use std::f64::consts::PI;
use thiserror::Error;

use super::{
    tolerance::Tolerance,
    vec::{Vec2, Vec3},
};

#[derive(Error, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConstructionError {
    #[error("non-finite construction input")]
    NonFinite,
    #[error("invalid construction tolerance")]
    InvalidTolerance,
    #[error("degenerate construction")]
    Degenerate,
    #[error("construction contains a self-intersection")]
    SelfIntersection,
    #[error("invalid construction dimensions")]
    InvalidDimensions,
    #[error("construction parameter is outside its domain")]
    OutOfDomain,
    #[error("construction is outside the certified domain")]
    Unsupported,
    #[error("singular construction frame")]
    SingularFrame,
    #[error("construction result is not finite")]
    Overflow,
    #[error("continuity is indeterminate")]
    Indeterminate,
}

fn valid_tolerance(t: Tolerance) -> Result<(), ConstructionError> {
    if t.absolute.is_finite() && t.relative.is_finite() && t.absolute >= 0.0 && t.relative >= 0.0 {
        Ok(())
    } else {
        Err(ConstructionError::InvalidTolerance)
    }
}

fn scale2(points: &[Vec2]) -> f64 {
    if points.is_empty() {
        return 1.0;
    }
    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    let mut max_edge: f64 = 0.0;
    for p in points {
        min_x = min_x.min(p.x);
        min_y = min_y.min(p.y);
        max_x = max_x.max(p.x);
        max_y = max_y.max(p.y);
    }
    for i in 0..points.len() {
        max_edge = max_edge.max(points[i].sub(points[(i + 1) % points.len()]).length());
    }
    (max_x - min_x).hypot(max_y - min_y).max(max_edge).max(1.0)
}

fn loft_family_convex(
    lower: &PlanarPolygon,
    upper: &PlanarPolygon,
    tolerance: Tolerance,
) -> Result<(), ConstructionError> {
    let scale = scale2(&lower.vertices).max(scale2(&upper.vertices));
    let eps = tolerance.threshold(scale)
        .map_err(|_| ConstructionError::InvalidTolerance)?;
    let eps2 = eps * eps;
    if !eps2.is_finite() {
        return Err(ConstructionError::Overflow);
    }
    let winding = lower.signed_area().signum();
    if winding == 0.0 || upper.signed_area().signum() != winding {
        return Err(ConstructionError::InvalidDimensions);
    }
    for i in 0..lower.vertices.len() {
        let a0 = lower.vertices[i];
        let b0 = lower.vertices[(i + 1) % lower.vertices.len()];
        let c0 = lower.vertices[(i + 2) % lower.vertices.len()];
        let a1 = upper.vertices[i];
        let b1 = upper.vertices[(i + 1) % upper.vertices.len()];
        let c1 = upper.vertices[(i + 2) % upper.vertices.len()];
        let e00 = b0.sub(a0);
        let e10 = c0.sub(b0);
        let e01 = b1.sub(a1);
        let e11 = c1.sub(b1);
        let bernstein = [
            e00.cross(e10),
            e01.cross(e10) + e00.cross(e11),
            e01.cross(e11),
        ];
        if bernstein.iter().any(|v| !v.is_finite() || *v * winding <= eps2) {
            return Err(ConstructionError::Unsupported);
        }
    }
    Ok(())
}

fn orient(a: Vec2, b: Vec2, c: Vec2) -> f64 {
    b.sub(a).cross(c.sub(a))
}

fn on_segment(a: Vec2, b: Vec2, p: Vec2, eps: f64) -> bool {
    p.x >= a.x.min(b.x) - eps
        && p.x <= a.x.max(b.x) + eps
        && p.y >= a.y.min(b.y) - eps
        && p.y <= a.y.max(b.y) + eps
}

fn segment_intersects(a: Vec2, b: Vec2, c: Vec2, d: Vec2, eps: f64) -> bool {
    let ab = b.sub(a);
    let cd = d.sub(c);
    let ab_c = ab.cross(c.sub(a));
    let ab_d = ab.cross(d.sub(a));
    let cd_a = cd.cross(a.sub(c));
    let cd_b = cd.cross(b.sub(c));
    if [ab_c, ab_d, cd_a, cd_b].iter().any(|v| !v.is_finite()) {
        return false;
    }
    // Orientations have units of length². Scale the positional tolerance by
    // the geometry involved rather than comparing an area directly to a length.
    let geometric_scale = ab.length()
        .max(cd.length())
        .max(a.sub(c).length())
        .max(a.sub(d).length())
        .max(eps);
    let orient_eps = eps * geometric_scale;
    if !orient_eps.is_finite() {
        return false;
    }
    if ab_c.abs() <= orient_eps && on_segment(a, b, c, eps) { return true; }
    if ab_d.abs() <= orient_eps && on_segment(a, b, d, eps) { return true; }
    if cd_a.abs() <= orient_eps && on_segment(c, d, a, eps) { return true; }
    if cd_b.abs() <= orient_eps && on_segment(c, d, b, eps) { return true; }
    ((ab_c > orient_eps && ab_d < -orient_eps) || (ab_c < -orient_eps && ab_d > orient_eps))
        && ((cd_a > orient_eps && cd_b < -orient_eps) || (cd_a < -orient_eps && cd_b > orient_eps))
}

#[derive(Clone, Debug, PartialEq)]
pub struct PlanarPolygon {
    pub vertices: Vec<Vec2>,
}

impl PlanarPolygon {
    pub fn new(vertices: Vec<Vec2>) -> Self { Self { vertices } }

    pub fn validate(&self, tolerance: Tolerance) -> Result<(), ConstructionError> {
        valid_tolerance(tolerance)?;
        if self.vertices.len() < 3 || self.vertices.iter().any(|p| !p.is_finite()) {
            return Err(ConstructionError::Degenerate);
        }
        let eps = tolerance.threshold(scale2(&self.vertices))
            .map_err(|_| ConstructionError::InvalidTolerance)?;
        let n = self.vertices.len();
        for i in 0..n {
            let a = self.vertices[i];
            let b = self.vertices[(i + 1) % n];
            if a.sub(b).length() <= eps { return Err(ConstructionError::Degenerate); }
            for j in (i + 1)..n {
                let j2 = (j + 1) % n;
                if i == j || (i + 1) % n == j || j2 == i { continue; }
                if segment_intersects(a, b, self.vertices[j], self.vertices[j2], eps) {
                    return Err(ConstructionError::SelfIntersection);
                }
            }
        }
        let area = self.signed_area();
        if !area.is_finite() || area.abs() <= eps * eps {
            return Err(ConstructionError::Degenerate);
        }
        Ok(())
    }

    fn signed_area(&self) -> f64 {
        if self.vertices.is_empty() {
            return 0.0;
        }
        // Compute the shoelace sum in a local frame to prevent cancellation
        // when the polygon is translated far from the coordinate origin.
        let reference = self.vertices[0];
        0.5 * self.vertices.iter().enumerate()
            .map(|(i, p)| {
                let q = self.vertices[(i + 1) % self.vertices.len()];
                p.sub(reference).cross(q.sub(reference))
            })
            .sum::<f64>()
    }

    pub fn area(&self, tolerance: Tolerance) -> Result<f64, ConstructionError> {
        self.validate(tolerance)?;
        Ok(self.signed_area().abs())
    }

    pub fn centroid(&self, tolerance: Tolerance) -> Result<Vec2, ConstructionError> {
        self.validate(tolerance)?;
        let area = self.signed_area();
        let reference = self.vertices[0];
        let mut cx = 0.0;
        let mut cy = 0.0;
        for i in 0..self.vertices.len() {
            let a = self.vertices[i].sub(reference);
            let b = self.vertices[(i + 1) % self.vertices.len()].sub(reference);
            let cross = a.cross(b);
            cx += (a.x + b.x) * cross;
            cy += (a.y + b.y) * cross;
        }
        let local = Vec2::new(cx / (6.0 * area), cy / (6.0 * area));
        let result = reference.add(local);
        if result.is_finite() { Ok(result) } else { Err(ConstructionError::Overflow) }
    }

    pub fn is_convex(&self, tolerance: Tolerance) -> Result<bool, ConstructionError> {
        self.validate(tolerance)?;
        let eps = tolerance.threshold(scale2(&self.vertices))
            .map_err(|_| ConstructionError::InvalidTolerance)?;
        let mut sign = 0.0;
        for i in 0..self.vertices.len() {
            let value = orient(
                self.vertices[i],
                self.vertices[(i + 1) % self.vertices.len()],
                self.vertices[(i + 2) % self.vertices.len()],
            );
            let edge_scale = self.vertices[(i + 1) % self.vertices.len()]
                .sub(self.vertices[i])
                .length()
                .max(self.vertices[(i + 2) % self.vertices.len()]
                    .sub(self.vertices[(i + 1) % self.vertices.len()])
                    .length())
                .max(eps);
            if value.abs() <= eps * edge_scale { return Ok(false); }
            if sign == 0.0 { sign = value.signum(); }
            else if value.signum() != sign { return Ok(false); }
        }
        Ok(true)
    }
}

fn line_intersection(
    p: Vec2,
    r: Vec2,
    q: Vec2,
    s: Vec2,
    eps: f64,
) -> Result<Vec2, ConstructionError> {
    let denominator = r.cross(s);
    if !denominator.is_finite() {
        return Err(ConstructionError::Overflow);
    }
    let determinant_eps = eps * r.length().max(s.length()).max(eps);
    if !determinant_eps.is_finite() {
        return Err(ConstructionError::Overflow);
    }
    if denominator.abs() <= determinant_eps {
        return Err(ConstructionError::Degenerate);
    }
    let t = q.sub(p).cross(s) / denominator;
    let result = p.add(r.scale(t));
    if result.is_finite() { Ok(result) } else { Err(ConstructionError::Overflow) }
}

/// Exact signed offset of a strictly convex polygon using mitered joins.
///
/// Positive distance offsets toward the geometric exterior for CCW input and
/// toward the geometric exterior for CW input as well; the polygon winding is
/// preserved. Concave and near-180-degree vertices are deliberately outside
/// the certified domain because their exact trim/topology behavior requires a
/// separate intersection/topology contract.
pub fn offset_convex_polygon(
    polygon: &PlanarPolygon,
    distance: f64,
    tolerance: Tolerance,
) -> Result<PlanarPolygon, ConstructionError> {
    valid_tolerance(tolerance)?;
    polygon.validate(tolerance)?;
    if !distance.is_finite() {
        return Err(ConstructionError::NonFinite);
    }
    if !polygon.is_convex(tolerance)? {
        return Err(ConstructionError::Unsupported);
    }
    let eps = tolerance
        .threshold(scale2(&polygon.vertices))
        .map_err(|_| ConstructionError::InvalidTolerance)?;
    let winding = polygon.signed_area().signum();
    let n = polygon.vertices.len();
    let mut shifted = Vec::with_capacity(n);

    for i in 0..n {
        let a = polygon.vertices[i];
        let b = polygon.vertices[(i + 1) % n];
        let edge = b.sub(a);
        let length = edge.length();
        if !length.is_finite() || length <= eps {
            return Err(ConstructionError::Degenerate);
        }
        let outward = if winding > 0.0 {
            Vec2::new(edge.y, -edge.x).scale(1.0 / length)
        } else {
            Vec2::new(-edge.y, edge.x).scale(1.0 / length)
        };
        shifted.push((a.add(outward.scale(distance)), b.add(outward.scale(distance))));
    }

    let mut vertices = Vec::with_capacity(n);
    for i in 0..n {
        let previous = shifted[(i + n - 1) % n];
        let current = shifted[i];
        vertices.push(line_intersection(
            previous.0,
            previous.1.sub(previous.0),
            current.0,
            current.1.sub(current.0),
            eps,
        )?);
    }

    let result = PlanarPolygon::new(vertices);
    result.validate(tolerance)?;
    Ok(result)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CircularPipe {
    pub path_start: Vec3,
    pub path_end: Vec3,
    pub radius: f64,
    pub reference: Vec3,
}

impl CircularPipe {
    pub fn validate(&self, tolerance: Tolerance) -> Result<(), ConstructionError> {
        valid_tolerance(tolerance)?;
        if [self.path_start, self.path_end, self.reference]
            .iter()
            .any(|v| !v.is_finite()) || !self.radius.is_finite() {
            return Err(ConstructionError::NonFinite);
        }
        let path = self.path_end.sub(self.path_start);
        let scale = path.length().max(self.radius.abs()).max(1.0);
        let eps = tolerance.threshold(scale)
            .map_err(|_| ConstructionError::InvalidTolerance)?;
        if path.length() <= eps || self.radius <= eps {
            return Err(ConstructionError::Degenerate);
        }
        if self.reference.length() == 0.0 {
            return Err(ConstructionError::SingularFrame);
        }
        Frame3::from_tangent(path, self.reference)?;
        Ok(())
    }

    pub fn length(&self, tolerance: Tolerance) -> Result<f64, ConstructionError> {
        self.validate(tolerance)?;
        Ok(self.path_end.sub(self.path_start).length())
    }

    pub fn volume(&self, tolerance: Tolerance) -> Result<f64, ConstructionError> {
        self.validate(tolerance)?;
        let value = PI * self.radius * self.radius * self.length(tolerance)?;
        if value.is_finite() { Ok(value) } else { Err(ConstructionError::Overflow) }
    }

    pub fn surface_point_at(
        &self,
        path_parameter: f64,
        profile_angle: f64,
        tolerance: Tolerance,
    ) -> Result<Vec3, ConstructionError> {
        self.validate(tolerance)?;
        if !path_parameter.is_finite() || !profile_angle.is_finite()
            || !(0.0..=1.0).contains(&path_parameter)
        {
            return Err(ConstructionError::OutOfDomain);
        }
        let delta = self.path_end.sub(self.path_start);
        let frame = Frame3::from_tangent(delta, self.reference)?;
        let center = self.path_start.add(delta.scale(path_parameter));
        let radial = frame.normal.scale(self.radius * profile_angle.cos())
            .add(frame.binormal.scale(self.radius * profile_angle.sin()));
        let result = center.add(radial);
        if result.is_finite() { Ok(result) } else { Err(ConstructionError::Overflow) }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VariableRadiusPipe {
    pub path_start: Vec3,
    pub path_end: Vec3,
    pub start_radius: f64,
    pub end_radius: f64,
    pub reference: Vec3,
}

impl VariableRadiusPipe {
    pub fn validate(&self, tolerance: Tolerance) -> Result<(), ConstructionError> {
        valid_tolerance(tolerance)?;
        if [self.path_start, self.path_end, self.reference]
            .iter()
            .any(|v| !v.is_finite())
            || !self.start_radius.is_finite()
            || !self.end_radius.is_finite()
        {
            return Err(ConstructionError::NonFinite);
        }
        let path = self.path_end.sub(self.path_start);
        let scale = path.length()
            .max(self.start_radius.abs())
            .max(self.end_radius.abs())
            .max(1.0);
        let eps = tolerance.threshold(scale)
            .map_err(|_| ConstructionError::InvalidTolerance)?;
        if path.length() <= eps
            || self.start_radius <= eps
            || self.end_radius <= eps
        {
            return Err(ConstructionError::InvalidDimensions);
        }
        Frame3::from_tangent(path, self.reference)?;
        Ok(())
    }

    pub fn radius_at(&self, parameter: f64, tolerance: Tolerance) -> Result<f64, ConstructionError> {
        self.validate(tolerance)?;
        if !parameter.is_finite() || !(0.0..=1.0).contains(&parameter) {
            return Err(ConstructionError::OutOfDomain);
        }
        Ok(self.start_radius + (self.end_radius - self.start_radius) * parameter)
    }

    pub fn length(&self, tolerance: Tolerance) -> Result<f64, ConstructionError> {
        self.validate(tolerance)?;
        Ok(self.path_end.sub(self.path_start).length())
    }

    pub fn volume(&self, tolerance: Tolerance) -> Result<f64, ConstructionError> {
        self.validate(tolerance)?;
        let l = self.length(tolerance)?;
        let a = self.start_radius;
        let b = self.end_radius;
        let value = PI * l * (a * a + a * b + b * b) / 3.0;
        if value.is_finite() { Ok(value) } else { Err(ConstructionError::Overflow) }
    }

    pub fn surface_point_at(
        &self,
        path_parameter: f64,
        profile_angle: f64,
        tolerance: Tolerance,
    ) -> Result<Vec3, ConstructionError> {
        self.validate(tolerance)?;
        if !path_parameter.is_finite() || !profile_angle.is_finite()
            || !(0.0..=1.0).contains(&path_parameter)
        {
            return Err(ConstructionError::OutOfDomain);
        }
        let delta = self.path_end.sub(self.path_start);
        let frame = Frame3::from_tangent(delta, self.reference)?;
        let radius = self.radius_at(path_parameter, tolerance)?;
        let center = self.path_start.add(delta.scale(path_parameter));
        let radial = frame.normal.scale(radius * profile_angle.cos())
            .add(frame.binormal.scale(radius * profile_angle.sin()));
        let result = center.add(radial);
        if result.is_finite() { Ok(result) } else { Err(ConstructionError::Overflow) }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CircularArcPipe {
    pub center: Vec3,
    pub path_radius: f64,
    pub profile_radius: f64,
    pub start_angle: f64,
    pub end_angle: f64,
}

impl CircularArcPipe {
    pub fn validate(&self, tolerance: Tolerance) -> Result<(), ConstructionError> {
        valid_tolerance(tolerance)?;
        if [self.center].iter().any(|v| !v.is_finite())
            || !self.path_radius.is_finite()
            || !self.profile_radius.is_finite()
            || !self.start_angle.is_finite()
            || !self.end_angle.is_finite()
        {
            return Err(ConstructionError::NonFinite);
        }
        let scale = self.path_radius.max(self.profile_radius).max(1.0);
        let eps = tolerance.threshold(scale)
            .map_err(|_| ConstructionError::InvalidTolerance)?;
        let span = self.end_angle - self.start_angle;
        if self.path_radius <= eps || self.profile_radius <= eps {
            return Err(ConstructionError::InvalidDimensions);
        }
        if self.profile_radius >= self.path_radius {
            return Err(ConstructionError::Unsupported);
        }
        if !span.is_finite() || span.abs() <= eps || span.abs() > 2.0 * PI {
            return Err(ConstructionError::InvalidDimensions);
        }
        Ok(())
    }

    pub fn path_length(&self, tolerance: Tolerance) -> Result<f64, ConstructionError> {
        self.validate(tolerance)?;
        let value = self.path_radius * (self.end_angle - self.start_angle).abs();
        if value.is_finite() { Ok(value) } else { Err(ConstructionError::Overflow) }
    }

    pub fn volume(&self, tolerance: Tolerance) -> Result<f64, ConstructionError> {
        self.validate(tolerance)?;
        let value = PI * self.profile_radius * self.profile_radius * self.path_length(tolerance)?;
        if value.is_finite() { Ok(value) } else { Err(ConstructionError::Overflow) }
    }

    pub fn surface_point_at(
        &self,
        path_parameter: f64,
        profile_angle: f64,
        tolerance: Tolerance,
    ) -> Result<Vec3, ConstructionError> {
        self.validate(tolerance)?;
        if !path_parameter.is_finite() || !profile_angle.is_finite()
            || !(0.0..=1.0).contains(&path_parameter)
        {
            return Err(ConstructionError::OutOfDomain);
        }
        let theta = self.start_angle + (self.end_angle - self.start_angle) * path_parameter;
        let radial = self.path_radius + self.profile_radius * profile_angle.cos();
        let result = Vec3::new(
            self.center.x + radial * theta.cos(),
            self.center.y + radial * theta.sin(),
            self.center.z + self.profile_radius * profile_angle.sin(),
        );
        if result.is_finite() { Ok(result) } else { Err(ConstructionError::Overflow) }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LinearExtrusion {
    pub profile: PlanarPolygon,
    pub height: f64,
}

impl LinearExtrusion {
    pub fn validate(&self, tolerance: Tolerance) -> Result<(), ConstructionError> {
        valid_tolerance(tolerance)?;
        self.profile.validate(tolerance)?;
        if !self.height.is_finite() { return Err(ConstructionError::NonFinite); }
        let eps = tolerance.threshold(scale2(&self.profile.vertices))
            .map_err(|_| ConstructionError::InvalidTolerance)?;
        if self.height.abs() <= eps { return Err(ConstructionError::InvalidDimensions); }
        Ok(())
    }

    pub fn volume(&self, tolerance: Tolerance) -> Result<f64, ConstructionError> {
        self.validate(tolerance)?;
        let value = self.profile.area(tolerance)? * self.height.abs();
        if value.is_finite() && value > 0.0 { Ok(value) } else { Err(ConstructionError::Overflow) }
    }

    pub fn point_at_vertex(&self, index: usize, tolerance: Tolerance) -> Result<Vec3, ConstructionError> {
        self.validate(tolerance)?;
        let p = *self.profile.vertices.get(index).ok_or(ConstructionError::OutOfDomain)?;
        let result = Vec3::new(p.x, p.y, self.height);
        if result.is_finite() { Ok(result) } else { Err(ConstructionError::Overflow) }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RevolvedPolygon {
    /// Profile coordinates are (radius, axial-z) and must remain off-axis.
    pub profile: PlanarPolygon,
    pub angle: f64,
}

impl RevolvedPolygon {
    pub fn validate(&self, tolerance: Tolerance) -> Result<(), ConstructionError> {
        valid_tolerance(tolerance)?;
        self.profile.validate(tolerance)?;
        if !self.angle.is_finite() || self.angle.abs() <= 0.0 || self.angle.abs() > 2.0 * PI {
            return Err(ConstructionError::InvalidDimensions);
        }
        let eps = tolerance.threshold(scale2(&self.profile.vertices))
            .map_err(|_| ConstructionError::InvalidTolerance)?;
        if self.profile.vertices.iter().any(|p| p.x.abs() <= eps) {
            return Err(ConstructionError::Unsupported);
        }
        Ok(())
    }

    pub fn volume(&self, tolerance: Tolerance) -> Result<f64, ConstructionError> {
        self.validate(tolerance)?;
        let area = self.profile.area(tolerance)?;
        let centroid = self.profile.centroid(tolerance)?;
        let value = area * centroid.x.abs() * self.angle.abs();
        if value.is_finite() && value > 0.0 { Ok(value) } else { Err(ConstructionError::Overflow) }
    }

    pub fn point_at(&self, index: usize, parameter: f64, tolerance: Tolerance) -> Result<Vec3, ConstructionError> {
        self.validate(tolerance)?;
        if !parameter.is_finite() || !(0.0..=1.0).contains(&parameter) {
            return Err(ConstructionError::OutOfDomain);
        }
        let p = *self.profile.vertices.get(index).ok_or(ConstructionError::OutOfDomain)?;
        let theta = self.angle * parameter;
        let result = Vec3::new(p.x * theta.cos(), p.x * theta.sin(), p.y);
        if result.is_finite() { Ok(result) } else { Err(ConstructionError::Overflow) }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PolygonLoft {
    pub lower: PlanarPolygon,
    pub lower_z: f64,
    pub upper: PlanarPolygon,
    pub upper_z: f64,
}

impl PolygonLoft {
    pub fn validate(&self, tolerance: Tolerance) -> Result<(), ConstructionError> {
        valid_tolerance(tolerance)?;
        self.lower.validate(tolerance)?;
        self.upper.validate(tolerance)?;
        if self.lower.vertices.len() != self.upper.vertices.len() {
            return Err(ConstructionError::Unsupported);
        }
        if !self.lower.is_convex(tolerance)? || !self.upper.is_convex(tolerance)? {
            return Err(ConstructionError::Unsupported);
        }
        if !self.lower_z.is_finite() || !self.upper_z.is_finite() {
            return Err(ConstructionError::NonFinite);
        }
        let scale = scale2(&self.lower.vertices).max(scale2(&self.upper.vertices));
        if (self.upper_z - self.lower_z).abs() <= tolerance.threshold(scale)
            .map_err(|_| ConstructionError::InvalidTolerance)? {
            return Err(ConstructionError::InvalidDimensions);
        }
        if self.lower.signed_area().signum() != self.upper.signed_area().signum() {
            return Err(ConstructionError::InvalidDimensions);
        }
        loft_family_convex(&self.lower, &self.upper, tolerance)?;
        Ok(())
    }

    pub fn section(&self, parameter: f64, tolerance: Tolerance) -> Result<PlanarPolygon, ConstructionError> {
        self.validate(tolerance)?;
        if !parameter.is_finite() || !(0.0..=1.0).contains(&parameter) {
            return Err(ConstructionError::OutOfDomain);
        }
        let vertices = self.lower.vertices.iter().zip(self.upper.vertices.iter())
            .map(|(a, b)| Vec2::new(
                a.x + (b.x - a.x) * parameter,
                a.y + (b.y - a.y) * parameter,
            ))
            .collect::<Vec<_>>();
        let result = PlanarPolygon::new(vertices);
        result.validate(tolerance)?;
        Ok(result)
    }

    pub fn volume(&self, tolerance: Tolerance) -> Result<f64, ConstructionError> {
        self.validate(tolerance)?;
        let a0 = self.lower.area(tolerance)?;
        let am = self.section(0.5, tolerance)?.area(tolerance)?;
        let a1 = self.upper.area(tolerance)?;
        let value = (self.upper_z - self.lower_z).abs() * (a0 + 4.0 * am + a1) / 6.0;
        if value.is_finite() && value > 0.0 { Ok(value) } else { Err(ConstructionError::Overflow) }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CubicHermite3 {
    pub p0: Vec3,
    pub p1: Vec3,
    pub t0: Vec3,
    pub t1: Vec3,
}

impl CubicHermite3 {
    pub fn validate(&self) -> Result<(), ConstructionError> {
        if [self.p0, self.p1, self.t0, self.t1].iter().any(|p| !p.is_finite()) {
            return Err(ConstructionError::NonFinite);
        }
        if self.p0 == self.p1 && self.t0.length() == 0.0 && self.t1.length() == 0.0 {
            return Err(ConstructionError::Degenerate);
        }
        Ok(())
    }

    pub fn point_at(&self, parameter: f64) -> Result<Vec3, ConstructionError> {
        self.validate()?;
        if !parameter.is_finite() || !(0.0..=1.0).contains(&parameter) {
            return Err(ConstructionError::OutOfDomain);
        }
        let t = parameter;
        let u = 1.0 - t;
        let p = self.p0.scale((1.0 + 2.0 * t) * u * u)
            .add(self.t0.scale(t * u * u))
            .add(self.p1.scale(t * t * (3.0 - 2.0 * t)))
            .add(self.t1.scale(t * t * (t - 1.0)));
        if p.is_finite() { Ok(p) } else { Err(ConstructionError::Overflow) }
    }

    pub fn derivative_at(&self, parameter: f64) -> Result<Vec3, ConstructionError> {
        self.validate()?;
        if !parameter.is_finite() || !(0.0..=1.0).contains(&parameter) {
            return Err(ConstructionError::OutOfDomain);
        }
        let t = parameter;
        let u = 1.0 - t;
        let d = self.p0.scale(-6.0 * t * u)
            .add(self.t0.scale(u * (1.0 - 3.0 * t)))
            .add(self.p1.scale(6.0 * t * u))
            .add(self.t1.scale(t * (3.0 * t - 2.0)));
        if d.is_finite() { Ok(d) } else { Err(ConstructionError::Overflow) }
    }

    pub fn second_derivative_at(&self, parameter: f64) -> Result<Vec3, ConstructionError> {
        self.validate()?;
        if !parameter.is_finite() || !(0.0..=1.0).contains(&parameter) {
            return Err(ConstructionError::OutOfDomain);
        }
        let t = parameter;
        let d = self.p0.scale(6.0 * t - 6.0)
            .add(self.t0.scale(6.0 * t - 4.0))
            .add(self.p1.scale(-6.0 * t + 6.0))
            .add(self.t1.scale(6.0 * t - 2.0));
        if d.is_finite() { Ok(d) } else { Err(ConstructionError::Overflow) }
    }
}

fn curvature_vector(v: Vec3, a: Vec3) -> Result<Vec3, ConstructionError> {
    let speed2 = v.dot(v);
    if !speed2.is_finite() { return Err(ConstructionError::Overflow); }
    if speed2 == 0.0 { return Err(ConstructionError::Indeterminate); }
    let result = a.scale(speed2).sub(v.scale(v.dot(a))).scale(1.0 / (speed2 * speed2));
    if result.is_finite() { Ok(result) } else { Err(ConstructionError::Overflow) }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContinuityGrade { G0, G1, G2, Discontinuous, Indeterminate }

fn approx_vec3(a: Vec3, b: Vec3, tolerance: Tolerance) -> Result<bool, ConstructionError> {
    let scale = a.length().max(b.length()).max(1.0);
    Ok(a.sub(b).length() <= tolerance.threshold(scale)
        .map_err(|_| ConstructionError::InvalidTolerance)?)
}

pub fn verify_continuity(left: &CubicHermite3, right: &CubicHermite3, tolerance: Tolerance)
    -> Result<ContinuityGrade, ConstructionError>
{
    valid_tolerance(tolerance)?;
    left.validate()?;
    right.validate()?;
    if !approx_vec3(left.point_at(1.0)?, right.point_at(0.0)?, tolerance)? {
        return Ok(ContinuityGrade::Discontinuous);
    }
    let vl = left.derivative_at(1.0)?;
    let vr = right.derivative_at(0.0)?;
    let sl = vl.length();
    let sr = vr.length();
    if sl == 0.0 || sr == 0.0 { return Ok(ContinuityGrade::Indeterminate); }
    if !approx_vec3(vl.scale(1.0 / sl), vr.scale(1.0 / sr), tolerance)? {
        return Ok(ContinuityGrade::G0);
    }
    if approx_vec3(
        curvature_vector(vl, left.second_derivative_at(1.0)?)?,
        curvature_vector(vr, right.second_derivative_at(0.0)?)?,
        tolerance,
    )? {
        Ok(ContinuityGrade::G2)
    } else {
        Ok(ContinuityGrade::G1)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Frame3 {
    pub tangent: Vec3,
    pub normal: Vec3,
    pub binormal: Vec3,
}

impl Frame3 {
    pub fn from_tangent(tangent: Vec3, reference: Vec3) -> Result<Self, ConstructionError> {
        if !tangent.is_finite() || !reference.is_finite() { return Err(ConstructionError::NonFinite); }
        let tangent = tangent.normalized().map_err(|_| ConstructionError::SingularFrame)?;
        let canonical = [
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
        ];
        let mut reference = if reference.length() == 0.0 { canonical[0] } else { reference };
        if tangent.dot(reference).abs() > 1.0 - 1.0e-10 {
            reference = *canonical.iter()
                .min_by(|a, b| tangent.dot(**a).abs().total_cmp(&tangent.dot(**b).abs()))
                .ok_or(ConstructionError::SingularFrame)?;
        }
        let normal = reference.sub(tangent.scale(tangent.dot(reference)))
            .normalized().map_err(|_| ConstructionError::SingularFrame)?;
        let binormal = tangent.cross(normal).normalized()
            .map_err(|_| ConstructionError::SingularFrame)?;
        Ok(Self { tangent, normal, binormal })
    }

    pub fn twisted(&self, angle: f64) -> Result<Self, ConstructionError> {
        if !angle.is_finite() { return Err(ConstructionError::NonFinite); }
        let c = angle.cos();
        let s = angle.sin();
        let normal = self.normal.scale(c).add(self.binormal.scale(s));
        let binormal = self.binormal.scale(c).sub(self.normal.scale(s));
        if normal.is_finite() && binormal.is_finite() {
            Ok(Self { tangent: self.tangent, normal, binormal })
        } else {
            Err(ConstructionError::Overflow)
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoxFillet { pub width: f64, pub depth: f64, pub height: f64, pub radius: f64 }

impl BoxFillet {
    pub fn validate(&self, tolerance: Tolerance) -> Result<(), ConstructionError> {
        valid_tolerance(tolerance)?;
        let v = [self.width, self.depth, self.height, self.radius];
        if v.iter().any(|x| !x.is_finite()) { return Err(ConstructionError::NonFinite); }
        let eps = tolerance.threshold(self.width.max(self.depth).max(self.height))
            .map_err(|_| ConstructionError::InvalidTolerance)?;
        if self.width <= eps || self.depth <= eps || self.height <= eps
            || self.radius <= eps
            || 2.0 * self.radius >= self.width.min(self.depth).min(self.height) {
            return Err(ConstructionError::InvalidDimensions);
        }
        Ok(())
    }

    pub fn volume(&self, tolerance: Tolerance) -> Result<f64, ConstructionError> {
        self.validate(tolerance)?;
        let a = self.width;
        let b = self.depth;
        let c = self.height;
        let r = self.radius;
        let value = a * b * c
            - 2.0 * r * (a * b + a * c + b * c)
            + PI * r * r * (a + b + c)
            + (4.0 * PI / 3.0 - 8.0) * r * r * r;
        if value.is_finite() && value > 0.0 { Ok(value) } else { Err(ConstructionError::Overflow) }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoxChamfer { pub width: f64, pub depth: f64, pub height: f64, pub distance: f64 }

impl BoxChamfer {
    pub fn validate(&self, tolerance: Tolerance) -> Result<(), ConstructionError> {
        valid_tolerance(tolerance)?;
        let v = [self.width, self.depth, self.height, self.distance];
        if v.iter().any(|x| !x.is_finite()) { return Err(ConstructionError::NonFinite); }
        let eps = tolerance.threshold(self.width.max(self.depth).max(self.height))
            .map_err(|_| ConstructionError::InvalidTolerance)?;
        if self.width <= eps || self.depth <= eps || self.height <= eps
            || self.distance <= eps
            || 2.0 * self.distance >= self.width.min(self.depth).min(self.height) {
            return Err(ConstructionError::InvalidDimensions);
        }
        Ok(())
    }

    /// Exact material volume for equal-distance cuts on all twelve box edges.
    ///
    /// The removed volume is the twelve central triangular prisms plus the
    /// eight independent corner unions, yielding
    /// 2*d^2*(width + depth + height) - 6*d^3.
    pub fn volume(&self, tolerance: Tolerance) -> Result<f64, ConstructionError> {
        self.validate(tolerance)?;
        let d = self.distance;
        let value = self.width * self.depth * self.height
            - 2.0 * d * d * (self.width + self.depth + self.height)
            + 6.0 * d * d * d;
        if value.is_finite() && value > 0.0 { Ok(value) } else { Err(ConstructionError::Overflow) }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClosedBoxShell { pub width: f64, pub depth: f64, pub height: f64, pub thickness: f64 }

impl ClosedBoxShell {
    pub fn validate(&self, tolerance: Tolerance) -> Result<(), ConstructionError> {
        valid_tolerance(tolerance)?;
        let v = [self.width, self.depth, self.height, self.thickness];
        if v.iter().any(|x| !x.is_finite()) { return Err(ConstructionError::NonFinite); }
        let eps = tolerance.threshold(self.width.max(self.depth).max(self.height))
            .map_err(|_| ConstructionError::InvalidTolerance)?;
        if self.width <= eps || self.depth <= eps || self.height <= eps
            || self.thickness <= eps
            || 2.0 * self.thickness >= self.width.min(self.depth).min(self.height) {
            return Err(ConstructionError::InvalidDimensions);
        }
        Ok(())
    }

    pub fn inner_dimensions(&self, tolerance: Tolerance)
        -> Result<(f64, f64, f64), ConstructionError>
    {
        self.validate(tolerance)?;
        Ok((
            self.width - 2.0 * self.thickness,
            self.depth - 2.0 * self.thickness,
            self.height - 2.0 * self.thickness,
        ))
    }

    pub fn material_volume(&self, tolerance: Tolerance) -> Result<f64, ConstructionError> {
        let (w, d, h) = self.inner_dimensions(tolerance)?;
        let value = self.width * self.depth * self.height - w * d * h;
        if value.is_finite() && value > 0.0 { Ok(value) } else { Err(ConstructionError::Overflow) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tol() -> Tolerance { Tolerance::new(1.0e-9, 1.0e-12).unwrap() }

    fn rect(w: f64, h: f64) -> PlanarPolygon {
        PlanarPolygon::new(vec![
            Vec2::new(0.0, 0.0), Vec2::new(w, 0.0),
            Vec2::new(w, h), Vec2::new(0.0, h),
        ])
    }

    #[test]
    fn planar_polygon_area_and_centroid_are_translation_invariant() {
        let local = rect(20.0, 10.0);
        let shifted = PlanarPolygon::new(vec![
            Vec2::new(1.0e12, 1.0e12),
            Vec2::new(1.0e12 + 20.0, 1.0e12),
            Vec2::new(1.0e12 + 20.0, 1.0e12 + 10.0),
            Vec2::new(1.0e12, 1.0e12 + 10.0),
        ]);
        assert!((local.area(tol()).unwrap() - 200.0).abs() <= 1.0e-9);
        assert!((shifted.area(tol()).unwrap() - 200.0).abs() <= 1.0e-6);
        let centroid = shifted.centroid(tol()).unwrap();
        assert!((centroid.x - (1.0e12 + 10.0)).abs() <= 1.0e-3);
        assert!((centroid.y - (1.0e12 + 5.0)).abs() <= 1.0e-3);
    }

    #[test]
    fn determinant_and_orientation_tolerances_are_scale_consistent() {
        let small = PlanarPolygon::new(vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0e-6, 0.0),
            Vec2::new(1.0e-6, 1.0e-6),
            Vec2::new(0.0, 1.0e-6),
        ]);
        let large = PlanarPolygon::new(vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0e6, 0.0),
            Vec2::new(1.0e6, 1.0e6),
            Vec2::new(0.0, 1.0e6),
        ]);
        assert!(small.validate(tol()).is_ok());
        assert!(large.validate(tol()).is_ok());
        assert!(offset_convex_polygon(&small, 1.0e-7, tol()).is_ok());
        assert!(offset_convex_polygon(&large, 1.0e5, tol()).is_ok());
    }

    #[test]
    fn geometric_scale_is_translation_invariant_for_construction_tolerance() {
        let local = rect(20.0, 10.0);
        let shifted = PlanarPolygon::new(vec![
            Vec2::new(1.0e12, 1.0e12),
            Vec2::new(1.0e12 + 20.0, 1.0e12),
            Vec2::new(1.0e12 + 20.0, 1.0e12 + 10.0),
            Vec2::new(1.0e12, 1.0e12 + 10.0),
        ]);
        let local_offset = offset_convex_polygon(&local, 2.0, tol()).unwrap();
        let shifted_offset = offset_convex_polygon(&shifted, 2.0, tol()).unwrap();
        assert!((local_offset.vertices[1].x - local_offset.vertices[0].x - 24.0).abs() <= 1.0e-9);
        assert!((shifted_offset.vertices[1].x - shifted_offset.vertices[0].x - 24.0).abs() <= 1.0e-6);
        assert!((shifted_offset.vertices[0].x - local_offset.vertices[0].x - 1.0e12).abs() <= 1.0e-3);
    }

    #[test]
    fn convex_polygon_offset_preserves_winding_and_has_expected_rectangle_bounds() {
        let p = rect(20.0, 10.0);
        let outward = offset_convex_polygon(&p, 2.0, tol()).unwrap();
        assert_eq!(
            outward.vertices,
            vec![
                Vec2::new(-2.0, -2.0),
                Vec2::new(22.0, -2.0),
                Vec2::new(22.0, 12.0),
                Vec2::new(-2.0, 12.0),
            ]
        );
        assert!((outward.area(tol()).unwrap() - 336.0).abs() <= 1.0e-9);

        let inward = offset_convex_polygon(&p, -2.0, tol()).unwrap();
        assert!((inward.area(tol()).unwrap() - 96.0).abs() <= 1.0e-9);
    }

    #[test]
    fn offset_rejects_concave_and_parallel_corner_singularities() {
        let concave = PlanarPolygon::new(vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(4.0, 0.0),
            Vec2::new(4.0, 4.0),
            Vec2::new(2.0, 2.0),
            Vec2::new(0.0, 4.0),
        ]);
        assert_eq!(
            offset_convex_polygon(&concave, 1.0, tol()),
            Err(ConstructionError::Unsupported)
        );
        let mut bad = rect(10.0, 10.0);
        bad.vertices[2] = Vec2::new(10.0, 0.0);
        assert!(matches!(
            offset_convex_polygon(&bad, 1.0, tol()),
            Err(ConstructionError::Degenerate) | Err(ConstructionError::SelfIntersection)
        ));
        assert_eq!(
            offset_convex_polygon(&rect(10.0, 10.0), f64::NAN, tol()),
            Err(ConstructionError::NonFinite)
        );
    }

    #[test]
    fn circular_and_variable_radius_pipes_have_exact_volumes_and_endpoints() {
        let pipe = CircularPipe {
            path_start: Vec3::new(0.0, 0.0, 0.0),
            path_end: Vec3::new(0.0, 0.0, 10.0),
            radius: 2.0,
            reference: Vec3::new(1.0, 0.0, 0.0),
        };
        assert!((pipe.length(tol()).unwrap() - 10.0).abs() <= 1.0e-12);
        assert!((pipe.volume(tol()).unwrap() - 40.0 * PI).abs() <= 1.0e-9);
        assert_eq!(
            pipe.surface_point_at(0.0, 0.0, tol()).unwrap(),
            Vec3::new(2.0, 0.0, 0.0)
        );
        let end = pipe.surface_point_at(1.0, PI, tol()).unwrap();
        assert!((end.x + 2.0).abs() <= 1.0e-12);
        assert!(end.y.abs() <= 1.0e-12);
        assert!((end.z - 10.0).abs() <= 1.0e-12);

        let variable = VariableRadiusPipe {
            path_start: Vec3::new(0.0, 0.0, 0.0),
            path_end: Vec3::new(0.0, 0.0, 10.0),
            start_radius: 1.0,
            end_radius: 2.0,
            reference: Vec3::new(1.0, 0.0, 0.0),
        };
        assert_eq!(variable.radius_at(0.0, tol()).unwrap(), 1.0);
        assert_eq!(variable.radius_at(1.0, tol()).unwrap(), 2.0);
        let expected = PI * 10.0 * (1.0 + 2.0 + 4.0) / 3.0;
        assert!((variable.volume(tol()).unwrap() - expected).abs() <= 1.0e-9);
        assert!(variable.surface_point_at(0.5, 0.0, tol()).unwrap().is_finite());
    }

    #[test]
    fn circular_arc_pipe_has_exact_path_and_volume_and_rejects_self_intersection() {
        let pipe = CircularArcPipe {
            center: Vec3::new(0.0, 0.0, 0.0),
            path_radius: 10.0,
            profile_radius: 2.0,
            start_angle: 0.0,
            end_angle: PI / 2.0,
        };
        assert!((pipe.path_length(tol()).unwrap() - 5.0 * PI).abs() <= 1.0e-12);
        assert!((pipe.volume(tol()).unwrap() - 20.0 * PI * PI).abs() <= 1.0e-9);
        assert_eq!(
            pipe.surface_point_at(0.0, 0.0, tol()).unwrap(),
            Vec3::new(12.0, 0.0, 0.0)
        );
        let end = pipe.surface_point_at(1.0, 0.0, tol()).unwrap();
        assert!((end.x).abs() <= 1.0e-12);
        assert!((end.y - 12.0).abs() <= 1.0e-12);
        assert!((end.z).abs() <= 1.0e-12);

        let mut invalid = pipe;
        invalid.profile_radius = invalid.path_radius;
        assert_eq!(invalid.validate(tol()), Err(ConstructionError::Unsupported));
        invalid.profile_radius = f64::NAN;
        assert_eq!(invalid.validate(tol()), Err(ConstructionError::NonFinite));
    }

    #[test]
    fn pipes_use_deterministic_frame_fallback_and_reject_invalid_parameters() {
        let pipe = CircularPipe {
            path_start: Vec3::new(0.0, 0.0, 0.0),
            path_end: Vec3::new(0.0, 0.0, 10.0),
            radius: 1.0,
            reference: Vec3::new(0.0, 0.0, 1.0),
        };
        assert!(pipe.validate(tol()).is_ok());
        assert_eq!(
            pipe.surface_point_at(0.0, 0.0, tol()).unwrap(),
            Vec3::new(1.0, 0.0, 0.0)
        );
        let mut variable = VariableRadiusPipe {
            path_start: Vec3::new(0.0, 0.0, 0.0),
            path_end: Vec3::new(0.0, 0.0, 10.0),
            start_radius: 1.0,
            end_radius: 2.0,
            reference: Vec3::new(1.0, 0.0, 0.0),
        };
        assert_eq!(
            variable.radius_at(1.1, tol()),
            Err(ConstructionError::OutOfDomain)
        );
        variable.start_radius = 0.0;
        assert_eq!(
            variable.validate(tol()),
            Err(ConstructionError::InvalidDimensions)
        );
    }
    #[test]
    fn extrusion_volume_is_exact() {
        let x = LinearExtrusion { profile: rect(20.0, 10.0), height: 30.0 };
        assert!((x.volume(tol()).unwrap() - 6000.0).abs() <= 1.0e-9);
    }

    #[test]
    fn revolution_matches_pappus_and_rejects_axis_contact() {
        let x = RevolvedPolygon {
            profile: PlanarPolygon::new(vec![
                Vec2::new(5.0, 0.0), Vec2::new(10.0, 0.0),
                Vec2::new(10.0, 20.0), Vec2::new(5.0, 20.0),
            ]),
            angle: 2.0 * PI,
        };
        let expected = PI * (100.0 - 25.0) * 20.0;
        assert!((x.volume(tol()).unwrap() - expected).abs() <= 1.0e-9);
        let mut axis = x.clone();
        axis.profile.vertices[0].x = 0.0;
        assert_eq!(axis.validate(tol()), Err(ConstructionError::Unsupported));
    }

    #[test]
    fn loft_volume_uses_exact_quadratic_area_integration() {
        let x = PolygonLoft {
            lower: rect(20.0, 10.0),
            lower_z: 0.0,
            upper: PlanarPolygon::new(vec![
                Vec2::new(2.0, 1.0), Vec2::new(18.0, 1.0),
                Vec2::new(18.0, 9.0), Vec2::new(2.0, 9.0),
            ]),
            upper_z: 30.0,
        };
        let expected = 30.0 * (200.0 + 4.0 * 162.0 + 128.0) / 6.0;
        assert!((x.volume(tol()).unwrap() - expected).abs() <= 1.0e-9);
    }

    #[test]
    fn loft_rejects_mismatched_nonconvex_and_reversed_sections() {
        let lower = rect(10.0, 10.0);
        let mut x = PolygonLoft {
            lower: lower.clone(),
            lower_z: 0.0,
            upper: PlanarPolygon::new(vec![
                Vec2::new(0.0, 0.0), Vec2::new(10.0, 0.0), Vec2::new(0.0, 10.0),
            ]),
            upper_z: 10.0,
        };
        assert_eq!(x.validate(tol()), Err(ConstructionError::Unsupported));
        x.upper = PlanarPolygon::new(vec![
            Vec2::new(0.0, 10.0), Vec2::new(10.0, 10.0),
            Vec2::new(10.0, 0.0), Vec2::new(0.0, 0.0),
        ]);
        assert_eq!(x.validate(tol()), Err(ConstructionError::InvalidDimensions));
    }

    #[test]
    fn polygon_self_intersection_is_fail_closed() {
        let bow = PlanarPolygon::new(vec![
            Vec2::new(0.0, 0.0), Vec2::new(2.0, 2.0),
            Vec2::new(0.0, 2.0), Vec2::new(2.0, 0.0),
        ]);
        assert_eq!(bow.validate(tol()), Err(ConstructionError::SelfIntersection));
    }

    #[test]
    fn hermite_blend_and_geometric_continuity_are_analytic() {
        let a = CubicHermite3 {
            p0: Vec3::new(0.0, 0.0, 0.0), p1: Vec3::new(1.0, 0.0, 0.0),
            t0: Vec3::new(1.0, 0.0, 0.0), t1: Vec3::new(1.0, 0.0, 0.0),
        };
        let b = CubicHermite3 {
            p0: Vec3::new(1.0, 0.0, 0.0), p1: Vec3::new(2.0, 0.0, 0.0),
            t0: Vec3::new(1.0, 0.0, 0.0), t1: Vec3::new(1.0, 0.0, 0.0),
        };
        assert_eq!(verify_continuity(&a, &b, tol()).unwrap(), ContinuityGrade::G2);
        let mut c = b;
        c.p0 = Vec3::new(1.0, 1.0, 0.0);
        assert_eq!(verify_continuity(&a, &c, tol()).unwrap(), ContinuityGrade::Discontinuous);
    }

    #[test]
    fn frame_reference_fallback_and_twist_are_deterministic() {
        let f = Frame3::from_tangent(
            Vec3::new(1.0, 2.0, 3.0),
            Vec3::new(0.0, 0.0, 1.0),
        ).unwrap();
        assert!((f.tangent.length() - 1.0).abs() <= 1.0e-12);
        assert!((f.normal.length() - 1.0).abs() <= 1.0e-12);
        assert!((f.binormal.length() - 1.0).abs() <= 1.0e-12);
        assert!(f.tangent.dot(f.normal).abs() <= 1.0e-12);
        let t = f.twisted(0.7).unwrap();
        assert!((t.normal.dot(t.binormal)).abs() <= 1.0e-12);
    }

    #[test]
    fn fillet_chamfer_shell_match_analytic_contracts() {
        let fillet = BoxFillet { width: 20.0, depth: 30.0, height: 40.0, radius: 2.0 };
        let expected_fillet = 24000.0
            - 2.0 * 2.0 * (600.0 + 800.0 + 1200.0)
            + PI * 4.0 * (20.0 + 30.0 + 40.0)
            + (4.0 * PI / 3.0 - 8.0) * 8.0;
        assert!((fillet.volume(tol()).unwrap() - expected_fillet).abs() <= 1.0e-9);

        let chamfer = BoxChamfer { width: 20.0, depth: 30.0, height: 40.0, distance: 2.0 };
        assert_eq!(chamfer.volume(tol()).unwrap(), 23328.0);

        let shell = ClosedBoxShell { width: 20.0, depth: 30.0, height: 40.0, thickness: 2.0 };
        assert_eq!(shell.inner_dimensions(tol()).unwrap(), (16.0, 26.0, 36.0));
        assert!((shell.material_volume(tol()).unwrap() - 9024.0).abs() <= 1.0e-9);
    }

    #[test]
    fn construction_rejects_singular_and_nonfinite_inputs() {
        assert_eq!(
            Frame3::from_tangent(Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 0.0, 0.0)),
            Err(ConstructionError::SingularFrame)
        );
        let mut x = LinearExtrusion { profile: rect(10.0, 10.0), height: 1.0 };
        x.height = f64::NAN;
        assert_eq!(x.validate(tol()), Err(ConstructionError::NonFinite));
    }

    #[test]
    fn construction_stays_finite_at_large_scale() {
        let s = 1.0e9;
        let x = LinearExtrusion { profile: rect(20.0 * s, 10.0 * s), height: 30.0 * s };
        assert!(x.volume(tol()).unwrap().is_finite());
        let q = RevolvedPolygon {
            profile: PlanarPolygon::new(vec![
                Vec2::new(5.0 * s, 0.0), Vec2::new(10.0 * s, 0.0),
                Vec2::new(10.0 * s, 20.0 * s), Vec2::new(5.0 * s, 20.0 * s),
            ]),
            angle: 2.0 * PI,
        };
        assert!(q.volume(tol()).unwrap().is_finite());
    }
}
