//! Second-order tensor-product NURBS surface differential geometry.
//!
//! The homogeneous control net is normalized by a common positive weight scale.
//! This preserves the rational surface while preventing avoidable overflow when
//! the caller uniformly rescales all projective weights.

use super::nurbs_surface::{NurbsSurface2D, NurbsSurfaceError, Point3};

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

#[derive(Clone, Copy, Debug)]
struct H {
    x: f64,
    y: f64,
    z: f64,
    w: f64,
}

fn zero_h() -> H {
    H { x: 0.0, y: 0.0, z: 0.0, w: 0.0 }
}

fn finite(h: H) -> bool {
    h.x.is_finite() && h.y.is_finite() && h.z.is_finite() && h.w.is_finite()
}

fn lerp(a: H, b: H, t: f64) -> H {
    H {
        x: a.x * (1.0 - t) + b.x * t,
        y: a.y * (1.0 - t) + b.y * t,
        z: a.z * (1.0 - t) + b.z * t,
        w: a.w * (1.0 - t) + b.w * t,
    }
}

fn control_count(knots: &[f64], degree: usize) -> usize {
    knots.len() - degree - 1
}

fn span(parameter: f64, degree: usize, knots: &[f64], control_count: usize) -> usize {
    let last = control_count - 1;
    if parameter >= knots[control_count] {
        return last;
    }
    if parameter <= knots[degree] {
        return degree;
    }
    let mut low = degree;
    let mut high = control_count;
    let mut mid = (low + high) / 2;
    while parameter < knots[mid] || parameter >= knots[mid + 1] {
        if parameter < knots[mid] {
            high = mid;
        } else {
            low = mid;
        }
        mid = (low + high) / 2;
    }
    mid
}

fn deboor_with_span(
    parameter: f64,
    degree: usize,
    knots: &[f64],
    span_index: usize,
    local_control: &[H],
) -> H {
    debug_assert_eq!(local_control.len(), degree + 1);
    if degree == 0 {
        return local_control[0];
    }
    let mut work = local_control.to_vec();
    for level in 1..=degree {
        for j in (level..=degree).rev() {
            let global_i = span_index - degree + j;
            let denominator = knots[global_i + degree + 1 - level] - knots[global_i];
            let alpha = if denominator == 0.0 {
                0.0
            } else {
                (parameter - knots[global_i]) / denominator
            };
            work[j] = lerp(work[j - 1], work[j], alpha);
        }
    }
    work[degree]
}

fn eval_net(
    net: &[Vec<H>],
    degree_u: usize,
    degree_v: usize,
    knots_u: &[f64],
    knots_v: &[f64],
    u: f64,
    v: f64,
) -> Result<H, NurbsSurfaceError> {
    let nu = net.len();
    let nv = net[0].len();
    let span_u = span(u, degree_u, knots_u, nu);
    let span_v = span(v, degree_v, knots_v, nv);
    let mut rows = Vec::with_capacity(degree_v + 1);

    for j in span_v - degree_v..=span_v {
        let local_u = (span_u - degree_u..=span_u)
            .map(|i| net[i][j])
            .collect::<Vec<_>>();
        rows.push(deboor_with_span(
            u,
            degree_u,
            knots_u,
            span_u,
            &local_u,
        ));
    }

    let result = deboor_with_span(v, degree_v, knots_v, span_v, &rows);
    if finite(result) {
        Ok(result)
    } else {
        Err(NurbsSurfaceError::Overflow)
    }
}

fn base_net(
    surface: &NurbsSurface2D,
    weight_scale: f64,
) -> Result<Vec<Vec<H>>, NurbsSurfaceError> {
    let nu = control_count(&surface.knots_u, surface.degree_u);
    let nv = control_count(&surface.knots_v, surface.degree_v);
    let mut net = Vec::with_capacity(nu);
    for i in 0..nu {
        let mut row = Vec::with_capacity(nv);
        for j in 0..nv {
            let index = i * nv + j;
            let point = surface.control_points[index];
            let weight = surface.weights[index] / weight_scale;
            let h = H {
                x: point.x * weight,
                y: point.y * weight,
                z: point.z * weight,
                w: weight,
            };
            if !finite(h) {
                return Err(NurbsSurfaceError::Overflow);
            }
            row.push(h);
        }
        net.push(row);
    }
    Ok(net)
}

