use crate::{
    bilinear_branch_pairing::pair_bilinear_roots,
    IntersectionStatus,
    NurbsSurface3DDefinition,
    Plane3D,
    SurfaceIntersectionEndpoint,
    SurfaceIntersectionSegment,
    SurfaceSurfaceIntersectionError,
    SurfaceSurfaceIntersectionResult,
};
use umlcad_v6_nurbs_surface_api::Point3;

#[derive(Clone, Copy)]
struct Patch {
    points: [Point3; 4],
    weights: [f64; 4],
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
    surface.validate().map_err(|_| SurfaceSurfaceIntersectionError::InvalidSurface)?;
    if surface.degree_u != 1 || surface.degree_v != 1 || surface.count_u != 2 || surface.count_v != 2 {
        return Ok(None);
    }
    let points = [surface.control_points[0], surface.control_points[1], surface.control_points[2], surface.control_points[3]];
    let weights = [surface.weights[0], surface.weights[1], surface.weights[2], surface.weights[3]];
    let ((u0, u1), (v0, v1)) = surface.parameter_domain().map_err(|_| SurfaceSurfaceIntersectionError::InvalidSurface)?;
    if u1 <= u0 || v1 <= v0 {
        return Err(SurfaceSurfaceIntersectionError::InvalidSurface);
    }
    if points.into_iter().flat_map(|p| [p.x, p.y, p.z]).any(|x| !x.is_finite())
        || weights.iter().any(|w| !w.is_finite() || *w <= 0.0)
    {
        return Err(SurfaceSurfaceIntersectionError::InvalidSurface);
    }
    Ok(Some(Patch { points, weights, u0, u1, v0, v1 }))
}

fn rational_point(p: Patch, s: f64, t: f64) -> Result<Point3, SurfaceSurfaceIntersectionError> {
    let basis = [(1.0 - s) * (1.0 - t), (1.0 - s) * t, s * (1.0 - t), s * t];
    let mut numerator = Point3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut denominator = 0.0;
    for ((point, basis_value), weight) in p.points.into_iter().zip(basis).zip(p.weights) {
        let factor = basis_value * weight;
        numerator = add(numerator, scale(point, factor));
        denominator += factor;
    }
    if !denominator.is_finite() || denominator <= 0.0 {
        return Err(SurfaceSurfaceIntersectionError::NumericalFailure);
    }
    let point = scale(numerator, 1.0 / denominator);
    if [point.x, point.y, point.z].into_iter().any(|x| !x.is_finite()) {
        return Err(SurfaceSurfaceIntersectionError::NumericalFailure);
    }
    Ok(point)
}

fn uv(p: Patch, s: f64, t: f64) -> (f64, f64) {
    (p.u0 + s * (p.u1 - p.u0), p.v0 + t * (p.v1 - p.v0))
}

fn is_planar(p: Patch, tolerance: f64) -> Result<bool, SurfaceSurfaceIntersectionError> {
    let n = cross(sub(p.points[2], p.points[0]), sub(p.points[1], p.points[0]));
    let nn = norm(n);
    let scale = p.points.into_iter().map(norm).fold(1.0, f64::max);
    if !nn.is_finite() || nn <= 1e-14 * scale * scale {
        return Err(SurfaceSurfaceIntersectionError::UnsupportedSurfaceFamily);
    }
    let plane = Plane3D { origin: p.points[0], normal: n };
    for point in p.points {
        let distance = plane.signed_distance(point).abs();
        if !distance.is_finite() {
            return Err(SurfaceSurfaceIntersectionError::NumericalFailure);
        }
        if distance > tolerance.max(1e-12 * scale) {
            return Ok(false);
        }
    }
    Ok(true)
}

fn plane_from_patch(p: Patch) -> Result<Plane3D, SurfaceSurfaceIntersectionError> {
    if p.weights.into_iter().any(|w| (w - 1.0).abs() > 1e-14) {
        return Err(SurfaceSurfaceIntersectionError::UnsupportedSurfaceFamily);
    }
    let normal = cross(sub(p.points[2], p.points[0]), sub(p.points[1], p.points[0]));
    if norm(normal) <= 1e-14 {
        return Err(SurfaceSurfaceIntersectionError::UnsupportedSurfaceFamily);
    }
    Ok(Plane3D { origin: p.points[0], normal })
}

