#[path = "trimmed_surface_backend.rs"]
mod trimmed_surface_backend;

use std::ptr::NonNull;

use crate::{NativeShape, OcctBackend, OcctShape, OCCT_OK};
use umlcad_v6_geometry_api::{
    GeometryBackend, GeometryError, GeometryEvidence, GeometryKind, GeometryResult, GeometryStatus,
    ToleranceContext,
};
use umlcad_v6_nurbs_surface_api::{
    NurbsSurface3DDefinition, NurbsSurfaceBackend, NurbsSurfaceDifferential,
    NurbsSurfaceDifferentialBackend, Point3,
};

unsafe extern "C" {
    fn umlcad_occt_nurbs_surface3d(
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
        face_tolerance: f64,
        out_shape: *mut *mut NativeShape,
    ) -> i32;
    fn umlcad_occt_nurbs_surface3d_differential(
        shape: *const NativeShape,
        u: f64,
        v: f64,
        out_values: *mut f64,
    ) -> i32;
    #[cfg(test)]
    fn umlcad_occt_surface_section_bbox(
        first: *const NativeShape,
        second: *const NativeShape,
        tolerance: f64,
        out_bounds: *mut f64,
        out_edge_count: *mut u32,
    ) -> i32;
}

impl NurbsSurfaceBackend for OcctBackend {
    fn nurbs_surface3d(
        &self,
        definition: &NurbsSurface3DDefinition,
        tolerance: ToleranceContext,
    ) -> Result<GeometryResult<Self::Shape>, GeometryError> {
        tolerance.validate()?;
        definition
            .validate()
            .map_err(|_| GeometryError::InvalidInput("invalid NURBS surface definition"))?;

        let mut poles_xyz = Vec::with_capacity(definition.control_points.len() * 3);
        for point in &definition.control_points {
            poles_xyz.extend([point.x, point.y, point.z]);
        }

        let mut raw = std::ptr::null_mut();
        let status = unsafe {
            umlcad_occt_nurbs_surface3d(
                poles_xyz.as_ptr(),
                definition.count_u as u32,
                definition.count_v as u32,
                definition.weights.as_ptr(),
                definition.knots_u.as_ptr(),
                definition.knots_u.len() as u32,
                definition.knots_v.as_ptr(),
                definition.knots_v.len() as u32,
                definition.degree_u as u32,
                definition.degree_v as u32,
                tolerance.modeling,
                &mut raw,
            )
        };
        if status != OCCT_OK {
            return Err(Self::status(status, "OCCT NURBS surface construction failed"));
        }

        let raw = NonNull::new(raw).ok_or(GeometryError::Unsupported("OCCT returned null"))?;
        Ok(GeometryResult {
            shape: OcctShape::from_raw(raw, GeometryKind::Surface),
            kind: GeometryKind::Surface,
            evidence: GeometryEvidence {
                status: GeometryStatus::Success,
                backend: self.backend_name(),
                tolerance,
                message: None,
            },
        })
    }
}

