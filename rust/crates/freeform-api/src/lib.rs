use umlcad_v6_geometry_api::{GeometryBackend, GeometryError, GeometryResult, ToleranceContext};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NurbsCurve3DDefinition {
    pub degree: usize,
    pub control_points: Vec<Point3>,
    pub weights: Vec<f64>,
    pub knots: Vec<f64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NurbsDefinitionError {
    NonFinite,
    InvalidDegree,
    InvalidControlPointCount,
    InvalidWeightCount,
    InvalidWeight,
    InvalidKnotCount,
    KnotsMustBeNondecreasing,
    NotClamped,
    InvalidDomain,
}

impl NurbsCurve3DDefinition {
    pub fn new(degree: usize, control_points: Vec<Point3>, weights: Vec<f64>, knots: Vec<f64>) -> Self {
        Self { degree, control_points, weights, knots }
    }

    pub fn validate(&self) -> Result<(), NurbsDefinitionError> {
        if self.degree == 0 {
            return Err(NurbsDefinitionError::InvalidDegree);
        }
        if self.knots.len() != self.control_points.len() + self.degree + 1 {
            return Err(NurbsDefinitionError::InvalidKnotCount);
        }
        if self.control_points.len() < self.degree + 1 {
            return Err(NurbsDefinitionError::InvalidControlPointCount);
        }
        if self.weights.len() != self.control_points.len() {
            return Err(NurbsDefinitionError::InvalidWeightCount);
        }
        if self.control_points.iter().any(|p| !p.x.is_finite() || !p.y.is_finite() || !p.z.is_finite())
            || self.weights.iter().any(|w| !w.is_finite())
            || self.knots.iter().any(|k| !k.is_finite())
        {
            return Err(NurbsDefinitionError::NonFinite);
        }
        if self.weights.iter().any(|w| *w <= 0.0) {
            return Err(NurbsDefinitionError::InvalidWeight);
        }
        if self.knots.windows(2).any(|pair| pair[1] < pair[0]) {
            return Err(NurbsDefinitionError::KnotsMustBeNondecreasing);
        }
        let start = self.knots[self.degree];
        let count = self.control_points.len();
        let end = self.knots[count];
        if end <= start {
            return Err(NurbsDefinitionError::InvalidDomain);
        }
        if !self.knots[..=self.degree].iter().all(|k| *k == start)
            || !self.knots[count..].iter().all(|k| *k == end)
        {
            return Err(NurbsDefinitionError::NotClamped);
        }
        Ok(())
    }

    pub fn parameter_domain(&self) -> Result<(f64, f64), NurbsDefinitionError> {
        self.validate()?;
        Ok((self.knots[self.degree], self.knots[self.control_points.len()]))
    }
}

pub trait NurbsCurveBackend: GeometryBackend {
    fn nurbs_curve3d(
        &self,
        definition: &NurbsCurve3DDefinition,
        tolerance: ToleranceContext,
    ) -> Result<GeometryResult<Self::Shape>, GeometryError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn quarter_circle() -> NurbsCurve3DDefinition {
        NurbsCurve3DDefinition::new(
            2,
            vec![
                Point3 { x: 1.0, y: 0.0, z: 0.0 },
                Point3 { x: 1.0, y: 1.0, z: 0.0 },
                Point3 { x: 0.0, y: 1.0, z: 0.0 },
            ],
            vec![1.0, std::f64::consts::FRAC_1_SQRT_2, 1.0],
            vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
        )
    }

    #[test]
    fn rational_quarter_circle_definition_is_valid() {
        assert!(quarter_circle().validate().is_ok());
        assert_eq!(quarter_circle().parameter_domain().unwrap(), (0.0, 1.0));
    }

    #[test]
    fn definition_rejects_weight_and_knot_contract_violations() {
        let mut curve = quarter_circle();
        curve.weights[1] = 0.0;
        assert_eq!(curve.validate(), Err(NurbsDefinitionError::InvalidWeight));

        let mut curve = quarter_circle();
        curve.knots[3] = -0.1;
        assert_eq!(curve.validate(), Err(NurbsDefinitionError::KnotsMustBeNondecreasing));
    }

    #[test]
    fn definition_rejects_non_clamped_domain() {
        let mut curve = quarter_circle();
        curve.knots[0] = -1.0;
        assert_eq!(curve.validate(), Err(NurbsDefinitionError::NotClamped));
    }
}