fn plane_restriction(p: Patch, plane: Plane3D) -> Result<[f64; 4], SurfaceSurfaceIntersectionError> {
    let values = [
        p.weights[0] * plane.signed_distance(p.points[0]),
        p.weights[1] * plane.signed_distance(p.points[1]),
        p.weights[2] * plane.signed_distance(p.points[2]),
        p.weights[3] * plane.signed_distance(p.points[3]),
    ];
    if values.iter().any(|x| !x.is_finite()) {
        return Err(SurfaceSurfaceIntersectionError::NumericalFailure);
    }
    Ok([
        values[0],
        values[2] - values[0],
        values[1] - values[0],
        values[3] - values[2] - values[1] + values[0],
    ])
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
    if let Some(t) = div(-a, c) { add_root(&mut roots, 0.0, t, tol); }
    else if a.abs() <= tol && (a + c).abs() <= tol { add_root(&mut roots, 0.0, 0.0, tol); add_root(&mut roots, 0.0, 1.0, tol); }
    if let Some(t) = div(-(a + b), c + d) { add_root(&mut roots, 1.0, t, tol); }
    else if (a + b).abs() <= tol && (a + b + c + d).abs() <= tol { add_root(&mut roots, 1.0, 0.0, tol); add_root(&mut roots, 1.0, 1.0, tol); }
    if let Some(s) = div(-a, b) { add_root(&mut roots, s, 0.0, tol); }
    else if a.abs() <= tol && (a + b).abs() <= tol { add_root(&mut roots, 0.0, 0.0, tol); add_root(&mut roots, 1.0, 0.0, tol); }
    if let Some(s) = div(-(a + c), b + d) { add_root(&mut roots, s, 1.0, tol); }
    else if (a + c).abs() <= tol && (a + b + c + d).abs() <= tol { add_root(&mut roots, 0.0, 1.0, tol); add_root(&mut roots, 1.0, 1.0, tol); }
    roots.sort_by(|x, y| x.0.total_cmp(&y.0).then(x.1.total_cmp(&y.1)));
    Ok(roots)
}

fn inverse_planar_parameters(p: Patch, point: Point3) -> Result<(f64, f64), SurfaceSurfaceIntersectionError> {
    let r = sub(point, p.points[0]);
    let du = sub(p.points[2], p.points[0]);
    let dv = sub(p.points[1], p.points[0]);
    let aa = dot(du, du); let ab = dot(du, dv); let bb = dot(dv, dv);
    let det = aa * bb - ab * ab;
    if !det.is_finite() || det.abs() <= 1e-14 * aa.max(bb).max(1.0) {
        return Err(SurfaceSurfaceIntersectionError::NumericalFailure);
    }
    let ru = dot(du, r); let rv = dot(dv, r);
    let s = (ru * bb - rv * ab) / det;
    let t = (rv * aa - ru * ab) / det;
    if !s.is_finite() || !t.is_finite() { return Err(SurfaceSurfaceIntersectionError::NumericalFailure); }
    Ok((s.clamp(0.0, 1.0), t.clamp(0.0, 1.0)))
}

