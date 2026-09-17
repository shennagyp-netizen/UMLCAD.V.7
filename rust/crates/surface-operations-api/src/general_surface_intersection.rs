use crate::{
    classify_nurbs_surface_pair_relation, intersect_planar_nurbs_surfaces,
    IntersectionStatus, NurbsSurface3DDefinition, NurbsSurfacePairRelation,
    Plane3D, SurfaceIntersectionEndpoint, SurfaceIntersectionSegment,
    SurfaceSurfaceIntersectionError, SurfaceSurfaceIntersectionResult,
};
use umlcad_v6_nurbs_surface_api::Point3;

/// Intersects two NURBS surfaces with a conservative general-family front end.
///
/// The positive-weight NURBS control-net convex-hull property makes a separated
/// control-net AABB a valid certificate that the represented surfaces cannot
/// intersect. The currently certified contact families are exact affine-planar
/// patch/patch intersection and unweighted bilinear-patch / planar-patch
/// intersection. Other potential contacts remain explicitly unsupported rather
/// than being converted into guessed geometry.
pub fn intersect_nurbs_surfaces(
    first: &NurbsSurface3DDefinition,
    second: &NurbsSurface3DDefinition,
    tolerance: f64,
) -> Result<SurfaceSurfaceIntersectionResult, SurfaceSurfaceIntersectionError> {
    if !tolerance.is_finite() || tolerance < 0.0 {
        return Err(SurfaceSurfaceIntersectionError::NonFinite);
    }
    let relation = classify_nurbs_surface_pair_relation(first, second, tolerance).map_err(|error| {
        match error {
            crate::NurbsSurfacePairRelationError::InvalidFirstSurface => {
                SurfaceSurfaceIntersectionError::InvalidSurface
            }
            crate::NurbsSurfacePairRelationError::InvalidSecondSurface => {
                SurfaceSurfaceIntersectionError::InvalidSurface
            }
            crate::NurbsSurfacePairRelationError::InvalidTolerance => {
                SurfaceSurfaceIntersectionError::NonFinite
            }
            crate::NurbsSurfacePairRelationError::NonFinite
            | crate::NurbsSurfacePairRelationError::NumericalFailure => {
                SurfaceSurfaceIntersectionError::NumericalFailure
            }
        }
    })?;

    match relation {
        NurbsSurfacePairRelation::DisjointCertified => Ok(SurfaceSurfaceIntersectionResult {
            status: IntersectionStatus::NoIntersection,
            segments: Vec::new(),
        }),
        NurbsSurfacePairRelation::PotentialContact => {
            if let Some(result) = intersect_bilinear_planar_pair(first, second, tolerance)? {
                return Ok(result);
            }
            intersect_planar_nurbs_surfaces(first, second, tolerance)
        }
    }
}

#[derive(Clone, Copy)]
struct BilinearPatch {
    p00: Point3,
    du: Point3,
    dv: Point3,
    duv: Point3,
    u0: f64,
    u1: f64,
    v0: f64,
    v1: f64,
}

fn bilinear_patch(
    surface: &NurbsSurface3DDefinition,
) -> Result<Option<BilinearPatch>, SurfaceSurfaceIntersectionError> {
    surface
        .validate()
        .map_err(|_| SurfaceSurfaceIntersectionError::InvalidSurface)?;
    if surface.degree_u != 1
        || surface.degree_v != 1
        || surface.count_u != 2
        || surface.count_v != 2
        || surface.weights.iter().any(|w| (*w - 1.0).abs() > 1e-14)
    {
        return Ok(None);
    }
    let p00 = surface.control_points[0];
    let p01 = surface.control_points[1];
    let p10 = surface.control_points[2];
    let p11 = surface.control_points[3];
    let du = sub(p10, p00);
    let dv = sub(p01, p00);
    let duv = sub(sub(p11, p10), dv);
    let ((u0, u1), (v0, v1)) = surface
        .parameter_domain()
        .map_err(|_| SurfaceSurfaceIntersectionError::InvalidSurface)?;
    if ![u0, u1, v0, v1]
        .iter()
        .all(|value| value.is_finite())
        || u1 <= u0
        || v1 <= v0
    {
        return Err(SurfaceSurfaceIntersectionError::InvalidSurface);
    }
    if [p00, p01, p10, p11]
        .into_iter()
        .flat_map(|p| [p.x, p.y, p.z])
        .any(|value| !value.is_finite())
    {
        return Err(SurfaceSurfaceIntersectionError::NumericalFailure);
    }
    Ok(Some(BilinearPatch { p00, du, dv, duv, u0, u1, v0, v1 }))
}

