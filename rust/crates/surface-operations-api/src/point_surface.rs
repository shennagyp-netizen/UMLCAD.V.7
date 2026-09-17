use crate::IntersectionStatus;
use umlcad_v6_nurbs_surface_api::{NurbsSurface3DDefinition, Point3};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PointSurfaceClosestPoint {
    pub point: Point3,
    pub surface_uv: (f64, f64),
    pub distance: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PointSurfaceClosestPointResult {
    pub status: IntersectionStatus,
    pub closest: Option<PointSurfaceClosestPoint>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PointSurfaceClosestPointError {
    NonFinite,
    InvalidSurface,
    UnsupportedSurfaceFamily,
    DegenerateSurface,
    NumericalFailure,
}

#[derive(Clone, Copy)]
struct Patch {
    origin: Point3,
    du: Point3,
    dv: Point3,
    u0: f64,
    u1: f64,
    v0: f64,
    v1: f64,
}

fn sub(a: Point3, b: Point3) -> Point3 {
    Point3 { x: a.x - b.x, y: a.y - b.y, z: a.z - b.z }
}
fn add(a: Point3, b: Point3) -> Point3 {
    Point3 { x: a.x + b.x, y: a.y + b.y, z: a.z + b.z }
}
fn scale(a: Point3, s: f64) -> Point3 {
    Point3 { x: a.x * s, y: a.y * s, z: a.z * s }
}
fn dot(a: Point3, b: Point3) -> f64 { a.x * b.x + a.y * b.y + a.z * b.z }
fn norm(a: Point3) -> f64 { dot(a, a).sqrt() }
fn point_at(p: Patch, su: f64, sv: f64) -> Point3 {
    add(p.origin, add(scale(p.du, su), scale(p.dv, sv)))
}
fn parameter_from_normalized(p: Patch, su: f64, sv: f64) -> (f64, f64) {
    (p.u0 + su * (p.u1 - p.u0), p.v0 + sv * (p.v1 - p.v0))
}

fn patch(surface: &NurbsSurface3DDefinition) -> Result<Patch, PointSurfaceClosestPointError> {
    surface.validate().map_err(|_| PointSurfaceClosestPointError::InvalidSurface)?;
    if surface.degree_u != 1
        || surface.degree_v != 1
        || surface.count_u != 2
        || surface.count_v != 2
        || surface.weights.iter().any(|w| (*w - 1.0).abs() > 1e-14)
    {
        return Err(PointSurfaceClosestPointError::UnsupportedSurfaceFamily);
    }
    let origin = surface.control_points[0];
    let du = sub(surface.control_points[2], origin);
    let dv = sub(surface.control_points[1], origin);
    let closure = sub(sub(surface.control_points[3], surface.control_points[2]), dv);
    let reference = norm(du).max(norm(dv)).max(1.0);
    if norm(closure) > 1e-12 * reference {
        return Err(PointSurfaceClosestPointError::UnsupportedSurfaceFamily);
    }
    let ((u0, u1), (v0, v1)) = surface
        .parameter_domain()
        .map_err(|_| PointSurfaceClosestPointError::InvalidSurface)?;
    if !norm(du).is_finite() || !norm(dv).is_finite() || norm(du) == 0.0 || norm(dv) == 0.0 {
        return Err(PointSurfaceClosestPointError::DegenerateSurface);
    }
    Ok(Patch { origin, du, dv, u0, u1, v0, v1 })
}

fn normalized_uv_unconstrained(
    p: Patch,
    point: Point3,
) -> Result<(f64, f64), PointSurfaceClosestPointError> {
    let r = sub(point, p.origin);
    let aa = dot(p.du, p.du);
    let ab = dot(p.du, p.dv);
    let bb = dot(p.dv, p.dv);
    let det = aa * bb - ab * ab;
    if !det.is_finite() || det <= 1e-24 * aa.max(bb).powi(2) {
        return Err(PointSurfaceClosestPointError::DegenerateSurface);
    }
    let ru = dot(p.du, r);
    let rv = dot(p.dv, r);
    Ok(((ru * bb - rv * ab) / det, (rv * aa - ru * ab) / det))
}

fn objective(p: Patch, point: Point3, su: f64, sv: f64) -> f64 {
    norm(sub(point_at(p, su, sv), point)).powi(2)
}

fn edge_min_u(p: Patch, point: Point3, fixed_su: f64) -> (f64, f64) {
    let base = sub(add(p.origin, scale(p.du, fixed_su)), point);
    let denom = dot(p.dv, p.dv);
    let sv = if denom == 0.0 { 0.0 } else { (-dot(p.dv, base) / denom).clamp(0.0, 1.0) };
    (fixed_su, sv)
}

fn edge_min_v(p: Patch, point: Point3, fixed_sv: f64) -> (f64, f64) {
    let base = sub(add(p.origin, scale(p.dv, fixed_sv)), point);
    let denom = dot(p.du, p.du);
    let su = if denom == 0.0 { 0.0 } else { (-dot(p.du, base) / denom).clamp(0.0, 1.0) };
    (su, fixed_sv)
}

pub fn closest_point_on_planar_nurbs_surface(
    point: Point3,
    surface: &NurbsSurface3DDefinition,
    tolerance: f64,
) -> Result<PointSurfaceClosestPointResult, PointSurfaceClosestPointError> {
    if [point.x, point.y, point.z].iter().any(|v| !v.is_finite())
        || !tolerance.is_finite()
        || tolerance < 0.0
    {
        return Err(PointSurfaceClosestPointError::NonFinite);
    }
    let p = patch(surface)?;
    let (su_star, sv_star) = normalized_uv_unconstrained(p, point)?;
    let mut candidates = Vec::with_capacity(9);
    candidates.push((su_star.clamp(0.0, 1.0), sv_star.clamp(0.0, 1.0)));
    candidates.push(edge_min_u(p, point, 0.0));
    candidates.push(edge_min_u(p, point, 1.0));
    candidates.push(edge_min_v(p, point, 0.0));
    candidates.push(edge_min_v(p, point, 1.0));
    candidates.extend([(0.0, 0.0), (0.0, 1.0), (1.0, 0.0), (1.0, 1.0)]);
    let mut best = None::<(f64, f64, f64)>;
    for (su, sv) in candidates {
        let d2 = objective(p, point, su, sv);
        if !d2.is_finite() {
            return Err(PointSurfaceClosestPointError::NumericalFailure);
        }
        if best.is_none_or(|current| d2 < current.2) {
            best = Some((su, sv, d2));
        }
    }
    let (su, sv, d2) = best.ok_or(PointSurfaceClosestPointError::NumericalFailure)?;
    let closest = PointSurfaceClosestPoint {
        point: point_at(p, su, sv),
        surface_uv: parameter_from_normalized(p, su, sv),
        distance: d2.sqrt(),
    };
    Ok(PointSurfaceClosestPointResult {
        status: IntersectionStatus::Unique,
        closest: Some(closest),
    })
}

pub trait PointSurfaceOperations {
    fn closest_point_on_planar_nurbs_surface(
        &self,
        point: Point3,
        surface: &NurbsSurface3DDefinition,
        tolerance: f64,
    ) -> Result<PointSurfaceClosestPointResult, PointSurfaceClosestPointError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_patch_with_domain(u0: f64, u1: f64, v0: f64, v1: f64) -> NurbsSurface3DDefinition {
        NurbsSurface3DDefinition::new(
            (1, 1),
            vec![
                Point3 { x: 0.0, y: 0.0, z: 0.0 },
                Point3 { x: 0.0, y: 1.0, z: 0.0 },
                Point3 { x: 1.0, y: 0.0, z: 0.0 },
                Point3 { x: 1.0, y: 1.0, z: 0.0 },
            ],
            vec![1.0; 4],
            (2, 2),
            vec![u0, u0, u1, u1],
            vec![v0, v0, v1, v1],
        )
    }

    #[test]
    fn interior_projection_is_exact_on_non_unit_domain() {
        let result = closest_point_on_planar_nurbs_surface(
            Point3 { x: 0.25, y: 0.75, z: 2.0 },
            &unit_patch_with_domain(2.0, 4.0, 10.0, 20.0),
            1e-10,
        )
        .unwrap();
        let closest = result.closest.unwrap();
        assert_eq!(result.status, IntersectionStatus::Unique);
        assert!((closest.surface_uv.0 - 2.5).abs() < 1e-12);
        assert!((closest.surface_uv.1 - 17.5).abs() < 1e-12);
        assert!((closest.distance - 2.0).abs() < 1e-12);
    }

    #[test]
    fn outside_point_closest_point_is_boundary() {
        let result = closest_point_on_planar_nurbs_surface(
            Point3 { x: 2.0, y: 0.5, z: 0.0 },
            &unit_patch_with_domain(2.0, 4.0, 10.0, 20.0),
            1e-10,
        )
        .unwrap();
        let closest = result.closest.unwrap();
        assert!((closest.surface_uv.0 - 4.0).abs() < 1e-12);
        assert!((closest.surface_uv.1 - 15.0).abs() < 1e-12);
        assert!((closest.distance - 1.0).abs() < 1e-12);
    }
}
