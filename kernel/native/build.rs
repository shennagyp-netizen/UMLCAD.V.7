use std::{env, fs, path::PathBuf};

const TARGET_BLOCK: &str = r#"        let (base_raw, _, _, relation_equations, relations_satisfied) = residuals(
            snapshot,
            &values,
            &ids,
            &offsets,
            scale,
            options.residual_tolerance,
        )?;
        let jacobian = jac(snapshot, &values, &ids, &offsets)?;
        let linear = match scaled_damped_qr(&jacobian, &base_raw, damping, 1.0e-10) {"#;

const PATCHED_BLOCK: &str = r#"        let (base_raw, base_scaled, _, relation_equations, relations_satisfied) = residuals(
            snapshot,
            &values,
            &ids,
            &offsets,
            scale,
            options.residual_tolerance,
        )?;
        let jacobian = jac(snapshot, &values, &ids, &offsets)?;
        let row_scales = production_row_scales(snapshot, &values, &ids, &offsets, scale)?;
        let scaled_system = super::linearization::scale_linearization(
            &jacobian,
            &base_raw,
            &row_scales,
        )?;
        if scaled_system.residual.len() != base_scaled.len() {
            return Err("scaled linearization/residual row mismatch".into());
        }
        for (scaled_from_system, scaled_from_residuals) in
            scaled_system.residual.iter().zip(base_scaled.iter())
        {
            let comparison_scale = scaled_from_system
                .abs()
                .max(scaled_from_residuals.abs())
                .max(1.0);
            if (scaled_from_system - scaled_from_residuals).abs()
                > 1.0e-12 * comparison_scale
            {
                return Err("linearization scale disagrees with residual scaling".into());
            }
        }
        let linear = match scaled_damped_qr(
            &scaled_system.jacobian,
            &scaled_system.residual,
            damping,
            1.0e-10,
        ) {"#;

const HELPER: &str = r#"

fn production_row_scales(
    snapshot: &SemanticSnapshot,
    values: &[f64],
    ids: &[String],
    offsets: &[usize],
    scale: f64,
) -> Result<Vec<f64>, String> {
    let current = candidate(snapshot, values, ids, offsets)?;
    let mut scales = Vec::new();

    for (id, constraint) in &snapshot.constraints {
        let residual = match constraint {
            Constraint::Fixed { entity_id } => {
                let actual = current
                    .geometry(entity_id)
                    .ok_or_else(|| format!("Unknown geometry: {entity_id}"))?;
                let reference = snapshot
                    .geometry(entity_id)
                    .ok_or_else(|| format!("Unknown geometry: {entity_id}"))?;
                geometry_difference(actual, reference)
                    .ok_or_else(|| format!("Fixed geometry type mismatch: {id}"))?
            }
            _ => constraint_residual(
                |geometry_id| current.geometry(geometry_id).cloned(),
                constraint,
            )
            .ok_or_else(|| format!("Invalid constraint domain: {id}"))?,
        };
        let row_scales = constraint_scales(snapshot, constraint, scale)?;
        if row_scales.len() != residual.len()
            || row_scales
                .iter()
                .any(|value| !value.is_finite() || *value <= 0.0)
        {
            return Err(format!("invalid constraint linearization scale: {id}"));
        }
        scales.extend(row_scales);
    }

    for (index, (_, relation)) in snapshot.relations.iter().enumerate() {
        let evaluation = evaluate_relation(&current, relation)?;
        if evaluation.residuals.len() != evaluation.scales.len()
            || evaluation
                .scales
                .iter()
                .any(|value| !value.is_finite() || *value <= 0.0)
        {
            return Err(format!(
                "invalid relation linearization scale: {}",
                index + 1
            ));
        }
        scales.extend(evaluation.scales);
    }

    Ok(scales)
}
"#;

const PRODUCTION_TESTS: &str = r#"

#[cfg(test)]
mod production_row_scaling_tests {
    use super::*;
    use crate::GeometryItem;

    fn scaled_snapshot(scale: f64) -> SemanticSnapshot {
        SemanticSnapshot {
            parameters: Vec::new(),
            geometry: vec![GeometryItem {
                id: "l".into(),
                geometry: Geometry::Line(Line {
                    start: Point { x: 0.0, y: 1.0 * scale },
                    end: Point { x: 2.0 * scale, y: 2.0 * scale },
                }),
                parameter_dependencies: Vec::new(),
            }],
            constraints: vec![(
                "horizontal".into(),
                Constraint::Horizontal { entity_id: "l".into() },
            )],
            relations: vec![],
        }
        .deterministic()
    }

    #[test]
    fn production_solver_uses_dimensionless_linearization_for_uniform_geometry_scale() {
        let small = solve_snapshot(&scaled_snapshot(1.0), SolveOptions::default()).unwrap();
        let large = solve_snapshot(&scaled_snapshot(1.0e9), SolveOptions::default()).unwrap();
        assert_eq!(small.converged, large.converged);
        assert_eq!(small.reason, large.reason);
        assert_eq!(small.iterations, large.iterations);
        assert!((small.final_scaled_residual_norm - large.final_scaled_residual_norm).abs() <= 1.0e-12);
        assert!((small.final_step_norm - large.final_step_norm).abs() <= 1.0e-12);
    }
}
"#;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let solver_path = manifest_dir.join("../math/solver.rs");
    let source = fs::read_to_string(&solver_path).expect("failed to read kernel/math/solver.rs");

    println!("cargo:rerun-if-changed={}", solver_path.display());
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../math/linearization.rs");

    let mut generated = source;
    let first_line = "#![allow(clippy::should_implement_trait)]\n\n";
    assert!(generated.starts_with(first_line), "unexpected solver.rs header");
    generated = generated.replacen(first_line, "", 1);

    assert_eq!(generated.matches(TARGET_BLOCK).count(), 1, "solver loop patch anchor must occur exactly once");
    generated = generated.replacen(TARGET_BLOCK, &format!("{}{}", HELPER, PATCHED_BLOCK), 1);
    generated.push_str(PRODUCTION_TESTS);

    let out = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));
    fs::write(out.join("solver_generated.rs"), generated)
        .expect("failed to write generated solver source");
}
