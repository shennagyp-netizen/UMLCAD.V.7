use umlcad_v6_nurbs_surface_api::{NurbsSurface3DDefinition, Point3};
use umlcad_v6_surface_operations_api::{
    intersect_planar_nurbs_surfaces, IntersectionStatus, SurfaceSurfaceIntersectionError,
    SurfaceSurfaceIntersectionResult, SurfaceSurfaceOperations,
};
use crate::{OcctBackend, OCCT_OK};

unsafe extern "C" {
    fn umlcad_occt_planar_surface_intersection(
        poles_a: *const f64, count_ua: u32, count_va: u32, weights_a: *const f64,
        knots_ua: *const f64, knot_count_ua: u32, knots_va: *const f64, knot_count_va: u32,
        degree_ua: u32, degree_va: u32, poles_b: *const f64, count_ub: u32, count_vb: u32,
        weights_b: *const f64, knots_ub: *const f64, knot_count_ub: u32, knots_vb: *const f64,
        knot_count_vb: u32, degree_ub: u32, degree_vb: u32, tolerance: f64,
        out_line_count: *mut u32, out_endpoints: *mut f64,
    ) -> i32;
}

impl SurfaceSurfaceOperations for OcctBackend {
    fn intersect_planar_nurbs_surfaces(
        &self,
        first: &NurbsSurface3DDefinition,
        second: &NurbsSurface3DDefinition,
        tolerance: f64,
    ) -> Result<SurfaceSurfaceIntersectionResult, SurfaceSurfaceIntersectionError> {
        let expected = intersect_planar_nurbs_surfaces(first, second, tolerance)?;
        let mut a = Vec::with_capacity(first.control_points.len() * 3);
        for p in &first.control_points { a.extend([p.x, p.y, p.z]); }
        let mut b = Vec::with_capacity(second.control_points.len() * 3);
        for p in &second.control_points { b.extend([p.x, p.y, p.z]); }
        let mut lines = 0u32;
        let mut endpoints = [0.0f64; 6];
        let st = unsafe {
            umlcad_occt_planar_surface_intersection(
                a.as_ptr(), first.count_u as u32, first.count_v as u32, first.weights.as_ptr(),
                first.knots_u.as_ptr(), first.knots_u.len() as u32, first.knots_v.as_ptr(),
                first.knots_v.len() as u32, first.degree_u as u32, first.degree_v as u32,
                b.as_ptr(), second.count_u as u32, second.count_v as u32, second.weights.as_ptr(),
                second.knots_u.as_ptr(), second.knots_u.len() as u32, second.knots_v.as_ptr(),
                second.knots_v.len() as u32, second.degree_u as u32, second.degree_v as u32,
                tolerance, &mut lines, endpoints.as_mut_ptr(),
            )
        };
        if st != OCCT_OK { return Err(SurfaceSurfaceIntersectionError::NumericalFailure); }
        match expected.status {
            IntersectionStatus::NoIntersection => {
                if lines != 0 { return Err(SurfaceSurfaceIntersectionError::NumericalFailure); }
            }
            IntersectionStatus::Unique => {
                if lines == 0 { return Err(SurfaceSurfaceIntersectionError::NumericalFailure); }
                let segment = expected.segments.first().ok_or(SurfaceSurfaceIntersectionError::NumericalFailure)?;
                let native_start = Point3 { x: endpoints[0], y: endpoints[1], z: endpoints[2] };
                let native_end = Point3 { x: endpoints[3], y: endpoints[4], z: endpoints[5] };
                let start_error = (segment.start.point.x - native_start.x)
                    .hypot((segment.start.point.y - native_start.y).hypot(segment.start.point.z - native_start.z));
                let end_error = (segment.end.point.x - native_end.x)
                    .hypot((segment.end.point.y - native_end.y).hypot(segment.end.point.z - native_end.z));
                let direct = start_error + end_error;
                let reversed = (segment.start.point.x - native_end.x)
                    .hypot((segment.start.point.y - native_end.y).hypot(segment.start.point.z - native_end.z))
                    + (segment.end.point.x - native_start.x)
                        .hypot((segment.end.point.y - native_start.y).hypot(segment.end.point.z - native_start.z));
                let comparison_tol = tolerance.max(1e-9) * 20.0;
                if direct > comparison_tol * 2.0 && reversed > comparison_tol * 2.0 {
                    return Err(SurfaceSurfaceIntersectionError::NumericalFailure);
                }
            }
            IntersectionStatus::Ambiguous => {
                if lines == 0 { return Err(SurfaceSurfaceIntersectionError::NumericalFailure); }
            }
        }
        Ok(expected)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn horizontal() -> NurbsSurface3DDefinition {
        NurbsSurface3DDefinition::new((1,1),vec![Point3{x:0.,y:0.,z:0.},Point3{x:0.,y:1.,z:0.},Point3{x:1.,y:0.,z:0.},Point3{x:1.,y:1.,z:0.}],vec![1.;4],(2,2),vec![0.,0.,1.,1.],vec![0.,0.,1.,1.])
    }
    fn vertical() -> NurbsSurface3DDefinition {
        NurbsSurface3DDefinition::new((1,1),vec![Point3{x:0.5,y:0.,z:-1.},Point3{x:0.5,y:0.,z:1.},Point3{x:0.5,y:1.,z:-1.},Point3{x:0.5,y:1.,z:1.}],vec![1.;4],(2,2),vec![0.,0.,1.,1.],vec![0.,0.,1.,1.])
    }
    #[test]
    fn native_segment_geometry_conforms() {
        let backend = OcctBackend::new();
        let result = backend.intersect_planar_nurbs_surfaces(&horizontal(), &vertical(), 1e-9).unwrap();
        assert_eq!(result.status, IntersectionStatus::Unique);
        assert_eq!(result.segments.len(), 1);
        assert!((result.segments[0].start.point.x - 0.5).abs() < 1e-9);
        assert!((result.segments[0].end.point.x - 0.5).abs() < 1e-9);
    }
}
