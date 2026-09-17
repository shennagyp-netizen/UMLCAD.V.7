use umlcad_v6_nurbs_surface_api::{NurbsSurface3DDefinition, Point3};
use umlcad_v6_surface_operations_api::{
    closest_point_on_planar_nurbs_surface, PointSurfaceClosestPointError,
    PointSurfaceClosestPointResult, PointSurfaceOperations,
};

use crate::{OcctBackend, OCCT_OK};

unsafe extern "C" {
    fn umlcad_occt_point_surface_closest_point(
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
        point_xyz: *const f64,
        tolerance: f64,
        out_uv: *mut f64,
        out_point_xyz: *mut f64,
        out_distance: *mut f64,
    ) -> i32;
}

impl PointSurfaceOperations for OcctBackend {
    fn closest_point_on_planar_nurbs_surface(
        &self,
        point: Point3,
        surface: &NurbsSurface3DDefinition,
        tolerance: f64,
    ) -> Result<PointSurfaceClosestPointResult, PointSurfaceClosestPointError> {
        let expected = closest_point_on_planar_nurbs_surface(point, surface, tolerance)?;
        let mut poles_xyz = Vec::with_capacity(surface.control_points.len() * 3);
        for p in &surface.control_points {
            poles_xyz.extend([p.x, p.y, p.z]);
        }
        let input = [point.x, point.y, point.z];
        let mut uv = [0.0f64; 2];
        let mut native_point = [0.0f64; 3];
        let mut native_distance = 0.0f64;
        let status = unsafe {
            umlcad_occt_point_surface_closest_point(
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
                input.as_ptr(),
                tolerance,
                uv.as_mut_ptr(),
                native_point.as_mut_ptr(),
                &mut native_distance,
            )
        };
        if status != OCCT_OK {
            return Err(PointSurfaceClosestPointError::NumericalFailure);
        }
        if !uv.iter().all(|v| v.is_finite())
            || !native_point.iter().all(|v| v.is_finite())
            || !native_distance.is_finite()
        {
            return Err(PointSurfaceClosestPointError::NumericalFailure);
        }
        let expected_point = expected
            .closest
            .ok_or(PointSurfaceClosestPointError::NumericalFailure)?;
        let point_error = Point3 {
            x: expected_point.point.x - native_point[0],
            y: expected_point.point.y - native_point[1],
            z: expected_point.point.z - native_point[2],
        }
        .norm();
        let uv_error = (expected_point.surface_uv.0 - uv[0])
            .hypot(expected_point.surface_uv.1 - uv[1]);
        let distance_error = (expected_point.distance - native_distance).abs();
        let comparison_tol = tolerance.max(1e-9) * 20.0;
        if point_error > comparison_tol
            || uv_error > comparison_tol
            || distance_error > comparison_tol
        {
            return Err(PointSurfaceClosestPointError::NumericalFailure);
        }
        Ok(expected)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_patch() -> NurbsSurface3DDefinition {
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
    fn native_interior_projection_conforms() {
        let backend = OcctBackend::new();
        let result = backend
            .closest_point_on_planar_nurbs_surface(
                Point3 { x: 0.25, y: 0.75, z: 2.0 },
                &unit_patch(),
                1e-9,
            )
            .unwrap();
        let closest = result.closest.unwrap();
        assert!((closest.distance - 2.0).abs() < 1e-9);
        assert!((closest.surface_uv.0 - 0.25).abs() < 1e-9);
        assert!((closest.surface_uv.1 - 0.75).abs() < 1e-9);
    }

    #[test]
    fn native_boundary_projection_conforms() {
        let backend = OcctBackend::new();
        let result = backend
            .closest_point_on_planar_nurbs_surface(
                Point3 { x: 2.0, y: 0.5, z: 0.0 },
                &unit_patch(),
                1e-9,
            )
            .unwrap();
        let closest = result.closest.unwrap();
        assert!((closest.point.x - 1.0).abs() < 1e-9);
        assert!((closest.point.y - 0.5).abs() < 1e-9);
        assert!((closest.distance - 1.0).abs() < 1e-9);
    }
}
