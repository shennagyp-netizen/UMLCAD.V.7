use super::nurbs_surface::{NurbsSurface2D, NurbsSurfaceError, Point3};

pub trait NurbsSurfaceDifferential {
    fn derivative_u_at(&self, u: f64, v: f64) -> Result<Point3, NurbsSurfaceError>;
    fn derivative_v_at(&self, u: f64, v: f64) -> Result<Point3, NurbsSurfaceError>;
    fn normal_at(&self, u: f64, v: f64) -> Result<Point3, NurbsSurfaceError>;
}

impl NurbsSurfaceDifferential for NurbsSurface2D {
    fn derivative_u_at(&self, u: f64, v: f64) -> Result<Point3, NurbsSurfaceError> {
        let base = evaluate_homogeneous(self, u, v)?;
        let derivative = evaluate_homogeneous_derivative_u(self, u, v)?;
        quotient_derivative(base, derivative)
    }

    fn derivative_v_at(&self, u: f64, v: f64) -> Result<Point3, NurbsSurfaceError> {
        let base = evaluate_homogeneous(self, u, v)?;
        let derivative = evaluate_homogeneous_derivative_v(self, u, v)?;
        quotient_derivative(base, derivative)
    }

    fn normal_at(&self, u: f64, v: f64) -> Result<Point3, NurbsSurfaceError> {
        let du = self.derivative_u_at(u, v)?;
        let dv = self.derivative_v_at(u, v)?;
        let cross = Point3 {
            x: du.y * dv.z - du.z * dv.y,
            y: du.z * dv.x - du.x * dv.z,
            z: du.x * dv.y - du.y * dv.x,
        };
        let magnitude = cross.x.hypot(cross.y.hypot(cross.z));
        if !magnitude.is_finite() {
            return Err(NurbsSurfaceError::Overflow);
        }
        if magnitude == 0.0 {
            return Err(NurbsSurfaceError::Degenerate);
        }
        Ok(Point3 {
            x: cross.x / magnitude,
            y: cross.y / magnitude,
            z: cross.z / magnitude,
        })
    }
}

#[derive(Clone, Copy)]
struct HomogeneousPoint {
    xw: f64,
    yw: f64,
    zw: f64,
    w: f64,
}

impl HomogeneousPoint {
    fn lerp(self, other: Self, alpha: f64) -> Self {
        Self {
            xw: self.xw * (1.0 - alpha) + other.xw * alpha,
            yw: self.yw * (1.0 - alpha) + other.yw * alpha,
            zw: self.zw * (1.0 - alpha) + other.zw * alpha,
            w: self.w * (1.0 - alpha) + other.w * alpha,
        }
    }
}

fn validate_query(surface: &NurbsSurface2D, u: f64, v: f64) -> Result<(), NurbsSurfaceError> {
    surface.validate()?;
    if !u.is_finite() || !v.is_finite() {
        return Err(NurbsSurfaceError::NonFinite);
    }
    let control_u = control_count_u(surface);
    let control_v = control_count_v(surface);
    let u0 = surface.knots_u[surface.degree_u];
    let u1 = surface.knots_u[control_u];
    let v0 = surface.knots_v[surface.degree_v];
    let v1 = surface.knots_v[control_v];
    if u < u0 || u > u1 || v < v0 || v > v1 {
        return Err(NurbsSurfaceError::OutOfDomain);
    }
    Ok(())
}

fn control_count_u(surface: &NurbsSurface2D) -> usize {
    surface.knots_u.len() - surface.degree_u - 1
}

fn control_count_v(surface: &NurbsSurface2D) -> usize {
    surface.knots_v.len() - surface.degree_v - 1
}