impl NurbsSurfaceDifferentialBackend for OcctBackend {
    fn nurbs_surface3d_differential_at(
        &self,
        shape: &Self::Shape,
        u: f64,
        v: f64,
        tolerance: ToleranceContext,
    ) -> Result<NurbsSurfaceDifferential, GeometryError> {
        tolerance.validate()?;
        if shape.kind != GeometryKind::Surface {
            return Err(GeometryError::InvalidInput(
                "NURBS differential requires a surface shape",
            ));
        }
        if !u.is_finite() || !v.is_finite() {
            return Err(GeometryError::InvalidInput(
                "NURBS parameters must be finite",
            ));
        }

        let mut values = [0.0_f64; 18];
        let status = unsafe {
            umlcad_occt_nurbs_surface3d_differential(
                shape.raw.as_ptr(),
                u,
                v,
                values.as_mut_ptr(),
            )
        };
        if status != OCCT_OK {
            return Err(Self::status(
                status,
                "OCCT NURBS surface differential evaluation failed",
            ));
        }
        if values.iter().any(|x| !x.is_finite()) {
            return Err(GeometryError::Unsupported(
                "OCCT returned non-finite NURBS differential values",
            ));
        }

        Ok(NurbsSurfaceDifferential {
            point: Point3 { x: values[0], y: values[1], z: values[2] },
            du: Point3 { x: values[3], y: values[4], z: values[5] },
            dv: Point3 { x: values[6], y: values[7], z: values[8] },
            duu: Point3 { x: values[9], y: values[10], z: values[11] },
            duv: Point3 { x: values[12], y: values[13], z: values[14] },
            dvv: Point3 { x: values[15], y: values[16], z: values[17] },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use umlcad_v6_geometry_api::GeometryBackend;
    use umlcad_v6_surface_operations_api::{intersect_nurbs_surfaces, IntersectionStatus};

    const TOLERANCE: ToleranceContext = ToleranceContext {
        modeling: 1e-9,
        validation: 1e-9,
    };

    fn bilinear() -> NurbsSurface3DDefinition {
        NurbsSurface3DDefinition::new(
            (1, 1),
            vec![
                Point3 { x: 0.0, y: 0.0, z: 0.0 },
                Point3 { x: 0.0, y: 1.0, z: 1.0 },
                Point3 { x: 1.0, y: 0.0, z: 1.0 },
                Point3 { x: 1.0, y: 1.0, z: 2.0 },
            ],
            vec![1.0; 4],
            (2, 2),
            vec![0.0, 0.0, 1.0, 1.0],
            vec![0.0, 0.0, 1.0, 1.0],
        )
    }

    fn freeform_intersection_patch() -> NurbsSurface3DDefinition {
        NurbsSurface3DDefinition::new(
            (1, 1),
            vec![
                Point3 { x: 0.0, y: 0.0, z: -0.25 },
                Point3 { x: 0.0, y: 1.0, z: 0.75 },
                Point3 { x: 1.0, y: 0.0, z: 0.75 },
                Point3 { x: 1.0, y: 1.0, z: 0.75 },
            ],
            vec![1.0; 4],
            (2, 2),
            vec![2.0, 2.0, 4.0, 4.0],
            vec![10.0, 10.0, 20.0, 20.0],
        )
    }

    fn plane_patch() -> NurbsSurface3DDefinition {
        NurbsSurface3DDefinition::new(
            (1, 1),
            vec![
                Point3 { x: -0.25, y: -0.25, z: 0.5 },
                Point3 { x: -0.25, y: 1.25, z: 0.5 },
                Point3 { x: 1.25, y: -0.25, z: 0.5 },
                Point3 { x: 1.25, y: 1.25, z: 0.5 },
            ],
            vec![1.0; 4],
            (2, 2),
            vec![-1.0, -1.0, 1.0, 1.0],
            vec![4.0, 4.0, 8.0, 8.0],
        )
    }

    fn assert_close(a: Point3, b: Point3, tol: f64) {
        assert!(
            (a.x - b.x).abs() <= tol
                && (a.y - b.y).abs() <= tol
                && (a.z - b.z).abs() <= tol,
            "left={a:?} right={b:?}"
        );
    }

    #[test]
    fn bilinear_surface_is_a_valid_surface() {
        let backend = OcctBackend::new();
        let result = backend.nurbs_surface3d(&bilinear(), TOLERANCE).unwrap();
        assert_eq!(result.kind, GeometryKind::Surface);
        assert!(backend.validate(&result.shape, TOLERANCE).unwrap().valid);
        assert_eq!(backend.topology_counts(&result.shape, TOLERANCE).unwrap().faces, 1);
    }

    #[test]
    fn rational_surface_is_deterministic() {
        let backend = OcctBackend::new();
        let mut definition = bilinear();
        definition.weights[3] = 2.0;
        let first = backend.nurbs_surface3d(&definition, TOLERANCE).unwrap().shape;
        let second = backend.nurbs_surface3d(&definition, TOLERANCE).unwrap().shape;
        assert_eq!(
            backend.bounding_box(&first, TOLERANCE).unwrap(),
            backend.bounding_box(&second, TOLERANCE).unwrap()
        );
        assert_eq!(
            backend.topology_counts(&first, TOLERANCE).unwrap(),
            backend.topology_counts(&second, TOLERANCE).unwrap()
        );
    }

    #[test]
    fn invalid_surface_definition_fails_before_native_construction() {
        let backend = OcctBackend::new();
        let mut definition = bilinear();
        definition.weights[0] = 0.0;
        assert!(matches!(
            backend.nurbs_surface3d(&definition, TOLERANCE),
            Err(GeometryError::InvalidInput("invalid NURBS surface definition"))
        ));
    }

    #[test]
    fn native_differential_matches_exact_semantic_differential() {
        let backend = OcctBackend::new();
        let definition = bilinear();
        let shape = backend.nurbs_surface3d(&definition, TOLERANCE).unwrap().shape;
        let expected = definition.differential_at(0.25, 0.75).unwrap();
        let actual = backend
            .nurbs_surface3d_differential_at(&shape, 0.25, 0.75, TOLERANCE)
            .unwrap();

        assert_close(actual.point, expected.point, 1e-11);
        assert_close(actual.du, expected.du, 1e-11);
        assert_close(actual.dv, expected.dv, 1e-11);
        assert_close(actual.duu, expected.duu, 1e-11);
        assert_close(actual.duv, expected.duv, 1e-11);
        assert_close(actual.dvv, expected.dvv, 1e-11);
        assert_close(actual.normal().unwrap(), expected.normal().unwrap(), 1e-11);
    }

    #[test]
    fn native_differential_rejects_non_finite_parameters() {
        let backend = OcctBackend::new();
        let shape = backend.nurbs_surface3d(&bilinear(), TOLERANCE).unwrap().shape;
        assert!(matches!(
            backend.nurbs_surface3d_differential_at(&shape, f64::NAN, 0.5, TOLERANCE),
            Err(GeometryError::InvalidInput(_))
        ));
    }

    #[test]
    fn native_surface_section_conforms_to_semantic_endpoint_envelope() {
        let semantic = intersect_nurbs_surfaces(&freeform_intersection_patch(), &plane_patch(), 1e-10).unwrap();
        assert_eq!(semantic.status, IntersectionStatus::Unique);
        assert_eq!(semantic.segments.len(), 1);
        let segment = semantic.segments[0];

        let backend = OcctBackend::new();
        let first = backend
            .nurbs_surface3d(&freeform_intersection_patch(), TOLERANCE)
            .unwrap()
            .shape;
        let second = backend.nurbs_surface3d(&plane_patch(), TOLERANCE).unwrap().shape;
        let mut bounds = [0.0_f64; 6];
        let mut edge_count = 0_u32;
        let status = unsafe {
            umlcad_occt_surface_section_bbox(
                first.raw.as_ptr(),
                second.raw.as_ptr(),
                TOLERANCE.modeling,
                bounds.as_mut_ptr(),
                &mut edge_count,
            )
        };
        assert_eq!(status, OCCT_OK);
        assert!(edge_count > 0);
        for endpoint in [segment.start.point, segment.end.point] {
            assert!(endpoint.x >= bounds[0] - 1e-8 && endpoint.x <= bounds[3] + 1e-8);
            assert!(endpoint.y >= bounds[1] - 1e-8 && endpoint.y <= bounds[4] + 1e-8);
            assert!(endpoint.z >= bounds[2] - 1e-8 && endpoint.z <= bounds[5] + 1e-8);
        }
    }
}