fn derivative_u_net(
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
        let denominator = knots[i + degree + 1] - knots[i + 1];
        if denominator == 0.0 {
            return Err(NurbsSurfaceError::InvalidDomain);
        }
        let factor = degree as f64 / denominator;
        for j in 0..nv {
            let a = net[i][j];
            let b = net[i + 1][j];
            let h = H {
                x: (b.x - a.x) * factor,
                y: (b.y - a.y) * factor,
                z: (b.z - a.z) * factor,
                w: (b.w - a.w) * factor,
            };
            if !finite(h) {
                return Err(NurbsSurfaceError::Overflow);
            }
            out[i][j] = h;
        }
    }
    Ok((out, degree - 1, knots[1..knots.len() - 1].to_vec()))
}

fn derivative_v_net(
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
            let denominator = knots[j + degree + 1] - knots[j + 1];
            if denominator == 0.0 {
                return Err(NurbsSurfaceError::InvalidDomain);
            }
            let factor = degree as f64 / denominator;
            let a = net[i][j];
            let b = net[i][j + 1];
            let h = H {
                x: (b.x - a.x) * factor,
                y: (b.y - a.y) * factor,
                z: (b.z - a.z) * factor,
                w: (b.w - a.w) * factor,
            };
            if !finite(h) {
                return Err(NurbsSurfaceError::Overflow);
            }
            out[i][j] = h;
        }
    }
    Ok((out, degree - 1, knots[1..knots.len() - 1].to_vec()))
}

fn quotient_first(base: H, derivative: H) -> Result<Point3, NurbsSurfaceError> {
    if base.w <= 0.0 {
        return Err(NurbsSurfaceError::ZeroProjectiveWeight);
    }
    let w2 = base.w * base.w;
    if !w2.is_finite() || w2 == 0.0 {
        return Err(NurbsSurfaceError::Overflow);
    }
    let result = Point3 {
        x: (derivative.x * base.w - base.x * derivative.w) / w2,
        y: (derivative.y * base.w - base.y * derivative.w) / w2,
        z: (derivative.z * base.w - base.z * derivative.w) / w2,
    };
    if result.x.is_finite() && result.y.is_finite() && result.z.is_finite() {
        Ok(result)
    } else {
        Err(NurbsSurfaceError::Overflow)
    }
}

