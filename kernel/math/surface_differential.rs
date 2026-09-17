//! Second-order tensor-product NURBS surface differential geometry.
//!
//! Homogeneous differentiated control nets provide analytic `Suu`, `Suv`, and
//! `Svv`. Rational derivatives use the quotient rule. Curvature quantities are
//! derived from the first and second fundamental forms and are undefined at
//! singular parameterizations.

use super::{nurbs_surface::{NurbsSurface2D, NurbsSurfaceError, Point3}};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SurfaceDerivatives {
    pub du: Point3,
    pub dv: Point3,
    pub duu: Point3,
    pub duv: Point3,
    pub dvv: Point3,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CurvatureData {
    pub gaussian: f64,
    pub mean: f64,
    pub principal_min: f64,
    pub principal_max: f64,
}

#[derive(Clone, Copy)]
struct H {
    x: f64,
    y: f64,
    z: f64,
    w: f64,
}

fn zero_h() -> H {
    H { x: 0.0, y: 0.0, z: 0.0, w: 0.0 }
}

fn lerp(a: H, b: H, t: f64) -> H {
    H {
        x: a.x * (1.0 - t) + b.x * t,
        y: a.y * (1.0 - t) + b.y * t,
        z: a.z * (1.0 - t) + b.z * t,
        w: a.w * (1.0 - t) + b.w * t,
    }
}

fn count(knots: &[f64], degree: usize) -> usize {
    knots.len() - degree - 1
}

fn span(t: f64, degree: usize, knots: &[f64], ncp: usize) -> usize {
    let n = ncp - 1;
    if t >= knots[n + 1] {
        n
    } else if t <= knots[degree] {
        degree
    } else {
        let mut lo = degree;
        let mut hi = n + 1;
        let mut m = (lo + hi) / 2;
        while t < knots[m] || t >= knots[m + 1] {
            if t < knots[m] {
                hi = m;
            } else {
                lo = m;
            }
            m = (lo + hi) / 2;
        }
        m
    }
}

fn deboor(t: f64, degree: usize, knots: &[f64], control: &[H]) -> H {
    if degree == 0 {
        return control[0];
    }
    let s = span(t, degree, knots, control.len());
    let mut work = control[s - degree..=s].to_vec();
    for level in 1..=degree {
        for j in (level..=degree).rev() {
            let i = s - degree + j;
            let den = knots[i + degree + 1 - level] - knots[i];
            let a = if den == 0.0 { 0.0 } else { (t - knots[i]) / den };
            work[j] = lerp(work[j - 1], work[j], a);
        }
    }
    work[degree]
}

fn base_net(surface: &NurbsSurface2D, weight_scale: f64) -> Result<Vec<Vec<H>>, NurbsSurfaceError> {
    let nu = count(&surface.knots_u, surface.degree_u);
    let nv = count(&surface.knots_v, surface.degree_v);
    let net = (0..nu)
        .map(|i| {
            (0..nv)
                .map(|j| {
                    let k = i * nv + j;
                    let p = surface.control_points[k];
                    let w = surface.weights[k] / weight_scale;
                    H { x: p.x * w, y: p.y * w, z: p.z * w, w }
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    if net.iter().flatten().any(|point| {
        !point.x.is_finite() || !point.y.is_finite() || !point.z.is_finite() || !point.w.is_finite()
    }) {
        return Err(NurbsSurfaceError::Overflow);
    }
    Ok(net)
}

fn du_net(
    net: &[Vec<H>],
    degree: usize,
    knots: &[f64],
) -> Result<(Vec<Vec<H>>, usize, Vec<f64>), NurbsSurfaceError> {
    if degree == 0 {
        return Err(NurbsSurfaceError::InvalidDegree);
    }
    let nu = net.len();
    let nv = net[0].len();
    let mut out = vec![vec![zero_h(); nv]; nu - 1];
    for i in 0..nu - 1 {
        let den = knots[i + degree + 1] - knots[i + 1];
        if den == 0.0 {
            return Err(NurbsSurfaceError::InvalidDomain);
        }
        let factor = degree as f64 / den;
        for j in 0..nv {
            let a = net[i][j];
            let b = net[i + 1][j];
            let value = H {
                x: (b.x - a.x) * factor,
                y: (b.y - a.y) * factor,
                z: (b.z - a.z) * factor,
                w: (b.w - a.w) * factor,
            };
            if ![value.x, value.y, value.z, value.w]
                .iter()
                .all(|component| component.is_finite())
            {
                return Err(NurbsSurfaceError::Overflow);
            }
            out[i][j] = value;
        }
    }
    Ok((out, degree - 1, knots[1..knots.len() - 1].to_vec()))
}

fn dv_net(
    net: &[Vec<H>],
    degree: usize,
    knots: &[f64],
) -> Result<(Vec<Vec<H>>, usize, Vec<f64>), NurbsSurfaceError> {
    if degree == 0 {
        return Err(NurbsSurfaceError::InvalidDegree);
    }
    let nu = net.len();
    let nv = net[0].len();
    let mut out = vec![vec![zero_h(); nv - 1]; nu];
    for i in 0..nu {
        for j in 0..nv - 1 {
            let den = knots[j + degree + 1] - knots[j + 1];
            if den == 0.0 {
                return Err(NurbsSurfaceError::InvalidDomain);
            }
            let factor = degree as f64 / den;
            let a = net[i][j];
            let b = net[i][j + 1];
            let value = H {
                x: (b.x - a.x) * factor,
                y: (b.y - a.y) * factor,
                z: (b.z - a.z) * factor,
                w: (b.w - a.w) * factor,
            };
            if ![value.x, value.y, value.z, value.w]
                .iter()
                .all(|component| component.is_finite())
            {
                return Err(NurbsSurfaceError::Overflow);
            }
            out[i][j] = value;
        }
    }
    Ok((out, degree - 1, knots[1..knots.len() - 1].to_vec()))
}

fn eval_net(
    net: &[Vec<H>],
    du: usize,
    dv: usize,
    ku: &[f64],
    kv: &[f64],
    u: f64,
    v: f64,
) -> H {
    let nv = net[0].len();
    let su = span(u, du, ku, net.len());
    let sv = span(v, dv, kv, nv);
    let mut rows = Vec::with_capacity(dv + 1);
    for j in sv - dv..=sv {
        let mut c = Vec::with_capacity(du + 1);
        for i in su - du..=su {
            c.push(net[i][j]);
        }
        rows.push(deboor(u, du, ku, &c));
    }
    deboor(v, dv, kv, &rows)
}

fn quotient1(p: H, d: H) -> Result<Point3, NurbsSurfaceError> {
    if p.w <= 0.0 {
        return Err(NurbsSurfaceError::ZeroProjectiveWeight);
    }
    let w2 = p.w * p.w;
    if !w2.is_finite() || w2 == 0.0 {
        return Err(NurbsSurfaceError::Overflow);
    }
    let q = Point3 {
        x: (d.x * p.w - p.x * d.w) / w2,
        y: (d.y * p.w - p.y * d.w) / w2,
        z: (d.z * p.w - p.z * d.w) / w2,
    };
    if q.x.is_finite() && q.y.is_finite() && q.z.is_finite() {
        Ok(q)
    } else {
        Err(NurbsSurfaceError::Overflow)
    }
}

fn quotient2(p: H, d: H, dd: H) -> Result<Point3, NurbsSurfaceError> {
    if p.w <= 0.0 {
        return Err(NurbsSurfaceError::ZeroProjectiveWeight);
    }
    let w = p.w;
    let w2 = w * w;
    let w3 = w2 * w;
    if !w2.is_finite() || !w3.is_finite() || w3 == 0.0 {
        return Err(NurbsSurfaceError::Overflow);
    }
    let q = Point3 {
        x: (dd.x * w2 - p.x * w * dd.w - 2.0 * d.x * w * d.w + 2.0 * p.x * d.w * d.w) / w3,
        y: (dd.y * w2 - p.y * w * dd.w - 2.0 * d.y * w * d.w + 2.0 * p.y * d.w * d.w) / w3,
        z: (dd.z * w2 - p.z * w * dd.w - 2.0 * d.z * w * dd.w + 2.0 * p.z * d.w * d.w) / w3,
    };
    if q.x.is_finite() && q.y.is_finite() && q.z.is_finite() {
        Ok(q)
    } else {
        Err(NurbsSurfaceError::Overflow)
    }
}

pub fn derivatives_at(
    surface: &NurbsSurface2D,
    u: f64,
    v: f64,
) -> Result<SurfaceDerivatives, NurbsSurfaceError> {
    surface.validate()?;
    let (u_start, u_end, v_start, v_end) = surface.parameter_domain()?;
    if !u.is_finite() || !v.is_finite() {
        return Err(NurbsSurfaceError::NonFinite);
    }
    if u < u_start || u > u_end || v < v_start || v > v_end {
        return Err(NurbsSurfaceError::OutOfDomain);
    }

    let weight_scale = surface.normalized_weight_scale()?;
    let base_net = base_net(surface, weight_scale)?;
    let base_h = eval_net(
        &base_net,
        surface.degree_u,
        surface.degree_v,
        &surface.knots_u,
        &surface.knots_v,
        u,
        v,
    );
    let (du_net1, p_u, ku1) = du_net(&base_net, surface.degree_u, &surface.knots_u)?;
    let (dv_net1, p_v, kv1) = dv_net(&base_net, surface.degree_v, &surface.knots_v)?;
    let du_h = eval_net(
        &du_net1,
        p_u,
        surface.degree_v,
        &ku1,
        &surface.knots_v,
        u,
        v,
    );
    let dv_h = eval_net(
        &dv_net1,
        surface.degree_u,
        p_v,
        &surface.knots_u,
        &kv1,
        u,
        v,
    );
    let duu_h = if surface.degree_u >= 2 {
        let x = du_net(&du_net1, p_u, &ku1)?;
        eval_net(&x.0, x.1, surface.degree_v, &x.2, &surface.knots_v, u, v)
    } else {
        zero_h()
    };
    let dvv_h = if surface.degree_v >= 2 {
        let x = dv_net(&dv_net1, p_v, &kv1)?;
        eval_net(&x.0, surface.degree_u, x.1, &surface.knots_u, &x.2, u, v)
    } else {
        zero_h()
    };
    let duv_h = if surface.degree_u >= 1 && surface.degree_v >= 1 {
        let x = dv_net(&du_net1, surface.degree_v, &surface.knots_v)?;
        eval_net(&x.0, p_u, x.1, &ku1, &x.2, u, v)
    } else {
        zero_h()
    };

    let du = quotient1(base_h, du_h)?;
    let dv = quotient1(base_h, dv_h)?;
    let duu = quotient2(base_h, du_h, duu_h)?;
    let dvv = quotient2(base_h, dv_h, dvv_h)?;

    let w = base_h.w;
    let w2 = w * w;
    let w3 = w2 * w;
    if !w2.is_finite() || !w3.is_finite() || w3 == 0.0 {
        return Err(NurbsSurfaceError::Overflow);
    }
    let duv = Point3 {
        x: (duv_h.x * w2
            - du_h.x * w * dv_h.w
            - dv_h.x * w * du_h.w
            + 2.0 * base_h.x * du_h.w * dv_h.w
            - base_h.x * w * duv_h.w)
            / w3,
        y: (duv_h.y * w2
            - du_h.y * w * dv_h.w
            - dv_h.y * w * du_h.w
            + 2.0 * base_h.y * du_h.w * dv_h.w
            - base_h.y * w * duv_h.w)
            / w3,
        z: (duv_h.z * w2
            - du_h.z * w * dv_h.w
            - dv_h.z * w * du_h.w
            + 2.0 * base_h.z * du_h.w * dv_h.w
            - base_h.z * w * duv_h.w)
            / w3,
    };
    if !duv.x.is_finite() || !duv.y.is_finite() || !duv.z.is_finite() {
        return Err(NurbsSurfaceError::Overflow);
    }
    Ok(SurfaceDerivatives { du, dv, duu, duv, dvv })
}

pub fn curvature_at(
    surface: &NurbsSurface2D,
    u: f64,
    v: f64,
    tolerance: f64,
) -> Result<CurvatureData, NurbsSurfaceError> {
    if !tolerance.is_finite() || tolerance < 0.0 {
        return Err(NurbsSurfaceError::InvalidDomain);
    }
    let d = derivatives_at(surface, u, v)?;
    let cross = Point3 {
        x: d.du.y * d.dv.z - d.du.z * d.dv.y,
        y: d.du.z * d.dv.x - d.du.x * d.dv.z,
        z: d.du.x * d.dv.y - d.du.y * d.dv.x,
    };
    let norm = cross.x.hypot(cross.y).hypot(cross.z);
    if !norm.is_finite() {
        return Err(NurbsSurfaceError::Overflow);
    }
    if norm == 0.0 {
        return Err(NurbsSurfaceError::Degenerate);
    }
    let n = Point3 {
        x: cross.x / norm,
        y: cross.y / norm,
        z: cross.z / norm,
    };
    let e = n.x * d.duu.x + n.y * d.duu.y + n.z * d.duu.z;
    let f = n.x * d.duv.x + n.y * d.duv.y + n.z * d.duv.z;
    let g = n.x * d.dvv.x + n.y * d.dvv.y + n.z * d.dvv.z;
    let e_first = d.du.x * d.du.x + d.du.y * d.du.y + d.du.z * d.du.z;
    let f_first = d.du.x * d.dv.x + d.du.y * d.dv.y + d.du.z * d.dv.z;
    let g_first = d.dv.x * d.dv.x + d.dv.y * d.dv.y + d.dv.z * d.dv.z;
    let det = e_first * g_first - f_first * f_first;
    if ![e, f, g, e_first, f_first, g_first, det]
        .iter()
        .all(|value| value.is_finite())
    {
        return Err(NurbsSurfaceError::Overflow);
    }
    if det <= 0.0 {
        return Err(NurbsSurfaceError::Degenerate);
    }

    let gaussian = (e * g - f * f) / det;
    let mean = (e * g_first + g * e_first - 2.0 * f * f_first) / (2.0 * det);
    let b = e * g_first + g * e_first - 2.0 * f * f_first;
    let c = e * g - f * f;
    let mut discriminant = b * b - 4.0 * det * c;
    if !discriminant.is_finite() {
        return Err(NurbsSurfaceError::Overflow);
    }
    let scale_term = b * b;
    let second_term = (4.0 * det * c).abs();
    if !scale_term.is_finite() || !second_term.is_finite() {
        return Err(NurbsSurfaceError::Overflow);
    }
    let scale = scale_term + second_term;
    if !scale.is_finite() {
        return Err(NurbsSurfaceError::Overflow);
    }
    let tolerance_band = tolerance * scale.max(f64::MIN_POSITIVE);
    if !tolerance_band.is_finite() {
        return Err(NurbsSurfaceError::Overflow);
    }
    if discriminant < 0.0 {
        if discriminant >= -tolerance_band {
            discriminant = 0.0;
        } else {
            return Err(NurbsSurfaceError::Degenerate);
        }
    }

    let root = discriminant.sqrt();
    let denominator = 2.0 * det;
    if denominator == 0.0 || !denominator.is_finite() {
        return Err(NurbsSurfaceError::Overflow);
    }
    let k1 = (b - root) / denominator;
    let k2 = (b + root) / denominator;
    if [gaussian, mean, k1, k2].iter().all(|value| value.is_finite()) {
        Ok(CurvatureData {
            gaussian,
            mean,
            principal_min: k1.min(k2),
            principal_max: k1.max(k2),
        })
    } else {
        Err(NurbsSurfaceError::Overflow)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn patch() -> NurbsSurface2D {
        NurbsSurface2D::new(
            1,
            1,
            vec![
                Point3 { x: 0.0, y: 0.0, z: 0.0 },
                Point3 { x: 0.0, y: 1.0, z: 0.0 },
                Point3 { x: 1.0, y: 0.0, z: 0.0 },
                Point3 { x: 1.0, y: 1.0, z: 0.0 },
            ],
            vec![1.0; 4],
            vec![0.0, 0.0, 1.0, 1.0],
            vec![0.0, 0.0, 1.0, 1.0],
        )
    }

    #[test]
    fn bilinear_surface_has_finite_second_differentials() {
        let d = derivatives_at(&patch(), 0.3, 0.7).unwrap();
        assert!(
            d.du.x.is_finite()
                && d.dv.y.is_finite()
                && d.duu.x.abs() < 1.0e-12
                && d.dvv.y.abs() < 1.0e-12
        );
    }

    #[test]
    fn planar_patch_has_zero_gaussian_curvature() {
        let k = curvature_at(&patch(), 0.5, 0.5, 1.0e-12).unwrap();
        assert!(k.gaussian.abs() < 1.0e-12 && k.mean.abs() < 1.0e-12);
    }

    #[test]
    fn second_differentials_are_invariant_under_extreme_uniform_weight_scaling() {
        for scale in [1.0e-200, 1.0e200] {
            let surface = NurbsSurface2D::new(
                1,
                1,
                vec![
                    Point3 { x: 0.0, y: 0.0, z: 0.0 },
                    Point3 { x: 0.0, y: 1.0, z: 0.0 },
                    Point3 { x: 1.0, y: 0.0, z: 0.0 },
                    Point3 { x: 1.0, y: 1.0, z: 0.0 },
                ],
                vec![scale; 4],
                vec![0.0, 0.0, 1.0, 1.0],
                vec![0.0, 0.0, 1.0, 1.0],
            );
            let d = derivatives_at(&surface, 0.3, 0.7).unwrap();
            assert!((d.du.x - 1.0).abs() < 1.0e-12);
            assert!(d.du.y.abs() < 1.0e-12 && d.du.z.abs() < 1.0e-12);
            assert!((d.dv.y - 1.0).abs() < 1.0e-12);
            assert!(d.dv.x.abs() < 1.0e-12 && d.dv.z.abs() < 1.0e-12);
            assert!(d.duu.x.abs() < 1.0e-12 && d.duu.y.abs() < 1.0e-12 && d.duu.z.abs() < 1.0e-12);
            assert!(d.duv.x.abs() < 1.0e-12 && d.duv.y.abs() < 1.0e-12 && d.duv.z.abs() < 1.0e-12);
            assert!(d.dvv.x.abs() < 1.0e-12 && d.dvv.y.abs() < 1.0e-12 && d.dvv.z.abs() < 1.0e-12);
            let k = curvature_at(&surface, 0.3, 0.7, 1.0e-12).unwrap();
            assert!(k.gaussian.abs() < 1.0e-12 && k.mean.abs() < 1.0e-12);
        }
    }
}
