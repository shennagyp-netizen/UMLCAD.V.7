use umlcad_v6_nurbs_surface_api::{
    NurbsSurface3DDefinition, NurbsSurfaceEvaluationError, Point3,
};

mod bilinear_branch_pairing;
mod bilinear_multisegment;
mod curve_split;
mod curve_surface;
mod general_surface_intersection;
mod planar_line_surface;
mod planar_nurbs_relation;
mod point_surface;
mod rational_bilinear;
mod surface_surface;
mod weighted_rational_bilinear;

pub use bilinear_multisegment::try_intersect_bilinear_planar_multisegment;
pub use curve_split::{
    split_line_segment_at_intersections, SplitLineSegmentError, SplitLineSegmentResult,
};
pub use curve_surface::{
    intersect_nurbs_curve_surface, CurveSurfaceIntersectionError, CurveSurfaceIntersectionPoint,
    CurveSurfaceIntersectionResult, NurbsCurveSurfaceOperations,
};
pub use general_surface_intersection::intersect_nurbs_surfaces as intersect_nurbs_surfaces_general;
pub use planar_nurbs_relation::{
    classify_planar_nurbs_surface_relation, PlanarNurbsSurfaceRelation,
    PlanarNurbsSurfaceRelationError, Plane3D,
};
pub use point_surface::{
    closest_point_on_planar_nurbs_surface, PointSurfaceClosestPoint,
    PointSurfaceClosestPointError, PointSurfaceClosestPointResult, PointSurfaceOperations,
};
pub use rational_bilinear::try_intersect_rational_bilinear_planar;
pub use surface_surface::{
    classify_nurbs_surface_pair_relation, intersect_planar_nurbs_surfaces,
    NurbsSurfacePairRelation, NurbsSurfacePairRelationError, SurfaceIntersectionEndpoint,
    SurfaceIntersectionSegment, SurfaceSurfaceIntersectionError, SurfaceSurfaceIntersectionResult,
    SurfaceSurfaceOperations,
};

