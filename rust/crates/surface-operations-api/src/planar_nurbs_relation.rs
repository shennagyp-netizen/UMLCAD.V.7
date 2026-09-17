use umlcad_v6_nurbs_surface_api::{NurbsSurface3DDefinition, Point3};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlanarNurbsSurfaceRelation {
    DisjointCertified,
    CoincidentWithinTolerance,
    Undetermined,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlanarNurbsSurfaceRelationError {
    NonFinite,
    InvalidSurface,
    DegeneratePlane,
    NumericalFailure,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Plane3D {
    pub origin: Point3,
    pub normal: Point3,
}

impl Plane3D {
    pub fn signed_distance(self, point: Point3) -> f64 {
        let n = self.normal.norm();
        if n == 0.0 {
            return f64::NAN;
        }
        self.normal.dot(Point3 {
            x: point.x - self.origin.x,
            y: point.y - self.origin.y,
            z: point.z - self.origin.z,
        }) / n
    }
}

pub fn classify_planar_nurbs_surface_relation(
    plane: Plane3D,
    surface: &NurbsSurface3DDefinition,
    tolerance: f64,
) -> Result<PlanarNurbsSurfaceRelation, PlanarNurbsSurfaceRelationError> {
    if !tolerance.is_finite() || tolerance < 0.0 {
        return Err(PlanarNurbsSurfaceRelationError::NonFinite);
    }
    surface
        .validate()
        .map_err(|_| PlanarNurbsSurfaceRelationError::InvalidSurface)?;
    if !plane.normal.norm().is_finite() || plane.normal.norm() == 0.0 {
        return Err(PlanarNurbsSurfaceRelationError::DegeneratePlane);
    }
    if [plane.origin.x, plane.origin.y, plane.origin.z]
        .iter()
        .any(|v| !v.is_finite())
    {
        return Err(PlanarNurbsSurfaceRelationError::NonFinite);
    }

    let distances: Vec<f64> = surface
        .control_points
        .iter()
        .map(|p| plane.signed_distance(*p))
        .collect();
    if distances.iter().any(|d| !d.is_finite()) {
        return Err(PlanarNurbsSurfaceRelationError::NumericalFailure);
    }
    let scale = surface
        .control_points
        .iter()
        .map(|p| p.norm())
        .fold(1.0, f64::max)
        .max(plane.origin.norm());
    let tol = tolerance.max(1e-12 * scale);

    if distances.iter().all(|d| *d > tol) || distances.iter().all(|d| *d < -tol) {
        Ok(PlanarNurbsSurfaceRelation::DisjointCertified)
    } else if distances.iter().all(|d| d.abs() <= tol) {
        Ok(PlanarNurbsSurfaceRelation::CoincidentWithinTolerance)
    } else {
        Ok(PlanarNurbsSurfaceRelation::Undetermined)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plane() -> Plane3D {
        Plane3D {
            origin: Point3 { x: 0.0, y: 0.0, z: 0.0 },
            normal: Point3 { x: 0.0, y: 0.0, z: 1.0 },
        }
    }

    fn patch(z: f64) -> NurbsSurface3DDefinition {
        NurbsSurface3DDefinition::new(
            (1, 1),
            vec![
                Point3 { x: 0.0, y: 0.0, z },
                Point3 { x: 0.0, y: 1.0, z },
                Point3 { x: 1.0, y: 0.0, z },
                Point3 { x: 1.0, y: 1.0, z },
            ],
            vec![1.0; 4],
            (2, 2),
            vec![0.0, 0.0, 1.0, 1.0],
            vec![0.0, 0.0, 1.0, 1.0],
        )
    }

    #[test]
    fn positive_control_net_certifies_disjointness() {
        assert_eq!(
            classify_planar_nurbs_surface_relation(plane(), &patch(1.0), 1e-10).unwrap(),
            PlanarNurbsSurfaceRelation::DisjointCertified
        );
    }

    #[test]
    fn coplanar_control_net_is_coincident() {
        assert_eq!(
            classify_planar_nurbs_surface_relation(plane(), &patch(0.0), 1e-10).unwrap(),
            PlanarNurbsSurfaceRelation::CoincidentWithinTolerance
        );
    }

    #[test]
    fn mixed_control_net_is_not_mistaken_for_an_intersection_certificate() {
        let mut s = patch(1.0);
        s.control_points[0].z = -1.0;
        assert_eq!(
            classify_planar_nurbs_surface_relation(plane(), &s, 1e-10).unwrap(),
            PlanarNurbsSurfaceRelation::Undetermined
        );
    }

    #[test]
    fn invalid_plane_is_rejected() {
        assert_eq!(
            classify_planar_nurbs_surface_relation(
                Plane3D {
                    origin: Point3 { x: 0.0, y: 0.0, z: 0.0 },
                    normal: Point3 { x: 0.0, y: 0.0, z: 0.0 },
                },
                &patch(1.0),
                1e-10,
            ),
            Err(PlanarNurbsSurfaceRelationError::DegeneratePlane)
        );
    }
}
