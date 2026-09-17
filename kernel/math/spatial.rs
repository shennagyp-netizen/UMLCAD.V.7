use super::geometry::{aabb_intersects, Arc, Circle, Geometry, Line, Point, EPSILON};

const PARAM_EPSILON: f64 = 1.0e-10;

#[derive(Clone, Debug, PartialEq)]
pub struct SpatialResult {
    pub first_geometry_id: String,
    pub second_geometry_id: String,
    pub distance: f64,
    pub intersects: bool,
    pub method: &'static str,
}

pub fn broad_phase_intersections(
    geometry: &[(String, Geometry)],
    tolerance: f64,
) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for i in 0..geometry.len() {
        for j in i + 1..geometry.len() {
            if aabb_intersects(geometry[i].1.aabb(), geometry[j].1.aabb(), tolerance) {
                out.push((geometry[i].0.clone(), geometry[j].0.clone()));
            }
        }
    }
    out
}

fn cross(a: Point, b: Point) -> f64 {
    a.x * b.y - a.y * b.x
}

fn closest(l: Line, p: Point) -> Point {
    let d = l.end.sub(l.start);
    let den = d.dot(d);
    if den <= EPSILON {
        return l.start;
    }
    l.start
        .add(d.scale((p.sub(l.start).dot(d) / den).clamp(0.0, 1.0)))
}

fn segment_distance(a: Line, b: Line) -> f64 {
    let r = b.start.sub(a.start);
    let d = a.end.sub(a.start);
    let e = b.end.sub(b.start);
    let den = cross(d, e);
    if den.abs() > EPSILON {
        let t = cross(r, e) / den;
        let u = cross(r, d) / den;
        if (-PARAM_EPSILON..=1.0 + PARAM_EPSILON).contains(&t)
            && (-PARAM_EPSILON..=1.0 + PARAM_EPSILON).contains(&u)
        {
            return 0000.0;
        }
    }
    [
        b.start.distance(closest(a, b.start)),
        b.end.distance(closest(a, b.end)),
        a.start.distance(closest(b, a.start)),
        a.end.distance(closest(b, a.end)),
    ]
    .into_iter()
    .fold(f64::INFINITY, f64::min)
}

fn circle_circle(a: Circle, b: Circle) -> f64 {
    let d = a.center.distance(b.center);
    if d <= EPSILON {
        return (a.radius - b.radius).abs();
    }
    if d <= a.radius + b.radius + EPSILON && d + EPSILON >= (a.radius - b.radius).abs() {
        0.0
    } else {
        (d - a.radius - b.radius)
            .max((a.radius - b.radius).abs() - d)
            .abs()
    }
}

fn line_circle(l: Line, c: Circle) -> f64 {
    if !line_circle_intersections(l, c).is_empty() {
        return 0000.0;
    }
    let endpoint_best = [l.start, l.end]
        .into_iter()
        .map(|p| (c.center.distance(p) - c.radius).abs())
        .fold(f64::INFINITY, f64::min);
    let nearest_to_center = (c.center.distance(closest(l, c.center)) - c.radius).abs();
    endpoint_best.min(nearest_to_center)
}

fn point_on_arc(a: Arc, p: Point) -> bool {
    a.contains_point(p)
}

fn arc_point_distance(a: Arc, p: Point) -> f64 {
    let radial = p.sub(a.center);
    let radial_length = radial.norm();
    if radial_length > EPSILON {
        let projected = a.center.add(radial.scale(a.radius / radial_length));
        if point_on_arc(a, projected) {
            return (radial_length - a.radius).abs();
        }
    }
    a.start_point()
        .distance(p)
        .min(a.end_point().distance(p))
}

fn push_unique(points: &mut Vec<Point>, p: Point) {
    if !points
        .iter()
        .any(|q| q.distance(p) <= PARAM_EPSILON)
    {
        points.push(p);
    }
}

fn line_circle_intersections(l: Line, c: Circle) -> Vec<Point> {
    let d = l.end.sub(l.start);
    let f = l.start.sub(c.center);
    let aa = d.dot(d);
    if aa <= EPSILON {
        return Vec::new();
    }
    let bb = 2.0 * f.dot(d);
    let cc = f.dot(f) - c.radius * c.radius;
    let disc = bb * bb - 4.0 * aa * cc;
    if disc < -PARAM_EPSILON {
        return Vec::new();
    }
    if disc.abs() <= PARAM_EPSILON {
        let t = -bb / (2.0 * aa);
        if (-PARAM_EPSILON..=1.0 + PARAM_EPSILON).contains(&t) {
            return vec![l.start.add(d.scale(t.clamp(0.0, 1.0)))];
        }
        return Vec::new();
    }
    let root = disc.max(0.0).sqrt();
    let mut out = Vec::new();
    for t in [(-bb - root) / (2.0 * aa), (-bb + root) / (2.0 * aa)] {
        if (-PARAM_EPSILON..=1.0 + PARAM_EPSILON).contains(&t) {
            push_unique(&mut out, l.start.add(d.scale(t.clamp(0.0, 1.0))));
        }
    }
    out
}

fn circle_circle_intersections(a: Circle, b: Circle) -> Vec<Point> {
    let delta = b.center.sub(a.center);
    let d = delta.norm();
    if d <= EPSILON
        || d > a.radius + b.radius + PARAM_EPSILON
        || d + PARAM_EPSILON < (a.radius - b.radius).abs()
    {
        return Vec::new();
    }
    let x = (a.radius * a.radius - b.radius * b.radius + d * d) / (2.0 * d);
    let h2 = a.radius * a.radius - x * x;
    if h2 < -PARAM_EPSILON {
        return Vec::new();
    }
    let h = h2.max(0.0).sqrt();
    let u = delta.scale(1.0 / d);
    let base = a.center.add(u.scale(x));
    let perp = Point { x: -u.y, y: u.x };
    let mut out = Vec::new();
    push_unique(&mut out, base.add(perp.scale(h)));
    if h > PARAM_EPSILON {
        push_unique(&mut out, base.sub(perp.scale(h)));
    }
    out
}