fn quotient_second(base: H, first: H, second: H) -> Result<Point3, NurbsSurfaceError> {
    if base.w <= 0.0 {
        return Err(NurbsSurfaceError::ZeroProjectiveWeight);
    }
    let w2 = base.w * base.w;
    let w3 = w2 * base.w;
    if !w2.is_finite() || !w3.is_finite() || w3 == 0.0 {
        return Err(NurbsSurfaceError::Overflow);
    }
    let result = Point3 {
        x: (second.x * w2 - base.x * base.w * second.w
            - 2.0 * first.x * base.w * first.w
            + 2.0 * base.x * first.w * first.w)
            / w3,
        y: (second.y * w2 - base.y * base.w * second.w
            - 2.0 * first.y * base.w * first.w
            + 2.0 * base.y * first.w * first.w)
            / w3,
        z: (second.z * w2 - base.z * base.w * second.w
            - 2.0 * first.z * base.w * first.w
            + 2.0 * base.z * first.w * first.w)
            / w3,
    };
    if result.x.is_finite() && result.y.is_finite() && result.z.is_finite() {
        Ok(result)
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
    let base = base_net(surface, weight_scale)?;
    let base_h = eval_net(
        &base,
        surface.degree_u,
        surface.degree_v,
        &surface.knots_u,
        &surface.knots_v,
        u,
        v,
    )?;

    let (du_net, du_degree, du_knots) = derivative_u_net(&base, surface.degree_u, &surface.knots_u)?;
    let du_h = eval_net(
        &du_net,
        du_degree,
        surface.degree_v,
        &du_knots,
        &surface.knots_v,
        u,
        v,
    )?;

    let (dv_net, dv_degree, dv_knots) = derivative_v_net(&base, surface.degree_v, &surface.knots_v)?;
    let dv_h = eval_net(
        &dv_net,
        surface.degree_u,
        dv_degree,
        &surface.knots_u,
        &dv_knots,
        u,
        v,
    )?;

    let du = quotient_first(base_h, du_h)?;
    let dv = quotient_first(base_h, dv_h)?;

    let duu_h = if surface.degree_u >= 2 {
        let (second_net, second_degree, second_knots) =
            derivative_u_net(&du_net, du_degree, &du_knots)?;
        eval_net(
            &second_net,
            second_degree,
            surface.degree_v,
            &second_knots,
            &surface.knots_v,
            u,
            v,
        )?
    } else {
        zero_h()
    };

    let dvv_h = if surface.degree_v >= 2 {
        let (second_net, second_degree, second_knots) =
            derivative_v_net(&dv_net, dv_degree, &dv_knots)?;
        eval_net(
            &second_net,
            surface.degree_u,
            second_degree,
            &surface.knots_u,
            &second_knots,
            u,
            v,
        )?
    } else {
        zero_h()
    };

    let duv_h = {
        let (mixed_net, mixed_degree_v, mixed_knots_v) =
            derivative_v_net(&du_net, surface.degree_v, &surface.knots_v)?;
        eval_net(
            &mixed_net,
            du_degree,
            mixed_degree_v,
            &du_knots,
            &mixed_knots_v,
            u,
            v,
        )?
    };

    let duu = quotient_second(base_h, du_h, duu_h)?;
    let dvv = quotient_second(base_h, dv_h, dvv_h)?;

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

    Ok(SurfaceDerivatives {
        du,
        dv,
        duu,
        duv,
        dvv,
    })
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
    let normal_raw = Point3 {
        x: d.du.y * d.dv.z - d.du.z * d.dv.y,
        y: d.du.z * d.dv.x - d.du.x * d.dv.z,
        z: d.du.x * d.dv.y - d.du.y * d.dv.x,
    };
    let normal_length = normal_raw
        .x
        .hypot(normal_raw.y)
        .hypot(normal_raw.z);
    if !normal_length.is_finite() {
        return Err(NurbsSurfaceError::Overflow);
    }
    if normal_length == 0.0 {
        return Err(NurbsSurfaceError::Degenerate);
    }
    let normal = Point3 {
        x: normal_raw.x / normal_length,
        y: normal_raw.y / normal_length,
        z: normal_raw.z / normal_length,
    };

    let first_e = d.du.x * d.du.x + d.du.y * d.du.y + d.du.z * d.du.z;
    let first_f = d.du.x * d.dv.x + d.du.y * d.dv.y + d.du.z * d.dv.z;
    let first_g = d.dv.x * d.dv.x + d.dv.y * d.dv.y + d.dv.z * d.dv.z;
    let second_e = normal.x * d.duu.x + normal.y * d.duu.y + normal.z * d.duu.z;
    let second_f = normal.x * d.duv.x + normal.y * d.duv.y + normal.z * d.duv.z;
    let second_g = normal.x * d.dvv.x + normal.y * d.dvv.y + normal.z * d.dvv.z;
    let determinant = first_e * first_g - first_f * first_f;

    if ![
        first_e,
        first_f,
        first_g,
        second_e,
        second_f,
        second_g,
        determinant,
    ]
    .iter()
    .all(|value| value.is_finite())
    {
        return Err(NurbsSurfaceError::Overflow);
    }
    if determinant <= 0.0 {
        return Err(NurbsSurfaceError::Degenerate);
    }

    let gaussian = (second_e * second_g - second_f * second_f) / determinant;
    let mean =
        (second_e * first_g + second_g * first_e - 2.0 * second_f * first_f)
            / (2.0 * determinant);
    if !gaussian.is_finite() || !mean.is_finite() {
        return Err(NurbsSurfaceError::Overflow);
    }

    let trace_numerator = second_e * first_g + second_g * first_e - 2.0 * second_f * first_f;
    let discriminant_numerator = trace_numerator * trace_numerator
        - 4.0 * determinant * (second_e * second_g - second_f * second_f);
    if !discriminant_numerator.is_finite() {
        return Err(NurbsSurfaceError::Overflow);
    }
    let tolerance_band = tolerance
        * (trace_numerator
            .abs()
            .max((4.0 * determinant * (second_e * second_g - second_f * second_f)).abs())
            .max(f64::MIN_POSITIVE));
    if !tolerance_band.is_finite() {
        return Err(NurbsSurfaceError::Overflow);
    }
    let discriminant = if discriminant_numerator < 0.0 && discriminant_numerator >= -tolerance_band {
        0.0
    } else if discriminant_numerator < 0.0 {
        return Err(NurbsSurfaceError::Degenerate);
    } else {
        discriminant_numerator
    };

    let denominator = 2.0 * determinant;
    if !denominator.is_finite() || denominator == 0.0 {
        return Err(NurbsSurfaceError::Overflow);
    }
    let root = discriminant.sqrt();
    let principal_a = (trace_numerator - root) / denominator;
    let principal_b = (trace_numerator + root) / denominator;
    if !principal_a.is_finite() || !principal_b.is_finite() {
        return Err(NurbsSurfaceError::Overflow);
    }

    Ok(CurvatureData {
        gaussian,
        mean,
        principal_min: principal_a.min(principal_b),
        principal_max: principal_a.max(principal_b),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn planar_patch(weights: Vec<f64>) -> NurbsSurface2D {
        NurbsSurface2D::new(
            1,
            1,
            vec![
                Point3 { x: 0.0, y: 0.0, z: 0.0 },
                Point3 { x: 0.0, y: 1.0, z: 0.0 },
                Point3 { x: 1.0, y: 0.0, z: 0.0 },
                Point3 { x: 1.0, y: 1.0, z: 0.0 },
            ],
            weights,
            vec![0.0, 0.0, 1.0, 1.0],
            vec![0.0, 0.0, 1.0, 1.0],
        )
    }

    fn assert_close(actual: Point3, expected: Point3, tolerance: f64) {
        assert!((actual.x - expected.x).abs() <= tolerance);
        assert!((actual.y - expected.y).abs() <= tolerance);
        assert!((actual.z - expected.z).abs() <= tolerance);
    }

    #[test]
    fn planar_patch_has_known_first_and_second_differentials() {
        let surface = planar_patch(vec![1.0; 4]);
        let derivatives = derivatives_at(&surface, 0.3, 0.7).unwrap();
        assert_close(
            derivatives.du,
            Point3 { x: 1.0, y: 0.0, z: 0.0 },
            1.0e-12,
        );
        assert_close(
            derivatives.dv,
            Point3 { x: 0.0, y: 1.0, z: 0.0 },
            1.0e-12,
        );
        assert_close(
            derivatives.duu,
            Point3 { x: 0.0, y: 0.0, z: 0.0 },
            1.0e-12,
        );
        assert_close(
            derivatives.duv,
            Point3 { x: 0.0, y: 0.0, z: 0.0 },
            1.0e-12,
        );
        assert_close(
            derivatives.dvv,
            Point3 { x: 0.0, y: 0.0, z: 0.0 },
            1.0e-12,
        );
    }

    #[test]
    fn planar_patch_has_zero_curvature() {
        let curvature = curvature_at(&planar_patch(vec![1.0; 4]), 0.5, 0.5, 1.0e-12).unwrap();
        assert!(curvature.gaussian.abs() <= 1.0e-12);
        assert!(curvature.mean.abs() <= 1.0e-12);
        assert!(curvature.principal_min.abs() <= 1.0e-12);
        assert!(curvature.principal_max.abs() <= 1.0e-12);
    }

    #[test]
    fn differential_geometry_is_invariant_under_extreme_uniform_weight_scaling() {
        for scale in [1.0e-200, 1.0e200] {
            let surface = planar_patch(vec![scale; 4]);
            let derivatives = derivatives_at(&surface, 0.3, 0.7).unwrap();
            assert_close(
                derivatives.du,
                Point3 { x: 1.0, y: 0.0, z: 0.0 },
                1.0e-12,
            );
            assert_close(
                derivatives.dv,
                Point3 { x: 0.0, y: 1.0, z: 0.0 },
                1.0e-12,
            );
            assert_close(
                derivatives.duu,
                Point3 { x: 0.0, y: 0.0, z: 0.0 },
                1.0e-12,
            );
            assert_close(
                derivatives.duv,
                Point3 { x: 0.0, y: 0.0, z: 0.0 },
                1.0e-12,
            );
            assert_close(
                derivatives.dvv,
                Point3 { x: 0.0, y: 0.0, z: 0.0 },
                1.0e-12,
            );
            let curvature = curvature_at(&surface, 0.3, 0.7, 1.0e-12).unwrap();
            assert!(curvature.gaussian.abs() <= 1.0e-12);
            assert!(curvature.mean.abs() <= 1.0e-12);
        }
    }
}
