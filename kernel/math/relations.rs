use super::constraints::endpoint;
use super::geometry::{Geometry, Line, Point};
use super::snapshot::{Relation, RelationPoint, RelationSnapshot, SemanticSnapshot, TangentMode};
use std::f64::consts::PI;

#[derive(Clone, Debug, PartialEq)]
pub struct RelationResidual {
    pub relation_kind: String,
    pub residuals: Vec<f64>,
    pub scales: Vec<f64>,
    pub norm: f64,
    pub scaled_norm: f64,
}

fn wrap_angle(a: f64) -> f64 {
    a.sin().atan2(a.cos())
}
fn distance(a: Point, b: Point) -> f64 {
    a.distance(b)
}
fn cross(a: Point, b: Point) -> f64 {
    a.x * b.y - a.y * b.x
}
fn dot(a: Point, b: Point) -> f64 {
    a.x * b.x + a.y * b.y
}
fn vector(g: &Geometry) -> Result<Point, String> {
    match g {
        Geometry::Line(l) => Ok(l.end.sub(l.start)),
        _ => Err("Relation requires a line".into()),
    }
}
fn length(g: &Geometry) -> f64 {
    match g {
        Geometry::Line(l) => l.length(),
        Geometry::Circle(c) => 2.0 * PI * c.radius,
        Geometry::Arc(a) => a.length(),
    }
}
fn radius_of(g: &Geometry) -> Result<f64, String> {
    match g {
        Geometry::Line(_) => Err("Relation requires circular geometry".into()),
        Geometry::Circle(c) => Ok(c.radius),
        Geometry::Arc(a) => Ok(a.radius),
    }
}
fn geometry(snapshot: &SemanticSnapshot, id: &str) -> Result<Geometry, String> {
    snapshot
        .geometry(id)
        .cloned()
        .ok_or_else(|| format!("Unknown geometry: {id}"))
}
fn resolve_point(snapshot: &SemanticSnapshot, p: &RelationPoint) -> Result<Point, String> {
    let id = match p {
        RelationPoint::Endpoint { geometry_id, .. } | RelationPoint::Center { geometry_id } => {
            geometry_id
        }
    };
    let g = snapshot
        .geometry(id)
        .ok_or_else(|| format!("Unknown geometry: {id}"))?;
    match p {
        RelationPoint::Center { .. } => match g {
            Geometry::Line(_) => Err("Geometry has no center".into()),
            Geometry::Circle(c) => Ok(c.center),
            Geometry::Arc(a) => Ok(a.center),
        },
        RelationPoint::Endpoint { point, .. } => {
            endpoint(g, *point).ok_or_else(|| "Circular geometry has no endpoints".into())
        }
    }
}
fn point_line_distance(p: Point, l: &Line) -> Result<f64, String> {
    let d = l.end.sub(l.start);
    let n = d.norm();
    if n <= 1e-12 {
        return Err("Degenerate line".into());
    }
    Ok(cross(p.sub(l.start), d).abs() / n)
}
fn relation_kind(r: &Relation) -> &'static str {
    match r {
        Relation::Parallel { .. } => "parallel",
        Relation::Perpendicular { .. } => "perpendicular",
        Relation::EqualLength { .. } => "equal-length",
        Relation::Angle { .. } => "angle",
        Relation::Collinear { .. } => "collinear",
        Relation::Concentric { .. } => "concentric",
        Relation::EqualRadius { .. } => "equal-radius",
        Relation::Radius { .. } => "radius",
        Relation::Diameter { .. } => "diameter",
        Relation::Tangent { .. } => "tangent",
        Relation::Midpoint { .. } => "midpoint",
        Relation::PointOnLine { .. } => "point-on-line",
        Relation::PointOnCircle { .. } => "point-on-circle",
        Relation::DistancePoints { .. } => "distance-points",
        Relation::Symmetric { .. } => "symmetric",
    }
}
fn make(r: &Relation, residuals: Vec<f64>, scales: Vec<f64>) -> RelationResidual {
    let norm = residuals.iter().map(|x| x * x).sum::<f64>().sqrt();
    let scaled_norm = residuals
        .iter()
        .enumerate()
        .map(|(i, x)| {
            let s = scales.get(i).copied().unwrap_or(1.0);
            let y = if s.abs() > 0.0 { *x / s } else { *x };
            y * y
        })
        .sum::<f64>()
        .sqrt();
    RelationResidual {
        relation_kind: relation_kind(r).into(),
        residuals,
        scales,
        norm,
        scaled_norm,
    }
}

