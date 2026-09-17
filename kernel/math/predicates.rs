//! Tri-state geometric predicates for the UMLCAD mathematical authority.
//!
//! `Indeterminate` is deliberately observable. It represents non-finite input,
//! degenerate input, or a determinant that lies inside the caller-provided
//! tolerance band. No predicate silently maps that state to `False`.

use super::vec::{Vec2, Vec3};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tri {
    True,
    False,
    Indeterminate,
}

impl Tri {
    #[inline]
    pub fn is_true(self) -> bool {
        matches!(self, Self::True)
    }

    #[inline]
    pub fn is_false(self) -> bool {
        matches!(self, Self::False)
    }

    #[inline]
    pub fn is_indeterminate(self) -> bool {
        matches!(self, Self::Indeterminate)
    }

    /// Opt into fail-closed boolean behavior at an explicit caller boundary.
    #[inline]
    pub fn or_false(self) -> bool {
        self.is_true()
    }
}

fn valid_tolerance(tol: f64) -> bool {
    tol.is_finite() && tol >= 0.0
}

/// Sign of the 2D orientation determinant of `(a,b,c)`.
///
/// The vectors are normalized before the determinant is formed. Therefore the
/// tolerance is dimensionless and the mathematical classification is invariant
/// under uniform positive scaling, subject to finite floating-point arithmetic.
pub fn orient2d(a: Vec2, b: Vec2, c: Vec2, tol: f64) -> Tri {
    if !a.is_finite() || !b.is_finite() || !c.is_finite() || !valid_tolerance(tol) {
        return Tri::Indeterminate;
    }
    let ab = match b.sub(a).normalized() {
        Ok(v) => v,
        Err(_) => return Tri::Indeterminate,
    };
    let ac = match c.sub(a).normalized() {
        Ok(v) => v,
        Err(_) => return Tri::Indeterminate,
    };
    let det = ab.cross(ac);
    if !det.is_finite() {
        return Tri::Indeterminate;
    }
    if det > tol {
        Tri::True
    } else if det < -tol {
        Tri::False
    } else {
        Tri::Indeterminate
    }
}

pub fn is_collinear2d(a: Vec2, b: Vec2, c: Vec2, tol: f64) -> Tri {
    if !a.is_finite() || !b.is_finite() || !c.is_finite() || !valid_tolerance(tol) {
        return Tri::Indeterminate;
    }
    match orient2d(a, b, c, tol) {
        Tri::True | Tri::False => Tri::False,
        Tri::Indeterminate => Tri::True,
    }
}

pub fn is_parallel2d(a: Vec2, b: Vec2, tol: f64) -> Tri {
    if !a.is_finite() || !b.is_finite() || !valid_tolerance(tol) {
        return Tri::Indeterminate;
    }
    let a = match a.normalized() {
        Ok(v) => v,
        Err(_) => return Tri::Indeterminate,
    };
    let b = match b.normalized() {
        Ok(v) => v,
        Err(_) => return Tri::Indeterminate,
    };
    let cross = a.cross(b).abs();
    if !cross.is_finite() {
        Tri::Indeterminate
    } else if cross <= tol {
        Tri::True
    } else {
        Tri::False
    }
}

pub fn is_perpendicular2d(a: Vec2, b: Vec2, tol: f64) -> Tri {
    if !a.is_finite() || !b.is_finite() || !valid_tolerance(tol) {
        return Tri::Indeterminate;
    }
    let a = match a.normalized() {
        Ok(v) => v,
        Err(_) => return Tri::Indeterminate,
    };
    let b = match b.normalized() {
        Ok(v) => v,
        Err(_) => return Tri::Indeterminate,
    };
    let dot = a.dot(b).abs();
    if !dot.is_finite() {
        Tri::Indeterminate
    } else if dot <= tol {
        Tri::True
    } else {
        Tri::False
    }
}

