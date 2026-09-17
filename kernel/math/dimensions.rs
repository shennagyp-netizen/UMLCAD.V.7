use super::constraints::endpoint;
use super::geometry::Geometry;
use super::snapshot::Endpoint;
use std::f64::consts::PI;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DimensionKind {
    Length,
    Radius,
    Diameter,
    Distance,
    Angle,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DimensionSpec {
    pub id: String,
    pub kind: DimensionKind,
    pub first_geometry_id: String,
    pub second_geometry_id: Option<String>,
    pub first_point: Option<Endpoint>,
    pub second_point: Option<Endpoint>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EvaluatedDimension {
    pub id: String,
    pub kind: DimensionKind,
    pub value: f64,
    pub units: DimensionUnits,
    pub valid: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DimensionUnits {
    Model,
    Radians,
}

fn length(g: &Geometry) -> f64 {
    match g {
        Geometry::Line(l) => l.length(),
        Geometry::Circle(c) => 2.0 * PI * c.radius,
        Geometry::Arc(a) => a.radius * (a.end_angle - a.start_angle).abs(),
    }
}

pub fn evaluate_dimensions(
    snapshot: &super::snapshot::SemanticSnapshot,
    dimensions: &[DimensionSpec],
) -> Result<Vec<EvaluatedDimension>, String> {
    dimensions
        .iter()
        .map(|d| {
            let a = snapshot
                .geometry(&d.first_geometry_id)
                .ok_or_else(|| format!("Unknown geometry: {}", d.first_geometry_id))?;
            match d.kind {
                DimensionKind::Length => Ok(EvaluatedDimension {
                    id: d.id.clone(),
                    kind: d.kind,
                    value: length(a),
                    units: DimensionUnits::Model,
                    valid: true,
                }),
                DimensionKind::Radius => Ok(EvaluatedDimension {
                    id: d.id.clone(),
                    kind: d.kind,
                    value: if matches!(a, Geometry::Line(_)) {
                        f64::NAN
                    } else {
                        match a {
                            Geometry::Circle(c) => c.radius,
                            Geometry::Arc(a) => a.radius,
                            _ => f64::NAN,
                        }
                    },
                    units: DimensionUnits::Model,
                    valid: !matches!(a, Geometry::Line(_)),
                }),
                DimensionKind::Diameter => Ok(EvaluatedDimension {
                    id: d.id.clone(),
                    kind: d.kind,
                    value: if matches!(a, Geometry::Line(_)) {
                        f64::NAN
                    } else {
                        200.0 * match a {
                            Geometry::Circle(c) => c.radius,
                            Geometry::Arc(a) => a.radius,
                            _ => f64::NAN,
                        }
                    },
                    units: DimensionUnits::Model,
                    valid: !matches!(a, Geometry::Line(_)),
                }),
                DimensionKind::Distance => {
                    let second_id = d
                        .second_geometry_id
                        .as_ref()
                        .ok_or_else(|| "Distance dimension requires second geometry".to_string())?;
                    let b = snapshot
                        .geometry(second_id)
                        .ok_or_else(|| format!("Unknown geometry: {second_id}"))?;
                    if matches!(a, Geometry::Circle(_)) || matches!(b, Geometry::Circle(_)) {
                        return Ok(EvaluatedDimension {
                            id: d.id.clone(),
                            kind: d.kind,
                            value: f64::NAN,
                            units: DimensionUnits::Model,
                            valid: false,
                        });
                    }
                    let pa = endpoint(a, d.first_point.unwrap_or(Endpoint::Start))
                        .ok_or_else(|| "First geometry has no endpoint".to_string())?;
                    let pb = endpoint(b, d.second_point.unwrap_or(Endpoint::Start))
                        .ok_or_else(|| "Second geometry has no endpoint".to_string())?;
                    Ok(EvaluatedDimension {
                        id: d.id.clone(),
                        kind: d.kind,
                        value: pa.distance(pb),
                        units: DimensionUnits::Model,
                        valid: true,
                    })
                }
                DimensionKind::Angle => {
                    let second_id = d
                        .second_geometry_id
                        .as_ref()
                        .ok_or_else(|| "Angle dimension requires second geometry".to_string())?;
                    let b = snapshot
                        .geometry(second_id)
                        .ok_or_else(|| format!("Unknown geometry: {second_id}"))?;
                    let (Geometry::Line(x), Geometry::Line(y)) = (a, b) else {
                        return Ok(EvaluatedDimension {
                            id: d.id.clone(),
                            kind: d.kind,
                            value: f64::NAN,
                            units: DimensionUnits::Radians,
                            valid: false,
                        });
                    };
                    let ax = x.end.x - x.start.x;
                    let ay = x.end.y - x.start.y;
                    let bx = y.end.x - y.start.x;
                    let by = y.end.y - y.start.y;
                    Ok(EvaluatedDimension {
                        id: d.id.clone(),
                        kind: d.kind,
                        value: (ax * by - ay * bx).atan2(ax * bx + ay * by),
                        units: DimensionUnits::Radians,
                        valid: true,
                    })
                }
            }
        })
        .collect()
}