fn span(t: f64, degree: usize, knots: &[f64], count: usize) -> usize {
    let last = count - 1;
    if t >= knots[count] {
        return last;
    }
    if t <= knots[degree] {
        return degree;
    }
    let mut low = degree;
    let mut high = count;
    let mut mid = (low + high) / 2;
    while t < knots[mid] || t >= knots[mid + 1] {
        if t < knots[mid] {
            high = mid;
        } else {
            low = mid;
        }
        mid = (low + high) / 2;
    }
    mid
}

fn de_boor(t: f64, span: usize, degree: usize, knots: &[f64], work: &mut [HomogeneousPoint]) -> HomogeneousPoint {
    if degree == 0 {
        return work[0];
    }
    for level in 1..=degree {
        for j in (level..=degree).rev() {
            let i = span - degree + j;
            let denominator = knots[i + degree + 1 - level] - knots[i];
            let alpha = if denominator == 0.0 { 0.0 } else { (t - knots[i]) / denominator };
            work[j] = work[j - 1].lerp(work[j], alpha);
        }
    }
    work[degree]
}

fn evaluate_homogeneous(surface: &NurbsSurface2D, u: f64, v: f64) -> Result<HomogeneousPoint, NurbsSurfaceError> {
    validate_query(surface, u, v)?;
    let control_u = control_count_u(surface);
    let control_v = control_count_v(surface);
    let span_u = span(u, surface.degree_u, &surface.knots_u, control_u);
    let mut rows = Vec::with_capacity(control_v);
    for v_index in 0..control_v {
        let mut work = Vec::with_capacity(surface.degree_u + 1);
        for local in 0..=surface.degree_u {
            let u_index = span_u - surface.degree_u + local;
            let index = u_index * control_v + v_index;
            let point = surface.control_points[index];
            let weight = surface.weights[index];
            work.push(HomogeneousPoint {
                xw: point.x * weight,
                yw: point.y * weight,
                zw: point.z * weight,
                w: weight,
            });
        }
        rows.push(de_boor(u, span_u, surface.degree_u, &surface.knots_u, &mut work));
    }
    let span_v = span(v, surface.degree_v, &surface.knots_v, control_v);
    let mut work = Vec::with_capacity(surface.degree_v + 1);
    for local in 0..=surface.degree_v {
        work.push(rows[span_v - surface.degree_v + local]);
    }
    Ok(de_boor(v, span_v, surface.degree_v, &surface.knots_v, &mut work))
}

fn evaluate_homogeneous_derivative_u(surface: &NurbsSurface2D, u: f64, v: f64) -> Result<HomogeneousPoint, NurbsSurfaceError> {
    validate_query(surface, u, v)?;
    if surface.degree_u == 0 {
        return Err(NurbsSurfaceError::InvalidDegree);
    }
    let control_u = control_count_u(surface);
    let control_v = control_count_v(surface);
    let derivative_count = control_u - 1;
    let derivative_knots = &surface.knots_u[1..surface.knots_u.len() - 1];
    let derivative_degree = surface.degree_u - 1;
    let span_u = span(u, derivative_degree, derivative_knots, derivative_count);
    let degree = surface.degree_u as f64;
    let mut rows = Vec::with_capacity(control_v);
    for v_index in 0..control_v {
        let mut work = Vec::with_capacity(surface.degree_u);
        for local in 0..surface.degree_u {
            let u_index = span_u - derivative_degree + local;
            let a = u_index * control_v + v_index;
            let b = (u_index + 1) * control_v + v_index;
            let pa = surface.control_points[a];
            let pb = surface.control_points[b];
            let wa = surface.weights[a];
            let wb = surface.weights[b];
            let denominator = surface.knots_u[u_index + surface.degree_u + 1] - surface.knots_u[u_index + 1];
            if denominator == 0.0 {
                return Err(NurbsSurfaceError::InvalidDomain);
            }
            let factor = degree / denominator;
            work.push(HomogeneousPoint {
                xw: (pb.x * wb - pa.x * wa) * factor,
                yw: (pb.y * wb - pa.y * wa) * factor,
                zw: (pb.z * wb - pa.z * wa) * factor,
                w: (wb - wa) * factor,
            });
        }
        rows.push(de_boor(u, span_u, derivative_degree, derivative_knots, &mut work));
    }
    let span_v = span(v, surface.degree_v, &surface.knots_v, control_v);
    let mut work = Vec::with_capacity(surface.degree_v + 1);
    for local in 0..=surface.degree_v {
        work.push(rows[span_v - surface.degree_v + local]);
    }
    Ok(de_boor(v, span_v, surface.degree_v, &surface.knots_v, &mut work))
}

