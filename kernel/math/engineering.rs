use super::solver::{solve_snapshot, SolveOptions};
use super::{
    diagnostics::classify_constraints, dxf::export_dxf, relations::evaluate_relation,
    snapshot::SemanticSnapshot, spatial::spatial_analysis, topology::build_topology,
    validation::validate_snapshot,
};

#[derive(Clone, Debug, PartialEq)]
pub struct EngineeringEvidence {
    pub structural_validity: bool,
    pub constraint_validity: bool,
    pub relation_validity: bool,
    pub numerical_conditioning: bool,
    pub reference_validity: bool,
    pub topology_validity: bool,
    pub spatial_validity: bool,
    pub engineering_rule_validity: bool,
    pub export_validity: bool,
    pub diagnostics: Vec<super::validation::Diagnostic>,
}

pub fn validate_engineering(snapshot: &SemanticSnapshot) -> EngineeringEvidence {
    let mut diagnostics = validate_snapshot(snapshot);
    let structural_validity = diagnostics
        .iter()
        .all(|d| !matches!(d.severity, super::validation::Severity::Error));
    let classification = classify_constraints(snapshot);
    for id in &classification.contradictory_constraint_ids {
        diagnostics.push(super::validation::Diagnostic {
            code: "contradictory-constraint".into(),
            message: format!("Contradictory constraint: {id}"),
            severity: super::validation::Severity::Error,
        });
    }
    for index in &classification.contradictory_relation_indexes {
        diagnostics.push(super::validation::Diagnostic {
            code: "contradictory-relation".into(),
            message: format!("Contradictory relation index: {}", index + 1),
            severity: super::validation::Severity::Error,
        });
    }
    let solve = solve_snapshot(snapshot, SolveOptions::default()).ok();
    let constraint_validity = structural_validity
        && classification.contradictory_constraint_ids.is_empty()
        && solve
            .as_ref()
            .map(|r| r.analysis.satisfied)
            .unwrap_or(false);
    let relation_validity = structural_validity
        && classification.contradictory_relation_indexes.is_empty()
        && snapshot.relations.iter().all(|(_, r)| {
            evaluate_relation(snapshot, r)
                .map(|x| x.scaled_norm.is_finite() && x.scaled_norm <= 1e-8)
                .unwrap_or(false)
        });
    let numerical_conditioning = solve
        .as_ref()
        .map(|r| r.analysis.well_conditioned || r.analysis.variable_count == 0)
        .unwrap_or(false);
    let reference_validity = !diagnostics.iter().any(|d| {
        d.code == "STALE_REFERENCE" && matches!(d.severity, super::validation::Severity::Error)
    });
    let topology_validity = structural_validity && build_topology(snapshot).is_ok();
    let spatial_validity = structural_validity
        && spatial_analysis(&snapshot.geometry, 1e-8)
            .iter()
            .all(|x| x.distance.is_finite());
    let export_validity = export_dxf(snapshot).ends_with("EOF\n");
    let engineering_rule_validity = structural_validity
        && constraint_validity
        && relation_validity
        && numerical_conditioning
        && reference_validity
        && topology_validity
        && spatial_validity
        && export_validity;
    EngineeringEvidence {
        structural_validity,
        constraint_validity,
        relation_validity,
        numerical_conditioning,
        reference_validity,
        topology_validity,
        spatial_validity,
        engineering_rule_validity,
        export_validity,
        diagnostics,
    }
}
