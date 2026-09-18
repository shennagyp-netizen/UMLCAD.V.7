use super::analytic::Cylinder3;
use super::tolerance::Tolerance;
use super::vec::Vec3;

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum CircularPrismError {
    #[error("circular-prism inputs are non-finite")]
    NonFinite,
    #[error("circular-prism axis is invalid")]
    InvalidAxis,
    #[error("circular-prism radius is invalid")]
    InvalidRadius,
    #[error("circular-prism depth is invalid")]
    InvalidDepth,
    #[error("circular-prism tolerance is invalid")]
    InvalidTolerance,
    #[error("circular-prism result overflowed")]
    Overflow,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CircularPrismSolid {
    pub origin: Vec3,
    pub axis: Vec3,
    pub radius: f64,
    pub depth: f64,
    pub volume: f64,
    pub surface_area: f64,
    pub centroid: Vec3,
    pub bounds_min: Vec3,
    pub bounds_max: Vec3,
}

impl CircularPrismSolid {
    pub fn validate(&self, tolerance: Tolerance) -> Result<(), CircularPrismError> {
        if !self.origin.is_finite()
            || !self.axis.is_finite()
            || !self.centroid.is_finite()
            || !self.bounds_min.is_finite()
            || !self.bounds_max.is_finite()
            || !self.radius.is_finite()
            || !self.depth.is_finite()
            || !self.volume.is_finite()
            || !self.surface_area.is_finite()
        {
            return Err(CircularPrismError::NonFinite);
        }

        if self.radius <= 0.0 {
            return Err(CircularPrismError::InvalidRadius);
        }
        if self.depth <= 0.0 {
            return Err(CircularPrismError::InvalidDepth);
        }

        let axis_length = self.axis.length();
        if !axis_length.is_finite() || (axis_length - 1.0).abs() > tolerance.threshold(1.0).map_err(|_| CircularPrismError::InvalidTolerance)? {
            return Err(CircularPrismError::InvalidAxis);
        }

        if self.bounds_min.x > self.bounds_max.x
            || self.bounds_min.y > self.bounds_max.y
            || self.bounds_min.z > self.bounds_max.z
            || self.volume <= 0.0
            || self.surface_area <= 0.0
        {
            return Err(CircularPrismError::Overflow);
        }

        Ok(())
    }
}

pub fn evaluate_circular_prism(
    origin: Vec3,
    axis: Vec3,
    radius: f64,
    depth: f64,
    tolerance: Tolerance,
) -> Result<CircularPrismSolid, CircularPrismError> {
    if !origin.is_finite()
        || !axis.is_finite()
        || !radius.is_finite()
        || !depth.is_finite()
    {
        return Err(CircularPrismError::NonFinite);
    }
    if radius <= 0.0 {
        return Err(CircularPrismError::InvalidRadius);
    }
    if !depth.is_finite() || depth <= 0.0 {
        return Err(CircularPrismError::InvalidDepth);
    }
    if tolerance.threshold(radius.max(depth).max(1.0)).is_err() {
        return Err(CircularPrismError::InvalidTolerance);
    }

    let cylinder = Cylinder3 {
        origin,
        axis,
        radius,
    };
    let unit_axis = cylinder
        .unit_axis()
        .map_err(|error| match error {
            super::analytic::AnalyticError::NonFinite => CircularPrismError::NonFinite,
            super::analytic::AnalyticError::InvalidAxis => CircularPrismError::InvalidAxis,
            super::analytic::AnalyticError::InvalidRadius => CircularPrismError::InvalidRadius,
            _ => CircularPrismError::Overflow,
        })?;

    let end = origin.add(unit_axis.scale(depth));
    let radial_extent = |component: f64| {
        radius * (1.0 - component * component).max(0.0).sqrt()
    };

    let rx = radial_extent(unit_axis.x);
    let ry = radial_extent(unit_axis.y);
    let rz = radial_extent(unit_axis.z);

    let bounds_min = Vec3::new(
        origin.x.min(end.x) - rx,
        origin.y.min(end.y) - ry,
        origin.z.min(end.z) - rz,
    );
    let bounds_max = Vec3::new(
        origin.x.max(end.x) + rx,
        origin.y.max(end.y) + ry,
        origin.z.max(end.z) + rz,
    );

    let volume = std::f64::consts::PI * radius * radius * depth;
    let surface_area =
        2.0 * std::f64::consts::PI * radius * (radius + depth);
    let centroid = origin.add(unit_axis.scale(depth * 0.5));

    let result = CircularPrismSolid {
        origin,
        axis: unit_axis,
        radius,
        depth,
        volume,
        surface_area,
        centroid,
        bounds_min,
        bounds_max,
    };

    result
        .validate(tolerance)
        .map(|()| result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tolerance() -> Tolerance {
        Tolerance::new(1.0e-9, 1.0e-9).unwrap()
    }

    #[test]
    fn negative_axis_is_normalized_without_changing_intrinsic_metrics() {
        let result = evaluate_circular_prism(
            Vec3::new(0.0, 0.0, 10.0),
            Vec3::new(0.0, 0.0, -2.0),
            3.0,
            5.0,
            tolerance(),
        )
        .unwrap();

        assert_eq!(result.axis, Vec3::new(0.0, 0.0, -1.0));
        assert_eq!(result.centroid, Vec3::new(0.0, 0.0, 7.5));
        assert_eq!(result.bounds_min, Vec3::new(-3.0, -3.0, 5.0));
        assert_eq!(result.bounds_max, Vec3::new(3.0, 3.0, 10.0));
    }

    #[test]
    fn translation_metamorphic_property_preserves_intrinsic_metrics() {
        let a = evaluate_circular_prism(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
            2.0,
            4.0,
            tolerance(),
        )
        .unwrap();
        let shift = Vec3::new(11.0, -3.0, 7.0);
        let b = evaluate_circular_prism(
            shift,
            Vec3::new(0.0, 0.0, 1.0),
            2.0,
            4.0,
            tolerance(),
        )
        .unwrap();

        assert_eq!(a.volume, b.volume);
        assert_eq!(a.surface_area, b.surface_area);
        assert_eq!(b.centroid, a.centroid.add(shift));
        assert_eq!(b.bounds_min, a.bounds_min.add(shift));
        assert_eq!(b.bounds_max, a.bounds_max.add(shift));
    }
}
