use umlcad_v6_geometry_api::{GeometryBackend, GeometryError};

pub mod validation;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vertex {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Triangle {
    pub a: u32,
    pub b: u32,
    pub c: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub triangles: Vec<Triangle>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TessellationOptions {
    pub linear_deflection: f64,
    pub angular_deflection_radians: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeshError {
    NonFinite,
    InvalidDeflection,
    IndexOutOfBounds,
    DegenerateTriangle,
    NonManifoldConnectivity,
}

impl TessellationOptions {
    pub fn validate(&self) -> Result<(), MeshError> {
        if !self.linear_deflection.is_finite() || !self.angular_deflection_radians.is_finite() {
            return Err(MeshError::NonFinite);
        }
        if self.linear_deflection <= 0.0 || self.angular_deflection_radians <= 0.0 {
            return Err(MeshError::InvalidDeflection);
        }
        Ok(())
    }
}

impl Mesh {
    pub fn validate(&self) -> Result<(), MeshError> {
        if self
            .vertices
            .iter()
            .any(|v| !v.x.is_finite() || !v.y.is_finite() || !v.z.is_finite())
        {
            return Err(MeshError::NonFinite);
        }

        let mut edge_use_counts = std::collections::BTreeMap::<(u32, u32), u8>::new();
        for triangle in &self.triangles {
            let indices = [triangle.a, triangle.b, triangle.c];
            if indices.iter().any(|index| *index as usize >= self.vertices.len()) {
                return Err(MeshError::IndexOutOfBounds);
            }
            if triangle.a == triangle.b || triangle.b == triangle.c || triangle.a == triangle.c {
                return Err(MeshError::DegenerateTriangle);
            }
            let area2 = validation::triangle_area2(self, triangle.a as usize, triangle.b as usize, triangle.c as usize)?;
            if !area2.is_finite() {
                return Err(MeshError::NonFinite);
            }
            if area2 == 0.0 {
                return Err(MeshError::DegenerateTriangle);
            }

            for (a, b) in [
                (triangle.a, triangle.b),
                (triangle.b, triangle.c),
                (triangle.c, triangle.a),
            ] {
                let edge = if a < b { (a, b) } else { (b, a) };
                let count = edge_use_counts.entry(edge).or_insert(0);
                *count = count.saturating_add(1);
                if *count > 2 {
                    return Err(MeshError::NonManifoldConnectivity);
                }
            }
        }
        Ok(())
    }
}

pub trait TessellationBackend: GeometryBackend {
    fn tessellate(
        &self,
        shape: &Self::Shape,
        options: TessellationOptions,
    ) -> Result<Mesh, GeometryError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn triangle_mesh() -> Mesh {
        Mesh {
            vertices: vec![
                Vertex { x: 0.0, y: 0.0, z: 0.0 },
                Vertex { x: 1.0, y: 0.0, z: 0.0 },
                Vertex { x: 0.0, y: 1.0, z: 0.0 },
                Vertex { x: 1.0, y: 1.0, z: 0.0 },
            ],
            triangles: vec![
                Triangle { a: 0, b: 1, c: 2 },
                Triangle { a: 1, b: 3, c: 2 },
            ],
        }
    }

    #[test]
    fn options_are_fail_closed() {
        assert!(TessellationOptions {
            linear_deflection: 0.1,
            angular_deflection_radians: 0.2,
        }
        .validate()
        .is_ok());
        assert_eq!(
            TessellationOptions {
                linear_deflection: 0.0,
                angular_deflection_radians: 0.2,
            }
            .validate(),
            Err(MeshError::InvalidDeflection)
        );
    }

    #[test]
    fn valid_triangle_mesh_passes_connectivity_and_area_validation() {
        assert!(triangle_mesh().validate().is_ok());
        assert_eq!(validation::triangle_area2(&triangle_mesh(), 0, 1, 2).unwrap(), 1.0);
    }

    #[test]
    fn mesh_rejects_invalid_triangle_indices() {
        let mesh = Mesh {
            vertices: vec![Vertex { x: 0.0, y: 0.0, z: 0.0 }],
            triangles: vec![Triangle { a: 0, b: 1, c: 2 }],
        };
        assert_eq!(mesh.validate(), Err(MeshError::IndexOutOfBounds));
    }

    #[test]
    fn mesh_rejects_zero_area_triangle() {
        let mesh = Mesh {
            vertices: vec![
                Vertex { x: 0.0, y: 0.0, z: 0.0 },
                Vertex { x: 1.0, y: 0.0, z: 0.0 },
                Vertex { x: 2.0, y: 0.0, z: 0.0 },
            ],
            triangles: vec![Triangle { a: 0, b: 1, c: 2 }],
        };
        assert_eq!(mesh.validate(), Err(MeshError::DegenerateTriangle));
    }

    #[test]
    fn mesh_rejects_non_manifold_edge() {
        let mesh = Mesh {
            vertices: vec![
                Vertex { x: 0.0, y: 0.0, z: 0.0 },
                Vertex { x: 1.0, y: 0.0, z: 0.0 },
                Vertex { x: 0.0, y: 1.0, z: 0.0 },
                Vertex { x: 0.0, y: -1.0, z: 0.0 },
                Vertex { x: 0.0, y: 0.0, z: 1.0 },
            ],
            triangles: vec![
                Triangle { a: 0, b: 1, c: 2 },
                Triangle { a: 1, b: 0, c: 3 },
                Triangle { a: 0, b: 1, c: 4 },
            ],
        };
        assert_eq!(mesh.validate(), Err(MeshError::NonManifoldConnectivity));
    }
}
