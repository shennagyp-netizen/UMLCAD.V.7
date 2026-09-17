//! Analytic differential operations for the existing B-spline/NURBS curves.
//!
//! Derivatives are obtained from differentiated control nets rather than finite
//! differences. Rational derivatives apply the quotient rule to internally
//! normalized homogeneous coordinates, so uniform weight scaling cannot cause
//! avoidable overflow.

use super::{bspline::BSplineCurve2D, nurbs::NurbsCurve2D};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DifferentialError {
    NonFinite,
    InvalidDomain,
    Degenerate,
    Singular,
    Overflow,
}

fn bspline_eval(
    degree: usize,
    control: &[(f64, f64)],
    knots: &[f64],
    u: f64,
) -> Result<(f64, f64), DifferentialError> {
    if degree == 0 || control.len() < degree + 1 || knots.len() != control.len() + degree + 1 {
        return Err(DifferentialError::InvalidDomain);
    }
    let n = control.len() - 1;
    let start = knots[degree];
    let end = knots[n + 1];
    if !u.is_finite() || u < start || u > end {
        return Err(DifferentialError::InvalidDomain);
    }
    let span = if u >= end {
        n
    } else {
        let mut lo = degree;
        let mut hi = n + 1;
        let mut mid = (lo + hi) / 2;
        while u < knots[mid] || u >= knots[mid + 1] {
            if u < knots[mid] {
                hi = mid;
            } else {
                lo = mid;
            }
            mid = (lo + hi) / 2;
        }
        mid
    };
    let mut work = control[span - degree..=span].to_vec();
    for level in 1..=degree {
        for j in (level..=degree).rev() {
            let i = span - degree + j;
            let den = knots[i + degree + 1 - level] - knots[i];
            let a = if den == 0.0 { 0.0 } else { (u - knots[i]) / den };
            if !a.is_finite() {
                return Err(DifferentialError::Overflow);
            }
            work[j] = (
                work[j - 1].0 * (1.0 - a) + work[j].0 * a,
                work[j - 1].1 * (1.0 - a) + work[j].1 * a,
            );
        }
    }
    let p = work[degree];
    if p.0.is_finite() && p.1.is_finite() {
        Ok(p)
    } else {
        Err(DifferentialError::Overflow)
    }
}

fn derivative_control(
    degree: usize,
    control: &[(f64, f64)],
    knots: &[f64],
) -> Result<(usize, Vec<(f64, f64)>, Vec<f64>), DifferentialError> {
    if degree == 0 || control.len() < 2 {
        return Err(DifferentialError::Degenerate);
    }
    if knots.len() != control.len() + degree + 1 {
        return Err(DifferentialError::InvalidDomain);
    }
    let mut result = Vec::with_capacity(control.len() - 1);
    for i in 0..control.len() - 1 {
        let den = knots[i + degree + 1] - knots[i + 1];
        if den == 0.0 {
            return Err(DifferentialError::Singular);
        }
        let f = degree as f64 / den;
        let q = (
            (control[i + 1].0 - control[i].0) * f,
            (control[i + 1].1 - control[i].1) * f,
        );
        if !q.0.is_finite() || !q.1.is_finite() {
            return Err(DifferentialError::Overflow);
        }
        result.push(q);
    }
    Ok((degree - 1, result, knots[1..knots.len() - 1].to_vec()))
}

pub fn bspline_derivative(
    curve: &BSplineCurve2D,
    u: f64,
) -> Result<(f64, f64), DifferentialError> {
    curve
        .validate()
        .map_err(|_| DifferentialError::InvalidDomain)?;
    let control: Vec<_> = curve.control_points.iter().map(|p| (p.x, p.y)).collect();
    let (degree, control, knots) = derivative_control(curve.degree, &control, &curve.knots)?;
    bspline_eval(degree, &control, &knots, u)
}

pub fn bspline_second_derivative(
    curve: &BSplineCurve2D,
    u: f64,
) -> Result<(f64, f64), DifferentialError> {
    curve
        .validate()
        .map_err(|_| DifferentialError::InvalidDomain)?;
    let control: Vec<_> = curve.control_points.iter().map(|p| (p.x, p.y)).collect();
    let (degree1, control1, knots1) = derivative_control(curve.degree, &control, &curve.knots)?;
    let (degree2, control2, knots2) = derivative_control(degree1, &control1, &knots1)?;
    bspline_eval(degree2, &control2, &knots2, u)
}

#[derive(Clone, Copy)]
struct H {
    x: f64,
    y: f64,
    w: f64,
}

