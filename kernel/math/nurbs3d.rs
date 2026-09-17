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

    fn scale(self, factor: f64) -> Self {
        Self {
            x: self.x * factor,
            y: self.y * factor,
            z: self.z * factor,
        }
    }

    fn norm(self) -> f64 {
        self.x.hypot(self.y.hypot(self.z))
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoundingBox3 {
    pub min: Point3,
    pub max: Point3,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Nurbs3DError {
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
    ZeroDerivative,
    Overflow,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NurbsCurve3D {
    pub degree: usize,
    pub control_points: Vec<Point3>,
    pub weights: Vec<f64>,
    pub knots: Vec<f64>,
}

impl NurbsCurve3D {
    pub fn new(
        degree: usize,
        control_points: Vec<Point3>,
        weights: Vec<f64>,
        knots: Vec<f64>,
    ) -> Self {
        Self {
            degree,
            control_points,
            weights,
            knots,
        }
    }

    pub fn validate(&self) -> Result<(), Nurbs3DError> {
        if self.degree == 0 {
            return Err(Nurbs3DError::InvalidDegree);
        }
        if self.control_points.len() < self.degree + 1 {
            return Err(Nurbs3DError::InvalidControlPointCount);
        }
        if self.weights.len() != self.control_points.len() {
            return Err(Nurbs3DError::InvalidWeightCount);
        }
        if self.knots.len() != self.control_points.len() + self.degree + 1 {
            return Err(Nurbs3DError::InvalidKnotCount);
        }
        if self.control_points.iter().any(|p| {
            !p.x.is_finite() || !p.y.is_finite() || !p.z.is_finite()
        }) {
            return Err(Nurbs3DError::NonFinite);
        }
        if self.weights.iter().any(|w| !w.is_finite()) {
            return Err(Nurbs3DError::NonFinite);
        }
        if self.weights.iter().any(|w| *w <= 0.0) {
            return Err(Nurbs3DError::InvalidWeight);
        }
        if self.knots.iter().any(|k| !k.is_finite()) {
            return Err(Nurbs3DError::NonFinite);
        }
        if self.knots.windows(2).any(|pair| pair[1] < pair[0]) {
            return Err(Nurbs3DError::KnotsMustBeNondecreasing);
        }

        let start = self.knots[self.degree];
        let end = self.knots[self.control_points.len()];
        if end <= start {
            return Err(Nurbs3DError::InvalidDomain);
        }
        if !self.knots[..=self.degree].iter().all(|k| *k == start)
            || !self.knots[self.control_points.len()..].iter().all(|k| *k == end)
        {
            return Err(Nurbs3DError::NotClamped);
        }
        if self.control_points.windows(2).all(|pair| pair[0] == pair[1]) {
            return Err(Nurbs3DError::Degenerate);
        }
        Ok(())
    }

    pub fn parameter_domain(&self) -> Result<(f64, f64), Nurbs3DError> {
        self.validate()?;
        Ok((
            self.knots[self.degree],
            self.knots[self.control_points.len()],
        ))
    }

    pub fn point_at(&self, parameter: f64) -> Result<Point3, Nurbs3DError> {
        let value = self.evaluate_homogeneous(parameter)?;
        dehomogenize(value)
    }

    pub fn derivative_at(&self, parameter: f64) -> Result<Point3, Nurbs3DError> {
        self.validate_query(parameter)?;
        let base = self.evaluate_homogeneous(parameter)?;
        let derivative = self.evaluate_homogeneous_derivative(parameter)?;
        quotient_derivative(base, derivative)
    }

    pub fn tangent_at(&self, parameter: f64) -> Result<Point3, Nurbs3DError> {
        let derivative = self.derivative_at(parameter)?;
        let magnitude = derivative.norm();
        if !magnitude.is_finite() {
            return Err(Nurbs3DError::Overflow);
        }
        if magnitude == 0.0 {
            return Err(Nurbs3DError::ZeroDerivative);
        }
        Ok(derivative.scale(1.0 / magnitude))
    }

    pub fn control_hull_bounds(&self) -> Result<BoundingBox3, Nurbs3DError> {
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

    pub fn translated(&self, dx: f64, dy: f64, dz: f64) -> Result<Self, Nurbs3DError> {
        self.validate()?;
        if !dx.is_finite() || !dy.is_finite() || !dz.is_finite() {
            return Err(Nurbs3DError::NonFinite);
        }
        let shift = Point3 { x: dx, y: dy, z: dz };
        let control_points = self
            .control_points
            .iter()
            .map(|p| p.add(shift))
            .collect::<Vec<_>>();
        if control_points.iter().any(|p| {
            !p.x.is_finite() || !p.y.is_finite() || !p.z.is_finite()
        }) {
            return Err(Nurbs3DError::Overflow);
        }
        Ok(Self::new(
            self.degree,
            control_points,
            self.weights.clone(),
            self.knots.clone(),
        ))
    }

    fn validate_query(&self, parameter: f64) -> Result<(), Nurbs3DError> {
        self.validate()?;
        if !parameter.is_finite() {
            return Err(Nurbs3DError::NonFinite);
        }
        let (start, end) = self.parameter_domain_unchecked();
        if parameter < start || parameter > end {
            return Err(Nurbs3DError::OutOfDomain);
        }
        Ok(())
    }

    fn parameter_domain_unchecked(&self) -> (f64, f64) {
        (
            self.knots[self.degree],
            self.knots[self.control_points.len()],
        )
    }

    fn evaluate_homogeneous(&self, parameter: f64) -> Result<HomogeneousPoint, Nurbs3DError> {
        self.validate_query(parameter)?;
        let span = self.find_span(parameter);
        let mut work: Vec<HomogeneousPoint> = (span - self.degree..=span)
            .map(|index| {
                let point = self.control_points[index];
                let weight = self.weights[index];
                HomogeneousPoint {
                    xw: point.x * weight,
                    yw: point.y * weight,
                    zw: point.z * weight,
                    w: weight,
                }
            })
            .collect();
        Ok(de_boor(parameter, span, self.degree, &self.knots, &mut work))
    }

    fn evaluate_homogeneous_derivative(
        &self,
        parameter: f64,
    ) -> Result<HomogeneousPoint, Nurbs3DError> {
        if self.degree == 0 {
            return Err(Nurbs3DError::InvalidDegree);
        }
        let derivative_count = self.control_points.len() - 1;
        let derivative_degree = self.degree - 1;
        let derivative_knots = &self.knots[1..self.knots.len() - 1];
        let span = find_span(parameter, derivative_degree, derivative_knots, derivative_count);
        let factor = self.degree as f64;
        let mut work = Vec::with_capacity(self.degree);

        for local in 0..self.degree {
            let index = span - derivative_degree + local;
            let left = self.control_points[index];
            let right = self.control_points[index + 1];
            let left_weight = self.weights[index];
            let right_weight = self.weights[index + 1];
            let denominator =
                self.knots[index + self.degree + 1] - self.knots[index + 1];
            if denominator == 0.0 {
                return Err(Nurbs3DError::InvalidDomain);
            }
            let scale = factor / denominator;
            work.push(HomogeneousPoint {
                xw: (right.x * right_weight - left.x * left_weight) * scale,
                yw: (right.y * right_weight - left.y * left_weight) * scale,
                zw: (right.z * right_weight - left.z * left_weight) * scale,
                w: (right_weight - left_weight) * scale,
            });
        }

        Ok(de_boor(
            parameter,
            span,
            derivative_degree,
            derivative_knots,
            &mut work,
        ))
    }

    fn find_span(&self, parameter: f64) -> usize {
        find_span(parameter, self.degree, &self.knots, self.control_points.len())
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

fn dehomogenize(value: HomogeneousPoint) -> Result<Point3, Nurbs3DError> {
    if [value.xw, value.yw, value.zw, value.w]
        .iter()
        .any(|x| !x.is_finite())
    {
        return Err(Nurbs3DError::Overflow);
    }
    if value.w <= 0.0 {
        return Err(Nurbs3DError::ZeroProjectiveWeight);
    }
    let point = Point3 {
        x: value.xw / value.w,
        y: value.yw / value.w,
        z: value.zw / value.w,
    };
    if !point.x.is_finite() || !point.y.is_finite() || !point.z.is_finite() {
        return Err(Nurbs3DError::Overflow);
    }
    Ok(point)
}

fn quotient_derivative(
    base: HomogeneousPoint,
    derivative: HomogeneousPoint,
) -> Result<Point3, Nurbs3DError> {
    if [
        base.xw,
        base.yw,
        base.zw,
        base.w,
        derivative.xw,
        derivative.yw,
        derivative.zw,
        derivative.w,
    ]
    .iter()
    .any(|x| !x.is_finite())
    {
        return Err(Nurbs3DError::Overflow);
    }
    if base.w <= 0.0 {
        return Err(Nurbs3DError::ZeroProjectiveWeight);
    }
    let denominator = base.w * base.w;
    if !denominator.is_finite() || denominator == 0.0 {
        return Err(Nurbs3DError::Overflow);
    }
    let result = Point3 {
        x: (derivative.xw * base.w - base.xw * derivative.w) / denominator,
        y: (derivative.yw * base.w - base.yw * derivative.w) / denominator,
        z: (derivative.zw * base.w - base.zw * derivative.w) / denominator,
    };
    if !result.x.is_finite() || !result.y.is_finite() || !result.z.is_finite() {
        return Err(Nurbs3DError::Overflow);
    }
    Ok(result)
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
            let index = span - degree + j;
            let denominator = knots[index + degree + 1 - level] - knots[index];
            let alpha = if denominator == 0.0 {
                0.0
            } else {
                (parameter - knots[index]) / denominator
            };
            work[j] = work[j - 1].lerp(work[j], alpha);
        }
    }
    work[degree]
}
