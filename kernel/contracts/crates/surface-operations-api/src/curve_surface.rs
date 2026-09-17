use super::*;
use umlcad_v6_freeform_api::{NurbsCurve3DDefinition, Point3 as CurvePoint3};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CurveSurfaceIntersectionError { NonFinite, InvalidCurve, InvalidSurface, NumericalFailure, TangentialContact, CoincidentOrUnderdetermined }
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CurveSurfaceIntersectionPoint { pub curve_parameter: f64, pub u: f64, pub v: f64, pub point: Point3 }
#[derive(Clone, Debug, PartialEq)]
pub struct CurveSurfaceIntersectionResult { pub status: IntersectionStatus, pub points: Vec<CurveSurfaceIntersectionPoint> }

fn basis(i: usize, p: usize, t: f64, knots: &[f64]) -> f64 {
    if p == 0 { let last = knots.len() - 1; return if (knots[i] <= t && t < knots[i + 1]) || (t == knots[last] && i + 1 == last) { 1.0 } else { 0.0 }; }
    let ld = knots[i + p] - knots[i]; let rd = knots[i + p + 1] - knots[i + 1];
    let l = if ld == 0.0 { 0.0 } else { (t - knots[i]) / ld * basis(i, p - 1, t, knots) };
    let r = if rd == 0.0 { 0.0 } else { (knots[i + p + 1] - t) / rd * basis(i + 1, p - 1, t, knots) };
    l + r
}
fn basis_derivative(i: usize, p: usize, t: f64, knots: &[f64]) -> f64 {
    if p == 0 { return 0.0; }
    let ld = knots[i + p] - knots[i]; let rd = knots[i + p + 1] - knots[i + 1];
    let l = if ld == 0.0 { 0.0 } else { p as f64 / ld * basis(i, p - 1, t, knots) };
    let r = if rd == 0.0 { 0.0 } else { p as f64 / rd * basis(i + 1, p - 1, t, knots) };
    l - r
}
fn evaluate_curve(curve: &NurbsCurve3DDefinition, t: f64) -> Result<(CurvePoint3, CurvePoint3), CurveSurfaceIntersectionError> {
    let (t0, t1) = curve.parameter_domain().map_err(|_| CurveSurfaceIntersectionError::InvalidCurve)?;
    if !t.is_finite() || t < t0 || t > t1 { return Err(CurveSurfaceIntersectionError::InvalidCurve); }
    let mut h0 = CurvePoint3 { x: 0.0, y: 0.0, z: 0.0 }; let mut h1 = h0; let mut w0 = 0.0; let mut w1 = 0.0;
    for i in 0..curve.control_points.len() {
        let w = curve.weights[i]; let b0 = basis(i, curve.degree, t, &curve.knots); let b1 = basis_derivative(i, curve.degree, t, &curve.knots);
        let s0 = b0 * w; let s1 = b1 * w; let p = curve.control_points[i];
        h0.x += p.x * s0; h0.y += p.y * s0; h0.z += p.z * s0; h1.x += p.x * s1; h1.y += p.y * s1; h1.z += p.z * s1; w0 += s0; w1 += s1;
    }
    if !w0.is_finite() || w0 <= 0.0 { return Err(CurveSurfaceIntersectionError::NumericalFailure); }
    let point = CurvePoint3 { x: h0.x / w0, y: h0.y / w0, z: h0.z / w0 };
    let derivative = CurvePoint3 { x: (h1.x - point.x * w1) / w0, y: (h1.y - point.y * w1) / w0, z: (h1.z - point.z * w1) / w0 };
    if [point.x, point.y, point.z, derivative.x, derivative.y, derivative.z].iter().any(|v| !v.is_finite()) { return Err(CurveSurfaceIntersectionError::NumericalFailure); }
    Ok((point, derivative))
}
fn det3(a: [[f64; 3]; 3]) -> f64 { a[0][0] * (a[1][1] * a[2][2] - a[1][2] * a[2][1]) - a[0][1] * (a[1][0] * a[2][2] - a[1][2] * a[2][0]) + a[0][2] * (a[1][0] * a[2][1] - a[1][1] * a[2][0]) }
fn solve3(a: [[f64; 3]; 3], b: Point3) -> Option<[f64; 3]> {
    let d = det3(a); if !d.is_finite() || d.abs() <= 1e-14 { return None; }
    let mut m = a; m[0][0] = b.x; m[1][0] = b.y; m[2][0] = b.z; let x0 = det3(m) / d;
    m = a; m[0][1] = b.x; m[1][1] = b.y; m[2][1] = b.z; let x1 = det3(m) / d;
    m = a; m[0][2] = b.x; m[1][2] = b.y; m[2][2] = b.z; let x2 = det3(m) / d; Some([x0, x1, x2])
}
fn distance(a: Point3, b: CurvePoint3) -> f64 { (a.x - b.x).hypot((a.y - b.y).hypot(a.z - b.z)) }
fn point_norm(a: CurvePoint3) -> f64 { a.x.hypot(a.y.hypot(a.z)) }
fn dot_curve(a: CurvePoint3, b: Point3) -> f64 { a.x * b.x + a.y * b.y + a.z * b.z }
fn isolated_root_is_tangent(curve_tangent: CurvePoint3, surface_normal: Point3, tolerance: f64) -> bool {
    let tangent_norm = point_norm(curve_tangent); let normal_norm = surface_normal.norm();
    if !tangent_norm.is_finite() || !normal_norm.is_finite() { return false; }
    let scale = tangent_norm.max(normal_norm).max(1.0); let threshold = tolerance.max(1e-10) * tangent_norm * normal_norm * 10.0 / scale;
    tangent_norm > 1e-12 * scale && normal_norm > 1e-12 * scale && dot_curve(curve_tangent, surface_normal).abs() <= threshold
}
fn try_exact_linear_curve_surface(curve: &NurbsCurve3DDefinition, surface: &NurbsSurface3DDefinition, tolerance: f64, t0: f64, t1: f64) -> Result<Option<CurveSurfaceIntersectionResult>, CurveSurfaceIntersectionError> {
    if curve.degree != 1 || curve.control_points.len() != 2 || curve.weights.iter().any(|w| *w != 1.0) { return Ok(None); }
    let line = LineSegment3D { start: Point3 { x: curve.control_points[0].x, y: curve.control_points[0].y, z: curve.control_points[0].z }, end: Point3 { x: curve.control_points[1].x, y: curve.control_points[1].y, z: curve.control_points[1].z } };
    let Some(result) = crate::planar_line_surface::classify_planar_line_surface(line, surface, tolerance).map_err(|e| match e {
        LineSurfaceIntersectionError::CoincidentOrUnderdetermined => CurveSurfaceIntersectionError::CoincidentOrUnderdetermined,
        LineSurfaceIntersectionError::NumericalFailure => CurveSurfaceIntersectionError::NumericalFailure,
        LineSurfaceIntersectionError::NonFinite => CurveSurfaceIntersectionError::NonFinite,
        LineSurfaceIntersectionError::DegenerateLine => CurveSurfaceIntersectionError::InvalidCurve,
        LineSurfaceIntersectionError::InvalidSurface => CurveSurfaceIntersectionError::InvalidSurface,
        LineSurfaceIntersectionError::TangentialContact => CurveSurfaceIntersectionError::TangentialContact,
    })? else { return Ok(None); };
    let points = result.points.into_iter().map(|p| CurveSurfaceIntersectionPoint { curve_parameter: t0 + p.line_parameter * (t1 - t0), u: p.u, v: p.v, point: p.point }).collect();
    Ok(Some(CurveSurfaceIntersectionResult { status: result.status, points }))
}
pub fn intersect_nurbs_curve_surface(curve: &NurbsCurve3DDefinition, surface: &NurbsSurface3DDefinition, tolerance: f64) -> Result<CurveSurfaceIntersectionResult, CurveSurfaceIntersectionError> {
    if !tolerance.is_finite() || tolerance < 0.0 { return Err(CurveSurfaceIntersectionError::NonFinite); }
    curve.validate().map_err(|_| CurveSurfaceIntersectionError::InvalidCurve)?; surface.validate().map_err(|_| CurveSurfaceIntersectionError::InvalidSurface)?;
    let (t0, t1) = curve.parameter_domain().map_err(|_| CurveSurfaceIntersectionError::InvalidCurve)?; let ((u0, u1), (v0, v1)) = surface.parameter_domain().map_err(|_| CurveSurfaceIntersectionError::InvalidSurface)?;
    if let Some(result) = try_exact_linear_curve_surface(curve, surface, tolerance, t0, t1)? { return Ok(result); }
    let scale = curve.control_points.iter().map(|p| p.x.hypot(p.y.hypot(p.z))).fold(1.0, f64::max).max(surface.control_points.iter().map(|p| p.norm()).fold(1.0, f64::max));
    let residual = tolerance.max(1e-10 * scale); let mut roots = Vec::new(); let seeds = 5usize;
    for it in 0..seeds { for iu in 0..seeds { for iv in 0..seeds {
        let mut t = t0 + (t1 - t0) * (it as f64 + 0.5) / seeds as f64; let mut u = u0 + (u1 - u0) * (iu as f64 + 0.5) / seeds as f64; let mut v = v0 + (v1 - v0) * (iv as f64 + 0.5) / seeds as f64; let mut converged = false;
        for _ in 0..50 {
            let (cp, cdt) = match evaluate_curve(curve, t) { Ok(x) => x, Err(_) => break }; let sd = match surface.differential_at(u, v) { Ok(x) => x, Err(_) => break };
            let r = Point3 { x: cp.x - sd.point.x, y: cp.y - sd.point.y, z: cp.z - sd.point.z }; if r.norm() <= residual { converged = true; break; }
            let Some(d) = solve3([[cdt.x, -sd.du.x, -sd.dv.x], [cdt.y, -sd.du.y, -sd.dv.y], [cdt.z, -sd.du.z, -sd.dv.z]], Point3 { x: -r.x, y: -r.y, z: -r.z }) else { break };
            t += d[0]; u += d[1]; v += d[2]; if !t.is_finite() || !u.is_finite() || !v.is_finite() { break; }
            if t < t0 - residual || t > t1 + residual || u < u0 - residual || u > u1 + residual || v < v0 - residual || v > v1 + residual { break; }
        }
        if converged && t >= t0 - residual && t <= t1 + residual && u >= u0 - residual && u <= u1 + residual && v >= v0 - residual && v <= v1 + residual {
            let t = t.clamp(t0, t1); let u = u.clamp(u0, u1); let v = v.clamp(v0, v1); let (cp, _) = match evaluate_curve(curve, t) { Ok(x) => x, Err(_) => continue }; let sp = match surface.differential_at(u, v) { Ok(x) => x.point, Err(_) => continue };
            let point = Point3 { x: 0.5 * (cp.x + sp.x), y: 0.5 * (cp.y + sp.y), z: 0.5 * (cp.z + sp.z) };
            if roots.iter().all(|q: &CurveSurfaceIntersectionPoint| distance(q.point, CurvePoint3 { x: point.x, y: point.y, z: point.z }) > residual * 10.0) { roots.push(CurveSurfaceIntersectionPoint { curve_parameter: t, u, v, point }); }
        }
    }}}
    roots.sort_by(|a, b| a.curve_parameter.total_cmp(&b.curve_parameter).then(a.u.total_cmp(&b.u)).then(a.v.total_cmp(&b.v)));
    if !roots.is_empty() {
        let tangent_tol = (residual * scale).sqrt().max(residual * 10.0);
        for root in &roots {
            let (_, tangent) = evaluate_curve(curve, root.curve_parameter)?;
            let differential = surface.differential_at(root.u, root.v).map_err(|_| CurveSurfaceIntersectionError::NumericalFailure)?;
            let normal = differential.du.cross(differential.dv);
            if !normal.norm().is_finite() { return Err(CurveSurfaceIntersectionError::NumericalFailure); }
            if isolated_root_is_tangent(tangent, normal, tolerance) {
                let clustered = roots.iter().all(|candidate| distance(candidate.point, CurvePoint3 { x: root.point.x, y: root.point.y, z: root.point.z }) <= tangent_tol);
                if clustered { return Err(CurveSurfaceIntersectionError::TangentialContact); }
            }
        }
    }
    let status = match roots.len() { 0 => IntersectionStatus::NoIntersection, 1 => IntersectionStatus::Unique, _ => IntersectionStatus::Ambiguous };
    Ok(CurveSurfaceIntersectionResult { status, points: roots })
}
pub trait NurbsCurveSurfaceOperations {
    fn intersect_nurbs_curve_surface(&self, curve: &NurbsCurve3DDefinition, surface: &NurbsSurface3DDefinition, tolerance: f64) -> Result<CurveSurfaceIntersectionResult, CurveSurfaceIntersectionError>;
}
#[cfg(test)]
mod tests {
    use super::*;
    fn plane() -> NurbsSurface3DDefinition {
        NurbsSurface3DDefinition::new((1, 1), vec![Point3 { x: 0.0, y: 0.0, z: 0.0 }, Point3 { x: 0.0, y: 1.0, z: 0.0 }, Point3 { x: 1.0, y: 0.0, z: 0.0 }, Point3 { x: 1.0, y: 1.0, z: 0.0 }], vec![1.0; 4], (2, 2), vec![0.0, 0.0, 1.0, 1.0], vec![0.0, 0.0, 1.0, 1.0])
    }
    fn tangent_plane() -> NurbsSurface3DDefinition {
        NurbsSurface3DDefinition::new((1, 1), vec![Point3 { x: -1.0, y: -1.0, z: 0.0 }, Point3 { x: -1.0, y: 1.0, z: 0.0 }, Point3 { x: 1.0, y: -1.0, z: 0.0 }, Point3 { x: 1.0, y: 1.0, z: 0.0 }], vec![1.0; 4], (2, 2), vec![0.0, 0.0, 1.0, 1.0], vec![0.0, 0.0, 1.0, 1.0])
    }
    fn line() -> NurbsCurve3DDefinition {
        NurbsCurve3DDefinition::new(1, vec![CurvePoint3 { x: 0.25, y: 0.75, z: -1.0 }, CurvePoint3 { x: 0.25, y: 0.75, z: 1.0 }], vec![1.0, 1.0], vec![0.0, 0.0, 1.0, 1.0])
    }
    fn tangent_curve() -> NurbsCurve3DDefinition {
        NurbsCurve3DDefinition::new(2, vec![CurvePoint3 { x: -1.0, y: 0.0, z: 1.0 }, CurvePoint3 { x: 0.0, y: 0.0, z: -1.0 }, CurvePoint3 { x: 1.0, y: 0.0, z: 1.0 }], vec![1.0; 3], vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0])
    }
    fn planar_linear_curve() -> NurbsCurve3DDefinition {
        NurbsCurve3DDefinition::new(1, vec![CurvePoint3 { x: -1.0, y: 0.0, z: 0.0 }, CurvePoint3 { x: 1.0, y: 0.0, z: 0.0 }], vec![1.0, 1.0], vec![3.0, 3.0, 7.0, 7.0])
    }
    #[test]
    fn unique_linear_curve_surface_root() {
        let r = intersect_nurbs_curve_surface(&line(), &plane(), 1e-10).unwrap(); assert_eq!(r.status, IntersectionStatus::Unique); assert_eq!(r.points.len(), 1); assert!((r.points[0].curve_parameter - 0.5).abs() < 1e-10); assert!((r.points[0].u - 0.25).abs() < 1e-10); assert!((r.points[0].v - 0.75).abs() < 1e-10); assert!(r.points[0].point.z.abs() < 1e-10);
    }
    #[test]
    fn curve_parameter_domain_is_preserved() { let mut c = line(); c.knots = vec![3.0, 3.0, 7.0, 7.0]; let r = intersect_nurbs_curve_surface(&c, &plane(), 1e-10).unwrap(); assert_eq!(r.status, IntersectionStatus::Unique); assert!((r.points[0].curve_parameter - 5.0).abs() < 1e-10); }
    #[test]
    fn isolated_quadratic_tangent_is_explicitly_reported() { assert_eq!(intersect_nurbs_curve_surface(&tangent_curve(), &tangent_plane(), 1e-10), Err(CurveSurfaceIntersectionError::TangentialContact)); }
    #[test]
    fn exact_linear_coplanar_curve_is_explicitly_underdetermined() { assert_eq!(intersect_nurbs_curve_surface(&planar_linear_curve(), &plane(), 1e-10), Err(CurveSurfaceIntersectionError::CoincidentOrUnderdetermined)); }
}
