use super::{NativeShape, OcctBackend, OCCT_CONSTRUCTION_FAILED, OCCT_INTERNAL_ERROR, OCCT_INVALID_ARGUMENT, OCCT_NULL_SHAPE, OCCT_OK, OCCT_TRANSFORM_FAILED};
use umlcad_v6_geometry_api::GeometryError;
use umlcad_v6_mesh_api::{Mesh, TessellationBackend, TessellationOptions, Triangle, Vertex};

unsafe extern "C" {
    fn umlcad_occt_shape_tessellation(
        input: *const NativeShape,
        linear_deflection: f64,
        angular_deflection_radians: f64,
        out_vertices_xyz: *mut f64,
        vertex_capacity: u32,
        out_vertex_count: *mut u32,
        out_triangles_abc: *mut u32,
        triangle_capacity: u32,
        out_triangle_count: *mut u32,
    ) -> i32;
}

fn status_error(status: i32) -> GeometryError {
    match status {
        OCCT_INVALID_ARGUMENT => GeometryError::InvalidInput("invalid OCCT tessellation argument"),
        OCCT_NULL_SHAPE => GeometryError::InvalidInput("OCCT tessellation shape is null"),
        OCCT_CONSTRUCTION_FAILED => GeometryError::Unsupported("OCCT tessellation construction failed"),
        OCCT_TRANSFORM_FAILED => GeometryError::Unsupported("OCCT tessellation transform failed"),
        OCCT_INTERNAL_ERROR => GeometryError::Unsupported("OCCT tessellation internal failure"),
        _ => GeometryError::Unsupported("unknown OCCT tessellation status"),
    }
}

impl TessellationBackend for OcctBackend {
    fn tessellate(
        &self,
        shape: &Self::Shape,
        options: TessellationOptions,
    ) -> Result<Mesh, GeometryError> {
        options
            .validate()
            .map_err(|_| GeometryError::InvalidInput("invalid tessellation options"))?;

        let mut vertex_count = 0u32;
        let mut triangle_count = 0u32;
        let status = unsafe {
            umlcad_occt_shape_tessellation(
                shape.raw.as_ptr(),
                options.linear_deflection,
                options.angular_deflection_radians,
                std::ptr::null_mut(),
                0,
                &mut vertex_count,
                std::ptr::null_mut(),
                0,
                &mut triangle_count,
            )
        };
        if status != OCCT_INVALID_ARGUMENT && status != OCCT_OK {
            return Err(status_error(status));
        }
        if vertex_count == 0 || triangle_count == 0 {
            return Err(GeometryError::Unsupported("OCCT produced an empty tessellation"));
        }

        let mut vertices_xyz = vec![0.0f64; vertex_count as usize * 3];
        let mut triangles_abc = vec![0u32; triangle_count as usize * 3];
        let status = unsafe {
            umlcad_occt_shape_tessellation(
                shape.raw.as_ptr(),
                options.linear_deflection,
                options.angular_deflection_radians,
                vertices_xyz.as_mut_ptr(),
                vertex_count,
                &mut vertex_count,
                triangles_abc.as_mut_ptr(),
                triangle_count,
                &mut triangle_count,
            )
        };
        if status != OCCT_OK {
            return Err(status_error(status));
        }
        if vertex_count == 0 || triangle_count == 0 {
            return Err(GeometryError::Unsupported("OCCT produced an empty tessellation"));
        }

        let vertices = vertices_xyz[..vertex_count as usize * 3]
            .chunks_exact(3)
            .map(|chunk| Vertex { x: chunk[0], y: chunk[1], z: chunk[2] })
            .collect();
        let triangles = triangles_abc[..triangle_count as usize * 3]
            .chunks_exact(3)
            .map(|chunk| Triangle { a: chunk[0], b: chunk[1], c: chunk[2] })
            .collect();
        let mesh = Mesh { vertices, triangles };
        mesh.validate()
            .map_err(|_| GeometryError::Unsupported("OCCT tessellation failed semantic mesh validation"))?;
        Ok(mesh)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use umlcad_v6_geometry_api::{GeometryBackend, ToleranceContext};

    const OPTIONS: TessellationOptions = TessellationOptions {
        linear_deflection: 0.1,
        angular_deflection_radians: 0.2,
    };

    #[test]
    fn box_tessellation_is_valid_and_repeatable() {
        let backend = OcctBackend::new();
        let tolerance = ToleranceContext { modeling: 1e-9, validation: 1e-9 };
        let shape = backend.box_solid(10.0, 20.0, 30.0, tolerance).unwrap().shape;
        let first = backend.tessellate(&shape, OPTIONS).unwrap();
        let second = backend.tessellate(&shape, OPTIONS).unwrap();
        assert_eq!(first, second);
        assert!(!first.vertices.is_empty());
        assert!(!first.triangles.is_empty());
        assert!(first.validate().is_ok());
    }

    #[test]
    fn invalid_tessellation_options_fail_closed_before_native_call() {
        let backend = OcctBackend::new();
        let tolerance = ToleranceContext { modeling: 1e-9, validation: 1e-9 };
        let shape = backend.box_solid(10.0, 20.0, 30.0, tolerance).unwrap().shape;
        let err = backend.tessellate(
            &shape,
            TessellationOptions {
                linear_deflection: 0.0,
                angular_deflection_radians: 0.2,
            },
        );
        assert_eq!(err, Err(GeometryError::InvalidInput("invalid tessellation options")));
    }
}
