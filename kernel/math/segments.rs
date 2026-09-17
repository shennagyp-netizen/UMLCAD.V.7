//! Finite segment mathematics in 2D and 3D.
//!
//! Segments are bounded parameter domains `[0,1]`. Projection and containment
//! use explicit tolerances and never clamp invalid endpoints silently.

use super::{
    predicates::{is_between2d, is_between3d, Tri},
    vec::{Vec2, Vec3},
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Segment2 {
    pub start: Vec2,
    pub end: Vec2,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Segment3 {
    pub start: Vec3,
    pub end: Vec3,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SegmentError {
    NonFinite,
    Degenerate,
    InvalidTolerance,
    InvalidParameter,
    Overflow,
}

impl Segment2 {
    pub fn validate(&self) -> Result<(), SegmentError> {
        if !self.start.is_finite() || !self.end.is_finite() {
            return Err(SegmentError::NonFinite);
        }
        if self.end.sub(self.start).length() == 0.0 {
            return Err(SegmentError::Degenerate);
        }
        Ok(())
    }

    pub fn length(&self) -> f64 {
        self.end.sub(self.start).length()
    }

    pub fn point_at(&self, t: f64) -> Result<Vec2, SegmentError> {
        self.validate()?;
        if !t.is_finite() {
            return Err(SegmentError::NonFinite);
        }
        if !(0.0..=1.0).contains(&t) {
            return Err(SegmentError::InvalidParameter);
        }
        let displacement = self.end.sub(self.start);
        let point = self.start.add(displacement.scale(t));
        if point.is_finite() {
            Ok(point)
        } else {
            Err(SegmentError::Overflow)
        }
    }

    pub fn supporting_parameter(&self, p: Vec2) -> Result<f64, SegmentError> {
        self.validate()?;
        if !p.is_finite() {
            return Err(SegmentError::NonFinite);
        }
        let d = self.end.sub(self.start);
        let length = d.length();
        if !length.is_finite() || length == 0.0 {
            return Err(SegmentError::Degenerate);
        }
        let unit = d.scale(1.0 / length);
        let displacement = p.sub(self.start);
        if !displacement.is_finite() {
            return Err(SegmentError::Overflow);
        }
        let projection = displacement.dot(unit);
        if !projection.is_finite() {
            return Err(SegmentError::Overflow);
        }
        let parameter = projection / length;
        if parameter.is_finite() {
            Ok(parameter)
        } else {
            Err(SegmentError::Overflow)
        }
    }

    pub fn closest_parameter(&self, p: Vec2) -> Result<f64, SegmentError> {
        Ok(self.supporting_parameter(p)?.clamp(0.0, 1.0))
    }

    pub fn closest_point(&self, p: Vec2) -> Result<Vec2, SegmentError> {
        self.point_at(self.closest_parameter(p)?)
    }

    pub fn distance_to_point(&self, p: Vec2) -> Result<f64, SegmentError> {
        let closest = self.closest_point(p)?;
        let distance = p.sub(closest).length();
        if distance.is_finite() {
            Ok(distance)
        } else {
            Err(SegmentError::Overflow)
        }
    }

    pub fn contains(&self, p: Vec2, tol: f64) -> Tri {
        is_between2d(self.start, self.end, p, tol)
    }
}

impl Segment3 {
    pub fn validate(&self) -> Result<(), SegmentError> {
        if !self.start.is_finite() || !self.end.is_finite() {
            return Err(SegmentError::NonFinite);
        }
        if self.end.sub(self.start).length() == 0.0 {
            return Err(SegmentError::Degenerate);
        }
        Ok(())
    }

    pub fn length(&self) -> f64 {
        self.end.sub(self.start).length()
    }

    pub fn point_at(&self, t: f64) -> Result<Vec3, SegmentError> {
        self.validate()?;
        if !t.is_finite() {
            return Err(SegmentError::NonFinite);
        }
        if !(0.0..=1.0).contains(&t) {
            return Err(SegmentError::InvalidParameter);
        }
        let displacement = self.end.sub(self.start);
        let point = self.start.add(displacement.scale(t));
        if point.is_finite() {
            Ok(point)
        } else {
            Err(SegmentError::Overflow)
        }
    }

    pub fn supporting_parameter(&self, p: Vec3) -> Result<f64, SegmentError> {
        self.validate()?;
        if !p.is_finite() {
            return Err(SegmentError::NonFinite);
        }
        let d = self.end.sub(self.start);
        let length = d.length();
        if !length.is_finite() || length == 0.0 {
            return Err(SegmentError::Degenerate);
        }
        let unit = d.scale(1.0 / length);
        let displacement = p.sub(self.start);
        if !displacement.is_finite() {
            return Err(SegmentError::Overflow);
        }
        let projection = displacement.dot(unit);
        if !projection.is_finite() {
            return Err(SegmentError::Overflow);
        }
        let parameter = projection / length;
        if parameter.is_finite() {
            Ok(parameter)
        } else {
            Err(SegmentError::Overflow)
        }
    }

    pub fn closest_parameter(&self, p: Vec3) -> Result<f64, SegmentError> {
        Ok(self.supporting_parameter(p)?.clamp(0.0, 1.0))
    }

    pub fn closest_point(&self, p: Vec3) -> Result<Vec3, SegmentError> {
        self.point_at(self.closest_parameter(p)?)
    }

    pub fn distance_to_point(&self, p: Vec3) -> Result<f64, SegmentError> {
        let closest = self.closest_point(p)?;
        let distance = p.sub(closest).length();
        if distance.is_finite() {
            Ok(distance)
        } else {
            Err(SegmentError::Overflow)
        }
    }

    pub fn contains(&self, p: Vec3, tol: f64) -> Tri {
        is_between3d(self.start, self.end, p, tol)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn segment_projection_and_distance() {
        let s = Segment2 {
            start: Vec2::new(0.0, 0.0),
            end: Vec2::new(2.0, 0.0),
        };
        assert!((s.closest_parameter(Vec2::new(1.0, 3.0)).unwrap() - 0.5).abs() < 1.0e-15);
        assert!((s.distance_to_point(Vec2::new(1.0, 3.0)).unwrap() - 3.0).abs() < 1.0e-15);
    }

    #[test]
    fn segment_domain_is_closed() {
        let s = Segment3 {
            start: Vec3::new(0.0, 0.0, 0.0),
            end: Vec3::new(1.0, 0.0, 0.0),
        };
        assert_eq!(s.point_at(0.0).unwrap(), s.start);
        assert_eq!(s.point_at(1.0).unwrap(), s.end);
        assert_eq!(s.point_at(1.1), Err(SegmentError::InvalidParameter));
    }

    #[test]
    fn extreme_segment_length_does_not_square_direction() {
        let s = Segment3 {
            start: Vec3::new(0.0, 0.0, 0.0),
            end: Vec3::new(1.0e308, 0.0, 0.0),
        };
        let parameter = s.supporting_parameter(Vec3::new(5.0e307, 0.0, 0.0)).unwrap();
        assert!((parameter - 0.5).abs() < 1.0e-15);
    }

    #[test]
    fn degenerate_segment_fails_closed() {
        let s = Segment2 {
            start: Vec2::new(0.0, 0.0),
            end: Vec2::new(0.0, 0.0),
        };
        assert_eq!(s.validate(), Err(SegmentError::Degenerate));
    }
}
