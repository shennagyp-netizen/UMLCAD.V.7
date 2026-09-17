use super::{NurbsSurface3DDefinition, NurbsSurfaceDefinitionError, Point3};

#[derive(Clone, Copy, Debug, PartialEq)]
struct Homogeneous { xw: f64, yw: f64, zw: f64, w: f64 }

impl Homogeneous {
    fn lerp(self, other: Self, a: f64) -> Self {
        let b = 1.0 - a;
        Self { xw: self.xw*b + other.xw*a, yw: self.yw*b + other.yw*a, zw: self.zw*b + other.zw*a, w: self.w*b + other.w*a }
    }
}

fn span(t: f64, degree: usize, knots: &[f64], count: usize) -> usize {
    if t >= knots[count] { return count - 1; }
    if t <= knots[degree] { return degree; }
    let mut low = degree;
    let mut high = count;
    let mut mid = (low + high) / 2;
    while t < knots[mid] || t >= knots[mid + 1] {
        if t < knots[mid] { high = mid; } else { low = mid; }
        mid = (low + high) / 2;
    }
    mid
}

fn de_boor(t: f64, s: usize, p: usize, knots: &[f64], work: &mut [Homogeneous]) -> Homogeneous {
    if p == 0 { return work[0]; }
    for level in 1..=p {
        for j in (level..=p).rev() {
            let i = s - p + j;
            let d = knots[i + p + 1 - level] - knots[i];
            let a = if d == 0.0 { 0.0 } else { (t - knots[i]) / d };
            work[j] = work[j - 1].lerp(work[j], a);
        }
    }
    work[p]
}

pub fn point_at(surface: &NurbsSurface3DDefinition, u: f64, v: f64) -> Result<Point3, NurbsSurfaceDefinitionError> {
    surface.validate()?;
    let ((u0,u1),(v0,v1)) = surface.parameter_domain()?;
    if !u.is_finite() || !v.is_finite() || u < u0 || u > u1 || v < v0 || v > v1 { return Err(NurbsSurfaceDefinitionError::InvalidDomain); }
    let su = span(u, surface.degree_u, &surface.knots_u, surface.count_u);
    let mut rows = Vec::with_capacity(surface.count_v);
    for j in 0..surface.count_v {
        let mut work = Vec::with_capacity(surface.degree_u + 1);
        for local in 0..=surface.degree_u {
            let i = su - surface.degree_u + local;
            let idx = i * surface.count_v + j;
            let p = surface.control_points[idx]; let w = surface.weights[idx];
            work.push(Homogeneous { xw:p.x*w, yw:p.y*w, zw:p.z*w, w });
        }
        rows.push(de_boor(u, su, surface.degree_u, &surface.knots_u, &mut work));
    }
    let sv = span(v, surface.degree_v, &surface.knots_v, surface.count_v);
    let mut work = Vec::with_capacity(surface.degree_v + 1);
    for local in 0..=surface.degree_v { work.push(rows[sv - surface.degree_v + local]); }
    let h = de_boor(v, sv, surface.degree_v, &surface.knots_v, &mut work);
    if h.w <= 0.0 || !h.w.is_finite() { return Err(NurbsSurfaceDefinitionError::InvalidWeight); }
    Ok(Point3 { x:h.xw/h.w, y:h.yw/h.w, z:h.zw/h.w })
}
