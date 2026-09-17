use umlcad_v6_geometry_api::{GeometryBackend, GeometryError, GeometryResult, ToleranceContext};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClosedBoxThickness {
    pub width: f64,
    pub depth: f64,
    pub height: f64,
    pub thickness: f64,
}

impl ClosedBoxThickness {
    pub fn validate(self, tolerance: ToleranceContext) -> Result<(), GeometryError> {
        tolerance.validate()?;
        if !self.width.is_finite() || !self.depth.is_finite() || !self.height.is_finite() || !self.thickness.is_finite() {
            return Err(GeometryError::InvalidInput("box thickness dimensions must be finite"));
        }
        if self.width <= tolerance.modeling || self.depth <= tolerance.modeling || self.height <= tolerance.modeling {
            return Err(GeometryError::InvalidInput("box thickness outer dimensions must exceed modeling tolerance"));
        }
        if self.thickness <= tolerance.modeling {
            return Err(GeometryError::InvalidInput("box thickness must exceed modeling tolerance"));
        }
        if 2.0 * self.thickness >= self.width.min(self.depth).min(self.height) {
            return Err(GeometryError::InvalidInput("box thickness must be strictly less than half the minimum outer dimension"));
        }
        Ok(())
    }

    pub fn inner_dimensions(self, tolerance: ToleranceContext) -> Result<(f64, f64, f64), GeometryError> {
        self.validate(tolerance)?;
        Ok((self.width - 2.0 * self.thickness, self.depth - 2.0 * self.thickness, self.height - 2.0 * self.thickness))
    }

    pub fn outer_volume(self, tolerance: ToleranceContext) -> Result<f64, GeometryError> {
        self.validate(tolerance)?;
        let v = self.width * self.depth * self.height;
        if !v.is_finite() { return Err(GeometryError::InvalidInput("box thickness outer volume is not finite")); }
        Ok(v)
    }

    pub fn inner_volume(self, tolerance: ToleranceContext) -> Result<f64, GeometryError> {
        let (w, d, h) = self.inner_dimensions(tolerance)?;
        let v = w * d * h;
        if !v.is_finite() || v <= 0.0 { return Err(GeometryError::InvalidInput("box thickness inner volume is not finite and positive")); }
        Ok(v)
    }

    pub fn material_volume(self, tolerance: ToleranceContext) -> Result<f64, GeometryError> {
        Ok(self.outer_volume(tolerance)? - self.inner_volume(tolerance)?)
    }

    pub fn realize_with<B: GeometryBackend>(self, backend: &B, tolerance: ToleranceContext) -> Result<GeometryResult<B::Shape>, GeometryError> {
        self.validate(tolerance)?;
        let outer = backend.box_solid(self.width, self.depth, self.height, tolerance)?.shape;
        let (inner_width, inner_depth, inner_height) = self.inner_dimensions(tolerance)?;
        let inner = backend.box_solid(inner_width, inner_depth, inner_height, tolerance)?.shape;
        let inner = backend.translate(&inner, self.thickness, self.thickness, self.thickness, tolerance)?.shape;
        backend.cut(&outer, &inner, tolerance)
    }
}

pub trait ShellBackend: GeometryBackend {
    fn make_closed_box_thickness(&self, definition: ClosedBoxThickness, tolerance: ToleranceContext) -> Result<GeometryResult<Self::Shape>, GeometryError>;
}

impl<B: GeometryBackend> ShellBackend for B {
    fn make_closed_box_thickness(&self, definition: ClosedBoxThickness, tolerance: ToleranceContext) -> Result<GeometryResult<Self::Shape>, GeometryError> {
        definition.realize_with(self, tolerance)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tolerance() -> ToleranceContext { ToleranceContext { modeling: 1e-9, validation: 1e-9 } }
    fn definition() -> ClosedBoxThickness { ClosedBoxThickness { width: 20.0, depth: 30.0, height: 40.0, thickness: 2.0 } }

    #[test]
    fn validates_strict_thickness_bound() {
        let mut d = definition();
        assert!(d.validate(tolerance()).is_ok());
        d.thickness = 10.0;
        assert!(d.validate(tolerance()).is_err());
    }

    #[test]
    fn computes_inner_and_material_volume_exactly() {
        let d = definition();
        assert_eq!(d.inner_dimensions(tolerance()).unwrap(), (16.0, 26.0, 36.0));
        assert_eq!(d.outer_volume(tolerance()).unwrap(), 24000.0);
        assert_eq!(d.inner_volume(tolerance()).unwrap(), 14976.0);
        assert_eq!(d.material_volume(tolerance()).unwrap(), 9024.0);
    }

    #[test]
    fn rejects_nonfinite_and_degenerate_values() {
        let mut d = definition();
        d.width = f64::NAN;
        assert!(d.validate(tolerance()).is_err());
        let mut d = definition();
        d.thickness = 0.0;
        assert!(d.validate(tolerance()).is_err());
        let mut d = definition();
        d.height = 3.0;
        d.thickness = 2.0;
        assert!(d.validate(tolerance()).is_err());
    }
}
