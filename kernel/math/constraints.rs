use super::geometry::{Geometry, Point, EPSILON};
use super::snapshot::{Constraint, Endpoint};

pub fn endpoint(g: &Geometry, e: Endpoint) -> Option<Point> {
    match (g, e) {
        (Geometry::Line(l), Endpoint::Start) => Some(l.start),
        (Geometry::Line(l), Endpoint::End) => Some(l.end),
        (Geometry::Arc(a), Endpoint::Start) => Some(Point {
            x: a.center.x + a.radius * a.start_angle.cos(),
            y: a.center.y + a.radius * a.start_angle.sin(),
        }),
        (Geometry::Arc(a), Endpoint::End) => Some(Point {
            x: a.center.x + a.radius * a.end_angle.cos(),
            y: a.center.y + a.radius * a.end_angle.sin(),
        }),
        (Geometry::Circle(_), _) => None,
    }
}

pub fn geometry_difference(a: &Geometry, b: &Geometry) -> Option<Vec<f64>> {
    match (a, b) {
        (Geometry::Line(x), Geometry::Line(y)) => Some(vec![
            x.start.x - y.start.x,
            x.start.y - y.start.y,
            x.end.x - y.end.x,
            x.end.y - y.end.y,
        ]),
        (Geometry::Circle(x), Geometry::Circle(y)) => Some(vec![
            x.center.x - y.center.x,
            x.center.y - y.center.y,
            x.radius - y.radius,
        ]),
        (Geometry::Arc(x), Geometry::Arc(y)) => Some(vec![
            x.center.x - y.center.x,
            x.center.y - y.center.y,
            x.radius - y.radius,
            x.start_angle - y.start_angle,
            x.end_angle - y.end_angle,
        ]),
        _ => None,
    }
}

pub fn residual(geometry: impl Fn(&str) -> Option<Geometry>, c: &Constraint) -> Option<Vec<f64>> {
    match c {
        Constraint::Horizontal { entity_id } => match geometry(entity_id)? {
            Geometry::Line(l) => Some(vec![l.end.y - l.start.y]),
            _ => None,
        },
        Constraint::Vertical { entity_id } => match geometry(entity_id)? {
            Geometry::Line(l) => Some(vec![l.end.x - l.start.x]),
            _ => None,
        },
        Constraint::Coincident {
            first_geometry_id,
            first_point,
            second_geometry_id,
            second_point,
        } => {
            let a = endpoint(&geometry(first_geometry_id)?, *first_point)?;
            let b = endpoint(&geometry(second_geometry_id)?, *second_point)?;
            Some(vec![a.x - b.x, a.y - b.y])
        }
        Constraint::Fixed { .. } => Some(vec![]),
        Constraint::Distance {
            first_geometry_id,
            second_geometry_id,
            first_endpoint,
            second_endpoint,
            value,
        } => {
            let a = geometry(first_geometry_id)?;
            if let Some(second_id) = second_geometry_id {
                let b = geometry(second_id)?;
                let pa = endpoint(&a, first_endpoint.unwrap_or(Endpoint::Start))?;
                let pb = endpoint(&b, second_endpoint.unwrap_or(Endpoint::Start))?;
                Some(vec![pa.distance(pb) - *value])
            } else {
                Some(vec![match a {
                    Geometry::Line(l) => l.length() - *value,
                    Geometry::Circle(c) => c.radius - *value,
                    Geometry::Arc(a) => a.length() - *value,
                }])
            }
        }
    }
}

pub fn satisfied(residuals: &[f64], tolerance: f64) -> bool {
    residuals
        .iter()
        .all(|r| r.is_finite() && r.abs() <= tolerance.max(EPSILON))
}
