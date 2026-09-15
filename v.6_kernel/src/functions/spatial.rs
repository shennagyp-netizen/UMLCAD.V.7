use super::geometry::{aabb_intersects, Arc, Circle, Geometry, Line, Point, EPSILON};

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
        if (-EPSILON..=1.0 + EPSILON).contains(&t) && (-EPSILON..=1.0 + EPSILON).contains(&u) {
            return 0.0;
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
    (c.center.distance(closest(l, c.center)) - c.radius).max(0.0)
}

fn arc_point(a: Arc, t: f64) -> Point {
    Geometry::Arc(a).point_at(t)
}

fn arc_distance(a: Arc, p: Point) -> f64 {
    Geometry::Arc(a).distance_to_point(p)
}

fn point_on_arc(a: Arc, p: Point) -> bool {
    let ang = (p.y - a.center.y).atan2(p.x - a.center.x);
    [-2.0 * std::f64::consts::PI, 0.0, 2.0 * std::f64::consts::PI]
        .into_iter()
        .any(|k| {
            let t = (ang + k - a.start_angle) / (a.end_angle - a.start_angle);
            (-EPSILON..=1.0 + EPSILON).contains(&t)
        })
}

fn line_arc(l: Line, a: Arc) -> f64 {
    let circle_dist = line_circle(
        l,
        Circle {
            center: a.center,
            radius: a.radius,
        },
    );
    if circle_dist <= EPSILON {
        let q = closest(l, a.center);
        if point_on_arc(a, q) {
            return 0.0;
        }
    }
    let p0 = l.start.distance(arc_point(a, 0.0));
    let p1 = l.end.distance(arc_point(a, 1.0));
    let pa = arc_distance(a, l.start);
    let pb = arc_distance(a, l.end);
    circle_dist.min(p0.min(p1).min(pa.min(pb)))
}

fn arc_circle(a: Arc, c: Circle) -> f64 {
    let center_dist = arc_distance(a, c.center);
    (center_dist - c.radius)
        .abs()
        .min((arc_point(a, 0.0).distance(c.center) - c.radius).abs())
        .min((arc_point(a, 1.0).distance(c.center) - c.radius).abs())
}

fn arc_arc(a: Arc, b: Arc) -> f64 {
    let circle_dist = circle_circle(
        Circle {
            center: a.center,
            radius: a.radius,
        },
        Circle {
            center: b.center,
            radius: b.radius,
        },
    );
    if circle_dist <= EPSILON {
        for t in [0.0, 0.5, 1.0] {
            if point_on_arc(b, arc_point(a, t)) {
                return 0.0;
            }
        }
    }

    let mut d = arc_point(a, 0.0)
        .distance(arc_point(b, 0.0))
        .min(arc_point(a, 0.0).distance(arc_point(b, 1.0)))
        .min(arc_point(a, 1.0).distance(arc_point(b, 0.0)))
        .min(arc_point(a, 1.0).distance(arc_point(b, 1.0)));
    d = d
        .min(arc_distance(a, arc_point(b, 0.5)))
        .min(arc_distance(b, arc_point(a, 0.5)));
    d
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
