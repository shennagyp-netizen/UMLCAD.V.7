//! Backend-independent analytic geometric primitives.
//!
//! These are mathematical definitions only. They do not create topology,
//! renderer objects, or backend shapes.

use std::f64::consts::TAU;

use super::vec::{Vec2, Vec3};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnalyticError {
    NonFinite,
    Degenerate,
    InvalidParameter,
    InvalidRadius,
    InvalidAxis,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Plane3 {
    pub origin: Vec3,
    pub normal: Vec3,
}

impl Plane3 {
    pub fn validate(&self) -> Result<(), AnalyticError> {
        if !self.origin.is_finite() || !self.normal.is_finite() {
            return Err(AnalyticError::NonFinite);
        }
        if self.normal.length() == 0.0 {
            return Err(AnalyticError::InvalidAxis);
        }
        Ok(())
    }

    pub fn unit_normal(&self) -> Result<Vec3, AnalyticError> {
        self.validate()?;
        self.normal
            .normalized()
            .map_err(|_| AnalyticError::InvalidAxis)
    }

    pub fn signed_distance(&self, point: Vec3) -> Result<f64, AnalyticError> {
        if !point.is_finite() {
            return Err(AnalyticError::NonFinite);
        }
        let normal = self.unit_normal()?;
        let displacement = point.sub(self.origin);
        let distance = displacement.dot(normal);
        if distance.is_finite() {
            Ok(distance)
        } else {
            Err(AnalyticError::NonFinite)
        }
    }

    pub fn project_point(&self, point: Vec3) -> Result<Vec3, AnalyticError> {
        let normal = self.unit_normal()?;
        let distance = self.signed_distance(point)?;
        let projected = point.sub(normal.scale(distance));
        if projected.is_finite() {
            Ok(projected)
        } else {
            Err(AnalyticError::NonFinite)
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ray2 {
    pub origin: Vec2,
    pub direction: Vec2,
}

impl Ray2 {
    pub fn validate(&self) -> Result<(), AnalyticError> {
        if !self.origin.is_finite() || !self.direction.is_finite() {
            return Err(AnalyticError::NonFinite);
        }
        if self.direction.length() == 0.0 {
            return Err(AnalyticError::Degenerate);
        }
        Ok(())
    }

    pub fn unit_direction(&self) -> Result<Vec2, AnalyticError> {
        self.validate()?;
        self.direction
            .normalized()
            .map_err(|_| AnalyticError::Degenerate)
    }

    pub fn point_at(&self, t: f64) -> Result<Vec2, AnalyticError> {
        if !t.is_finite() {
            return Err(AnalyticError::NonFinite);
        }
        if t < 0.0 {
            return Err(AnalyticError::InvalidParameter);
        }
        let point = self.origin.add(self.direction.scale(t));
        if point.is_finite() {
            Ok(point)
        } else {
            Err(AnalyticError::NonFinite)
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ray3 {
    pub origin: Vec3,
    pub direction: Vec3,
}

impl Ray3 {
    pub fn validate(&self) -> Result<(), AnalyticError> {
        if !self.origin.is_finite() || !self.direction.is_finite() {
            return Err(AnalyticError::NonFinite);
        }
        if self.direction.length() == 0.0 {
            return Err(AnalyticError::Degenerate);
        }
        Ok(())
    }

    pub fn unit_direction(&self) -> Result<Vec3, AnalyticError> {
        self.validate()?;
        self.direction
            .normalized()
            .map_err(|_| AnalyticError::Degenerate)
    }

    pub fn point_at(&self, t: f64) -> Result<Vec3, AnalyticError> {
        if !t.is_finite() {
            return Err(AnalyticError::NonFinite);
        }
        if t < 0.0 {
            return Err(AnalyticError::InvalidParameter);
        }
        let point = self.origin.add(self.direction.scale(t));
        if point.is_finite() {
            Ok(point)
        } else {
            Err(AnalyticError::NonFinite)
        }
    }

    /// Signed ray parameter of the orthogonal projection of `point` onto the
    /// supporting line. It does not by itself establish that `point` is on the
    /// ray.
    pub fn supporting_parameter(&self, point: Vec3) -> Result<f64, AnalyticError> {
        if !point.is_finite() {
            return Err(AnalyticError::NonFinite);
        }
        self.validate()?;
        let denominator = self.direction.dot(self.direction);
        let parameter = point.sub(self.origin).dot(self.direction) / denominator;
        if parameter.is_finite() {
            Ok(parameter)
        } else {
            Err(AnalyticError::NonFinite)
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ellipse2 {
    pub center: Vec2,
    pub semi_axis_a: f64,
    pub semi_axis_b: f64,
    pub rotation: f64,
}

impl Ellipse2 {
    pub fn validate(&self) -> Result<(), AnalyticError> {
        if !self.center.is_finite()
            || !self.semi_axis_a.is_finite()
            || !self.semi_axis_b.is_finite()
            || !self.rotation.is_finite()
        {
            return Err(AnalyticError::NonFinite);
        }
        if self.semi_axis_a <= 0.0 || self.semi_axis_b <= 0.0 {
            return Err(AnalyticError::InvalidRadius);
        }
        Ok(())
    }

    pub fn point_at(&self, parameter: f64) -> Result<Vec2, AnalyticError> {
        self.validate()?;
        if !parameter.is_finite() || !(0.0..=1.0).contains(&parameter) {
            return Err(AnalyticError::InvalidParameter);
        }
        let theta = parameter * TAU;
        let cos_r = self.rotation.cos();
        let sin_r = self.rotation.sin();
        let local = Vec2::new(self.semi_axis_a * theta.cos(), self.semi_axis_b * theta.sin());
        let point = self.center.add(Vec2::new(
            cos_r * local.x - sin_r * local.y,
            sin_r * local.x + cos_r * local.y,
        ));
        if point.is_finite() {
            Ok(point)
        } else {
            Err(AnalyticError::NonFinite)
        }
    }

    /// Implicit ellipse equation value. Zero denotes the exact ellipse.
    pub fn implicit_value(&self, point: Vec2) -> Result<f64, AnalyticError> {
        self.validate()?;
        if !point.is_finite() {
            return Err(AnalyticError::NonFinite);
        }
        let delta = point.sub(self.center);
        let cos_r = self.rotation.cos();
        let sin_r = self.rotation.sin();
        let local_x = cos_r * delta.x + sin_r * delta.y;
        let local_y = -sin_r * delta.x + cos_r * delta.y;
        let value = (local_x / self.semi_axis_a).powi(2)
            + (local_y / self.semi_axis_b).powi(2)
            - 1.0;
        if value.is_finite() {
            Ok(value)
        } else {
            Err(AnalyticError::NonFinite)
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cylinder3 {
    pub origin: Vec3,
    pub axis: Vec3,
    pub radius: f64,
}

impl Cylinder3 {
    pub fn validate(&self) -> Result<(), AnalyticError> {
        if !self.origin.is_finite() || !self.axis.is_finite() || !self.radius.is_finite() {
            return Err(AnalyticError::NonFinite);
        }
        if self.axis.length() == 0.0 {
            return Err(AnalyticError::InvalidAxis);
        }
        if self.radius <= 0.0 {
            return Err(AnalyticError::InvalidRadius);
        }
        Ok(())
    }

    pub fn unit_axis(&self) -> Result<Vec3, AnalyticError> {
        self.validate()?;
        self.axis
            .normalized()
            .map_err(|_| AnalyticError::InvalidAxis)
    }

    pub fn radial_distance(&self, point: Vec3) -> Result<f64, AnalyticError> {
        if !point.is_finite() {
            return Err(AnalyticError::NonFinite);
        }
        let axis = self.unit_axis()?;
        let displacement = point.sub(self.origin);
        let axial = displacement.dot(axis);
        let radial_vector = displacement.sub(axis.scale(axial));
        let radial = radial_vector.length();
        if radial.is_finite() {
            Ok(radial)
        } else {
            Err(AnalyticError::NonFinite)
        }
    }

    pub fn implicit_value(&self, point: Vec3) -> Result<f64, AnalyticError> {
        Ok(self.radial_distance(point)? - self.radius)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cone3 {
    pub apex: Vec3,
    pub axis: Vec3,
    /// Half-angle from the axis, restricted to `(0, π/2)`.
    pub half_angle: f64,
}

impl Cone3 {
    pub fn validate(&self) -> Result<(), AnalyticError> {
        if !self.apex.is_finite() || !self.axis.is_finite() || !self.half_angle.is_finite() {
            return Err(AnalyticError::NonFinite);
        }
        if self.axis.length() == 0.0 {
            return Err(AnalyticError::InvalidAxis);
        }
        if !(0.0 < self.half_angle && self.half_angle < std::f64::consts::FRAC_PI_2) {
            return Err(AnalyticError::InvalidParameter);
        }
        Ok(())
    }

    pub fn unit_axis(&self) -> Result<Vec3, AnalyticError> {
        self.validate()?;
        self.axis
            .normalized()
            .map_err(|_| AnalyticError::InvalidAxis)
    }

    /// Residual of the double-cone equation `rho = |z| tan(alpha)`.
    ///
    /// The double-sided definition is intentional; a bounded one-nappe cone
    /// is a separate semantic primitive and should not be inferred here.
    pub fn implicit_value(&self, point: Vec3) -> Result<f64, AnalyticError> {
        if !point.is_finite() {
            return Err(AnalyticError::NonFinite);
        }
        let axis = self.unit_axis()?;
        let displacement = point.sub(self.apex);
        let axial = displacement.dot(axis);
        let radial = displacement.sub(axis.scale(axial)).length();
        let value = radial - axial.abs() * self.half_angle.tan();
        if value.is_finite() {
            Ok(value)
        } else {
            Err(AnalyticError::NonFinite)
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Torus3 {
    pub center: Vec3,
    pub axis: Vec3,
    pub major_radius: f64,
    pub minor_radius: f64,
}

impl Torus3 {
    pub fn validate(&self) -> Result<(), AnalyticError> {
        if !self.center.is_finite()
            || !self.axis.is_finite()
            || !self.major_radius.is_finite()
            || !self.minor_radius.is_finite()
        {
            return Err(AnalyticError::NonFinite);
        }
        if self.axis.length() == 0.0 {
            return Err(AnalyticError::InvalidAxis);
        }
        if self.major_radius <= 0.0 || self.minor_radius <= 0.0 || self.minor_radius >= self.major_radius {
            return Err(AnalyticError::InvalidRadius);
        }
        Ok(())
    }

    pub fn unit_axis(&self) -> Result<Vec3, AnalyticError> {
        self.validate()?;
        self.axis
            .normalized()
            .map_err(|_| AnalyticError::InvalidAxis)
    }

    /// Exact signed meridional distance residual of the rotational torus.
    pub fn implicit_value(&self, point: Vec3) -> Result<f64, AnalyticError> {
        if !point.is_finite() {
            return Err(AnalyticError::NonFinite);
        }
        let axis = self.unit_axis()?;
        let displacement = point.sub(self.center);
        let axial = displacement.dot(axis);
        let radial = displacement.sub(axis.scale(axial)).length();
        let meridian_distance = (radial - self.major_radius).hypot(axial);
        let value = meridian_distance - self.minor_radius;
        if value.is_finite() {
            Ok(value)
        } else {
            Err(AnalyticError::NonFinite)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plane_signed_distance_and_projection_are_exact_for_axis_aligned_case() {
        let plane = Plane3 {
            origin: Vec3::new(0.0, 0.0, 2.0),
            normal: Vec3::new(0.0, 0.0, 5.0),
        };
        let point = Vec3::new(1.0, -3.0, 7.0);
        assert!((plane.signed_distance(point).unwrap() - 5.0).abs() < 1.0e-14);
        assert_eq!(plane.project_point(point).unwrap(), Vec3::new(1.0, -3.0, 2.0));
    }

    #[test]
    fn rays_reject_negative_parameters_and_zero_directions() {
        let ray = Ray3 { origin: Vec3::new(0.0, 0.0, 0.0), direction: Vec3::new(1.0, 0.0, 0.0) };
        assert_eq!(ray.point_at(-1.0), Err(AnalyticError::InvalidParameter));
        assert_eq!(
            Ray3 { origin: Vec3::new(0.0, 0.0, 0.0), direction: Vec3::new(0.0, 0.0, 0.0) }.validate(),
            Err(AnalyticError::Degenerate)
        );
    }

    #[test]
    fn ellipse_parameterization_hits_axis_extrema() {
        let ellipse = Ellipse2 {
            center: Vec2::new(2.0, 3.0),
            semi_axis_a: 4.0,
            semi_axis_b: 2.0,
            rotation: 0.0,
        };
        assert_eq!(ellipse.point_at(0.0).unwrap(), Vec2::new(6.0, 3.0));
        assert_eq!(ellipse.point_at(0.25).unwrap(), Vec2::new(2.0, 5.0));
        assert!(ellipse.implicit_value(Vec2::new(6.0, 3.0)).unwrap().abs() < 1.0e-14);
    }

    #[test]
    fn cylinder_cone_and_torus_have_zero_residual_on_known_points() {
        let cylinder = Cylinder3 {
            origin: Vec3::new(0.0, 0.0, 0.0),
            axis: Vec3::new(0.0, 0.0, 2.0),
            radius: 3.0,
        };
        assert!(cylinder.implicit_value(Vec3::new(3.0, 0.0, 100.0)).unwrap().abs() < 1.0e-14);

        let cone = Cone3 {
            apex: Vec3::new(0.0, 0.0, 0.0),
            axis: Vec3::new(0.0, 0.0, 1.0),
            half_angle: std::f64::consts::FRAC_PI_4,
        };
        assert!(cone.implicit_value(Vec3::new(1.0, 0.0, 1.0)).unwrap().abs() < 1.0e-14);

        let torus = Torus3 {
            center: Vec3::new(0.0, 0.0, 0.0),
            axis: Vec3::new(0.0, 0.0, 1.0),
            major_radius: 5.0,
            minor_radius: 1.0,
        };
        assert!(torus.implicit_value(Vec3::new(6.0, 0.0, 0.0)).unwrap().abs() < 1.0e-14);
    }

    #[test]
    fn analytic_primitives_reject_nonfinite_inputs() {
        assert_eq!(
            Plane3 { origin: Vec3::new(0.0, 0.0, 0.0), normal: Vec3::new(f64::NAN, 0.0, 1.0) }.validate(),
            Err(AnalyticError::NonFinite)
        );
        assert_eq!(
            Ellipse2 { center: Vec2::new(0.0, 0.0), semi_axis_a: 1.0, semi_axis_b: 1.0, rotation: f64::INFINITY }.validate(),
            Err(AnalyticError::NonFinite)
        );
    }
}