pub fn try_intersect_rational_bilinear_planar(
    first: &NurbsSurface3DDefinition,
    second: &NurbsSurface3DDefinition,
    tolerance: f64,
) -> Result<Option<SurfaceSurfaceIntersectionResult>, SurfaceSurfaceIntersectionError> {
    if !tolerance.is_finite() || tolerance < 0.0 { return Err(SurfaceSurfaceIntersectionError::NonFinite); }
    let a = match patch(first)? { Some(x) => x, None => return Ok(None) };
    let b = match patch(second)? { Some(x) => x, None => return Ok(None) };
    let ap = is_planar(a, tolerance)?; let bp = is_planar(b, tolerance)?;
    if ap == bp { return Ok(None); }
    let (freeform, planar, freeform_first) = if !ap { (a, b, true) } else { (b, a, false) };
    let plane = plane_from_patch(planar)?;
    let coefficients = plane_restriction(freeform, plane)?;
    let roots = boundary_roots(coefficients, tolerance)?;
    if roots.is_empty() {
        return Ok(Some(SurfaceSurfaceIntersectionResult { status: IntersectionStatus::NoIntersection, segments: Vec::new() }));
    }
    if roots.len() == 1 || roots.len() == 3 {
        return Ok(Some(SurfaceSurfaceIntersectionResult { status: IntersectionStatus::Ambiguous, segments: Vec::new() }));
    }
    let pairs = pair_bilinear_roots(&roots, coefficients, tolerance)?;
    let mut segments = Vec::new();
    for pair in pairs {
        let p0 = rational_point(freeform, pair.0.0, pair.0.1)?;
        let p1 = rational_point(freeform, pair.1.0, pair.1.1)?;
        if norm(sub(p0, p1)) <= tolerance.max(1e-12) {
            return Ok(Some(SurfaceSurfaceIntersectionResult { status: IntersectionStatus::Ambiguous, segments: Vec::new() }));
        }
        let f0 = uv(freeform, pair.0.0, pair.0.1);
        let f1 = uv(freeform, pair.1.0, pair.1.1);
        let q0 = inverse_planar_parameters(planar, p0)?;
        let q1 = inverse_planar_parameters(planar, p1)?;
        let g0 = uv(planar, q0.0, q0.1);
        let g1 = uv(planar, q1.0, q1.1);
        let start = if freeform_first { SurfaceIntersectionEndpoint { point: p0, first_uv: f0, second_uv: g0 } } else { SurfaceIntersectionEndpoint { point: p0, first_uv: g0, second_uv: f0 } };
        let end = if freeform_first { SurfaceIntersectionEndpoint { point: p1, first_uv: f1, second_uv: g1 } } else { SurfaceIntersectionEndpoint { point: p1, first_uv: g1, second_uv: f1 } };
        segments.push(SurfaceIntersectionSegment { start, end });
    }
    Ok(Some(SurfaceSurfaceIntersectionResult { status: IntersectionStatus::Unique, segments }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rational_freeform() -> NurbsSurface3DDefinition {
        NurbsSurface3DDefinition::new(
            (1, 1),
            vec![
                Point3 { x: 0.0, y: 0.0, z: -0.25 },
                Point3 { x: 0.0, y: 1.0, z: 0.75 },
                Point3 { x: 1.0, y: 0.0, z: 0.75 },
                Point3 { x: 1.0, y: 1.0, z: 0.75 },
            ],
            vec![1.0, 1.0, 1.0, 2.0],
            (2, 2),
            vec![0.0, 0.0, 1.0, 1.0],
            vec![0.0, 0.0, 1.0, 1.0],
        )
    }

    fn four_root_rational_freeform() -> NurbsSurface3DDefinition {
        NurbsSurface3DDefinition::new(
            (1, 1),
            vec![
                Point3 { x: 0.0, y: 0.0, z: 0.01 },
                Point3 { x: 0.0, y: 1.0, z: -0.095 },
                Point3 { x: 1.0, y: 0.0, z: -0.2633333333333333 },
                Point3 { x: 1.0, y: 1.0, z: 0.0025 },
            ],
            vec![1.0, 2.0, 3.0, 4.0],
            (2, 2),
            vec![0.0, 0.0, 1.0, 1.0],
            vec![0.0, 0.0, 1.0, 1.0],
        )
    }

    fn plane() -> NurbsSurface3DDefinition {
        NurbsSurface3DDefinition::new(
            (1, 1),
            vec![
                Point3 { x: -0.25, y: -0.25, z: 0.0 },
                Point3 { x: -0.25, y: 1.25, z: 0.0 },
                Point3 { x: 1.25, y: -0.25, z: 0.0 },
                Point3 { x: 1.25, y: 1.25, z: 0.0 },
            ],
            vec![1.0; 4],
            (2, 2),
            vec![-1.0, -1.0, 1.0, 1.0],
            vec![4.0, 4.0, 8.0, 8.0],
        )
    }

    #[test]
    fn rational_bilinear_section_has_exact_two_endpoints() {
        let r = try_intersect_rational_bilinear_planar(&rational_freeform(), &plane(), 1e-12).unwrap().unwrap();
        assert_eq!(r.status, IntersectionStatus::Unique);
        assert_eq!(r.segments.len(), 1);
        for p in [r.segments[0].start.point, r.segments[0].end.point] { assert!(p.z.abs() <= 1e-12); }
    }

    #[test]
    fn rational_bilinear_four_boundary_roots_form_two_segments() {
        let r = try_intersect_rational_bilinear_planar(&four_root_rational_freeform(), &plane(), 1e-12).unwrap().unwrap();
        assert_eq!(r.status, IntersectionStatus::Unique);
        assert_eq!(r.segments.len(), 2);
        for segment in r.segments {
            assert!(segment.start.point.z.abs() <= 1e-12);
            assert!(segment.end.point.z.abs() <= 1e-12);
        }
    }

    #[test]
    fn rational_section_is_deterministic() {
        let first = try_intersect_rational_bilinear_planar(&rational_freeform(), &plane(), 1e-12).unwrap();
        for _ in 0..1000 { assert_eq!(first, try_intersect_rational_bilinear_planar(&rational_freeform(), &plane(), 1e-12).unwrap()); }
    }

    #[test]
    fn non_bilinear_surface_is_ignored_by_this_bounded_slice() {
        let quadratic = NurbsSurface3DDefinition::new(
            (2, 2),
            vec![
                Point3 { x: 0.0, y: 0.0, z: 0.0 },
                Point3 { x: 0.0, y: 0.5, z: 0.0 },
                Point3 { x: 0.0, y: 1.0, z: 0.0 },
                Point3 { x: 0.5, y: 0.0, z: 0.0 },
                Point3 { x: 0.5, y: 0.5, z: 0.0 },
                Point3 { x: 0.5, y: 1.0, z: 0.0 },
                Point3 { x: 1.0, y: 0.0, z: 0.0 },
                Point3 { x: 1.0, y: 0.5, z: 0.0 },
                Point3 { x: 1.0, y: 1.0, z: 0.0 },
            ],
            vec![1.0; 9],
            (3, 3),
            vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
            vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
        );
        assert_eq!(try_intersect_rational_bilinear_planar(&quadratic, &plane(), 1e-12).unwrap(), None);
    }
}
