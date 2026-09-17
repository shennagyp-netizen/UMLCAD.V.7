use super::*;

fn sub(a: Point3, b: Point3) -> Point3 {
    Point3 { x: a.x - b.x, y: a.y - b.y, z: a.z - b.z }
}

fn affine(a: Point3, b: Point3, t: f64) -> Point3 {
    Point3 { x: a.x + b.x * t, y: a.y + b.y * t, z: a.z + b.z * t }
}

fn is_exact_affine_planar_patch(surface: &NurbsSurface3DDefinition) -> Option<PlanarPatch> {
    if surface.degree_u != 1 || surface.degree_v != 1 || surface.count_u != 2 || surface.count_v != 2 || surface.weights.iter().any(|w| *w != 1.0) {
        return None;
    }
    let ((u0, u1), (v0, v1)) = surface.parameter_domain().ok()?;
    let p00 = surface.control_points[0];
    let p01 = surface.control_points[1];
    let p10 = surface.control_points[2];
    let p11 = surface.control_points[3];
    let du = sub(p10, p00);
    let dv = sub(p01, p00);
    let closure = sub(sub(p11, p00), Point3 { x: du.x + dv.x, y: du.y + dv.y, z: du.z + dv.z });
    if closure.norm() > 1e-12 * du.norm().max(dv.norm()).max(1.0) {
        return None;
    }
    let normal = du.cross(dv);
    if normal.norm() == 0.0 {
        return None;
    }
    Some(PlanarPatch { origin: p00, du, dv, normal, u0, u1, v0, v1 })
}

struct PlanarPatch {
    origin: Point3,
    du: Point3,
    dv: Point3,
    normal: Point3,
    u0: f64,
    u1: f64,
    v0: f64,
    v1: f64,
}

pub(crate) fn classify_planar_line_surface(
    line: LineSegment3D,
    surface: &NurbsSurface3DDefinition,
    tolerance: f64,
) -> Result<Option<LineSurfaceIntersectionResult>, LineSurfaceIntersectionError> {
    let Some(patch) = is_exact_affine_planar_patch(surface) else {
        return Ok(None);
    };
    let direction = sub(line.end, line.start);
    let normal_norm = patch.normal.norm();
    let direction_norm = direction.norm();
    let scale = direction_norm.max(patch.origin.norm()).max(patch.du.norm()).max(patch.dv.norm()).max(1.0);
    let tol_dist = tolerance.max(1e-12 * scale);
    let denominator = patch.normal.dot(direction);
    let parallel_bound = tol_dist * normal_norm * direction_norm;
    let offset = patch.normal.dot(sub(line.start, patch.origin));
    if denominator.abs() <= parallel_bound {
        if offset.abs() <= tol_dist * normal_norm {
            return Err(LineSurfaceIntersectionError::CoincidentOrUnderdetermined);
        }
        return Ok(Some(LineSurfaceIntersectionResult { status: IntersectionStatus::NoIntersection, points: Vec::new() }));
    }
    let t = -offset / denominator;
    if !t.is_finite() {
        return Err(LineSurfaceIntersectionError::NumericalFailure);
    }
    let t_tol = tol_dist / direction_norm;
    if t < -t_tol || t > 1.0 + t_tol {
        return Ok(Some(LineSurfaceIntersectionResult { status: IntersectionStatus::NoIntersection, points: Vec::new() }));
    }
    let t = t.clamp(0.0, 1.0);
    let point = affine(line.start, direction, t);
    let rhs = sub(point, patch.origin);
    let aa = patch.du.dot(patch.du);
    let ab = patch.du.dot(patch.dv);
    let bb = patch.dv.dot(patch.dv);
    let det = aa * bb - ab * ab;
    if !det.is_finite() || det <= tol_dist * tol_dist {
        return Err(LineSurfaceIntersectionError::NumericalFailure);
    }
    let a = rhs.dot(patch.du);
    let b = rhs.dot(patch.dv);
    let alpha = (a * bb - b * ab) / det;
    let beta = (b * aa - a * ab) / det;
    if !alpha.is_finite() || !beta.is_finite() {
        return Err(LineSurfaceIntersectionError::NumericalFailure);
    }
    let param_tol_u = tol_dist / patch.du.norm().max(1.0);
    let param_tol_v = tol_dist / patch.dv.norm().max(1.0);
    if alpha < -param_tol_u || alpha > 1.0 + param_tol_u || beta < -param_tol_v || beta > 1.0 + param_tol_v {
        return Ok(Some(LineSurfaceIntersectionResult { status: IntersectionStatus::NoIntersection, points: Vec::new() }));
    }
    let alpha = alpha.clamp(0.0, 1.0);
    let beta = beta.clamp(0.0, 1.0);
    Ok(Some(LineSurfaceIntersectionResult {
        status: IntersectionStatus::Unique,
        points: vec![LineSurfaceIntersectionPoint {
            line_parameter: t,
            u: patch.u0 + alpha * (patch.u1 - patch.u0),
            v: patch.v0 + beta * (patch.v1 - patch.v0),
            point,
        }],
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn plane_with_domain() -> NurbsSurface3DDefinition {
        NurbsSurface3DDefinition::new(
            (1, 1),
            vec![
                Point3 { x: 0.0, y: 0.0, z: 0.0 },
                Point3 { x: 0.0, y: 2.0, z: 0.0 },
                Point3 { x: 3.0, y: 0.0, z: 0.0 },
                Point3 { x: 3.0, y: 2.0, z: 0.0 },
            ],
            vec![1.0; 4],
            (2, 2),
            vec![2.0, 2.0, 4.0, 4.0],
            vec![10.0, 10.0, 20.0, 20.0],
        )
    }
    #[test]
    fn boundary_corner_intersection_is_unique() {
        let line = LineSegment3D { start: Point3 { x: -1.0, y: -1.0, z: -1.0 }, end: Point3 { x: 0.0, y: 0.0, z: 0.0 } };
        let result = classify_planar_line_surface(line, &plane_with_domain(), 1e-10).unwrap().unwrap();
        assert_eq!(result.status, IntersectionStatus::Unique);
        assert!((result.points[0].line_parameter - 1.0).abs() < 1e-12);
        assert!((result.points[0].u - 2.0).abs() < 1e-12);
        assert!((result.points[0].v - 10.0).abs() < 1e-12);
    }
    #[test]
    fn parallel_disjoint_line_has_no_intersection() {
        let line = LineSegment3D { start: Point3 { x: 0.5, y: 0.5, z: 1.0 }, end: Point3 { x: 2.5, y: 0.5, z: 1.0 } };
        let result = classify_planar_line_surface(line, &plane_with_domain(), 1e-10).unwrap().unwrap();
        assert_eq!(result.status, IntersectionStatus::NoIntersection);
    }
    #[test]
    fn coplanar_line_is_underdetermined() {
        let line = LineSegment3D { start: Point3 { x: -1.0, y: 0.5, z: 0.0 }, end: Point3 { x: 4.0, y: 0.5, z: 0.0 } };
        assert_eq!(classify_planar_line_surface(line, &plane_with_domain(), 1e-10), Err(LineSurfaceIntersectionError::CoincidentOrUnderdetermined))
    }
}