/// Whether `p` lies on the closed segment `[a,b]` within the dimensionless
/// angular tolerance `tol` and an exact/relative projection test.
pub fn is_between2d(a: Vec2, b: Vec2, p: Vec2, tol: f64) -> Tri {
    if !a.is_finite() || !b.is_finite() || !p.is_finite() || !valid_tolerance(tol) {
        return Tri::Indeterminate;
    }
    let ab_raw = b.sub(a);
    let ab = match ab_raw.normalized() {
        Ok(v) => v,
        Err(_) => return if p == a { Tri::True } else { Tri::False },
    };
    let ap = p.sub(a);
    let cross = ab.cross(ap).abs();
    let ap_len = ap.length();
    if !cross.is_finite() || !ap_len.is_finite() {
        return Tri::Indeterminate;
    }
    if cross > tol * ap_len {
        return Tri::False;
    }

    let projection = ap.dot(ab);
    let length = ab_raw.length();
    if !projection.is_finite() || !length.is_finite() {
        return Tri::Indeterminate;
    }
    let distance_scale = length.max(ap_len).max(1.0e-300);
    let projection_tol = tol * distance_scale;
    if projection < -projection_tol || projection > length + projection_tol {
        Tri::False
    } else {
        Tri::True
    }
}

pub fn is_same_side_2d(line_a: Vec2, line_b: Vec2, p: Vec2, q: Vec2, tol: f64) -> Tri {
    if !line_a.is_finite()
        || !line_b.is_finite()
        || !p.is_finite()
        || !q.is_finite()
        || !valid_tolerance(tol)
    {
        return Tri::Indeterminate;
    }
    if line_b.sub(line_a).length() == 0.0 {
        return Tri::Indeterminate;
    }
    match (
        orient2d(line_a, line_b, p, tol),
        orient2d(line_a, line_b, q, tol),
    ) {
        (Tri::True, Tri::True) | (Tri::False, Tri::False) => Tri::True,
        (Tri::True, Tri::False) | (Tri::False, Tri::True) => Tri::False,
        _ => Tri::Indeterminate,
    }
}

/// Sign of the 3D orientation determinant of `(a,b,c,d)`.
///
/// All three edge vectors are normalized before evaluation, preventing a
/// determinant overflow when the input scale is still representable.
pub fn orient3d(a: Vec3, b: Vec3, c: Vec3, d: Vec3, tol: f64) -> Tri {
    if !a.is_finite()
        || !b.is_finite()
        || !c.is_finite()
        || !d.is_finite()
        || !valid_tolerance(tol)
    {
        return Tri::Indeterminate;
    }
    let ab = match b.sub(a).normalized() {
        Ok(v) => v,
        Err(_) => return Tri::Indeterminate,
    };
    let ac = match c.sub(a).normalized() {
        Ok(v) => v,
        Err(_) => return Tri::Indeterminate,
    };
    let ad = match d.sub(a).normalized() {
        Ok(v) => v,
        Err(_) => return Tri::Indeterminate,
    };
    let det = ab.cross(ac).dot(ad);
    if !det.is_finite() {
        return Tri::Indeterminate;
    }
    if det > tol {
        Tri::True
    } else if det < -tol {
        Tri::False
    } else {
        Tri::Indeterminate
    }
}

pub fn is_coplanar(a: Vec3, b: Vec3, c: Vec3, d: Vec3, tol: f64) -> Tri {
    if !a.is_finite() || !b.is_finite() || !c.is_finite() || !d.is_finite() || !valid_tolerance(tol) {
        return Tri::Indeterminate;
    }
    match orient3d(a, b, c, d, tol) {
        Tri::True | Tri::False => Tri::False,
        Tri::Indeterminate => Tri::True,
    }
}

pub fn is_parallel3d(a: Vec3, b: Vec3, tol: f64) -> Tri {
    if !a.is_finite() || !b.is_finite() || !valid_tolerance(tol) {
        return Tri::Indeterminate;
    }
    let a = match a.normalized() {
        Ok(v) => v,
        Err(_) => return Tri::Indeterminate,
    };
    let b = match b.normalized() {
        Ok(v) => v,
        Err(_) => return Tri::Indeterminate,
    };
    let cross = a.cross(b).length();
    if !cross.is_finite() {
        Tri::Indeterminate
    } else if cross <= tol {
        Tri::True
    } else {
        Tri::False
    }
}

pub fn is_perpendicular3d(a: Vec3, b: Vec3, tol: f64) -> Tri {
    if !a.is_finite() || !b.is_finite() || !valid_tolerance(tol) {
        return Tri::Indeterminate;
    }
    let a = match a.normalized() {
        Ok(v) => v,
        Err(_) => return Tri::Indeterminate,
    };
    let b = match b.normalized() {
        Ok(v) => v,
        Err(_) => return Tri::Indeterminate,
    };
    let dot = a.dot(b).abs();
    if !dot.is_finite() {
        Tri::Indeterminate
    } else if dot <= tol {
        Tri::True
    } else {
        Tri::False
    }
}

