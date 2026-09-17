use umlcad_v6_freeform_api::NurbsCurve3DDefinition;
use umlcad_v6_nurbs_surface_api::NurbsSurface3DDefinition;
use umlcad_v6_surface_operations_api::{
    intersect_line_segment_nurbs_surface, intersect_nurbs_curve_surface, CurveSurfaceOperations,
    CurveSurfaceIntersectionError, CurveSurfaceIntersectionPoint, CurveSurfaceIntersectionResult,
    IntersectionStatus, LineSegment3D, LineSurfaceIntersectionError, LineSurfaceIntersectionPoint,
    LineSurfaceIntersectionResult, NurbsCurveSurfaceOperations,
};

use crate::{OcctBackend, OCCT_OK};

unsafe extern "C" {
    fn umlcad_occt_line_surface_intersection(
        poles_xyz: *const f64,
        count_u: u32,
        count_v: u32,
        weights: *const f64,
        knots_u: *const f64,
        knot_count_u: u32,
        knots_v: *const f64,
        knot_count_v: u32,
        degree_u: u32,
        degree_v: u32,
        start_xyz: *const f64,
        end_xyz: *const f64,
        tolerance: f64,
        out_values: *mut f64,
        capacity: u32,
        out_count: *mut u32,
        out_tangent: *mut i32,
    ) -> i32;

    fn umlcad_occt_nurbs_curve_surface_intersection(
        curve_poles_xyz: *const f64,
        curve_count: u32,
        curve_weights: *const f64,
        curve_knots: *const f64,
        curve_knot_count: u32,
        curve_degree: u32,
        surface_poles_xyz: *const f64,
        count_u: u32,
        count_v: u32,
        surface_weights: *const f64,
        surface_knots_u: *const f64,
        surface_knot_count_u: u32,
        surface_knots_v: *const f64,
        surface_knot_count_v: u32,
        surface_degree_u: u32,
        surface_degree_v: u32,
        tolerance: f64,
        out_values: *mut f64,
        capacity: u32,
        out_count: *mut u32,
        out_tangent: *mut i32,
    ) -> i32;
}

impl CurveSurfaceOperations for OcctBackend {
    fn intersect_line_segment_nurbs_surface(
        &self,
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

        let expected = intersect_line_segment_nurbs_surface(line, surface, tolerance)?;
        let mut poles_xyz = Vec::with_capacity(surface.control_points.len() * 3);
        for point in &surface.control_points {
            poles_xyz.extend([point.x, point.y, point.z]);
        }
        let start = [line.start.x, line.start.y, line.start.z];
        let end = [line.end.x, line.end.y, line.end.z];
        const CAPACITY: u32 = 64;
        let mut values = vec![0.0f64; CAPACITY as usize * 6];
        let mut count = 0u32;
        let mut tangent = 0i32;
        let status = unsafe {
            umlcad_occt_line_surface_intersection(
                poles_xyz.as_ptr(),
                surface.count_u as u32,
                surface.count_v as u32,
                surface.weights.as_ptr(),
                surface.knots_u.as_ptr(),
                surface.knots_u.len() as u32,
                surface.knots_v.as_ptr(),
                surface.knots_v.len() as u32,
                surface.degree_u as u32,
                surface.degree_v as u32,
                start.as_ptr(),
                end.as_ptr(),
                tolerance,
                values.as_mut_ptr(),
                CAPACITY,
                &mut count,
                &mut tangent,
            )
        };
        if status != OCCT_OK {
            return Err(LineSurfaceIntersectionError::NumericalFailure);
        }
        if tangent != 0 {
            return Err(LineSurfaceIntersectionError::TangentialContact);
        }
        if count > CAPACITY {
            return Err(LineSurfaceIntersectionError::NumericalFailure);
        }

        let dx = line.end.x - line.start.x;
        let dy = line.end.y - line.start.y;
        let dz = line.end.z - line.start.z;
        let denom = dx * dx + dy * dy + dz * dz;
        let native: Vec<LineSurfaceIntersectionPoint> = (0..count as usize)
            .map(|i| {
                let offset = 6 * i;
                let point = umlcad_v6_nurbs_surface_api::Point3 {
                    x: values[offset + 3],
                    y: values[offset + 4],
                    z: values[offset + 5],
                };
                let t = ((point.x - line.start.x) * dx
                    + (point.y - line.start.y) * dy
                    + (point.z - line.start.z) * dz)
                    / denom;
                LineSurfaceIntersectionPoint {
                    line_parameter: t,
                    u: values[offset],
                    v: values[offset + 1],
                    point,
                }
            })
            .collect();

        if native.len() != expected.points.len() {
            return Err(LineSurfaceIntersectionError::NumericalFailure);
        }
        let comparison_tol = tolerance.max(1e-9) * 20.0;
        for native_point in &native {
            if expected.points.iter().all(|expected_point| {
                (expected_point.point.x - native_point.point.x)
                    .hypot((expected_point.point.y - native_point.point.y)
                        .hypot(expected_point.point.z - native_point.point.z))
                    > comparison_tol
            }) {
                return Err(LineSurfaceIntersectionError::NumericalFailure);
            }
        }

        let status = match native.len() {
            0 => IntersectionStatus::NoIntersection,
            1 => IntersectionStatus::Unique,
            _ => IntersectionStatus::Ambiguous,
        };
        Ok(LineSurfaceIntersectionResult { status, points: native })
    }
}

