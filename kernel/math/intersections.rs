//! Analytic intersection mathematics for supported primitive families.
//!
//! Each operation classifies geometric degeneracy explicitly. Tolerances are
//! dimensionless relative inputs applied to the natural scale of each equation.

use super::{
    analytic::{Ellipse2, Plane3},
    conics3d::{Circle3, Sphere3},
    geometry::{Circle, Line},
    predicates::Tri,
    vec::{Vec2, Vec3},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IntersectionKind {
    None,
    Point,
    MultiplePoints,
    Circle,
    Line,
    Tangent,
    Parallel,
    Coincident,
    Skew,
    Degenerate,
    Indeterminate,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Intersection2D {
    pub kind: IntersectionKind,
    pub points: Vec<Vec2>,
    pub parameters: Vec<f64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Intersection3D {
    pub kind: IntersectionKind,
    pub points: Vec<Vec3>,
    pub parameters: Vec<f64>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Line2 {
    pub origin: Vec2,
    pub direction: Vec2,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Line3 {
    pub origin: Vec3,
    pub direction: Vec3,
}

impl Line2 {
    fn valid(&self) -> bool {
        self.origin.is_finite() && self.direction.is_finite() && self.direction.length() > 0.0
    }
}

impl Line3 {
    fn valid(&self) -> bool {
        self.origin.is_finite() && self.direction.is_finite() && self.direction.length() > 0.0
    }
}

fn valid_tol(tolerance: f64) -> bool {
    tolerance.is_finite() && tolerance >= 0.0
}

fn invalid_2d(kind: IntersectionKind) -> Intersection2D {
    Intersection2D {
        kind,
        points: Vec::new(),
        parameters: Vec::new(),
    }
}

fn invalid_3d(kind: IntersectionKind) -> Intersection3D {
    Intersection3D {
        kind,
        points: Vec::new(),
        parameters: Vec::new(),
    }
}

pub fn line_line_2d(a: Line2, b: Line2, tol: f64) -> Intersection2D {
    if !valid_tol(tol) || !a.valid() || !b.valid() {
        return invalid_2d(IntersectionKind::Degenerate);
    }
    let a_length = a.direction.length();
    let b_length = b.direction.length();
    let ua = match a.direction.normalized() {
        Ok(value) => value,
        Err(_) => return invalid_2d(IntersectionKind::Degenerate),
    };
    let ub = match b.direction.normalized() {
        Ok(value) => value,
        Err(_) => return invalid_2d(IntersectionKind::Degenerate),
    };
    let determinant = ua.cross(ub);
    let offset = b.origin.sub(a.origin);
    if !determinant.is_finite() || !offset.is_finite() || !a_length.is_finite() || !b_length.is_finite() {
        return invalid_2d(IntersectionKind::Indeterminate);
    }

    if determinant.abs() <= tol {
        let separation = offset.cross(ua).abs();
        let scale = offset.length();
        if !separation.is_finite() || !scale.is_finite() {
            return invalid_2d(IntersectionKind::Indeterminate);
        }
        let band = tol * scale;
        if !band.is_finite() {
            return invalid_2d(IntersectionKind::Indeterminate);
        }
        if separation <= band {
            invalid_2d(IntersectionKind::Coincident)
        } else {
            invalid_2d(IntersectionKind::Parallel)
        }
    } else {
        let ta_distance = offset.cross(ub) / determinant;
        let tb_distance = offset.cross(ua) / determinant;
        let ta = ta_distance / a_length;
        let tb = tb_distance / b_length;
        if !ta.is_finite() || !tb.is_finite() || !ta_distance.is_finite() || !tb_distance.is_finite() {
            return invalid_2d(IntersectionKind::Indeterminate);
        }
        let point = a.origin.add(ua.scale(ta_distance));
        if point.is_finite() {
            Intersection2D {
                kind: IntersectionKind::Point,
                points: vec![point],
                parameters: vec![ta, tb],
            }
        } else {
            invalid_2d(IntersectionKind::Indeterminate)
        }
    }
}

pub fn line_circle_2d(line: Line2, circle: Circle, tol: f64) -> Intersection2D {
    if !valid_tol(tol) || !line.valid() || circle.validate().is_err() {
        return invalid_2d(IntersectionKind::Degenerate);
    }
    let direction_length = line.direction.length();
    let unit = match line.direction.normalized() {
        Ok(value) => value,
        Err(_) => return invalid_2d(IntersectionKind::Degenerate),
    };
    let offset = line.origin.sub(circle.center);
    if !offset.is_finite() || !direction_length.is_finite() {
        return invalid_2d(IntersectionKind::Indeterminate);
    }
    let projection = offset.dot(unit);
    let radial_vector = offset.sub(unit.scale(projection));
    let radial_distance = radial_vector.length();
    if !projection.is_finite() || !radial_vector.is_finite() || !radial_distance.is_finite() {
        return invalid_2d(IntersectionKind::Indeterminate);
    }

    let gap = circle.radius - radial_distance;
    let scale = circle.radius.max(radial_distance);
    let band = tol * scale;
    if !gap.is_finite() || !band.is_finite() {
        return invalid_2d(IntersectionKind::Indeterminate);
    }
    if gap < -band {
        return invalid_2d(IntersectionKind::None);
    }
    if gap.abs() <= band {
        let distance = -projection;
        let parameter = distance / direction_length;
        let point = line.origin.add(unit.scale(distance));
        if parameter.is_finite() && point.is_finite() {
            return Intersection2D {
                kind: IntersectionKind::Tangent,
                points: vec![point],
                parameters: vec![parameter],
            };
        }
        return invalid_2d(IntersectionKind::Indeterminate);
    }

    let half_chord = (circle.radius - radial_distance).sqrt() * (circle.radius + radial_distance).sqrt();
    if !half_chord.is_finite() {
        return invalid_2d(IntersectionKind::Indeterminate);
    }
    let distance_a = -projection - half_chord;
    let distance_b = -projection + half_chord;
    let parameter_a = distance_a / direction_length;
    let parameter_b = distance_b / direction_length;
    let point_a = line.origin.add(unit.scale(distance_a));
    let point_b = line.origin.add(unit.scale(distance_b));
    if !parameter_a.is_finite()
        || !parameter_b.is_finite()
        || !point_a.is_finite()
        || !point_b.is_finite()
    {
        return invalid_2d(IntersectionKind::Indeterminate);
    }
    Intersection2D {
        kind: IntersectionKind::MultiplePoints,
        points: vec![point_a, point_b],
        parameters: vec![parameter_a, parameter_b],
    }
}

pub fn circle_circle_2d(a: Circle, b: Circle, tol: f64) -> Intersection2D {
    if !valid_tol(tol) || a.validate().is_err() || b.validate().is_err() {
        return invalid_2d(IntersectionKind::Degenerate);
    }
    let delta = b.center.sub(a.center);
    let distance = delta.length();
    if !delta.is_finite() || !distance.is_finite() {
        return invalid_2d(IntersectionKind::Indeterminate);
    }
    let scale = a.radius.max(b.radius).max(distance);
    let band = tol * scale;
    if !band.is_finite() {
        return invalid_2d(IntersectionKind::Indeterminate);
    }
    if distance <= band {
        if (a.radius - b.radius).abs() <= band {
            return invalid_2d(IntersectionKind::Coincident);
        }
        return invalid_2d(IntersectionKind::None);
    }

    let normalized_a = a.radius / scale;
    let normalized_b = b.radius / scale;
    let normalized_d = distance / scale;
    let sum = normalized_a + normalized_b;
    let difference = (normalized_a - normalized_b).abs();
    if normalized_d > sum + tol || normalized_d < difference - tol {
        return invalid_2d(IntersectionKind::None);
    }

    let x_normalized = (normalized_a * normalized_a - normalized_b * normalized_b
        + normalized_d * normalized_d)
        / (2.0 * normalized_d);
    if !x_normalized.is_finite() {
        return invalid_2d(IntersectionKind::Indeterminate);
    }
    let h2_normalized = normalized_a * normalized_a - x_normalized * x_normalized;
    let h_band = tol * scale;
    let h_band_squared = if h_band <= scale {
        tol * (scale / scale)
    } else {
        tol
    };
    if !h_band_squared.is_finite() {
        return invalid_2d(IntersectionKind::Indeterminate);
    }
    if h2_normalized < 0.0 && h2_normalized.abs() > tol * scale.max(f64::MIN_POSITIVE) / scale {
        return invalid_2d(IntersectionKind::Indeterminate);
    }
    if h2_normalized <= tol {
        let x = x_normalized * scale;
        let base = a.center.add(delta.scale(x / distance));
        if base.is_finite() {
            return Intersection2D {
                kind: IntersectionKind::Tangent,
                points: vec![base],
                parameters: Vec::new(),
            };
        }
        return invalid_2d(IntersectionKind::Indeterminate);
    }
    if h2_normalized < 0.0 {
        return invalid_2d(IntersectionKind::Indeterminate);
    }
    let h = h2_normalized.sqrt() * scale;
    let x = x_normalized * scale;
    let base = a.center.add(delta.scale(x / distance));
    let unit_perp = Vec2::new(-delta.y / distance, delta.x / distance);
    let point_a = base.add(unit_perp.scale(h));
    let point_b = base.sub(unit_perp.scale(h));
    if !h.is_finite() || !base.is_finite() || !point_a.is_finite() || !point_b.is_finite() {
        return invalid_2d(IntersectionKind::Indeterminate);
    }
    Intersection2D {
        kind: IntersectionKind::MultiplePoints,
        points: vec![point_a, point_b],
        parameters: Vec::new(),
    }
}

pub fn ellipse_line_2d(ellipse: Ellipse2, line: Line2, tol: f64) -> Intersection2D {
    if !valid_tol(tol) || !line.valid() || ellipse.validate().is_err() {
        return invalid_2d(IntersectionKind::Degenerate);
    }
    let direction_length = line.direction.length();
    let unit_direction = match line.direction.normalized() {
        Ok(value) => value,
        Err(_) => return invalid_2d(IntersectionKind::Degenerate),
    };
    let rotation_c = ellipse.rotation.cos();
    let rotation_s = ellipse.rotation.sin();
    let local = |p: Vec2| {
        let d = p.sub(ellipse.center);
        Vec2::new(
            rotation_c * d.x + rotation_s * d.y,
            -rotation_s * d.x + rotation_c * d.y,
        )
    };
    let origin = local(line.origin);
    let direction = Vec2::new(
        rotation_c * unit_direction.x + rotation_s * unit_direction.y,
        -rotation_s * unit_direction.x + rotation_c * unit_direction.y,
    );
    let ux = direction.x / ellipse.semi_axis_a;
    let uy = direction.y / ellipse.semi_axis_b;
    let ox = origin.x / ellipse.semi_axis_a;
    let oy = origin.y / ellipse.semi_axis_b;
    if [ux, uy, ox, oy].iter().any(|value| !value.is_finite()) {
        return invalid_2d(IntersectionKind::Indeterminate);
    }
    let coefficient_scale = 1.0_f64.max(ux.abs()).max(uy.abs()).max(ox.abs()).max(oy.abs());
    let ux = ux / coefficient_scale;
    let uy = uy / coefficient_scale;
    let ox = ox / coefficient_scale;
    let oy = oy / coefficient_scale;
    let inv_scale_sq = 1.0 / (coefficient_scale * coefficient_scale);
    if !inv_scale_sq.is_finite() {
        return invalid_2d(IntersectionKind::Indeterminate);
    }
    let quadratic_a = ux * ux + uy * uy;
    let quadratic_b = 2.0 * (ox * ux + oy * uy);
    let quadratic_c = ox * ox + oy * oy - inv_scale_sq;
    let discriminant = quadratic_b * quadratic_b - 4.0 * quadratic_a * quadratic_c;
    if ![quadratic_a, quadratic_b, quadratic_c, discriminant]
        .iter()
        .all(|value| value.is_finite())
    {
        return invalid_2d(IntersectionKind::Indeterminate);
    }
    if quadratic_a == 0.0 {
        return invalid_2d(IntersectionKind::Degenerate);
    }
    let band = tol * (quadratic_b * quadratic_b).abs().max((4.0 * quadratic_a * quadratic_c).abs());
    if !band.is_finite() {
        return invalid_2d(IntersectionKind::Indeterminate);
    }
    if discriminant < -band {
        return invalid_2d(IntersectionKind::None);
    }
    let t_distance = if discriminant.abs() <= band {
        -quadratic_b / (2.0 * quadratic_a)
    } else {
        if discriminant < 0.0 {
            return invalid_2d(IntersectionKind::Indeterminate);
        }
        let root = discriminant.sqrt();
        let t1 = (-quadratic_b - root) / (2.0 * quadratic_a);
        let t2 = (-quadratic_b + root) / (2.0 * quadratic_a);
        let parameter_1 = t1 * coefficient_scale / direction_length;
        let parameter_2 = t2 * coefficient_scale / direction_length;
        let point_1 = line.origin.add(line.direction.scale(parameter_1));
        let point_2 = line.origin.add(line.direction.scale(parameter_2));
        if !parameter_1.is_finite()
            || !parameter_2.is_finite()
            || !point_1.is_finite()
            || !point_2.is_finite()
        {
            return invalid_2d(IntersectionKind::Indeterminate);
        }
        return Intersection2D {
            kind: IntersectionKind::MultiplePoints,
            points: vec![point_1, point_2],
            parameters: vec![parameter_1, parameter_2],
        };
    };
    let parameter = t_distance * coefficient_scale / direction_length;
    let point = line.origin.add(line.direction.scale(parameter));
    if !parameter.is_finite() || !point.is_finite() {
        return invalid_2d(IntersectionKind::Indeterminate);
    }
    Intersection2D {
        kind: IntersectionKind::Tangent,
        points: vec![point],
        parameters: vec![parameter],
    }
}

pub fn line_plane_3d(line: Line3, plane: Plane3, tol: f64) -> Intersection3D {
    if !valid_tol(tol) || !line.valid() || plane.validate().is_err() {
        return invalid_3d(IntersectionKind::Degenerate);
    }
    let normal = match plane.unit_normal() {
        Ok(value) => value,
        Err(_) => return invalid_3d(IntersectionKind::Degenerate),
    };
    let direction_length = line.direction.length();
    let denominator = normal.dot(line.direction);
    let displacement = plane.origin.sub(line.origin);
    let numerator = normal.dot(displacement);
    if !direction_length.is_finite() || !denominator.is_finite() || !displacement.is_finite() || !numerator.is_finite() {
        return invalid_3d(IntersectionKind::Indeterminate);
    }
    let parallel_band = tol * direction_length;
    if !parallel_band.is_finite() {
        return invalid_3d(IntersectionKind::Indeterminate);
    }
    if denominator.abs() <= parallel_band {
        let separation = numerator.abs();
        let separation_band = tol * displacement.length();
        if !separation.is_finite() || !separation_band.is_finite() {
            return invalid_3d(IntersectionKind::Indeterminate);
        }
        if separation <= separation_band {
            invalid_3d(IntersectionKind::Coincident)
        } else {
            invalid_3d(IntersectionKind::Parallel)
        }
    } else {
        let parameter = numerator / denominator;
        let point = line.origin.add(line.direction.scale(parameter));
        if parameter.is_finite() && point.is_finite() {
            Intersection3D {
                kind: IntersectionKind::Point,
                points: vec![point],
                parameters: vec![parameter],
            }
        } else {
            invalid_3d(IntersectionKind::Indeterminate)
        }
    }
}

pub fn plane_plane_3d(a: Plane3, b: Plane3, tol: f64) -> Intersection3D {
    if !valid_tol(tol) || a.validate().is_err() || b.validate().is_err() {
        return invalid_3d(IntersectionKind::Degenerate);
    }
    let n1 = match a.unit_normal() {
        Ok(value) => value,
        Err(_) => return invalid_3d(IntersectionKind::Degenerate),
    };
    let n2 = match b.unit_normal() {
        Ok(value) => value,
        Err(_) => return invalid_3d(IntersectionKind::Degenerate),
    };
    let direction = n1.cross(n2);
    let direction_length = direction.length();
    if !direction.is_finite() || !direction_length.is_finite() {
        return invalid_3d(IntersectionKind::Indeterminate);
    }
    if direction_length <= tol {
        let distance = a.signed_distance(b.origin);
        let origin_delta = b.origin.sub(a.origin);
        let distance = match distance {
            Ok(value) => value.abs(),
            Err(_) => return invalid_3d(IntersectionKind::Indeterminate),
        };
        if !origin_delta.is_finite() {
            return invalid_3d(IntersectionKind::Indeterminate);
        }
        let band = tol * origin_delta.length();
        if !band.is_finite() {
            return invalid_3d(IntersectionKind::Indeterminate);
        }
        if distance <= band {
            return invalid_3d(IntersectionKind::Coincident);
        }
        return invalid_3d(IntersectionKind::Parallel);
    }

    // Choose a coordinate to zero along the strongest direction component and
    // solve the remaining 2×2 system. This avoids constructing squared normal
    // magnitudes or multiplying large plane offsets together.
    let dominant = if direction.x.abs() >= direction.y.abs() && direction.x.abs() >= direction.z.abs() {
        0
    } else if direction.y.abs() >= direction.z.abs() {
        1
    } else {
        2
    };
    let c1 = n1.dot(a.origin);
    let c2 = n2.dot(b.origin);
    if !c1.is_finite() || !c2.is_finite() {
        return invalid_3d(IntersectionKind::Indeterminate);
    }

    let (x, y, z) = match dominant {
        0 => {
            let det = n1.y * n2.z - n1.z * n2.y;
            if det.abs() <= f64::MIN_POSITIVE {
                return invalid_3d(IntersectionKind::Indeterminate);
            }
            ((0.0), (c1 * n2.z - n1.z * c2) / det, (n1.y * c2 - c1 * n2.y) / det)
        }
        1 => {
            let det = n1.x * n2.z - n1.z * n2.x;
            if det.abs() <= f64::MIN_POSITIVE {
                return invalid_3d(IntersectionKind::Indeterminate);
            }
            (((c1 * n2.z - n1.z * c2) / det), 0.0, ((n1.x * c2 - c1 * n2.x) / det))
        }
        _ => {
            let det = n1.x * n2.y - n1.y * n2.x;
            if det.abs() <= f64::MIN_POSITIVE {
                return invalid_3d(IntersectionKind::Indeterminate);
            }
            (((c1 * n2.y - n1.y * c2) / det), ((n1.x * c2 - c1 * n2.x) / det), 0.0)
        }
    };
    let point = Vec3::new(x, y, z);
    if point.is_finite() {
        Intersection3D {
            kind: IntersectionKind::Line,
            points: vec![point],
            parameters: vec![0.0],
        }
    } else {
        invalid_3d(IntersectionKind::Indeterminate)
    }
}

pub fn line_sphere_3d(line: Line3, sphere: Sphere3, tol: f64) -> Intersection3D {
    if !valid_tol(tol) || !line.valid() || sphere.validate().is_err() {
        return invalid_3d(IntersectionKind::Degenerate);
    }
    let direction_length = line.direction.length();
    let unit = match line.direction.normalized() {
        Ok(value) => value,
        Err(_) => return invalid_3d(IntersectionKind::Degenerate),
    };
    let offset = line.origin.sub(sphere.center);
    let projection = offset.dot(unit);
    let radial_vector = offset.sub(unit.scale(projection));
    let radial_distance = radial_vector.length();
    if !offset.is_finite()
        || !projection.is_finite()
        || !radial_vector.is_finite()
        || !radial_distance.is_finite()
    {
        return invalid_3d(IntersectionKind::Indeterminate);
    }
    let gap = sphere.radius - radial_distance;
    let band = tol * sphere.radius.max(radial_distance);
    if !gap.is_finite() || !band.is_finite() {
        return invalid_3d(IntersectionKind::Indeterminate);
    }
    if gap < -band {
        return invalid_3d(IntersectionKind::None);
    }
    if gap.abs() <= band {
        let distance = -projection;
        let parameter = distance / direction_length;
        let point = line.origin.add(unit.scale(distance));
        if parameter.is_finite() && point.is_finite() {
            return Intersection3D {
                kind: IntersectionKind::Tangent,
                points: vec![point],
                parameters: vec![parameter],
            };
        }
        return invalid_3d(IntersectionKind::Indeterminate);
    }
    let half_chord = (sphere.radius - radial_distance).sqrt()
        * (sphere.radius + radial_distance).sqrt();
    if !half_chord.is_finite() {
        return invalid_3d(IntersectionKind::Indeterminate);
    }
    let distance_a = -projection - half_chord;
    let distance_b = -projection + half_chord;
    let parameter_a = distance_a / direction_length;
    let parameter_b = distance_b / direction_length;
    let point_a = line.origin.add(unit.scale(distance_a));
    let point_b = line.origin.add(unit.scale(distance_b));
    if !parameter_a.is_finite()
        || !parameter_b.is_finite()
        || !point_a.is_finite()
        || !point_b.is_finite()
    {
        return invalid_3d(IntersectionKind::Indeterminate);
    }
    Intersection3D {
        kind: IntersectionKind::MultiplePoints,
        points: vec![point_a, point_b],
        parameters: vec![parameter_a, parameter_b],
    }
}

pub fn plane_sphere_3d(plane: Plane3, sphere: Sphere3, tol: f64) -> Intersection3D {
    if !valid_tol(tol) || plane.validate().is_err() || sphere.validate().is_err() {
        return invalid_3d(IntersectionKind::Degenerate);
    }
    let distance = match plane.signed_distance(sphere.center) {
        Ok(value) => value.abs(),
        Err(_) => return invalid_3d(IntersectionKind::Indeterminate),
    };
    let band = tol * sphere.radius;
    if !distance.is_finite() || !band.is_finite() {
        return invalid_3d(IntersectionKind::Indeterminate);
    }
    if distance > sphere.radius + band {
        return invalid_3d(IntersectionKind::None);
    }
    let projected = match plane.project_point(sphere.center) {
        Ok(value) => value,
        Err(_) => return invalid_3d(IntersectionKind::Indeterminate),
    };
    if (distance - sphere.radius).abs() <= band {
        return if projected.is_finite() {
            Intersection3D {
                kind: IntersectionKind::Tangent,
                points: vec![projected],
                parameters: Vec::new(),
            }
        } else {
            invalid_3d(IntersectionKind::Indeterminate)
        };
    }
    let half_chord = (sphere.radius - distance).sqrt() * (sphere.radius + distance).sqrt();
    if !half_chord.is_finite() || !projected.is_finite() {
        return invalid_3d(IntersectionKind::Indeterminate);
    }
    Intersection3D {
        kind: IntersectionKind::Circle,
        points: vec![projected],
        parameters: vec![half_chord],
    }
}

pub fn plane_circle_3d(plane: Plane3, circle: Circle3, tol: f64) -> Intersection3D {
    if !valid_tol(tol) || plane.validate().is_err() || circle.validate().is_err() {
        return invalid_3d(IntersectionKind::Degenerate);
    }
    let circle_normal = match circle.unit_normal() {
        Ok(value) => value,
        Err(_) => return invalid_3d(IntersectionKind::Degenerate),
    };
    let plane_normal = match plane.unit_normal() {
        Ok(value) => value,
        Err(_) => return invalid_3d(IntersectionKind::Degenerate),
    };
    let parallel = circle_normal.cross(plane_normal).length();
    if !parallel.is_finite() {
        return invalid_3d(IntersectionKind::Indeterminate);
    }
    if parallel <= tol {
        let distance = match plane.signed_distance(circle.center) {
            Ok(value) => value.abs(),
            Err(_) => return invalid_3d(IntersectionKind::Indeterminate),
        };
        let band = tol * circle.radius;
        if !band.is_finite() || !distance.is_finite() {
            return invalid_3d(IntersectionKind::Indeterminate);
        }
        if distance <= band {
            return invalid_3d(IntersectionKind::Coincident);
        }
        return invalid_3d(IntersectionKind::None);
    }

    let distance = match plane.signed_distance(circle.center) {
        Ok(value) => value.abs(),
        Err(_) => return invalid_3d(IntersectionKind::Indeterminate),
    };
    let band = tol * circle.radius;
    if !distance.is_finite() || !band.is_finite() {
        return invalid_3d(IntersectionKind::Indeterminate);
    }
    if distance > circle.radius + band {
        return invalid_3d(IntersectionKind::None);
    }
    let projected = match plane.project_point(circle.center) {
        Ok(value) => value,
        Err(_) => return invalid_3d(IntersectionKind::Indeterminate),
    };
    if (distance - circle.radius).abs() <= band {
        return if projected.is_finite() {
            Intersection3D {
                kind: IntersectionKind::Tangent,
                points: vec![projected],
                parameters: Vec::new(),
            }
        } else {
            invalid_3d(IntersectionKind::Indeterminate)
        };
    }
    let half_chord = (circle.radius - distance).sqrt() * (circle.radius + distance).sqrt();
    let chord_direction = match circle_normal.cross(plane_normal).normalized() {
        Ok(value) => value,
        Err(_) => return invalid_3d(IntersectionKind::Indeterminate),
    };
    if !half_chord.is_finite() || !projected.is_finite() {
        return invalid_3d(IntersectionKind::Indeterminate);
    }
    let point_a = projected.add(chord_direction.scale(half_chord));
    let point_b = projected.sub(chord_direction.scale(half_chord));
    if !point_a.is_finite() || !point_b.is_finite() {
        return invalid_3d(IntersectionKind::Indeterminate);
    }
    Intersection3D {
        kind: IntersectionKind::MultiplePoints,
        points: vec![point_a, point_b],
        parameters: Vec::new(),
    }
}

pub fn sphere_sphere_3d(a: Sphere3, b: Sphere3, tol: f64) -> Intersection3D {
    if !valid_tol(tol) || a.validate().is_err() || b.validate().is_err() {
        return invalid_3d(IntersectionKind::Degenerate);
    }
    let delta = b.center.sub(a.center);
    let distance = delta.length();
    if !delta.is_finite() || !distance.is_finite() {
        return invalid_3d(IntersectionKind::Indeterminate);
    }
    let scale = a.radius.max(b.radius).max(distance);
    let band = tol * scale;
    if !band.is_finite() {
        return invalid_3d(IntersectionKind::Indeterminate);
    }
    if distance <= band {
        if (a.radius - b.radius).abs() <= band {
            return invalid_3d(IntersectionKind::Coincident);
        }
        return invalid_3d(IntersectionKind::None);
    }

    let ra = a.radius / scale;
    let rb = b.radius / scale;
    let d = distance / scale;
    let sum = ra + rb;
    let difference = (ra - rb).abs();
    if d > sum + tol || d < difference - tol {
        return invalid_3d(IntersectionKind::None);
    }
    let x_normalized = (ra * ra - rb * rb + d * d) / (2.0 * d);
    if !x_normalized.is_finite() {
        return invalid_3d(IntersectionKind::Indeterminate);
    }
    let h2_normalized = ra * ra - x_normalized * x_normalized;
    if !h2_normalized.is_finite() {
        return invalid_3d(IntersectionKind::Indeterminate);
    }
    if h2_normalized < 0.0 && h2_normalized.abs() > tol {
        return invalid_3d(IntersectionKind::Indeterminate);
    }
    let center_offset = x_normalized * scale;
    let center = a.center.add(delta.scale(center_offset / distance));
    if !center.is_finite() {
        return invalid_3d(IntersectionKind::Indeterminate);
    }
    if h2_normalized.abs() <= tol {
        return Intersection3D {
            kind: IntersectionKind::Tangent,
            points: vec![center],
            parameters: Vec::new(),
        };
    }
    if h2_normalized < 0.0 {
        return invalid_3d(IntersectionKind::Indeterminate);
    }
    let half_chord = h2_normalized.sqrt() * scale;
    if !half_chord.is_finite() {
        return invalid_3d(IntersectionKind::Indeterminate);
    }
    Intersection3D {
        kind: IntersectionKind::Circle,
        points: vec![center],
        parameters: vec![half_chord],
    }
}

pub fn predicate_classification(value: Tri) -> IntersectionKind {
    match value {
        Tri::True => IntersectionKind::Point,
        Tri::False => IntersectionKind::None,
        Tri::Indeterminate => IntersectionKind::Indeterminate,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_line_crossing_is_exact() {
        let a = Line2 {
            origin: Vec2::new(0.0, 0.0),
            direction: Vec2::new(1.0, 0.0),
        };
        let b = Line2 {
            origin: Vec2::new(0.5, -1.0),
            direction: Vec2::new(0.0, 1.0),
        };
        let result = line_line_2d(a, b, 1.0e-12);
        assert_eq!(result.kind, IntersectionKind::Point);
        assert!((result.points[0].x - 0.5).abs() < 1.0e-14);
        assert!((result.parameters[0] - 0.5).abs() < 1.0e-14);
        assert!((result.parameters[1] - 1.0).abs() < 1.0e-14);
    }

    #[test]
    fn line_circle_returns_tangent_and_two_points() {
        let line = Line2 {
            origin: Vec2::new(-2.0, 0.0),
            direction: Vec2::new(1.0, 0.0),
        };
        let circle = Circle {
            center: Vec2::new(0.0, 0.0),
            radius: 1.0,
        };
        assert_eq!(
            line_circle_2d(
                Line2 {
                    origin: Vec2::new(-2.0, 1.0),
                    direction: Vec2::new(1.0, 0.0),
                },
                circle,
                1.0e-12,
            )
            .kind,
            IntersectionKind::Tangent
        );
        assert_eq!(line_circle_2d(line, circle, 1.0e-12).kind, IntersectionKind::MultiplePoints);
    }

    #[test]
    fn line_sphere_and_plane_sphere_use_length_domain_formulas() {
        let line = Line3 {
            origin: Vec3::new(-2.0, 0.0, 0.0),
            direction: Vec3::new(2.0, 0.0, 0.0),
        };
        let sphere = Sphere3 {
            center: Vec3::new(0.0, 0.0, 0.0),
            radius: 1.0,
        };
        let result = line_sphere_3d(line, sphere, 1.0e-12);
        assert_eq!(result.kind, IntersectionKind::MultiplePoints);
        assert!(result.points.iter().all(|point| point.is_finite()));

        let plane = Plane3 {
            origin: Vec3::new(0.0, 0.0, 0.0),
            normal: Vec3::new(0.0, 0.0, 1.0),
        };
        let circle = plane_sphere_3d(plane, sphere, 1.0e-12);
        assert_eq!(circle.kind, IntersectionKind::Circle);
        assert!((circle.parameters[0] - 1.0).abs() < 1.0e-14);
    }

    #[test]
    fn sphere_sphere_handles_large_radii_without_squaring_them() {
        let a = Sphere3 {
            center: Vec3::new(0.0, 0.0, 0.0),
            radius: 1.0e200,
        };
        let b = Sphere3 {
            center: Vec3::new(1.0e200, 0.0, 0.0),
            radius: 1.0e200,
        };
        let result = sphere_sphere_3d(a, b, 1.0e-12);
        assert_eq!(result.kind, IntersectionKind::Circle);
        assert!((result.points[0].x - 5.0e199).abs() / 5.0e199 < 1.0e-14);
        assert!((result.parameters[0] - (3.0_f64).sqrt() * 5.0e199).abs() / 1.0e200 < 1.0e-14);
    }

    #[test]
    fn nonfinite_or_invalid_inputs_do_not_create_nan_outputs() {
        let line = Line2 {
            origin: Vec2::new(f64::NAN, 0.0),
            direction: Vec2::new(1.0, 0.0),
        };
        assert_eq!(line_line_2d(line, line, 1.0e-12).kind, IntersectionKind::Degenerate);
        let ellipse = Ellipse2 {
            center: Vec2::new(0.0, 0.0),
            semi_axis_a: 1.0e-308,
            semi_axis_b: 1.0,
            rotation: 0.0,
        };
        let result = ellipse_line_2d(
            ellipse,
            Line2 {
                origin: Vec2::new(0.0, 2.0),
                direction: Vec2::new(1.0, 0.0),
            },
            1.0e-12,
        );
        assert!(matches!(
            result.kind,
            IntersectionKind::Indeterminate | IntersectionKind::Degenerate | IntersectionKind::None
        ));
    }
}