pub fn validate_relation(snapshot: &SemanticSnapshot, r: &Relation) -> Result<(), String> {
    match r {
        Relation::Parallel {
            first_geometry_id,
            second_geometry_id,
        }
        | Relation::Perpendicular {
            first_geometry_id,
            second_geometry_id,
        }
        | Relation::Angle {
            first_geometry_id,
            second_geometry_id,
            ..
        }
        | Relation::Collinear {
            first_geometry_id,
            second_geometry_id,
        } => {
            vector(&geometry(snapshot, first_geometry_id)?)?;
            vector(&geometry(snapshot, second_geometry_id)?)?;
            Ok(())
        }
        Relation::EqualLength {
            first_geometry_id,
            second_geometry_id,
        } => {
            length(&geometry(snapshot, first_geometry_id)?);
            length(&geometry(snapshot, second_geometry_id)?);
            Ok(())
        }
        Relation::Concentric {
            first_geometry_id,
            second_geometry_id,
        }
        | Relation::EqualRadius {
            first_geometry_id,
            second_geometry_id,
        } => {
            radius_of(&geometry(snapshot, first_geometry_id)?)?;
            radius_of(&geometry(snapshot, second_geometry_id)?)?;
            Ok(())
        }
        Relation::Radius { geometry_id, value } | Relation::Diameter { geometry_id, value } => {
            radius_of(&geometry(snapshot, geometry_id)?)?;
            if !value.is_finite() || *value <= 0.0 {
                Err("Radius/diameter value must be positive and finite".into())
            } else {
                Ok(())
            }
        }
        Relation::Tangent {
            first_geometry_id,
            second_geometry_id,
            ..
        } => {
            geometry(snapshot, first_geometry_id)?;
            geometry(snapshot, second_geometry_id)?;
            Ok(())
        }
        Relation::Midpoint {
            point: rp,
            line_geometry_id,
        }
        | Relation::PointOnLine {
            point: rp,
            line_geometry_id,
        } => {
            resolve_point(snapshot, rp)?;
            match geometry(snapshot, line_geometry_id)? {
                Geometry::Line(_) => Ok(()),
                _ => Err("Relation requires a line".into()),
            }
        }
        Relation::PointOnCircle {
            point: rp,
            circle_geometry_id,
        } => {
            resolve_point(snapshot, rp)?;
            radius_of(&geometry(snapshot, circle_geometry_id)?)?;
            Ok(())
        }
        Relation::DistancePoints {
            first,
            second,
            value,
        } => {
            resolve_point(snapshot, first)?;
            resolve_point(snapshot, second)?;
            if !value.is_finite() || *value < 0.0 {
                Err("Point distance must be finite and non-negative".into())
            } else {
                Ok(())
            }
        }
        Relation::Symmetric {
            first,
            second,
            about,
        } => {
            resolve_point(snapshot, first)?;
            resolve_point(snapshot, second)?;
            resolve_point(snapshot, about)?;
            Ok(())
        }
    }
}