pub fn is_between3d(a: Vec3, b: Vec3, p: Vec3, tol: f64) -> Tri {
    if !a.is_finite() || !b.is_finite() || !p.is_finite() || !valid_tolerance(tol) {
        return Tri::Indeterminate;
    }
    let ab_raw = b.sub(a);
    let ab = match ab_raw.normalized() {
        Ok(v) => v,
        Err(_) => return if p == a { Tri::True } else { Tri::False },
    };
    let ap = p.sub(a);
    let cross = ab.cross(ap).length();
    let ap_len = ap.length();
    if !cross.is_finite() || !ap_len.is_finite() {
        return Tri::Indeterminate;
    }
    if cross > tol * ap_len {
        return Tri::False;
    }
    let projection = ap.dot(ab);
    let length = ab_raw.length();
    if !projection.is_finite() || !length.is_finite() {
        return Tri::Indeterminate;
    }
    let projection_tol = tol * length.max(ap_len).max(1.0e-300);
    if projection < -projection_tol || projection > length + projection_tol {
        Tri::False
    } else {
        Tri::True
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOL: f64 = 1.0e-12;

    #[test]
    fn orientation_has_expected_signs() {
        assert_eq!(
            orient2d(Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0), Vec2::new(0.0, 1.0), TOL),
            Tri::True
        );
        assert_eq!(
            orient2d(Vec2::new(0.0, 0.0), Vec2::new(0.0, 1.0), Vec2::new(1.0, 0.0), TOL),
            Tri::False
        );
        assert_eq!(
            orient3d(
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(1.0, 0.0, 0.0),
                Vec3::new(0.0, 1.0, 0.0),
                Vec3::new(0.0, 0.0, 1.0),
                TOL,
            ),
            Tri::True
        );
    }

    #[test]
    fn uncertain_cases_are_not_false() {
        assert_eq!(
            orient2d(
                Vec2::new(0.0, 0.0),
                Vec2::new(1.0, 0.0),
                Vec2::new(2.0, 1.0e-15),
                TOL,
            ),
            Tri::Indeterminate
        );
        assert_eq!(
            is_parallel2d(Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0), TOL),
            Tri::Indeterminate
        );
    }

    #[test]
    fn parallel_and_perpendicular_are_scale_invariant() {
        assert_eq!(
            is_parallel2d(Vec2::new(1.0e-12, 0.0), Vec2::new(2.0e12, 0.0), TOL),
            Tri::True
        );
        assert_eq!(
            is_perpendicular3d(
                Vec3::new(1.0e-12, 0.0, 0.0),
                Vec3::new(0.0, 5.0e12, 0.0),
                TOL,
            ),
            Tri::True
        );
    }

    #[test]
    fn between_checks_collinearity_and_projection() {
        assert_eq!(
            is_between2d(Vec2::new(0.0, 0.0), Vec2::new(2.0, 0.0), Vec2::new(1.0, 0.0), TOL),
            Tri::True
        );
        assert_eq!(
            is_between2d(Vec2::new(0.0, 0.0), Vec2::new(2.0, 0.0), Vec2::new(3.0, 0.0), TOL),
            Tri::False
        );
        assert_eq!(
            is_between3d(Vec3::new(0.0, 0.0, 0.0), Vec3::new(2.0, 0.0, 0.0), Vec3::new(1.0, 1.0, 0.0), TOL),
            Tri::False
        );
    }

    #[test]
    fn nonfinite_input_is_indeterminate() {
        assert_eq!(
            orient2d(Vec2::new(0.0, 0.0), Vec2::new(f64::NAN, 1.0), Vec2::new(1.0, 1.0), TOL),
            Tri::Indeterminate
        );
        assert_eq!(
            orient3d(
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(1.0, 0.0, 0.0),
                Vec3::new(0.0, 1.0, 0.0),
                Vec3::new(f64::INFINITY, 0.0, 1.0),
                TOL,
            ),
            Tri::Indeterminate
        );
    }
}