fn bilinear_point(p: BilinearPatch, s: f64, t: f64) -> Point3 {
    add(p.p00, add(scale(p.du, s), add(scale(p.dv, t), scale(p.duv, s * t))))
}

fn parameter_from_normalized(p: BilinearPatch, s: f64, t: f64) -> (f64, f64) {
    (p.u0 + s * (p.u1 - p.u0), p.v0 + t * (p.v1 - p.v0))
}

fn patch_is_planar(p: BilinearPatch, tolerance: f64) -> Result<bool, SurfaceSurfaceIntersectionError> {
    let n = cross(p.du, p.dv);
    let n_norm = norm(n);
    let scale = norm(p.du).max(norm(p.dv)).max(norm(p.duv)).max(1.0);
    if !n_norm.is_finite() || n_norm <= 1e-14 * scale * scale {
        return Err(SurfaceSurfaceIntersectionError::UnsupportedSurfaceFamily);
    }
    let plane = Plane3D { origin: p.p00, normal: n };
    let expected = tolerance.max(1e-12 * scale);
    let corner_distance = plane
        .signed_distance(bilinear_point(p, 1.0, 1.0))
        .abs();
    if !corner_distance.is_finite() {
        return Err(SurfaceSurfaceIntersectionError::NumericalFailure);
    }
    Ok(corner_distance <= expected)
}

fn plane_coefficients(
    p: BilinearPatch,
    plane: Plane3D,
) -> Result<[f64; 4], SurfaceSurfaceIntersectionError> {
    let d00 = plane.signed_distance(p.p00);
    let d10 = plane.signed_distance(add(p.p00, p.du));
    let d01 = plane.signed_distance(add(p.p00, p.dv));
    let d11 = plane.signed_distance(bilinear_point(p, 1.0, 1.0));
    let coefficients = [d00, d10 - d00, d01 - d00, d11 - d10 - d01 + d00];
    if coefficients.iter().any(|x| !x.is_finite()) {
        return Err(SurfaceSurfaceIntersectionError::NumericalFailure);
    }
    Ok(coefficients)
}

fn add_root(roots: &mut Vec<(f64, f64)>, s: f64, t: f64, tol: f64) {
    if s >= -tol && s <= 1.0 + tol && t >= -tol && t <= 1.0 + tol {
        let s = s.clamp(0.0, 1.0);
        let t = t.clamp(0.0, 1.0);
        if roots
            .iter()
            .all(|(rs, rt)| (rs - s).hypot(rt - t) > tol.max(1e-12) * 10.0)
        {
            roots.push((s, t));
        }
    }
}

fn boundary_roots(
    coefficients: [f64; 4],
    tolerance: f64,
) -> Result<Vec<(f64, f64)>, SurfaceSurfaceIntersectionError> {
    let [a, b, c, d] = coefficients;
    let scale = coefficients.iter().map(|x| x.abs()).fold(1.0, f64::max);
    let tol = tolerance.max(1e-12 * scale);
    if coefficients.iter().all(|x| x.abs() <= tol) {
        return Err(SurfaceSurfaceIntersectionError::CoincidentOrUnderdetermined);
    }
    let mut roots = Vec::new();
    let safe_div = |num: f64, den: f64| -> Option<f64> {
        if den.abs() <= tol { None } else { Some(num / den) }
    };
    if let Some(t) = safe_div(-a, c) {
        add_root(&mut roots, 0.0, t, tol);
    } else if a.abs() <= tol && (a + c).abs() <= tol {
        add_root(&mut roots, 0.0, 0.0, tol);
        add_root(&mut roots, 0.0, 1.0, tol);
    }
    if let Some(t) = safe_div(-(a + b), c + d) {
        add_root(&mut roots, 1.0, t, tol);
    } else if (a + b).abs() <= tol && (a + b + c + d).abs() <= tol {
        add_root(&mut roots, 1.0, 0.0, tol);
        add_root(&mut roots, 1.0, 1.0, tol);
    }
    if let Some(s) = safe_div(-a, b) {
        add_root(&mut roots, s, 0.0, tol);
    } else if a.abs() <= tol && (a + b).abs() <= tol {
        add_root(&mut roots, 0.0, 0.0, tol);
        add_root(&mut roots, 1.0, 0.0, tol);
    }
    if let Some(s) = safe_div(-(a + c), b + d) {
        add_root(&mut roots, s, 1.0, tol);
    } else if (a + c).abs() <= tol && (a + b + c + d).abs() <= tol {
        add_root(&mut roots, 0.0, 1.0, tol);
        add_root(&mut roots, 1.0, 1.0, tol);
    }
    roots.sort_by(|lhs, rhs| lhs.0.total_cmp(&rhs.0).then(lhs.1.total_cmp(&rhs.1)));
    Ok(roots)
}

