use umlcad_v6_geometry_api::{GeometryBackend, GeometryError, GeometryResult, ToleranceContext};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DraftedRectangularSolid {
    pub width: f64,
    pub depth: f64,
    pub height: f64,
    pub draft_angle_radians: f64,
}

impl DraftedRectangularSolid {
    pub fn validate(self, tolerance: ToleranceContext) -> Result<(), GeometryError> {
        tolerance.validate()?;
        if !self.width.is_finite() || !self.depth.is_finite() || !self.height.is_finite() || !self.draft_angle_radians.is_finite() {
            return Err(GeometryError::InvalidInput("drafted dimensions and angle must be finite"));
        }
        if self.width <= tolerance.modeling || self.depth <= tolerance.modeling || self.height <= tolerance.modeling {
            return Err(GeometryError::InvalidInput("drafted dimensions must exceed modeling tolerance"));
        }
        if self.draft_angle_radians.abs() >= std::f64::consts::FRAC_PI_2 {
            return Err(GeometryError::InvalidInput("draft angle must have finite tangent and remain within +/- 90 degrees"));
        }
        let inset = self.height * self.draft_angle_radians.tan();
        if !inset.is_finite() {
            return Err(GeometryError::InvalidInput("draft displacement is not finite"));
        }
        let top_width = self.width + 2.0 * inset;
        let top_depth = self.depth + 2.0 * inset;
        if top_width <= tolerance.modeling || top_depth <= tolerance.modeling {
            return Err(GeometryError::InvalidInput("draft angle produces a non-positive top profile"));
        }
        Ok(())
    }

    pub fn top_dimensions(self, tolerance: ToleranceContext) -> Result<(f64, f64), GeometryError> {
        self.validate(tolerance)?;
        let inset = self.height * self.draft_angle_radians.tan();
        Ok((self.width + 2.0 * inset, self.depth + 2.0 * inset))
    }

    pub fn exact_volume(self, tolerance: ToleranceContext) -> Result<f64, GeometryError> {
        let (top_width, top_depth) = self.top_dimensions(tolerance)?;
        let a0 = self.width * self.depth;
        let a1 = top_width * top_depth;
        let v = self.height * (a0 + a1 + (a0 * a1).sqrt()) / 3.0;
        if !v.is_finite() || v <= 0.0 {
            return Err(GeometryError::InvalidInput("drafted solid volume is not finite and positive"));
        }
        Ok(v)
    }

    pub fn realize_with<B: GeometryBackend>(self, backend: &B, tolerance: ToleranceContext) -> Result<GeometryResult<B::Shape>, GeometryError> {
        self.validate(tolerance)?;
        let (top_width, top_depth) = self.top_dimensions(tolerance)?;
        let lower = [(-self.width / 2.0, -self.depth / 2.0), (self.width / 2.0, -self.depth / 2.0), (self.width / 2.0, self.depth / 2.0), (-self.width / 2.0, self.depth / 2.0)];
        let upper = [(-top_width / 2.0, -top_depth / 2.0), (top_width / 2.0, -top_depth / 2.0), (top_width / 2.0, top_depth / 2.0), (-top_width / 2.0, top_depth / 2.0)];
        backend.loft_between_polygons(&lower, 0.0, &upper, self.height, tolerance)
    }
}

pub trait DraftBackend: GeometryBackend {
    fn make_drafted_rectangular_solid(&self, definition: DraftedRectangularSolid, tolerance: ToleranceContext) -> Result<GeometryResult<Self::Shape>, GeometryError>;
}

impl<B: GeometryBackend> DraftBackend for B {
    fn make_drafted_rectangular_solid(&self, definition: DraftedRectangularSolid, tolerance: ToleranceContext) -> Result<GeometryResult<Self::Shape>, GeometryError> {
        definition.realize_with(self, tolerance)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn tolerance() -> ToleranceContext { ToleranceContext { modeling: 1e-9, validation: 1e-9 } }
    fn definition() -> DraftedRectangularSolid { DraftedRectangularSolid { width: 20.0, depth: 30.0, height: 10.0, draft_angle_radians: 0.1 } }

    #[test]
    fn computes_top_dimensions_and_volume_exactly() {
        let d = definition();
        let (w, dep) = d.top_dimensions(tolerance()).unwrap();
        let delta = 10.0 * 0.1_f64.tan();
        assert!((w - (20.0 + 2.0 * delta)).abs() < 1e-12);
        assert!((dep - (30.0 + 2.0 * delta)).abs() < 1e-12);
        let a0 = 20.0 * 30.0;
        let a1 = w * dep;
        let expected = 10.0 * (a0 + a1 + (a0 * a1).sqrt()) / 3.0;
        assert!((d.exact_volume(tolerance()).unwrap() - expected).abs() < 1e-12);
    }

    #[test]
    fn zero_draft_is_exact_prismatic_limit() {
        let mut d = definition();
        d.draft_angle_radians = 0.0;
        assert_eq!(d.top_dimensions(tolerance()).unwrap(), (20.0, 30.0));
        assert_eq!(d.exact_volume(tolerance()).unwrap(), 6000.0);
    }

    #[test]
    fn rejects_angles_that_collapse_top_profile() {
        let mut d = definition();
        d.draft_angle_radians = -1.4;
        assert!(d.validate(tolerance()).is_err());
    }

    #[test]
    fn rejects_nonfinite_and_degenerate_inputs() {
        let mut d = definition();
        d.width = f64::NAN;
        assert!(d.validate(tolerance()).is_err());
        let mut d = definition();
        d.draft_angle_radians = f64::INFINITY;
        assert!(d.validate(tolerance()).is_err());
        let mut d = definition();
        d.height = 0.0;
        assert!(d.validate(tolerance()).is_err());
    }
}
