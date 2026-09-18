//! Parameter-space trimming mathematics for analytic line/arc trim curves.
//!
//! A trim loop is an ordered closed boundary in the surface parameter domain.
//! Classification is performed in parameter space; no display mesh is involved.
//! Boundary cases are explicit, and unsupported pairwise degeneracies return
//! `Indeterminate` rather than an arbitrary inside/outside decision.

use super::{
    geometry::{Arc, Point},
    predicates::Tri,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrimError {
    NonFinite,
    InvalidDomain,
    Degenerate,
    NotClosed,
    SelfIntersection,
    Indeterminate,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegionClass {
    Inside,
    Outside,
    OnBoundary,
    Indeterminate,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TrimCurve2 {
    Line { start: Point, end: Point },
    Arc(Arc),
}

#[derive(Clone, Debug, PartialEq)]
pub struct TrimLoop2 {
    pub curves: Vec<TrimCurve2>,
}

impl TrimCurve2 {
    pub fn validate(&self) -> Result<(), TrimError> {
        match self {
            Self::Line { start, end } => {
                if !start.x.is_finite()
                    || !start.y.is_finite()
                    || !end.x.is_finite()
                    || !end.y.is_finite()
                {
                    return Err(TrimError::NonFinite);
                }
                if start.distance(*end) == 0.0 {
                    Err(TrimError::Degenerate)
                } else {
                    Ok(())
                }
            }
            Self::Arc(arc) => arc.validate().map_err(|_| TrimError::Degenerate),
        }
    }

    pub fn start(&self) -> Point {
        match self {
            Self::Line { start, .. } => *start,
            Self::Arc(arc) => arc.start_point(),
        }
    }

    pub fn end(&self) -> Point {
        match self {
            Self::Line { end, .. } => *end,
            Self::Arc(arc) => arc.end_point(),
        }
    }

    pub fn point_at(&self, t: f64) -> Result<Point, TrimError> {
        if !t.is_finite() || !(0.0..=1.0).contains(&t) {
            return Err(TrimError::InvalidDomain);
        }
        let point = match self {
            Self::Line { start, end } => Point {
                x: start.x + (end.x - start.x) * t,
                y: start.y + (end.y - start.y) * t,
            },
            Self::Arc(arc) => arc.point_at(t),
        };
        if point.x.is_finite() && point.y.is_finite() {
            Ok(point)
        } else {
            Err(TrimError::NonFinite)
        }
    }

    fn signed_area_contribution_about(&self, reference: Point) -> Result<f64, TrimError> {
        self.validate()?;
        let area = match self {
            Self::Line { start, end } => {
                0.5 * start.sub(reference).cross(end.sub(reference))
            }
            Self::Arc(arc) => {
                let t0 = arc.start_angle;
                let t1 = arc.end_angle;
                let p0 = arc.start_point().sub(reference);
                let p1 = arc.end_point().sub(reference);
                let center = arc.center.sub(reference);
                0.5
                    * (arc.radius * arc.radius * (t1 - t0)
                        + center.x * (p1.y - p0.y)
                        - center.y * (p1.x - p0.x))
            }
        };
        if area.is_finite() {
            Ok(area)
        } else {
            Err(TrimError::NonFinite)
        }
    }

    pub fn signed_area_contribution(&self) -> Result<f64, TrimError> {
        self.signed_area_contribution_about(Point { x: 0.0, y: 0.0 })
    }
}

impl TrimLoop2 {
    pub fn validate(&self, tolerance: f64) -> Result<(), TrimError> {
        if !tolerance.is_finite() || tolerance < 0.0 {
            return Err(TrimError::InvalidDomain);
        }
        if self.curves.is_empty() {
            return Err(TrimError::Degenerate);
        }
        for curve in &self.curves {
            curve.validate()?;
        }
        for pair in self.curves.windows(2) {
            if pair[0].end().distance(pair[1].start()) > tolerance {
                return Err(TrimError::NotClosed);
            }
        }
        if self.curves[self.curves.len() - 1]
            .end()
            .distance(self.curves[0].start())
            > tolerance
        {
            return Err(TrimError::NotClosed);
        }
        match self.has_self_intersection(tolerance) {
            Tri::True => Err(TrimError::SelfIntersection),
            Tri::False => Ok(()),
            Tri::Indeterminate => Err(TrimError::Indeterminate),
        }
    }

    pub fn signed_area(&self, tolerance: f64) -> Result<f64, TrimError> {
        self.validate(tolerance)?;
        let reference = self.curves[0].start();
        let mut area = 0.0;
        for curve in &self.curves {
            area += curve.signed_area_contribution_about(reference)?;
            if !area.is_finite() {
                return Err(TrimError::NonFinite);
            }
        }
        Ok(area)
    }

    pub fn orientation(&self, tolerance: f64) -> Result<Tri, TrimError> {
        let area = self.signed_area(tolerance)?;
        let mut min_x = f64::INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut max_y = f64::NEG_INFINITY;
        let mut geometric_scale: f64 = 0.0;
        for curve in &self.curves {
            let start = curve.start();
            let end = curve.end();
            min_x = min_x.min(start.x).min(end.x);
            min_y = min_y.min(start.y).min(end.y);
            max_x = max_x.max(start.x).max(end.x);
            max_y = max_y.max(start.y).max(end.y);
            geometric_scale = geometric_scale.max(start.distance(end));
            if let TrimCurve2::Arc(arc) = curve {
                geometric_scale = geometric_scale.max(arc.radius);
            }
        }
        let extent = (max_x - min_x).hypot(max_y - min_y);
        let scale = extent.max(geometric_scale).max(1.0);
        let eps = tolerance * scale * scale;
        if !eps.is_finite() {
            return Ok(Tri::Indeterminate);
        }
        if area > eps {
            Ok(Tri::True)
        } else if area < -eps {
            Ok(Tri::False)
        } else {
            Ok(Tri::Indeterminate)
        }
    }

    pub fn classify_point(
        &self,
        p: Point,
        tolerance: f64,
    ) -> Result<RegionClass, TrimError> {
        self.validate(tolerance)?;
        if !p.x.is_finite() || !p.y.is_finite() {
            return Err(TrimError::NonFinite);
        }
        let mut crossings = 0usize;
        for curve in &self.curves {
            match ray_crossing(curve, p, tolerance) {
                RayHit::Boundary => return Ok(RegionClass::OnBoundary),
                RayHit::Cross => crossings += 1,
                RayHit::None => {}
                RayHit::Indeterminate => return Ok(RegionClass::Indeterminate),
            }
        }
        Ok(if crossings % 2 == 1 {
            RegionClass::Inside
        } else {
            RegionClass::Outside
        })
    }

    fn has_self_intersection(&self, tolerance: f64) -> Tri {
        for i in 0..self.curves.len() {
            for j in i + 1..self.curves.len() {
                if j == i + 1 || (i == 0 && j + 1 == self.curves.len()) {
                    continue;
                }
                match curve_pair_intersects(&self.curves[i], &self.curves[j], tolerance) {
                    Tri::True => return Tri::True,
                    Tri::Indeterminate => return Tri::Indeterminate,
                    Tri::False => {}
                }
            }
        }
        Tri::False
    }
}

enum RayHit {
    None,
    Cross,
    Boundary,
    Indeterminate,
}

fn ray_crossing(c: &TrimCurve2, p: Point, tol: f64) -> RayHit {
    match c {
        TrimCurve2::Line { start, end } => {
            let min_x = start.x.min(end.x);
            let max_x = start.x.max(end.x);
            let min_y = start.y.min(end.y);
            let max_y = start.y.max(end.y);
            let d = end.sub(*start);
            let relative = p.sub(*start);
            let cross = d.cross(relative);
            let line_scale = d.length().max(f64::MIN_POSITIVE);
            if !cross.is_finite() || !line_scale.is_finite() {
                return RayHit::Indeterminate;
            }
            // cross has units of length²; compare it with tolerance × line
            // length so the boundary test is translation- and scale-consistent.
            if cross.abs() <= tol * line_scale
                && p.x >= min_x - tol
                && p.x <= max_x + tol
                && p.y >= min_y - tol
                && p.y <= max_y + tol
            {
                return RayHit::Boundary;
            }
            if (start.y > p.y) != (end.y > p.y) {
                let x = start.x + (p.y - start.y) * d.x / d.y;
                if !x.is_finite() {
                    return RayHit::Indeterminate;
                }
                if x > p.x {
                    return RayHit::Cross;
                }
            }
            RayHit::None
        }
        TrimCurve2::Arc(arc) => {
            let rel = (p.y - arc.center.y) / arc.radius;
            if !rel.is_finite() {
                return RayHit::Indeterminate;
            }
            if rel.abs() > 1.0 + tol {
                return RayHit::None;
            }
            if rel.abs() >= 1.0 - tol {
                return RayHit::Indeterminate;
            }
            let theta = rel.clamp(-1.0, 1.0).asin();
            let candidates = [theta, std::f64::consts::PI - theta];
            let mut hit = 0usize;
            for angle in candidates {
                if arc.contains_angle(angle) {
                    let x = arc.center.x + arc.radius * angle.cos();
                    if !x.is_finite() {
                        return RayHit::Indeterminate;
                    }
                    if (x - p.x).abs() <= tol {
                        return RayHit::Boundary;
                    }
                    if x > p.x {
                        hit += 1;
                    }
                }
            }
            if hit % 2 == 1 {
                RayHit::Cross
            } else {
                RayHit::None
            }
        }
    }
}

fn curve_pair_intersects(a: &TrimCurve2, b: &TrimCurve2, tol: f64) -> Tri {
    match (a, b) {
        (
            TrimCurve2::Line { start: a0, end: a1 },
            TrimCurve2::Line { start: b0, end: b1 },
        ) => segment_intersection(*a0, *a1, *b0, *b1, tol),
        (
            TrimCurve2::Line { start: a0, end: a1 },
            TrimCurve2::Arc(arc),
        ) => line_arc_intersection(*a0, *a1, *arc, tol),
        (
            TrimCurve2::Arc(arc),
            TrimCurve2::Line { start: b0, end: b1 },
        ) => line_arc_intersection(*b0, *b1, *arc, tol),
        (TrimCurve2::Arc(a), TrimCurve2::Arc(b)) => arc_arc_intersection(*a, *b, tol),
    }
}

fn angle_in_arc_tolerant(angle: f64, start: f64, end: f64, linear_tolerance: f64, radius: f64) -> bool {
    let delta = end - start;
    let angle_tolerance = linear_tolerance / radius.max(f64::MIN_POSITIVE);
    if !delta.is_finite() || !angle_tolerance.is_finite() {
        return false;
    }
    if delta.abs() >= 2.0 * std::f64::consts::PI - angle_tolerance {
        return true;
    }
    if delta > 0.0 {
        (angle - start).rem_euclid(2.0 * std::f64::consts::PI) <= delta + angle_tolerance
    } else {
        (start - angle).rem_euclid(2.0 * std::f64::consts::PI) <= -delta + angle_tolerance
    }
}

fn arc_contains_point_tolerant(arc: Arc, point: Point, tolerance: f64) -> Tri {
    let radial = point.sub(arc.center);
    let distance = radial.length();
    if !distance.is_finite() || !radial.is_finite() {
        return Tri::Indeterminate;
    }
    let radial_band = tolerance * arc.radius.max(1.0);
    if !radial_band.is_finite() {
        return Tri::Indeterminate;
    }
    if (distance - arc.radius).abs() > radial_band {
        return Tri::False;
    }
    if distance <= f64::MIN_POSITIVE {
        return Tri::Indeterminate;
    }
    let angle = radial.y.atan2(radial.x);
    if !angle.is_finite() {
        return Tri::Indeterminate;
    }
    if angle_in_arc_tolerant(angle, arc.start_angle, arc.end_angle, tolerance, arc.radius) {
        Tri::True
    } else {
        Tri::False
    }
}

fn line_arc_intersection(start: Point, end: Point, arc: Arc, tolerance: f64) -> Tri {
    if !start.is_finite() || !end.is_finite() || arc.validate().is_err()
        || !tolerance.is_finite() || tolerance < 0.0
    {
        return Tri::Indeterminate;
    }
    let direction = end.sub(start);
    let length = direction.length();
    if !length.is_finite() || length <= f64::MIN_POSITIVE {
        return Tri::Indeterminate;
    }
    let unit = direction.scale(1.0 / length);
    let center_offset = arc.center.sub(start);
    let along = center_offset.dot(unit);
    let perpendicular = center_offset.sub(unit.scale(along));
    let h2 = arc.radius * arc.radius - perpendicular.dot(perpendicular);
    let scale = arc.radius.max(length).max(center_offset.length()).max(1.0);
    let band2 = 2.0 * scale * tolerance + tolerance * tolerance;
    if [along, h2, scale, band2].iter().any(|v| !v.is_finite()) {
        return Tri::Indeterminate;
    }
    if h2 < -band2 {
        return Tri::False;
    }
    let half = h2.max(0.0).sqrt();
    let candidates = if half <= tolerance {
        vec![along]
    } else {
        vec![along - half, along + half]
    };
    for distance in candidates {
        let segment_band = tolerance.max(f64::MIN_POSITIVE);
        if distance < -segment_band || distance > length + segment_band {
            continue;
        }
        let point = start.add(unit.scale(distance));
        match arc_contains_point_tolerant(arc, point, tolerance) {
            Tri::True => return Tri::True,
            Tri::Indeterminate => return Tri::Indeterminate,
            Tri::False => {}
        }
    }
    Tri::False
}

fn same_circle_arcs_overlap(a: Arc, b: Arc, tolerance: f64) -> Tri {
    let radius_scale = a.radius.max(b.radius).max(1.0);
    if (a.center.sub(b.center)).length() > tolerance
        || (a.radius - b.radius).abs() > tolerance * radius_scale
    {
        return Tri::False;
    }
    let full_a = (a.end_angle - a.start_angle).abs() >= 2.0 * std::f64::consts::PI
        - tolerance / a.radius.max(f64::MIN_POSITIVE);
    let full_b = (b.end_angle - b.start_angle).abs() >= 2.0 * std::f64::consts::PI
        - tolerance / b.radius.max(f64::MIN_POSITIVE);
    if full_a || full_b {
        return Tri::True;
    }
    let endpoints = [
        (a.start_angle, b),
        (a.end_angle, b),
        (b.start_angle, a),
        (b.end_angle, a),
    ];
    for (angle, target) in endpoints {
        if angle_in_arc_tolerant(angle, target.start_angle, target.end_angle, tolerance, target.radius) {
            return Tri::True;
        }
    }
    Tri::False
}

fn arc_arc_intersection(a: Arc, b: Arc, tolerance: f64) -> Tri {
    if a.validate().is_err() || b.validate().is_err() || !tolerance.is_finite() || tolerance < 0.0 {
        return Tri::Indeterminate;
    }
    let delta = b.center.sub(a.center);
    let distance = delta.length();
    let scale = a.radius.max(b.radius).max(distance).max(1.0);
    let center_band = tolerance;
    let radius_band = tolerance * scale;
    if !distance.is_finite() || !scale.is_finite() || !radius_band.is_finite() {
        return Tri::Indeterminate;
    }
    if distance <= center_band && (a.radius - b.radius).abs() <= radius_band {
        return same_circle_arcs_overlap(a, b, tolerance);
    }
    if distance > a.radius + b.radius + center_band
        || distance < (a.radius - b.radius).abs() - center_band
    {
        return Tri::False;
    }
    if distance <= f64::MIN_POSITIVE {
        return Tri::Indeterminate;
    }
    let numerator = a.radius * a.radius - b.radius * b.radius + distance * distance;
    let along = numerator / (2.0 * distance);
    let h2 = a.radius * a.radius - along * along;
    let band2 = 2.0 * scale * tolerance + tolerance * tolerance;
    if !numerator.is_finite() || !along.is_finite() || !h2.is_finite() || !band2.is_finite() {
        return Tri::Indeterminate;
    }
    if h2 < -band2 {
        return Tri::False;
    }
    let half = h2.max(0.0).sqrt();
    let base = a.center.add(delta.scale(along / distance));
    let unit_perp = Point {
        x: -delta.y / distance,
        y: delta.x / distance,
    };
    let first = Point {
        x: base.x + unit_perp.x * half,
        y: base.y + unit_perp.y * half,
    };
    let second = Point {
        x: base.x - unit_perp.x * half,
        y: base.y - unit_perp.y * half,
    };
    let candidates = if half <= tolerance {
        vec![first]
    } else {
        vec![first, second]
    };
    for point in candidates {
        if !point.is_finite() {
            return Tri::Indeterminate;
        }
        match arc_contains_point_tolerant(a, point, tolerance) {
            Tri::True => match arc_contains_point_tolerant(b, point, tolerance) {
                Tri::True => return Tri::True,
                Tri::Indeterminate => return Tri::Indeterminate,
                Tri::False => {}
            },
            Tri::Indeterminate => return Tri::Indeterminate,
            Tri::False => {}
        }
    }
    Tri::False
}

fn segment_intersection(a: Point, b: Point, c: Point, d: Point, tol: f64) -> Tri {
    let ab = b.sub(a);
    let cd = d.sub(c);
    let c1 = ab.cross(c.sub(a));
    let c2 = ab.cross(d.sub(a));
    let c3 = cd.cross(a.sub(c));
    let c4 = cd.cross(b.sub(c));
    let scale = (ab.length() * cd.length()).max(f64::MIN_POSITIVE);
    let epsilon = tol * scale;
    if !epsilon.is_finite() {
        return Tri::Indeterminate;
    }
    let s1 = if c1 > epsilon { 1 } else if c1 < -epsilon { -1 } else { 0 };