fn intersect_bilinear_planar_pair(
    first: &NurbsSurface3DDefinition,
    second: &NurbsSurface3DDefinition,
    tolerance: f64,
) -> Result<Option<SurfaceSurfaceIntersectionResult>, SurfaceSurfaceIntersectionError> {
    let first_patch = match bilinear_patch(first)? {
        Some(patch) => patch,
        None => return Ok(None),
    };
    let second_patch = match bilinear_patch(second)? {
        Some(patch) => patch,
        None => return Ok(None),
    };
    let first_planar = patch_is_planar(first_patch, tolerance)?;
    let second_planar = patch_is_planar(second_patch, tolerance)?;
    if first_planar == second_planar {
        return Ok(None);
    }
    let (freeform, planar, freeform_is_first) = if !first_planar {
        (first_patch, second_patch, true)
    } else {
        (second_patch, first_patch, false)
    };
    let plane = Plane3D { origin: planar.p00, normal: cross(planar.du, planar.dv) };
    let coefficients = plane_coefficients(freeform, plane)?;
    let roots = boundary_roots(coefficients, tolerance)?;
    if roots.is_empty() {
        return Ok(Some(SurfaceSurfaceIntersectionResult {
            status: IntersectionStatus::NoIntersection,
            segments: Vec::new(),
        }));
    }
    if roots.len() != 2 {
        return Ok(Some(SurfaceSurfaceIntersectionResult {
            status: IntersectionStatus::Ambiguous,
            segments: Vec::new(),
        }));
    }
    let (s0, t0) = roots[0];
    let (s1, t1) = roots[1];
    let first_point = bilinear_point(freeform, s0, t0);
    let second_point = bilinear_point(freeform, s1, t1);
    if norm(sub(first_point, second_point)) <= tolerance.max(1e-12) {
        return Ok(Some(SurfaceSurfaceIntersectionResult {
            status: IntersectionStatus::Ambiguous,
            segments: Vec::new(),
        }));
    }
    let make_endpoint = |s: f64, t: f64, point: Point3| {
        let freeform_uv = parameter_from_normalized(freeform, s, t);
        let (planar_s, planar_t) = inverse_planar_parameters(planar, point)?;
        let planar_uv = parameter_from_normalized(planar, planar_s, planar_t);
        Ok::<SurfaceIntersectionEndpoint, SurfaceSurfaceIntersectionError>(if freeform_is_first {
            SurfaceIntersectionEndpoint { point, first_uv: freeform_uv, second_uv: planar_uv }
        } else {
            SurfaceIntersectionEndpoint { point, first_uv: planar_uv, second_uv: freeform_uv }
        })
    };
    let start = make_endpoint(s0, t0, first_point)?;
    let end = make_endpoint(s1, t1, second_point)?;
    Ok(Some(SurfaceSurfaceIntersectionResult {
        status: IntersectionStatus::Unique,
        segments: vec![SurfaceIntersectionSegment { start, end }],
    }))
}