pub fn intersect_nurbs_surfaces(
    first: &NurbsSurface3DDefinition,
    second: &NurbsSurface3DDefinition,
    tolerance: f64,
) -> Result<SurfaceSurfaceIntersectionResult, SurfaceSurfaceIntersectionError> {
    if let Some(result) = weighted_rational_bilinear::try_intersect_weighted_rational_bilinear_planar(
        first, second, tolerance,
    )? {
        return Ok(result);
    }
    if let Some(result) = try_intersect_rational_bilinear_planar(first, second, tolerance)? {
        return Ok(result);
    }
    if let Some(result) = try_intersect_bilinear_planar_multisegment(first, second, tolerance)? {
        return Ok(result);
    }
    intersect_nurbs_surfaces_general(first, second, tolerance)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LineSegment3D {
    pub start: Point3,
    pub end: Point3,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LineSurfaceIntersectionError {
    NonFinite,
    DegenerateLine,
    InvalidSurface,
    NumericalFailure,
    TangentialContact,
    CoincidentOrUnderdetermined,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IntersectionStatus {
    NoIntersection,
    Unique,
    Ambiguous,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LineSurfaceIntersectionPoint {
    pub line_parameter: f64,
    pub u: f64,
    pub v: f64,
    pub point: Point3,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LineSurfaceIntersectionResult {
    pub status: IntersectionStatus,
    pub points: Vec<LineSurfaceIntersectionPoint>,
}

impl LineSegment3D {
    pub fn validate(&self) -> Result<(), LineSurfaceIntersectionError> {
        let values = [
            self.start.x,
            self.start.y,
            self.start.z,
            self.end.x,
            self.end.y,
            self.end.z,
        ];
        if values.iter().any(|v| !v.is_finite()) {
            return Err(LineSurfaceIntersectionError::NonFinite);
        }
        let d = Point3 {
            x: self.end.x - self.start.x,
            y: self.end.y - self.start.y,
            z: self.end.z - self.start.z,
        };
        if d.norm() == 0.0 {
            return Err(LineSurfaceIntersectionError::DegenerateLine);
        }
        Ok(())
    }
}

fn det3(a: [[f64; 3]; 3]) -> f64 {
    a[0][0] * (a[1][1] * a[2][2] - a[1][2] * a[2][1])
        - a[0][1] * (a[1][0] * a[2][2] - a[1][2] * a[2][0])
        + a[0][2] * (a[1][0] * a[2][1] - a[1][1] * a[2][0])
}

fn solve3(a: [[f64; 3]; 3], b: Point3) -> Option<[f64; 3]> {
    let d = det3(a);
    if !d.is_finite() || d.abs() <= 1e-14 {
        return None;
    }
    let mut x = a;
    x[0][0] = b.x;
    x[1][0] = b.y;
    x[2][0] = b.z;
    let x0 = det3(x) / d;
    x = a;
    x[0][1] = b.x;
    x[1][1] = b.y;
    x[2][1] = b.z;
    let x1 = det3(x) / d;
    x = a;
    x[0][2] = b.x;
    x[1][2] = b.y;
    x[2][2] = b.z;
    let x2 = det3(x) / d;
    Some([x0, x1, x2])
}

pub fn intersect_line_segment_nurbs_surface(
    line: LineSegment3D,
    surface: &NurbsSurface3DDefinition,
    tolerance: f64,
) -> Result<LineSurfaceIntersectionResult, LineSurfaceIntersectionError> {
    line.validate()?;
    if !tolerance.is_finite() || tolerance < 0.0 {
        return Err(LineSurfaceIntersectionError::NonFinite);
    }
    surface
        .validate()
        .map_err(|_| LineSurfaceIntersectionError::InvalidSurface)?;
    if let Some(result) = planar_line_surface::classify_planar_line_surface(
        line,
        surface,
        tolerance,
    )? {
        return Ok(result);
    }
    let ((u0, u1), (v0, v1)) = surface
        .parameter_domain()
        .map_err(|_| LineSurfaceIntersectionError::InvalidSurface)?;
    let direction = Point3 {
        x: line.end.x - line.start.x,
        y: line.end.y - line.start.y,
        z: line.end.z - line.start.z,
    };
    let seeds = 5usize;
    let mut roots = Vec::new();
    let scale = direction
        .norm()
        .max(surface.control_points.iter().map(|p| p.norm()).fold(1.0, f64::max));
    let residual_tol = tolerance.max(1e-10 * scale);
    for iu in 0..seeds {
        for iv in 0..seeds {
            for it in 0..seeds {
                let mut u = u0 + (u1 - u0) * (iu as f64 + 0.5) / seeds as f64;
                let mut v = v0 + (v1 - v0) * (iv as f64 + 0.5) / seeds as f64;
                let mut t = (it as f64 + 0.5) / seeds as f64;
                let mut converged = false;
                for _ in 0..40 {
                    let d = surface.differential_at(u, v).map_err(|e| match e {
                        NurbsSurfaceEvaluationError::InsufficientContinuity => {
                            LineSurfaceIntersectionError::TangentialContact
                        }
                        _ => LineSurfaceIntersectionError::NumericalFailure,
                    })?;
                    let curve = Point3 {
                        x: line.start.x + direction.x * t,
                        y: line.start.y + direction.y * t,
                        z: line.start.z + direction.z * t,
                    };
                    let r = Point3 {
                        x: d.point.x - curve.x,
                        y: d.point.y - curve.y,
                        z: d.point.z - curve.z,
                    };
                    if r.norm() <= residual_tol {
                        converged = true;
                        break;
                    }
                    let delta = match solve3(
                        [
                            [d.du.x, d.dv.x, -direction.x],
                            [d.du.y, d.dv.y, -direction.y],
                            [d.du.z, d.dv.z, -direction.z],
                        ],
                        Point3 {
                            x: -r.x,
                            y: -r.y,
                            z: -r.z,
                        },
                    ) {
                        Some(x) => x,
                        None => break,
                    };
                    u += delta[0];
                    v += delta[1];
                    t += delta[2];
                    if !u.is_finite() || !v.is_finite() || !t.is_finite() {
                        break;
                    }
                    if u < u0 - residual_tol
                        || u > u1 + residual_tol
                        || v < v0 - residual_tol
                        || v > v1 + residual_tol
                        || t < -residual_tol
                        || t > 1.0 + residual_tol
                    {
                        break;
                    }
                }
                if converged
                    && u >= u0 - residual_tol
                    && u <= u1 + residual_tol
                    && v >= v0 - residual_tol
                    && v <= v1 + residual_tol
                    && t >= -residual_tol
                    && t <= 1.0 + residual_tol
                {
                    let u = u.clamp(u0, u1);
                    let v = v.clamp(v0, v1);
                    let t = t.clamp(0.0, 1.0);
                    let d = surface
                        .differential_at(u, v)
                        .map_err(|_| LineSurfaceIntersectionError::NumericalFailure)?;
                    let point = Point3 {
                        x: line.start.x + direction.x * t,
                        y: line.start.y + direction.y * t,
                        z: line.start.z + direction.z * t,
                    };
                    if !d.du.cross(d.dv).norm().is_finite() {
                        return Err(LineSurfaceIntersectionError::NumericalFailure);
                    }
                    if roots.iter().all(|q: &LineSurfaceIntersectionPoint| {
                        (q.point.x - point.x)
                            .hypot((q.point.y - point.y).hypot(q.point.z - point.z))
                            > residual_tol * 10.0
                    }) {
                        roots.push(LineSurfaceIntersectionPoint {
                            line_parameter: t,
                            u,
                            v,
                            point,
                        });
                    }
                }
            }
        }
    }
    roots.sort_by(|a, b| {
        a.line_parameter
            .total_cmp(&b.line_parameter)
            .then(a.u.total_cmp(&b.u))
            .then(a.v.total_cmp(&b.v))
    });
    let status = match roots.len() {
        0 => IntersectionStatus::NoIntersection,
        1 => IntersectionStatus::Unique,
        _ => IntersectionStatus::Ambiguous,
    };
    Ok(LineSurfaceIntersectionResult { status, points: roots })
}

pub trait CurveSurfaceOperations {
    fn intersect_line_segment_nurbs_surface(
        &self,
        line: LineSegment3D,
        surface: &NurbsSurface3DDefinition,
        tolerance: f64,
    ) -> Result<LineSurfaceIntersectionResult, LineSurfaceIntersectionError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plane() -> NurbsSurface3DDefinition {
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
            vec![0.0, 0.0, 1.0, 1.0],
            vec![0.0, 0.0, 1.0, 1.0],
        )
    }

    #[test]
    fn line_plane_has_one_exact_intersection() {
        let r = intersect_line_segment_nurbs_surface(
            LineSegment3D {
                start: Point3 { x: 0.25, y: 0.75, z: -1.0 },
                end: Point3 { x: 0.25, y: 0.75, z: 1.0 },
            },
            &plane(),
            1e-10,
        )
        .unwrap();
        assert_eq!(r.status, IntersectionStatus::Unique);
        assert_eq!(r.points.len(), 1);
        let p = r.points[0];
        assert!((p.line_parameter - 0.5).abs() < 1e-10);
        assert!((p.u - 0.25).abs() < 1e-10);
        assert!((p.v - 0.75).abs() < 1e-10);
        assert!(p.point.z.abs() < 1e-10);
    }

    #[test]
    fn parallel_line_has_no_intersection() {
        let r = intersect_line_segment_nurbs_surface(
            LineSegment3D {
                start: Point3 { x: 0.25, y: 0.75, z: 1.0 },
                end: Point3 { x: 0.75, y: 0.75, z: 1.0 },
            },
            &plane(),
            1e-10,
        )
        .unwrap();
        assert_eq!(r.status, IntersectionStatus::NoIntersection);
    }

    #[test]
    fn degenerate_line_is_rejected() {
        let p = Point3 { x: 0.0, y: 0.0, z: 0.0 };
        assert_eq!(
            LineSegment3D { start: p, end: p }.validate(),
            Err(LineSurfaceIntersectionError::DegenerateLine)
        );
    }
}