pub fn evaluate_relation(
    snapshot: &SemanticSnapshot,
    r: &Relation,
) -> Result<RelationResidual, String> {
    validate_relation(snapshot, r)?;
    match r {
        Relation::Parallel {
            first_geometry_id,
            second_geometry_id,
        } => {
            let a = vector(&geometry(snapshot, first_geometry_id)?)?;
            let b = vector(&geometry(snapshot, second_geometry_id)?)?;
            let s = a.norm().max(b.norm()).max(1.0);
            Ok(make(r, vec![cross(a, b)], vec![s * s]))
        }
        Relation::Perpendicular {
            first_geometry_id,
            second_geometry_id,
        } => {
            let a = vector(&geometry(snapshot, first_geometry_id)?)?;
            let b = vector(&geometry(snapshot, second_geometry_id)?)?;
            let s = a.norm().max(b.norm()).max(1.0);
            Ok(make(r, vec![dot(a, b)], vec![s * s]))
        }
        Relation::EqualLength {
            first_geometry_id,
            second_geometry_id,
        } => {
            let a = length(&geometry(snapshot, first_geometry_id)?);
            let b = length(&geometry(snapshot, second_geometry_id)?);
            Ok(make(r, vec![a - b], vec![a.max(b).max(1.0)]))
        }
        Relation::Angle {
            first_geometry_id,
            second_geometry_id,
            radians,
        } => {
            let a = vector(&geometry(snapshot, first_geometry_id)?)?;
            let b = vector(&geometry(snapshot, second_geometry_id)?)?;
            Ok(make(
                r,
                vec![wrap_angle(b.y.atan2(b.x) - a.y.atan2(a.x) - radians)],
                vec![1.0],
            ))
        }
        Relation::Collinear {
            first_geometry_id,
            second_geometry_id,
        } => {
            let Geometry::Line(a) = geometry(snapshot, first_geometry_id)? else {
                return Err("Collinear relation requires lines".into());
            };
            let Geometry::Line(b) = geometry(snapshot, second_geometry_id)? else {
                return Err("Collinear relation requires lines".into());
            };
            let d = a.end.sub(a.start);
            let s = d.norm().max(1.0);
            Ok(make(
                r,
                vec![cross(d, b.start.sub(a.start)), cross(d, b.end.sub(a.start))],
                vec![s * s, s * s],
            ))
        }
        Relation::Concentric {
            first_geometry_id,
            second_geometry_id,
        } => {
            let a = geometry(snapshot, first_geometry_id)?;
            let b = geometry(snapshot, second_geometry_id)?;
            let ca = match a {
                Geometry::Circle(c) => c.center,
                Geometry::Arc(a) => a.center,
                _ => return Err("Concentric relation requires circular geometries".into()),
            };
            let cb = match b {
                Geometry::Circle(c) => c.center,
                Geometry::Arc(a) => a.center,
                _ => return Err("Concentric relation requires circular geometries".into()),
            };
            let s = radius_of(&geometry(snapshot, first_geometry_id)?)?
                .max(radius_of(&geometry(snapshot, second_geometry_id)?)?)
                .max(1.0);
            Ok(make(r, vec![ca.x - cb.x, ca.y - cb.y], vec![s, s]))
        }
        Relation::EqualRadius {
            first_geometry_id,
            second_geometry_id,
        } => {
            let a = radius_of(&geometry(snapshot, first_geometry_id)?)?;
            let b = radius_of(&geometry(snapshot, second_geometry_id)?)?;
            Ok(make(r, vec![a - b], vec![a.max(b).max(1.0)]))
        }
        Relation::Radius { geometry_id, value } => {
            let a = radius_of(&geometry(snapshot, geometry_id)?)?;
            Ok(make(r, vec![a - value], vec![value.abs().max(1.0)]))
        }
        Relation::Diameter { geometry_id, value } => {
            let a = radius_of(&geometry(snapshot, geometry_id)?)?;
            Ok(make(r, vec![2.0 * a - value], vec![value.abs().max(1.0)]))
        }
        Relation::Tangent {
            first_geometry_id,
            second_geometry_id,
            mode,
        } => tangent(snapshot, r, first_geometry_id, second_geometry_id, *mode),
        Relation::Midpoint {
            point: rp,
            line_geometry_id,
        } => {
            let p = resolve_point(snapshot, rp)?;
            let Geometry::Line(l) = geometry(snapshot, line_geometry_id)? else {
                return Err("Midpoint relation requires a line".into());
            };
            let m = Point {
                x: (l.start.x + l.end.x) / 2.0,
                y: (l.start.y + l.end.y) / 2.0,
            };
            let s = l.length().max(1.0);
            Ok(make(r, vec![p.x - m.x, p.y - m.y], vec![s, s]))
        }
        Relation::PointOnLine {
            point: rp,
            line_geometry_id,
        } => {
            let p = resolve_point(snapshot, rp)?;
            let Geometry::Line(l) = geometry(snapshot, line_geometry_id)? else {
                return Err("Point-on-line relation requires a line".into());
            };
            Ok(make(
                r,
                vec![point_line_distance(p, &l)?],
                vec![l.length().max(1.0)],
            ))
        }
        Relation::PointOnCircle {
            point: rp,
            circle_geometry_id,
        } => {
            let p = resolve_point(snapshot, rp)?;
            let g = geometry(snapshot, circle_geometry_id)?;
            let (c, rad) = match g {
                Geometry::Circle(c) => (c.center, c.radius),
                Geometry::Arc(a) => (a.center, a.radius),
                _ => return Err("Point-on-circle relation requires circular geometry".into()),
            };
            Ok(make(r, vec![distance(p, c) - rad], vec![rad.max(1.0)]))
        }
        Relation::DistancePoints {
            first,
            second,
            value,
        } => {
            let a = resolve_point(snapshot, first)?;
            let b = resolve_point(snapshot, second)?;
            Ok(make(
                r,
                vec![distance(a, b) - value],
                vec![value.abs().max(1.0)],
            ))
        }
        Relation::Symmetric {
            first,
            second,
            about,
        } => {
            let a = resolve_point(snapshot, first)?;
            let b = resolve_point(snapshot, second)?;
            let o = resolve_point(snapshot, about)?;
            let d = distance(a, b).max(1.0);
            Ok(make(
                r,
                vec![(a.x + b.x) / 2.0 - o.x, (a.y + b.y) / 2.0 - o.y],
                vec![d, d],
            ))
        }
    }
}