impl NurbsCurveSurfaceOperations for OcctBackend {
    fn intersect_nurbs_curve_surface(
        &self,
        curve: &NurbsCurve3DDefinition,
        surface: &NurbsSurface3DDefinition,
        tolerance: f64,
    ) -> Result<CurveSurfaceIntersectionResult, CurveSurfaceIntersectionError> {
        curve.validate().map_err(|_| CurveSurfaceIntersectionError::InvalidCurve)?;
        surface.validate().map_err(|_| CurveSurfaceIntersectionError::InvalidSurface)?;
        if !tolerance.is_finite() || tolerance < 0.0 {
            return Err(CurveSurfaceIntersectionError::NonFinite);
        }
        let expected = intersect_nurbs_curve_surface(curve, surface, tolerance)?;
        let mut curve_poles = Vec::with_capacity(curve.control_points.len() * 3);
        for p in &curve.control_points {
            curve_poles.extend([p.x, p.y, p.z]);
        }
        let mut surface_poles = Vec::with_capacity(surface.control_points.len() * 3);
        for p in &surface.control_points {
            surface_poles.extend([p.x, p.y, p.z]);
        }
        const CAPACITY: u32 = 64;
        let mut values = vec![0.0; CAPACITY as usize * 6];
        let mut count = 0u32;
        let mut tangent = 0i32;
        let status = unsafe {
            umlcad_occt_nurbs_curve_surface_intersection(
                curve_poles.as_ptr(),
                curve.control_points.len() as u32,
                curve.weights.as_ptr(),
                curve.knots.as_ptr(),
                curve.knots.len() as u32,
                curve.degree as u32,
                surface_poles.as_ptr(),
                surface.count_u as u32,
                surface.count_v as u32,
                surface.weights.as_ptr(),
                surface.knots_u.as_ptr(),
                surface.knots_u.len() as u32,
                surface.knots_v.as_ptr(),
                surface.knots_v.len() as u32,
                surface.degree_u as u32,
                surface.degree_v as u32,
                tolerance,
                values.as_mut_ptr(),
                CAPACITY,
                &mut count,
                &mut tangent,
            )
        };
        if status != OCCT_OK || count > CAPACITY {
            return Err(CurveSurfaceIntersectionError::NumericalFailure);
        }
        if tangent != 0 {
            return Err(CurveSurfaceIntersectionError::TangentialContact);
        }
        let native: Vec<CurveSurfaceIntersectionPoint> = values
            .chunks_exact(6)
            .take(count as usize)
            .map(|v| CurveSurfaceIntersectionPoint {
                curve_parameter: v[0],
                u: v[1],
                v: v[2],
                point: umlcad_v6_nurbs_surface_api::Point3 {
                    x: v[3],
                    y: v[4],
                    z: v[5],
                },
            })
            .collect();
        let comparison_tol = tolerance.max(1e-9) * 20.0;
        if native.len() != expected.points.len()
            || native.iter().any(|n| {
                expected.points.iter().all(|e| {
                    (e.point.x - n.point.x)
                        .hypot((e.point.y - n.point.y).hypot(e.point.z - n.point.z))
                        > comparison_tol
                })
            })
        {
            return Err(CurveSurfaceIntersectionError::NumericalFailure);
        }
        let result_status = match native.len() {
            0 => IntersectionStatus::NoIntersection,
            1 => IntersectionStatus::Unique,
            _ => IntersectionStatus::Ambiguous,
        };
        Ok(CurveSurfaceIntersectionResult { status: result_status, points: native })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use umlcad_v6_freeform_api::Point3 as CurvePoint3;

    fn plane() -> NurbsSurface3DDefinition {
        NurbsSurface3DDefinition::new(
            (1, 1),
            vec![
                umlcad_v6_nurbs_surface_api::Point3 { x: 0.0, y: 0.0, z: 0.0 },
                umlcad_v6_nurbs_surface_api::Point3 { x: 0.0, y: 1.0, z: 0.0 },
                umlcad_v6_nurbs_surface_api::Point3 { x: 1.0, y: 0.0, z: 0.0 },
                umlcad_v6_nurbs_surface_api::Point3 { x: 1.0, y: 1.0, z: 0.0 },
            ],
            vec![1.0; 4],
            (2, 2),
            vec![0.0, 0.0, 1.0, 1.0],
            vec![0.0, 0.0, 1.0, 1.0],
        )
    }

    fn curve() -> NurbsCurve3DDefinition {
        NurbsCurve3DDefinition::new(
            1,
            vec![
                CurvePoint3 { x: 0.25, y: 0.75, z: -1.0 },
                CurvePoint3 { x: 0.25, y: 0.75, z: 1.0 },
            ],
            vec![1.0, 1.0],
            vec![0.0, 0.0, 1.0, 1.0],
        )
    }

    #[test]
    fn native_line_surface_matches_semantic_point() {
        let backend = OcctBackend::new();
        let line = LineSegment3D {
            start: umlcad_v6_nurbs_surface_api::Point3 { x: 0.25, y: 0.75, z: -1.0 },
            end: umlcad_v6_nurbs_surface_api::Point3 { x: 0.25, y: 0.75, z: 1.0 },
        };
        let result = backend.intersect_line_segment_nurbs_surface(line, &plane(), 1e-10).unwrap();
        assert_eq!(result.status, IntersectionStatus::Unique);
        assert_eq!(result.points.len(), 1);
    }

    #[test]
    fn native_nurbs_curve_surface_matches_semantic_point() {
        let backend = OcctBackend::new();
        let result = backend.intersect_nurbs_curve_surface(&curve(), &plane(), 1e-10).unwrap();
        assert_eq!(result.status, IntersectionStatus::Unique);
        assert_eq!(result.points.len(), 1);
        assert!(result.points[0].point.z.abs() < 1e-8);
    }
}
