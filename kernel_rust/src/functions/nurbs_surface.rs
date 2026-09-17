#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Point3 {
    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoundingBox3 {
    pub min: Point3,
    pub max: Point3,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NurbsSurfaceError {
    NonFinite,
    InvalidDegree,
    InvalidControlPointCount,
    InvalidWeightCount,
    InvalidWeight,
    InvalidKnotCount,
    KnotsMustBeNondecreasing,
    NotClamped,
    InvalidDomain,
    OutOfDomain,
    Degenerate,
    ZeroProjectiveWeight,
    Overflow,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NurbsSurface2D {
    pub degree_u: usize,
    pub degree_v: usize,
    pub control_points: Vec<Point3>,
    pub weights: Vec<f64>,
    pub knots_u: Vec<f64>,
    pub knots_v: Vec<f64>,
}

impl NurbsSurface2D {
    pub fn new(
        degree_u: usize,
        degree_v: usize,
        control_points: Vec<Point3>,
        weights: Vec<f64>,
        knots_u: Vec<f64>,
        knots_v: Vec<f64>,
    ) -> Self {
        Self {
            degree_u,
            degree_v,
            control_points,
            weights,
            knots_u,
            knots_v,
        }
    }

    pub fn validate(&self) -> Result<(), NurbsSurfaceError> {
        if self.degree_u == 0 || self.degree_v == 0 {
            return Err(NurbsSurfaceError::InvalidDegree);
        }
        let count_u = self.control_count_u()?;
        let count_v = self.control_count_v()?;
        if count_u < self.degree_u + 1 || count_v < self.degree_v + 1 {
            return Err(NurbsSurfaceError::InvalidControlPointCount);
        }
        if count_u * count_v != self.control_points.len() {
            return Err(NurbsSurfaceError::InvalidControlPointCount);
        }
        if self.weights.len() != self.control_points.len() {
            return Err(NurbsSurfaceError::InvalidWeightCount);
        }
        if self.control_points.iter().any(|p| {
            !p.x.is_finite() || !p.y.is_finite() || !p.z.is_finite()
        }) {
            return Err(NurbsSurfaceError::NonFinite);
        }
        if self.weights.iter().any(|w| !w.is_finite()) {
            return Err(NurbsSurfaceError::NonFinite);
        }
        if self.weights.iter().any(|w| *w <= 0.0) {
            return Err(NurbsSurfaceError::InvalidWeight);
        }
        validate_knot_vector(&self.knots_u)?;
        validate_knot_vector(&self.knots_v)?;
        validate_clamped(&self.knots_u, self.degree_u, count_u)?;
        validate_clamped(&self.knots_v, self.degree_v, count_v)?;
        Ok(())
    }

    pub fn parameter_domain(&self) -> Result<(f64, f64, f64, f64), NurbsSurfaceError> {
        self.validate()?;
        let count_u = self.control_count_u_unchecked();
        let count_v = self.control_count_v_unchecked();
        Ok((
            self.knots_u[self.degree_u],
            self.knots_u[count_u],
            self.knots_v[self.degree_v],
            self.knots_v[count_v],
        ))
    }

    pub fn point_at(&self, u: f64, v: f64) -> Result<Point3, NurbsSurfaceError> {
        self.validate()?;
        if !u.is_finite() || !v.is_finite() {
            return Err(NurbsSurfaceError::NonFinite);
        }
        let (u0, u1, v0, v1) = self.parameter_domain_unchecked();
        if u < u0 || u > u1 || v < v0 || v > v1 {
            return Err(NurbsSurfaceError::OutOfDomain);
        }
        dehomogenize(self.evaluate_homogeneous(u, v))
    }

    pub fn control_hull_bounds(&self) -> Result<BoundingBox3, NurbsSurfaceError> {
        self.validate()?;
        let min = Point3 {
            x: self.control_points.iter().map(|p| p.x).fold(f64::INFINITY, f64::min),
            y: self.control_points.iter().map(|p| p.y).fold(f64::INFINITY, f64::min),
            z: self.control_points.iter().map(|p| p.z).fold(f64::INFINITY, f64::min),
        };
        let max = Point3 {
            x: self.control_points.iter().map(|p| p.x).fold(f64::NEG_INFINITY, f64::max),
            y: self.control_points.iter().map(|p| p.y).fold(f64::NEG_INFINITY, f64::max),
            z: self.control_points.iter().map(|p| p.z).fold(f64::NEG_INFINITY, f64::max),
        };
        Ok(BoundingBox3 { min, max })
    }

    pub fn translated(
        &self,
        dx: f64,
        dy: f64,
        dz: f64,
    ) -> Result<Self, NurbsSurfaceError> {
        self.validate()?;
        if !dx.is_finite() || !dy.is_finite() || !dz.is_finite() {
            return Err(NurbsSurfaceError::NonFinite);
        }
        let delta = Point3 { x: dx, y: dy, z: dz };
        let control_points = self
            .control_points
            .iter()
            .map(|point| point.add(delta))
            .collect::<Vec<_>>();
        if control_points.iter().any(|point| {
            !point.x.is_finite() || !point.y.is_finite() || !point.z.is_finite()
        }) {
            return Err(NurbsSurfaceError::Overflow);
        }
        Ok(Self::new(
            self.degree_u,
            self.degree_v,
            control_points,
            self.weights.clone(),
            self.knots_u.clone(),
            self.knots_v.clone(),
        ))
    }

    fn control_count_u(&self) -> Result<usize, NurbsSurfaceError> {
        if self.knots_u.len() < self.degree_u + 1 {
            return Err(NurbsSurfaceError::InvalidKnotCount);
        }
        Ok(self.knots_u.len() - self.degree_u - 1)
    }

    fn control_count_v(&self) -> Result<usize, NurbsSurfaceError> {
        if self.knots_v.len() < self.degree_v + 1 {
            return Err(NurbsSurfaceError::InvalidKnotCount);
        }
        Ok(self.knots_v.len() - self.degree_v - 1)
    }

    fn control_count_u_unchecked(&self) -> usize {
        self.knots_u.len() - self.degree_u - 1
    }

    fn control_count_v_unchecked(&self) -> usize {
        self.knots_v.len() - self.degree_v - 1
    }

    fn parameter_domain_unchecked(&self) -> (f64, f64, f64, f64) {
        let count_u = self.control_count_u_unchecked();
        let count_v = self.control_count_v_unchecked();
        (
            self.knots_u[self.degree_u],
            self.knots_u[count_u],
            self.knots_v[self.degree_v],
            self.knots_v[count_v],
        )
    }

    fn evaluate_homogeneous(&self, u: f64, v: f64) -> HomogeneousPoint {
        let count_u = self.control_count_u_unchecked();
        let count_v = self.control_count_v_unchecked();
        let span_u = find_span(u, self.degree_u, &self.knots_u, count_u);
        let mut rows = Vec::with_capacity(count_v);
        for v_index in 0..count_v {
            let mut work = Vec::with_capacity(self.degree_u + 1);
            for local in 0..=self.degree_u {
                let u_index = span_u - self.degree_u + local;
                let index = u_index * count_v + v_index;
                let point = self.control_points[index];
                let weight = self.weights[index];
                work.push(HomogeneousPoint {
                    xw: point.x * weight,
                    yw: point.y * weight,
                    zw: point.z * weight,
                    w: weight,
                });
            }
            rows.push(de_boor(u, span_u, self.degree_u, &self.knots_u, &mut work));
        }

        let span_v = find_span(v, self.degree_v, &self.knots_v, count_v);
        let mut work = Vec::with_capacity(self.degree_v + 1);
        for local in 0..=self.degree_v {
            work.push(rows[span_v - self.degree_v + local]);
        }
        de_boor(v, span_v, self.degree_v, &self.knots_v, &mut work)
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

fn dehomogenize(value: HomogeneousPoint) -> Result<Point3, NurbsSurfaceError> {
    if [value.xw, value.yw, value.zw, value.w]
        .iter()
        .any(|x| !x.is_finite())
    {
        return Err(NurbsSurfaceError::Overflow);
    }
    if value.w <= 0.0 {
        return Err(NurbsSurfaceError::ZeroProjectiveWeight);
    }
    let point = Point3 {
        x: value.xw / value.w,
        y: value.yw / value.w,
        z: value.zw / value.w,
    };
    if !point.x.is_finite() || !point.y.is_finite() || !point.z.is_finite() {
        return Err(NurbsSurfaceError::Overflow);
    }
    Ok(point)
}

fn validate_knot_vector(knots: &[f64]) -> Result<(), NurbsSurfaceError> {
    if knots.iter().any(|knot| !knot.is_finite()) {
        return Err(NurbsSurfaceError::NonFinite);
    }
    if knots.len() < 2 || knots.windows(2).any(|pair| pair[1] < pair[0]) {
        return Err(NurbsSurfaceError::KnotsMustBeNondecreasing);
    }
    Ok(())
}

fn validate_clamped(
    knots: &[f64],
    degree: usize,
    control_count: usize,
) -> Result<(), NurbsSurfaceError> {
    if knots.len() <= control_count || degree >= knots.len() {
        return Err(NurbsSurfaceError::InvalidKnotCount);
    }
    let start = knots[degree];
    let end = knots[control_count];
    if end <= start {
        return Err(NurbsSurfaceError::InvalidDomain);
    }
    if !knots[..=degree].iter().all(|knot| *knot == start)
        || !knots[control_count..].iter().all(|knot| *knot == end)
    {
        return Err(NurbsSurfaceError::NotClamped);
    }
    Ok(())
}

fn find_span(parameter: f64, degree: usize, knots: &[f64], control_count: usize) -> usize {
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

fn de_boor(
    parameter: f64,
    span: usize,
    degree: usize,
    knots: &[f64],
    work: &mut [HomogeneousPoint],
) -> HomogeneousPoint {
    if degree == 0 {
        return work[0];
    }
    for level in 1..=degree {
        for j in (level..=degree).rev() {
            let i = span - degree + j;
            let denominator = knots[i + degree + 1 - level] - knots[i];
            let alpha = if denominator == 0.0 {
                0.0
            } else {
                (parameter - knots[i]) / denominator
            };
            work[j] = work[j - 1].lerp(work[j], alpha);
        }
    }
    work[degree]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bilinear_patch_matches_known_point() {
        let surface = NurbsSurface2D::new(
            1,
            1,
            vec![
                Point3 { x: 0.0, y: 0.0, z: 0.0 },
                Point3 { x: 0.0, y: 1.0, z: 1.0 },
                Point3 { x: 1.0, y: 0.0, z: 2.0 },
                Point3 { x: 1.0, y: 1.0, z: 3.0 },
            ],
            vec![1.0; 4],
            vec![0.0, 0.0, 1.0, 1.0],
            vec![0.0, 0.0, 1.0, 1.0],
        );
        let point = surface.point_at(0.25, 0.75).unwrap();
        assert!((point.x - 0.25).abs() < 1e-12);
        assert!((point.y - 0.75).abs() < 1e-12);
        assert!((point.z - 1.25).abs() < 1e-12);
    }
}