fn tangent(
    snapshot: &SemanticSnapshot,
    r: &Relation,
    a_id: &str,
    b_id: &str,
    mode: TangentMode,
) -> Result<RelationResidual, String> {
    let a = geometry(snapshot, a_id)?;
    let b = geometry(snapshot, b_id)?;
    match (&a, &b) {
        (Geometry::Line(_), Geometry::Line(_)) => {
            Err("Two lines do not define a tangent relation".into())
        }
        (Geometry::Line(l), Geometry::Circle(c)) => Ok(make(
            r,
            vec![point_line_distance(c.center, l)? - c.radius],
            vec![c.radius.max(1.0)],
        )),
        (Geometry::Line(l), Geometry::Arc(c)) => Ok(make(
            r,
            vec![point_line_distance(c.center, l)? - c.radius],
            vec![c.radius.max(1.0)],
        )),
        (Geometry::Circle(c), Geometry::Line(l)) => Ok(make(
            r,
            vec![point_line_distance(c.center, l)? - c.radius],
            vec![c.radius.max(1.0)],
        )),
        (Geometry::Arc(c), Geometry::Line(l)) => Ok(make(
            r,
            vec![point_line_distance(c.center, l)? - c.radius],
            vec![c.radius.max(1.0)],
        )),
        (Geometry::Circle(ca), Geometry::Circle(cb)) => {
            circle_tangent(r, ca.center, ca.radius, cb.center, cb.radius, mode)
        }
        (Geometry::Circle(ca), Geometry::Arc(cb)) => {
            circle_tangent(r, ca.center, ca.radius, cb.center, cb.radius, mode)
        }
        (Geometry::Arc(ca), Geometry::Circle(cb)) => {
            circle_tangent(r, ca.center, ca.radius, cb.center, cb.radius, mode)
        }
        (Geometry::Arc(ca), Geometry::Arc(cb)) => {
            circle_tangent(r, ca.center, ca.radius, cb.center, cb.radius, mode)
        }
    }
}
fn circle_tangent(
    r: &Relation,
    ca: Point,
    ra: f64,
    cb: Point,
    rb: f64,
    mode: TangentMode,
) -> Result<RelationResidual, String> {
    let d = distance(ca, cb);
    let ext = d - ra - rb;
    let int = d - (ra - rb).abs();
    let value = match mode {
        TangentMode::External => ext,
        TangentMode::Internal => int,
        TangentMode::Any => {
            if ext.abs() <= int.abs() {
                ext
            } else {
                int
            }
        }
    };
    Ok(make(r, vec![value], vec![(ra + rb).max(1.0)]))
}

pub fn relation_snapshot(
    base: SemanticSnapshot,
    relations: Vec<(String, Relation)>,
) -> RelationSnapshot {
    RelationSnapshot { base, relations }
}
