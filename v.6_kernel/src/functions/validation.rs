use super::snapshot::{Constraint, Relation, RelationPoint, SemanticSnapshot};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagnostic {
    pub code: String,
    pub message: String,
    pub severity: Severity,
}

fn push(diagnostics: &mut Vec<Diagnostic>, code: &str, message: String) {
    diagnostics.push(Diagnostic {
        code: code.into(),
        message,
        severity: Severity::Error,
    });
}

pub fn validate_snapshot(snapshot: &SemanticSnapshot) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    let mut parameter_ids = BTreeSet::new();
    for parameter in &snapshot.parameters {
        if parameter.id.is_empty() {
            push(
                &mut diagnostics,
                "INVALID_ID",
                "Parameter ID must not be empty".into(),
            );
        }
        if !parameter_ids.insert(parameter.id.clone()) {
            push(
                &mut diagnostics,
                "DUPLICATE_ID",
                format!("Duplicate parameter ID: {}", parameter.id),
            );
        }
        if !parameter.default_value.is_finite() {
            push(
                &mut diagnostics,
                "INVALID_PARAMETER",
                format!("Parameter {} has a non-finite default value", parameter.id),
            );
        }
    }

    let mut geometry_ids = BTreeSet::new();
    for geometry in &snapshot.geometry {
        if geometry.id.is_empty() {
            push(
                &mut diagnostics,
                "INVALID_ID",
                "Geometry ID must not be empty".into(),
            );
        }
        if !geometry_ids.insert(geometry.id.clone()) {
            push(
                &mut diagnostics,
                "DUPLICATE_ID",
                format!("Duplicate geometry ID: {}", geometry.id),
            );
        }
        if let Err(error) = geometry.geometry.validate() {
            push(
                &mut diagnostics,
                "INVALID_GEOMETRY",
                format!("{}: {}", geometry.id, error),
            );
        }
        for parameter_id in &geometry.parameter_dependencies {
            if !parameter_ids.contains(parameter_id) {
                push(
                    &mut diagnostics,
                    "STALE_PARAMETER_REFERENCE",
                    format!(
                        "Geometry {} references unknown parameter {}",
                        geometry.id, parameter_id
                    ),
                );
            }
        }
    }

    let mut constraint_ids = BTreeSet::new();
    for (id, constraint) in &snapshot.constraints {
        if id.is_empty() {
            push(
                &mut diagnostics,
                "INVALID_ID",
                "Constraint ID must not be empty".into(),
            );
        }
        if !constraint_ids.insert(id.clone()) {
            push(
                &mut diagnostics,
                "DUPLICATE_ID",
                format!("Duplicate constraint ID: {id}"),
            );
        }
        validate_constraint_values(&mut diagnostics, id, constraint);
        for geometry_id in constraint_ids_for(constraint) {
            if !geometry_ids.contains(geometry_id) {
                push(
                    &mut diagnostics,
                    "STALE_REFERENCE",
                    format!("{} references unknown geometry {}", id, geometry_id),
                );
            }
        }
    }

    let mut relation_ids_seen = BTreeSet::new();
    for (id, relation) in &snapshot.relations {
        if id.is_empty() {
            push(
                &mut diagnostics,
                "INVALID_ID",
                "Relation ID must not be empty".into(),
            );
        }
        if !relation_ids_seen.insert(id.clone()) {
            push(
                &mut diagnostics,
                "DUPLICATE_ID",
                format!("Duplicate relation ID: {id}"),
            );
        }
        validate_relation_values(&mut diagnostics, id, relation);
        for geometry_id in relation_ids_for(relation) {
            if !geometry_ids.contains(geometry_id) {
                push(
                    &mut diagnostics,
                    "STALE_REFERENCE",
                    format!("{} references unknown geometry {}", id, geometry_id),
                );
            }
        }
    }

    diagnostics
}

fn validate_constraint_values(diagnostics: &mut Vec<Diagnostic>, id: &str, c: &Constraint) {
    if let Constraint::Distance { value, .. } = c {
        if !value.is_finite() || *value < 0.0 {
            push(
                diagnostics,
                "INVALID_CONSTRAINT",
                format!("Constraint {id} has an invalid distance value"),
            );
        }
    }
}

fn validate_relation_values(diagnostics: &mut Vec<Diagnostic>, id: &str, r: &Relation) {
    match r {
        Relation::Angle { radians, .. } => {
            if !radians.is_finite() {
                push(
                    diagnostics,
                    "INVALID_RELATION",
                    format!("Relation {id} has a non-finite angle"),
                );
            }
        }
        Relation::Radius { value, .. } | Relation::Diameter { value, .. } => {
            if !value.is_finite() || *value <= 0.0 {
                push(
                    diagnostics,
                    "INVALID_RELATION",
                    format!("Relation {id} has an invalid radius/diameter value"),
                );
            }
        }
        Relation::DistancePoints { value, .. }
            if !value.is_finite() || *value < 0.0 =>
        {
            push(
                diagnostics,
                "INVALID_RELATION",
                format!("Relation {id} has an invalid point distance"),
            );
        }
        _ => {}
    }
}

fn relation_point_id(p: &RelationPoint) -> &str {
    match p {
        RelationPoint::Endpoint { geometry_id, .. } | RelationPoint::Center { geometry_id } => {
            geometry_id
        }
    }
}

fn constraint_ids_for(c: &Constraint) -> Vec<&str> {
    match c {
        Constraint::Horizontal { entity_id }
        | Constraint::Vertical { entity_id }
        | Constraint::Fixed { entity_id } => vec![entity_id],
        Constraint::Coincident {
            first_geometry_id,
            second_geometry_id,
            ..
        } => vec![first_geometry_id, second_geometry_id],
        Constraint::Distance {
            first_geometry_id,
            second_geometry_id,
            ..
        } => second_geometry_id.as_ref().map_or_else(
            || vec![first_geometry_id.as_str()],
            |second| vec![first_geometry_id.as_str(), second.as_str()],
        ),
    }
}

fn relation_ids_for(r: &Relation) -> Vec<&str> {
    use Relation::*;
    match r {
        Parallel {
            first_geometry_id,
            second_geometry_id,
        }
        | Perpendicular {
            first_geometry_id,
            second_geometry_id,
        }
        | EqualLength {
            first_geometry_id,
            second_geometry_id,
        }
        | Angle {
            first_geometry_id,
            second_geometry_id,
            ..
        }
        | Collinear {
            first_geometry_id,
            second_geometry_id,
        }
        | Concentric {
            first_geometry_id,
            second_geometry_id,
        }
        | EqualRadius {
            first_geometry_id,
            second_geometry_id,
        }
        | Tangent {
            first_geometry_id,
            second_geometry_id,
            ..
        } => vec![first_geometry_id, second_geometry_id],
        Radius { geometry_id, .. } | Diameter { geometry_id, .. } => vec![geometry_id],
        Midpoint {
            point,
            line_geometry_id,
        }
        | PointOnLine {
            point,
            line_geometry_id,
        } => vec![relation_point_id(point), line_geometry_id],
        PointOnCircle {
            point,
            circle_geometry_id,
        } => vec![relation_point_id(point), circle_geometry_id],
        DistancePoints { first, second, .. } => {
            vec![relation_point_id(first), relation_point_id(second)]
        }
        Symmetric {
            first,
            second,
            about,
        } => vec![
            relation_point_id(first),
            relation_point_id(second),
            relation_point_id(about),
        ],
    }
}
