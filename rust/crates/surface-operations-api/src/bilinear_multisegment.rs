use crate::{
    bilinear_branch_pairing::pair_bilinear_roots,
    IntersectionStatus, NurbsSurface3DDefinition, Plane3D, SurfaceIntersectionEndpoint,
    SurfaceIntersectionSegment, SurfaceSurfaceIntersectionError, SurfaceSurfaceIntersectionResult,
};
use umlcad_v6_nurbs_surface_api::Point3;

#[derive(Clone, Copy)]
struct Patch {
    p00: Point3,
    du: Point3,
    dv: Point3,
    duv: Point3,
    u0: f64,
    u1: f64,
    v0: f64,
    v1: f64,
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
fn dot(a: Point3, b: Point3) -> f64 {
    a.x * b.x + a.y * b.y + a.z * b.z
}
fn cross(a: Point3, b: Point3) -> Point3 {
    Point3 { x: a.y * b.z - a.z * b.y, y: a.z * b.x - a.x * b.z, z: a.x * b.y - a.y * b.x }
}
fn norm(a: Point3) -> f64 {
    dot(a, a).sqrt()
}

fn patch(surface: &NurbsSurface3DDefinition) -> Result<Option<Patch>, SurfaceSurfaceIntersectionError> {
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
    let ((u0, u1), (v0, v1)) = surface
        .parameter_domain()
        .map_err(|_| SurfaceSurfaceIntersectionError::InvalidSurface)?;
    if u1 <= u0 || v1 <= v0 {
        return Err(SurfaceSurfaceIntersectionError::InvalidSurface);
    }
    if [p00, p01, p10, p11]
        .into_iter()
        .flat_map(|p| [p.x, p.y, p.z])
        .any(|x| !x.is_finite())
    {
        return Err(SurfaceSurfaceIntersectionError::NumericalFailure);
    }
    Ok(Some(Patch {
        p00,
        du: sub(p10, p00),
        dv: sub(p01, p00),
        duv: sub(sub(p11, p10), sub(p01, p00)),
        u0,
        u1,
        v0,
        v1,
    }))
}

fn point_at(p: Patch, s: f64, t: f64) -> Point3 {
    add(p.p00, add(scale(p.du, s), add(scale(p.dv, t), scale(p.duv, s * t))))
}
fn uv(p: Patch, s: f64, t: f64) -> (f64, f64) {
    (p.u0 + s * (p.u1 - p.u0), p.v0 + t * (p.v1 - p.v0))
}
fn planar(p: Patch, tolerance: f64) -> Result<bool, SurfaceSurfaceIntersectionError> {
    let n = cross(p.du, p.dv);
    let n_norm = norm(n);
    let scale = norm(p.du).max(norm(p.dv)).max(norm(p.duv)).max(1.0);
    if !n_norm.is_finite() || n_norm <= 1e-14 * scale * scale {
        return Err(SurfaceSurfaceIntersectionError::UnsupportedSurfaceFamily);
    }
    let plane = Plane3D { origin: p.p00, normal: n };
    let distance = plane.signed_distance(point_at(p, 1.0, 1.0)).abs();
    if !distance.is_finite() {
        return Err(SurfaceSurfaceIntersectionError::NumericalFailure);
    }
    Ok(distance <= tolerance.max(1e-12 * scale))
}
fn plane_coefficients(p: Patch, plane: Plane3D) -> Result<[f64; 4], SurfaceSurfaceIntersectionError> {
    let d00 = plane.signed_distance(p.p00);
    let d10 = plane.signed_distance(add(p.p00, p.du));
    let d01 = plane.signed_distance(add(p.p00, p.dv));
    let d11 = plane.signed_distance(point_at(p, 1.0, 1.0));
    let c = [d00, d10 - d00, d01 - d00, d11 - d10 - d01 + d00];
    if c.iter().any(|x| !x.is_finite()) {
        return Err(SurfaceSurfaceIntersectionError::NumericalFailure);
    }
    Ok(c)
}
fn add_root(roots: &mut Vec<(f64, f64)>, s: f64, t: f64, tol: f64) {
    if s >= -tol && s <= 1.0 + tol && t >= -tol && t <= 1.0 + tol {
        let s = s.clamp(0.0, 1.0);
        let t = t.clamp(0.0, 1.0);
        if roots.iter().all(|(rs, rt)| (rs - s).hypot(rt - t) > tol.max(1e-12) * 10.0) {
            roots.push((s, t));
        }
    }
}
fn boundary_roots(c: [f64; 4], tolerance: f64) -> Result<Vec<(f64, f64)>, SurfaceSurfaceIntersectionError> {
    let [a, b, c, d] = c;
    let scale = [a, b, c, d].into_iter().map(f64::abs).fold(1.0, f64::max);
    let tol = tolerance.max(1e-12 * scale);
    if [a, b, c, d].into_iter().all(|x| x.abs() <= tol) {
        return Err(SurfaceSurfaceIntersectionError::CoincidentOrUnderdetermined);
    }
    let mut roots = Vec::new();
    let div = |n: f64, q: f64| if q.abs() <= tol { None } else { Some(n / q) };
    if let Some(t) = div(-a, c) {
        add_root(&mut roots, 0.0, t, tol);
    } else if a.abs() <= tol && (a + c).abs() <= tol {
        add_root(&mut roots, 0.0, 0.0, tol);
        add_root(&mut roots, 0.0, 1.0, tol);
    }
    if let Some(t) = div(-(a + b), c + d) {
        add_root(&mut roots, 1.0, t, tol);
    } else if (a + b).abs() <= tol && (a + b + c + d).abs() <= tol {
        add_root(&mut roots, 1.0, 0.0, tol);
        add_root(&mut roots, 1.0, 1.0, tol);
    }
    if let Some(s) = div(-a, b) {
        add_root(&mut roots, s, 0.0, tol);
    } else if a.abs() <= tol && (a + b).abs() <= tol {
        add_root(&mut roots, 0.0, 0.0, tol);
        add_root(&mut roots, 1.0, 0.0, tol);
    }
    if let Some(s) = div(-(a + c), b + d) {
        add_root(&mut roots, s, 1.0, tol);
    } else if (a + c).abs() <= tol && (a + b + c + d).abs() <= tol {
        add_root(&mut roots, 0.0, 1.0, tol);
        add_root(&mut roots, 1.0, 1.0, tol);
    }
    roots.sort_by(|x, y| x.0.total_cmp(&y.0).then(x.1.total_cmp(&y.1)));
    Ok(roots)
}
fn inverse_planar_parameters(p: Patch, point: Point3) -> Result<(f64, f64), SurfaceSurfaceIntersectionError> {
    let r = sub(point, p.p00);
    let aa = dot(p.du, p.du);
    let ab = dot(p.du, p.dv);
    let bb = dot(p.dv, p.dv);
    let det = aa * bb - ab * ab;
    if !det.is_finite() || det.abs() <= 1e-14 * aa.max(bb).max(1.0) {
        return Err(SurfaceSurfaceIntersectionError::NumericalFailure);
    }
    let ru = dot(p.du, r);
    let rv = dot(p.dv, r);
    let s = (ru * bb - rv * ab) / det;
    let t = (rv * aa - ru * ab) / det;
    if !s.is_finite() || !t.is_finite() {
        return Err(SurfaceSurfaceIntersectionError::NumericalFailure);
    }
    Ok((s.clamp(0.0, 1.0), t.clamp(0.0, 1.0)))
}
fn endpoint(
    freeform: Patch,
    planar: Patch,
    freeform_first: bool,
    s: f64,
    t: f64,
) -> Result<SurfaceIntersectionEndpoint, SurfaceSurfaceIntersectionError> {
    let point = point_at(freeform, s, t);
    let freeform_uv = uv(freeform, s, t);
    let (ps, pt) = inverse_planar_parameters(planar, point)?;
    let planar_uv = uv(planar, ps, pt);
    Ok(if freeform_first {
        SurfaceIntersectionEndpoint { point, first_uv: freeform_uv, second_uv: planar_uv }
    } else {
        SurfaceIntersectionEndpoint { point, first_uv: planar_uv, second_uv: freeform_uv }
    })
}

pub fn try_intersect_bilinear_planar_multisegment(
    first: &NurbsSurface3DDefinition,
    second: &NurbsSurface3DDefinition,
    tolerance: f64,
) -> Result<Option<SurfaceSurfaceIntersectionResult>, SurfaceSurfaceIntersectionError> {
    if !tolerance.is_finite() || tolerance < 0.0 {
        return Err(SurfaceSurfaceIntersectionError::NonFinite);
    }
    let a = match patch(first)? { Some(p) => p, None => return Ok(None) };
    let b = match patch(second)? { Some(p) => p, None => return Ok(None) };
    let ap = planar(a, tolerance)?;
    let bp = planar(b, tolerance)?;
    if ap == bp {
        return Ok(None);
    }
    let (freeform, plane_patch, freeform_first) = if !ap { (a, b, true) } else { (b, a, false) };
    let coefficients = plane_coefficients(freeform, Plane3D { origin: plane_patch.p00, normal: cross(plane_patch.du, plane_patch.dv) })?;
    let roots = boundary_roots(coefficients, tolerance)?;
    match roots.len() {
        0 => Ok(Some(SurfaceSurfaceIntersectionResult { status: IntersectionStatus::NoIntersection, segments: Vec::new() })),
        1 | 3 => Ok(Some(SurfaceSurfaceIntersectionResult { status: IntersectionStatus::Ambiguous, segments: Vec::new() })),
        2 | 4 => {
            let pairs = pair_bilinear_roots(&roots, coefficients, tolerance)?;
            let mut segments = Vec::new();
            for pair in pairs {
                let start = endpoint(freeform, plane_patch, freeform_first, pair.0.0, pair.0.1)?;
                let end = endpoint(freeform, plane_patch, freeform_first, pair.1.0, pair.1.1)?;
                if norm(sub(start.point, end.point)) <= tolerance.max(1e-12) {
                    return Ok(Some(SurfaceSurfaceIntersectionResult { status: IntersectionStatus::Ambiguous, segments: Vec::new() }));
                }
                segments.push(SurfaceIntersectionSegment { start, end });
            }
            Ok(Some(SurfaceSurfaceIntersectionResult { status: IntersectionStatus::Unique, segments }))
        }
        _ => Ok(Some(SurfaceSurfaceIntersectionResult { status: IntersectionStatus::Ambiguous, segments: Vec::new() })),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn freeform() -> NurbsSurface3DDefinition {
        NurbsSurface3DDefinition::new((1, 1), vec![
            Point3 { x: -1.0, y: -1.0, z: 0.01 }, Point3 { x: -1.0, y: 1.0, z: -0.19 },
            Point3 { x: 1.0, y: -1.0, z: -0.79 }, Point3 { x: 1.0, y: 1.0, z: 0.01 },
        ], vec![1.0; 4], (2, 2), vec![0.0, 0.0, 1.0, 1.0], vec![0.0, 0.0, 1.0, 1.0])
    }
    fn plane() -> NurbsSurface3DDefinition {
        NurbsSurface3DDefinition::new((1, 1), vec![
            Point3 { x: -2.0, y: -2.0, z: 0.0 }, Point3 { x: -2.0, y: 2.0, z: 0.0 },
            Point3 { x: 2.0, y: -2.0, z: 0.0 }, Point3 { x: 2.0, y: 2.0, z: 0.0 },
        ], vec![1.0; 4], (2, 2), vec![0.0, 0.0, 1.0, 1.0], vec![0.0, 0.0, 1.0, 1.0])
    }
    #[test]
    fn four_boundary_roots_form_two_non_crossing_segments() {
        let result = try_intersect_bilinear_planar_multisegment(&freeform(), &plane(), 1e-12).unwrap().unwrap();
        assert_eq!(result.status, IntersectionStatus::Unique);
        assert_eq!(result.segments.len(), 2);
        assert!(result.segments[0].start.point.x < 0.0);
        assert!(result.segments[0].end.point.x < 0.0);
        assert!(result.segments[1].start.point.x > 0.0);
        assert!(result.segments[1].end.point.x > 0.0);
        for segment in result.segments {
            assert!(segment.start.point.z.abs() <= 1e-12);
            assert!(segment.end.point.z.abs() <= 1e-12);
        }
    }
    #[test]
    fn multisegment_result_is_deterministic() {
        let first = try_intersect_bilinear_planar_multisegment(&freeform(), &plane(), 1e-12).unwrap();
        for _ in 0..1000 {
            assert_eq!(first, try_intersect_bilinear_planar_multisegment(&freeform(), &plane(), 1e-12).unwrap());
        }
    }
    #[test]
    fn identical_plane_pair_falls_through_to_existing_planar_solver() {
        assert_eq!(try_intersect_bilinear_planar_multisegment(&plane(), &plane(), 1e-12).unwrap(), None);
    }
}
