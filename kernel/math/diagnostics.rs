use super::snapshot::{Constraint, Relation, SemanticSnapshot};

#[derive(Clone, Debug, PartialEq)]
pub struct DiagnosticClassification {
    pub redundant_constraint_ids: Vec<String>,
    pub contradictory_constraint_ids: Vec<String>,
    pub contradictory_relation_indexes: Vec<usize>,
    pub warnings: Vec<String>,
}

pub fn classify_constraints(snapshot: &SemanticSnapshot) -> DiagnosticClassification {
    let mut result = DiagnosticClassification {
        redundant_constraint_ids: Vec::new(),
        contradictory_constraint_ids: Vec::new(),
        contradictory_relation_indexes: Vec::new(),
        warnings: Vec::new(),
    };
    for i in 0..snapshot.constraints.len() {
        for j in 0..i {
            if snapshot.constraints[i].1 == snapshot.constraints[j].1 {
                result
                    .redundant_constraint_ids
                    .push(snapshot.constraints[i].0.clone());
                result.warnings.push(format!(
                    "Constraint {} duplicates {}",
                    snapshot.constraints[i].0, snapshot.constraints[j].0
                ));
            }
        }
    }
    for (id, c) in &snapshot.constraints {
        if let Constraint::Horizontal { entity_id } = c {
            if snapshot
                .constraints
                .iter()
                .any(|(_, x)| matches!(x,Constraint::Vertical{entity_id:other} if other==entity_id))
            {
                result.contradictory_constraint_ids.push(id.clone());
            }
        }
        if let Constraint::Vertical { entity_id } = c {
            if snapshot.constraints.iter().any(
                |(_, x)| matches!(x,Constraint::Horizontal{entity_id:other} if other==entity_id),
            ) {
                result.contradictory_constraint_ids.push(id.clone());
            }
        }
    }
    for i in 0..snapshot.relations.len() {
        for j in 0..i {
            let a = &snapshot.relations[i].1;
            let b = &snapshot.relations[j].1;
            if conflicting_relations(a, b) {
                result.contradictory_relation_indexes.extend([j, i]);
            }
        }
    }
    result.contradictory_constraint_ids.sort();
    result.contradictory_constraint_ids.dedup();
    result.contradictory_relation_indexes.sort();
    result.contradictory_relation_indexes.dedup();
    result
}

fn pair(a: &str, b: &str, c: &str, d: &str) -> bool {
    (a == c && b == d) || (a == d && b == c)
}
fn conflicting_relations(a: &Relation, b: &Relation) -> bool {
    match (a, b) {
        (
            Relation::Parallel {
                first_geometry_id: a1,
                second_geometry_id: a2,
            },
            Relation::Perpendicular {
                first_geometry_id: b1,
                second_geometry_id: b2,
            },
        )
        | (
            Relation::Perpendicular {
                first_geometry_id: a1,
                second_geometry_id: a2,
            },
            Relation::Parallel {
                first_geometry_id: b1,
                second_geometry_id: b2,
            },
        ) => pair(a1, a2, b1, b2),
        (
            Relation::Radius {
                geometry_id: a,
                value: x,
            },
            Relation::Radius {
                geometry_id: b,
                value: y,
            },
        ) if a == b => x != y,
        (
            Relation::Diameter {
                geometry_id: a,
                value: x,
            },
            Relation::Diameter {
                geometry_id: b,
                value: y,
            },
        ) if a == b => x != y,
        _ => false,
    }
}