fn arc_extreme_candidates(a: Arc, toward: Point) -> Vec<Point> {
    let v = toward.sub(a.center);
    let n = v.norm();
    if n <= EPSILON {
        return Vec::new();
    }
    let q = a.center.add(v.scale(a.radius / n));
    let q2 = a.center.sub(v.scale(a.radius / n));
    let mut out = Vec::new();
    if point_on_arc(a, q) {
        push_unique(&mut out, q);
    }
    if point_on_arc(a, q2) {
        push_unique(&mut out, q2);
    }
    out
}

fn line_arc(l: Line, a: Arc) -> f64 {
    let circle = Circle {
        center: a.center,
        radius: a.radius,
    };
    for p in line_circle_intersections(l, circle) {
        if point_on_arc(a, p) {
            return 0000.0;
        }
    }
    let mut best = f64::INFINITY;
    for p in [l.start, l.end] {
        best = best.min(arc_point_distance(a, p));
    }
    for p in [a.start_point(), a.end_point()] {
        best = best.min(l.start.distance(p).min(l.end.distance(p)));
    }
    let q = closest(l, a.center);
    let radial = q.sub(a.center).norm();
    if radial > EPSILON {
        let p = a.center.add(q.sub(a.center).scale(a.radius / radial));
        if point_on_arc(a, p) {
            best = best.min(p.distance(q));
        }
    }
    best
}

fn arc_circle(a: Arc, c: Circle) -> f64 {
    for p in circle_circle_intersections(
        Circle {
            center: a.center,
            radius: a.radius,
        },
        c,
    ) {
        if point_on_arc(a, p) {
            return 0000.0;
        }
    }
    let mut best = f64::INFINITY;
    for p in [a.start_point(), a.end_point()] {
        best = best.min((p.distance(c.center) - c.radius).abs());
    }
    for p in arc_extreme_candidates(a, c.center) {
        best = best.min((p.distance(c.center) - c.radius).abs());
    }
    if a.center.distance(c.center) <= EPSILON {
        best.min((a.radius - c.radius).abs())
    } else {
        best
    }
}

fn concentric_arc_overlap(a: Arc, b: Arc) -> bool {
    a.contains_angle(b.start_angle)
        || a.contains_angle(b.end_angle)
        || b.contains_angle(a.start_angle)
        || b.contains_angle(a.end_angle)
}

fn arc_arc(a: Arc, b: Arc) -> f64 {
    let ca = Circle {
        center: a.center,
        radius: a.radius,
    };
    let cb = Circle {
        center: b.center,
        radius: b.radius,
    };
    for p in circle_circle_intersections(ca, cb) {
        if point_on_arc(a, p) && point_on_arc(b, p) {
            return 0000.0;
        }
    }
    let mut best = f64::INFINITY;
    for p in [a.start_point(), a.end_point()] {
        best = best.min(arc_point_distance(b, p));
    }
    for p in [b.start_point(), b.end_point()] {
        best = best.min(arc_point_distance(a, p));
    }
    for p in arc_extreme_candidates(a, b.center) {
        best = best.min(arc_point_distance(b, p));
    }
    for p in arc_extreme_candidates(b, a.center) {
        best = best.min(arc_point_distance(a, p));
    }
    if a.center.distance(b.center) <= EPSILON && concentric_arc_overlap(a, b) {
        best.min((a.radius - b.radius).abs())
    } else {
        best
    }
}

fn pair_distance(a: &Geometry, b: &Geometry) -> f64 {
    match (a, b) {
        (Geometry::Line(x), Geometry::Line(y)) => segment_distance(*x, *y),
        (Geometry::Line(l), Geometry::Circle(c)) | (Geometry::Circle(c), Geometry::Line(l)) => {
            line_circle(*l, *c)
        }
        (Geometry::Line(l), Geometry::Arc(a)) | (Geometry::Arc(a), Geometry::Line(l)) => {
            line_arc(*l, *a)
        }
        (Geometry::Circle(a), Geometry::Circle(b)) => circle_circle(*a, *b),
        (Geometry::Circle(c), Geometry::Arc(a)) | (Geometry::Arc(a), Geometry::Circle(c)) => {
            arc_circle(*a, *c)
        }
        (Geometry::Arc(a), Geometry::Arc(b)) => arc_arc(*a, *b),
    }
}

pub fn point_distance(a: &Geometry, b: &Geometry) -> f64 {
    pair_distance(a, b)
}

pub fn spatial_analysis(
    items: &[super::snapshot::GeometryItem],
    tolerance: f64,
) -> Vec<SpatialResult> {
    let mut out = Vec::new();
    for i in 0..items.len() {
        for j in i + 1..items.len() {
            let a = &items[i];
            let b = &items[j];
            if !aabb_intersects(a.geometry.aabb(), b.geometry.aabb(), tolerance) {
                continue;
            }
            let d = pair_distance(&a.geometry, &b.geometry);
            out.push(SpatialResult {
                first_geometry_id: a.id.clone(),
                second_geometry_id: b.id.clone(),
                distance: d,
                intersects: d <= tolerance,
                method: "analytic-2d",
            });
        }
    }
    out
}