fn evaluate_homogeneous_derivative_v(surface: &NurbsSurface2D, u: f64, v: f64) -> Result<HomogeneousPoint, NurbsSurfaceError> {
    validate_query(surface, u, v)?;
    if surface.degree_v == 0 {
        return Err(NurbsSurfaceError::InvalidDegree);
    }
    let control_u = control_count_u(surface);
    let control_v = control_count_v(surface);
    let derivative_count = control_v - 1;
    let derivative_knots = &surface.knots_v[1..surface.knots_v.len() - 1];
    let derivative_degree = surface.degree_v - 1;
    let span_v = span(v, derivative_degree, derivative_knots, derivative_count);
    let degree = surface.degree_v as f64;
    let span_u = span(u, surface.degree_u, &surface.knots_u, control_u);
    let mut columns = Vec::with_capacity(control_u);
    for u_index in 0..control_u {
        let mut work = Vec::with_capacity(surface.degree_v);
        for local in 0..surface.degree_v {
            let v_index = span_v - derivative_degree + local;
            let a = u_index * control_v + v_index;
            let b = u_index * control_v + v_index + 1;
            let pa = surface.control_points[a];
            let pb = surface.control_points[b];
            let wa = surface.weights[a];
            let wb = surface.weights[b];
            let denominator = surface.knots_v[v_index + surface.degree_v + 1] - surface.knots_v[v_index + 1];
            if denominator == 0.0 {
                return Err(NurbsSurfaceError::InvalidDomain);
            }
            let factor = degree / denominator;
            work.push(HomogeneousPoint {
                xw: (pb.x * wb - pa.x * wa) * factor,
                yw: (pb.y * wb - pa.y * wa) * factor,
                zw: (pb.z * wb - pa.z * wa) * factor,
                w: (wb - wa) * factor,
            });
        }
        columns.push(de_boor(v, span_v, derivative_degree, derivative_knots, &mut work));
    }
    let mut work = Vec::with_capacity(surface.degree_u + 1);
    for local in 0..=surface.degree_u {
        work.push(columns[span_u - surface.degree_u + local]);
    }
    Ok(de_boor(u, span_u, surface.degree_u, &surface.knots_u, &mut work))
}

fn quotient_derivative(base: HomogeneousPoint, derivative: HomogeneousPoint) -> Result<Point3, NurbsSurfaceError> {
    let values = [base.xw, base.yw, base.zw, base.w, derivative.xw, derivative.yw, derivative.zw, derivative.w];
    if values.iter().any(|value| !value.is_finite()) {
        return Err(NurbsSurfaceError::Overflow);
    }
    if base.w <= 0.0 {
        return Err(NurbsSurfaceError::ZeroProjectiveWeight);
    }
    let denominator = base.w * base.w;
    if !denominator.is_finite() || denominator == 0.0 {
        return Err(NurbsSurfaceError::Overflow);
    }
    let result = Point3 {
        x: (derivative.xw * base.w - base.xw * derivative.w) / denominator,
        y: (derivative.yw * base.w - base.yw * derivative.w) / denominator,
        z: (derivative.zw * base.w - base.zw * derivative.w) / denominator,
    };
    if !result.x.is_finite() || !result.y.is_finite() || !result.z.is_finite() {
        return Err(NurbsSurfaceError::Overflow);
    }
    Ok(result)
}