fn deboor_h(
    degree: usize,
    control: &[H],
    knots: &[f64],
    u: f64,
) -> Result<H, DifferentialError> {
    if control.len() < degree + 1 || knots.len() != control.len() + degree + 1 {
        return Err(DifferentialError::InvalidDomain);
    }
    let n = control.len() - 1;
    let start = knots[degree];
    let end = knots[n + 1];
    if !u.is_finite() || u < start || u > end {
        return Err(DifferentialError::InvalidDomain);
    }
    let span = if u >= end {
        n
    } else {
        let mut lo = degree;
        let mut hi = n + 1;
        let mut mid = (lo + hi) / 2;
        while u < knots[mid] || u >= knots[mid + 1] {
            if u < knots[mid] {
                hi = mid;
            } else {
                lo = mid;
            }
            mid = (lo + hi) / 2;
        }
        mid
    };
    let mut work = control[span - degree..=span].to_vec();
    for level in 1..=degree {
        for j in (level..=degree).rev() {
            let i = span - degree + j;
            let den = knots[i + degree + 1 - level] - knots[i];
            let a = if den == 0.0 { 0.0 } else { (u - knots[i]) / den };
            if !a.is_finite() {
                return Err(DifferentialError::Overflow);
            }
            let p = work[j - 1];
            let q = work[j];
            work[j] = H {
                x: p.x * (1.0 - a) + q.x * a,
                y: p.y * (1.0 - a) + q.y * a,
                w: p.w * (1.0 - a) + q.w * a,
            };
        }
    }
    let p = work[degree];
    if p.x.is_finite() && p.y.is_finite() && p.w.is_finite() {
        Ok(p)
    } else {
        Err(DifferentialError::Overflow)
    }
}

fn h_derivative_control(
    degree: usize,
    control: &[H],
    knots: &[f64],
) -> Result<(usize, Vec<H>, Vec<f64>), DifferentialError> {
    if degree == 0 || control.len() < 2 {
        return Err(DifferentialError::Degenerate);
    }
    if knots.len() != control.len() + degree + 1 {
        return Err(DifferentialError::InvalidDomain);
    }
    let mut next = Vec::with_capacity(control.len() - 1);
    for i in 0..control.len() - 1 {
        let den = knots[i + degree + 1] - knots[i + 1];
        if den == 0.0 {
            return Err(DifferentialError::Singular);
        }
        let f = degree as f64 / den;
        let p = control[i];
        let q = control[i + 1];
        let r = H {
            x: (q.x - p.x) * f,
            y: (q.y - p.y) * f,
            w: (q.w - p.w) * f,
        };
        if !r.x.is_finite() || !r.y.is_finite() || !r.w.is_finite() {
            return Err(DifferentialError::Overflow);
        }
        next.push(r);
    }
    Ok((degree - 1, next, knots[1..knots.len() - 1].to_vec()))
}

fn normalized_weight_scale(curve: &NurbsCurve2D) -> Result<f64, DifferentialError> {
    let scale = curve.weights.iter().copied().fold(0.0, f64::max);
    if !scale.is_finite() || scale <= 0.0 {
        return Err(DifferentialError::Singular);
    }
    Ok(scale)
}

fn nurbs_homogeneous_derivative(
    curve: &NurbsCurve2D,
    order: usize,
    u: f64,
) -> Result<H, DifferentialError> {
    curve
        .validate()
        .map_err(|_| DifferentialError::InvalidDomain)?;
    if order > curve.degree {
        return Ok(H {
            x: 0.0,
            y: 0.0,
            w: 0.0,
        });
    }
    let scale = normalized_weight_scale(curve)?;
    let mut degree = curve.degree;
    let mut control: Vec<H> = curve
        .control_points
        .iter()
        .zip(curve.weights.iter())
        .map(|(p, weight)| {
            let w = *weight / scale;
            H {
                x: p.x * w,
                y: p.y * w,
                w,
            }
        })
        .collect();
    if control.iter().any(|p| !p.x.is_finite() || !p.y.is_finite() || !p.w.is_finite()) {
        return Err(DifferentialError::Overflow);
    }
    let mut knots = curve.knots.clone();
    for _ in 0..order {
        let (next_degree, next, next_knots) = h_derivative_control(degree, &control, &knots)?;
        degree = next_degree;
        control = next;
        knots = next_knots;
    }
    deboor_h(degree, &control, &knots, u)
}

pub fn nurbs_derivative(
    curve: &NurbsCurve2D,
    u: f64,
) -> Result<(f64, f64), DifferentialError> {
    let p = nurbs_homogeneous_derivative(curve, 0, u)?;
    let d = nurbs_homogeneous_derivative(curve, 1, u)?;
    if p.w <= 0.0 || !p.w.is_finite() {
        return Err(DifferentialError::Singular);
    }
    let px = p.x / p.w;
    let py = p.y / p.w;
    let wx = d.w / p.w;
    let x = d.x / p.w - px * wx;
    let y = d.y / p.w - py * wx;
    if x.is_finite() && y.is_finite() {
        Ok((x, y))
    } else {
        Err(DifferentialError::Overflow)
    }
}