fn inverse_planar_parameters(
    p: BilinearPatch,
    point: Point3,
) -> Result<(f64, f64), SurfaceSurfaceIntersectionError> {
    let r = sub(point, p.p00);
    let aa = dot(p.du, p.du);
    let ab = dot(p.du, p.dv);
    let bb = dot(p.dv, p.dv);
    let determinant = aa * bb - ab * ab;
    if !determinant.is_finite() || determinant.abs() <= 1e-14 * aa.max(bb).max(1.0) {
        return Err(SurfaceSurfaceIntersectionError::NumericalFailure);
    }
    let ru = dot(p.du, r);
    let rv = dot(p.dv, r);
    let s = (ru * bb - rv * ab) / determinant;
    let t = (rv * aa - ru * ab) / determinant;
    if !s.is_finite() || !t.is_finite() {
        return Err(SurfaceSurfaceIntersectionError::NumericalFailure);
    }
    Ok((s.clamp(0.0, 1.0), t.clamp(0.0, 1.0)))
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
fn cross(a: Point3, b: Point3) -> Point3 {
    Point3 { x: a.y * b.z - a.z * b.y, y: a.z * b.x - a.x * b.z, z: a.x * b.y - a.y * b.x }
}
fn norm(a: Point3) -> f64 { dot(a, a).sqrt() }

#[cfg(test)]
mod tests {
    use super::*;

    fn quadratic_patch(z: f64) -> NurbsSurface3DDefinition {
        let mut points = Vec::with_capacity(9);
        for i in 0..3 {
            for j in 0..3 {
                points.push(crate::Point3 { x: i as f64 * 0.5, y: j as f64 * 0.5, z });
            }
        }
        NurbsSurface3DDefinition::new(
            (2, 2),
            points,
            vec![1.0; 9],
            (3, 3),
            vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
            vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
        )
    }

    fn bilinear_freeform() -> NurbsSurface3DDefinition {
        NurbsSurface3DDefinition::new(
            (1, 1),
            vec![
                crate::Point3 { x: 0.0, y: 0.0, z: -0.25 },
                crate::Point3 { x: 0.0, y: 1.0, z: 0.75 },
                crate::Point3 { x: 1.0, y: 0.0, z: 0.75 },
                crate::Point3 { x: 1.0, y: 1.0, z: 0.75 },
            ],
            vec![1.0; 4],
            (2, 2),
            vec![2.0, 2.0, 4.0, 4.0],
            vec![10.0, 10.0, 20.0, 20.0],
        )
    }

    fn plane_patch(z: f64) -> NurbsSurface3DDefinition {
        NurbsSurface3DDefinition::new(
            (1, 1),
            vec![
                crate::Point3 { x: -0.25, y: -0.25, z },
                crate::Point3 { x: -0.25, y: 1.25, z },
                crate::Point3 { x: 1.25, y: -0.25, z },
                crate::Point3 { x: 1.25, y: 1.25, z },
            ],
            vec![1.0; 4],
            (2, 2),
            vec![-1.0, -1.0, 1.0, 1.0],
            vec![4.0, 4.0, 8.0, 8.0],
        )
    }

    #[test]
    fn separated_general_nurbs_surfaces_are_certified_disjoint() {
        let first = quadratic_patch(0.0);
        let second = quadratic_patch(10.0);
        let result = intersect_nurbs_surfaces(&first, &second, 1e-10).unwrap();
        assert_eq!(result.status, IntersectionStatus::NoIntersection);
        assert!(result.segments.is_empty());
    }

    #[test]
    fn potentially_contacting_non_planar_family_remains_explicitly_unsupported() {
        let mut second = quadratic_patch(0.0);
        second.control_points[4].z = 0.25;
        assert_eq!(
            intersect_nurbs_surfaces(&quadratic_patch(0.0), &second, 1e-10),
            Err(SurfaceSurfaceIntersectionError::UnsupportedSurfaceFamily)
        );
    }

    #[test]
    fn bilinear_freeform_planar_patch_intersection_is_traced() {
        let freeform = bilinear_freeform();
        let plane = plane_patch(0.5);
        let result = intersect_nurbs_surfaces(&freeform, &plane, 1e-12).unwrap();
        assert_eq!(result.status, IntersectionStatus::Unique);
        assert_eq!(result.segments.len(), 1);
        let segment = result.segments[0];
        assert!((segment.start.point.z - 0.5).abs() <= 1e-12);
        assert!((segment.end.point.z - 0.5).abs() <= 1e-12);
        let ((u0, u1), (v0, v1)) = plane.parameter_domain().unwrap();
        for uv in [segment.start.second_uv, segment.end.second_uv] {
            assert!(uv.0 >= u0 - 1e-12 && uv.0 <= u1 + 1e-12);
            assert!(uv.1 >= v0 - 1e-12 && uv.1 <= v1 + 1e-12);
        }
    }

    #[test]
    fn bilinear_freeform_trace_is_deterministic() {
        let first = intersect_nurbs_surfaces(&bilinear_freeform(), &plane_patch(0.5), 1e-12).unwrap();
        for _ in 0..1000 {
            let next = intersect_nurbs_surfaces(&bilinear_freeform(), &plane_patch(0.5), 1e-12).unwrap();
            assert_eq!(first, next);
        }
    }

    #[test]
    fn bilinear_freeform_disjoint_from_plane_has_no_intersection() {
        let result = intersect_nurbs_surfaces(&bilinear_freeform(), &plane_patch(2.0), 1e-12).unwrap();
        assert_eq!(result.status, IntersectionStatus::NoIntersection);
        assert!(result.segments.is_empty());
    }

    #[test]
    fn higher_degree_pair_remains_closed() {
        assert_eq!(
            intersect_nurbs_surfaces(&quadratic_patch(0.0), &plane_patch(0.0), 1e-12),
            Err(SurfaceSurfaceIntersectionError::UnsupportedSurfaceFamily)
        );
    }

    #[test]
    fn invalid_tolerance_fails_closed() {
        assert_eq!(
            intersect_nurbs_surfaces(&quadratic_patch(0.0), &quadratic_patch(1.0), -1.0),
            Err(SurfaceSurfaceIntersectionError::NonFinite)
        );
    }
}