pub fn nurbs_second_derivative(
    curve: &NurbsCurve2D,
    u: f64,
) -> Result<(f64, f64), DifferentialError> {
    let p = nurbs_homogeneous_derivative(curve, 0, u)?;
    let d = nurbs_homogeneous_derivative(curve, 1, u)?;
    let dd = nurbs_homogeneous_derivative(curve, 2, u)?;
    if p.w <= 0.0 || !p.w.is_finite() {
        return Err(DifferentialError::Singular);
    }
    let px = p.x / p.w;
    let py = p.y / p.w;
    let w1 = d.w / p.w;
    let w2 = dd.w / p.w;
    let dx = d.x / p.w;
    let dy = d.y / p.w;
    let x = dd.x / p.w - px * w2 - 2.0 * dx * w1 + 2.0 * px * w1 * w1;
    let y = dd.y / p.w - py * w2 - 2.0 * dy * w1 + 2.0 * py * w1 * w1;
    if x.is_finite() && y.is_finite() {
        Ok((x, y))
    } else {
        Err(DifferentialError::Overflow)
    }
}

pub fn curvature_from_derivatives(
    first: (f64, f64),
    second: (f64, f64),
) -> Result<f64, DifferentialError> {
    if !first.0.is_finite()
        || !first.1.is_finite()
        || !second.0.is_finite()
        || !second.1.is_finite()
    {
        return Err(DifferentialError::NonFinite);
    }
    let speed = first.0.hypot(first.1);
    if speed == 0.0 || !speed.is_finite() {
        return Err(DifferentialError::Degenerate);
    }
    let numerator = first.0 * second.1 - first.1 * second.0;
    if !numerator.is_finite() {
        return Err(DifferentialError::Overflow);
    }
    let denominator = speed * speed * speed;
    if !denominator.is_finite() {
        return Err(DifferentialError::Overflow);
    }
    let k = numerator.abs() / denominator;
    if k.is_finite() {
        Ok(k)
    } else {
        Err(DifferentialError::Overflow)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bspline_linear_derivative_is_constant() {
        let c = BSplineCurve2D::new(
            1,
            vec![
                crate::math::bspline::Point2 { x: 0.0, y: 0.0 },
                crate::math::bspline::Point2 { x: 2.0, y: 4.0 },
            ],
            vec![0.0, 0.0, 1.0, 1.0],
        );
        assert_eq!(bspline_derivative(&c, 0.5).unwrap(), (2.0, 4.0));
    }

    #[test]
    fn bspline_quadratic_second_derivative_is_constant() {
        let c = BSplineCurve2D::new(
            2,
            vec![
                crate::math::bspline::Point2 { x: 0.0, y: 0.0 },
                crate::math::bspline::Point2 { x: 1.0, y: 1.0 },
                crate::math::bspline::Point2 { x: 2.0, y: 0.0 },
            ],
            vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
        );
        let d = bspline_second_derivative(&c, 0.5).unwrap();
        assert!(d.0.abs() < 1.0e-12 && (d.1 + 4.0).abs() < 1.0e-12);
    }

    #[test]
    fn rational_quarter_circle_derivative_is_finite() {
        let c = NurbsCurve2D::new(
            2,
            vec![
                crate::math::nurbs::Point2 { x: 1.0, y: 0.0 },
                crate::math::nurbs::Point2 { x: 1.0, y: 1.0 },
                crate::math::nurbs::Point2 { x: 0.0, y: 1.0 },
            ],
            vec![1.0, 2.0_f64.sqrt() / 2.0, 1.0],
            vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
        );
        let d = nurbs_derivative(&c, 0.5).unwrap();
        assert!(d.0.is_finite() && d.1.is_finite());
    }

    #[test]
    fn rational_derivative_is_invariant_under_uniform_weight_scaling() {
        let curve = NurbsCurve2D::new(
            2,
            vec![
                crate::math::nurbs::Point2 { x: 0.0, y: 0.0 },
                crate::math::nurbs::Point2 { x: 1.0, y: 2.0 },
                crate::math::nurbs::Point2 { x: 3.0, y: 0.0 },
            ],
            vec![1.0, 2.0, 4.0],
            vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
        );
        let scaled = NurbsCurve2D::new(
            curve.degree,
            curve.control_points.clone(),
            curve.weights.iter().map(|weight| weight * 1.0e200).collect(),
            curve.knots.clone(),
        );
        let a = nurbs_derivative(&curve, 0.5).unwrap();
        let b = nurbs_derivative(&scaled, 0.5).unwrap();
        assert!((a.0 - b.0).abs() < 1.0e-12);
        assert!((a.1 - b.1).abs() < 1.0e-12);
    }
}
