//! Existing solver implementation plus the production row-scaled linearization path.
//!
//! `solver.rs` remains the implementation corpus. The child production module can
//! access its private helpers without duplicating geometry/residual/analysis logic.
//! The only production numerical-path change here is explicit row scaling of each
//! Jacobian/residual equation before the existing nalgebra-backed SVD solve.

include!("solver.rs");

mod production {
    use super::*;
    use crate::math::linearization::scale_linearization;

    fn linearization_row_scales(
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

    pub fn solve_snapshot_row_scaled(
        snapshot: &SemanticSnapshot,
        options: SolveOptions,
    ) -> Result<ConstraintSolveResult, String> {
        if options.max_iterations == 0
            || !options.residual_tolerance.is_finite()
            || options.residual_tolerance < 0.0
            || !options.step_tolerance.is_finite()
            || options.step_tolerance < 0.0
            || !options.initial_damping.is_finite()
            || options.initial_damping < 0.0
            || !options.finite_difference_step.is_finite()
            || options.finite_difference_step <= 0.0
        {
            return Err("Invalid solver options".into());
        }

        let scale = model_scale(snapshot)?;
        let (ids, offsets) = vars(snapshot);
        let mut values = Vec::new();
        for geometry in &snapshot.geometry {
            values.extend(enc(&geometry.geometry));
        }

        let (
            initial_raw,
            initial_scaled,
            _,
            initial_relation_equations,
            initial_relations_satisfied,
        ) = residuals(
            snapshot,
            &values,
            &ids,
            &offsets,
            scale,
            options.residual_tolerance,
        )?;
        let initial_residual_norm = stable_norm(&initial_raw);
        let initial_scaled_residual_norm = stable_norm(&initial_scaled);
        if !initial_residual_norm.is_finite() || !initial_scaled_residual_norm.is_finite() {
            return Err("Non-finite initial residual".into());
        }

        if initial_scaled_residual_norm <= options.residual_tolerance
            && initial_relations_satisfied
        {
            let analysis = analysis(
                snapshot,
                &values,
                &ids,
                &offsets,
                scale,
                options.residual_tolerance,
                initial_relation_equations,
                initial_relations_satisfied,
                0,
                1.0,
            )?;
            return Ok(ConstraintSolveResult {
                converged: analysis.satisfied,
                reason: if analysis.satisfied {
                    SolveReason::Converged
                } else {
                    SolveReason::MaxIterations
                },
                iterations: 0,
                initial_residual_norm,
                final_residual_norm: initial_residual_norm,
                initial_scaled_residual_norm,
                final_scaled_residual_norm: initial_scaled_residual_norm,
                final_step_norm: 0.0,
                analysis,
                geometry: materialize(snapshot, &values, &ids, &offsets)?,
            });
        }

        let mut damping = options.initial_damping;
        let mut rank = 0usize;
        let mut condition = f64::INFINITY;
        let mut last_step_norm = 0.0;

        for iteration in 1..=options.max_iterations {
            let (
                base_raw,
                base_scaled,
                _,
                relation_equations,
                relations_satisfied,
            ) = residuals(
                snapshot,
                &values,
                &ids,
                &offsets,
                scale,
                options.residual_tolerance,
            )?;
            let jacobian = jac(snapshot, &values, &ids, &offsets)?;
            let row_scales = linearization_row_scales(snapshot, &values, &ids, &offsets, scale)?;
            let scaled_system = scale_linearization(&jacobian, &base_raw, &row_scales)?;

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
            ) {
                Ok(report) => report,
                Err(_) => {
                    let current_analysis = analysis(
                        snapshot,
                        &values,
                        &ids,
                        &offsets,
                        scale,
                        options.residual_tolerance,
                        relation_equations,
                        relations_satisfied,
                        rank,
                        condition,
                    )?;
                    return Ok(ConstraintSolveResult {
                        converged: false,
                        reason: SolveReason::Singular,
                        iterations: iteration,
                        initial_residual_norm,
                        final_residual_norm: current_analysis.residual_norm,
                        initial_scaled_residual_norm,
                        final_scaled_residual_norm: current_analysis.scaled_residual_norm,
                        final_step_norm: last_step_norm,
                        analysis: current_analysis,
                        geometry: materialize(snapshot, &values, &ids, &offsets)?,
                    });
                }
            };
            rank = linear.rank;
            condition = linear.condition_number;
            let candidate_step_norm = normalized_step_norm(snapshot, &linear.delta, scale)?;

            let mut proposal = values.clone();
            for (value, delta) in proposal.iter_mut().zip(linear.delta.iter()) {
                *value += *delta;
            }
            if !finite(&proposal) {
                damping = (damping * 10.0).min(1.0e12);
                continue;
            }

            let proposed = residuals(
                snapshot,
                &proposal,
                &ids,
                &offsets,
                scale,
                options.residual_tolerance,
            );
            let current_scaled_norm = stable_norm(&base_scaled);
            let Ok((proposed_raw, proposed_scaled, _, proposed_relation_equations, proposed_relations_satisfied)) = proposed else {
                damping = (damping * 10.0).min(1.0e12);
                continue;
            };
            let proposed_scaled_norm = stable_norm(&proposed_scaled);
            if !proposed_scaled_norm.is_finite() {
                damping = (damping * 10.0).min(1.0e12);
                continue;
            }

            if proposed_scaled_norm < current_scaled_norm {
                values = proposal;
                last_step_norm = candidate_step_norm;
                damping = (damping * 0.3).max(1.0e-12);
                let terminal = verify_terminal(
                    proposed_scaled_norm,
                    candidate_step_norm,
                    options.residual_tolerance,
                    options.step_tolerance,
                    iteration,
                );
                if terminal.status == TerminalConvergenceStatus::Converged
                    && proposed_relations_satisfied
                {
                    let final_analysis = analysis(
                        snapshot,
                        &values,
                        &ids,
                        &offsets,
                        scale,
                        options.residual_tolerance,
                        proposed_relation_equations,
                        proposed_relations_satisfied,
                        rank,
                        condition,
                    )?;
                    return Ok(ConstraintSolveResult {
                        converged: final_analysis.satisfied,
                        reason: if final_analysis.satisfied {
                            SolveReason::Converged
                        } else {
                            SolveReason::MaxIterations
                        },
                        iterations: iteration,
                        initial_residual_norm,
                        final_residual_norm: stable_norm(&proposed_raw),
                        initial_scaled_residual_norm,
                        final_scaled_residual_norm: proposed_scaled_norm,
                        final_step_norm: last_step_norm,
                        analysis: final_analysis,
                        geometry: materialize(snapshot, &values, &ids, &offsets)?,
                    });
                }
            } else {
                damping = (damping * 10.0).min(1.0e12);
            }
        }

        let (final_raw, final_scaled, _, relation_equations, relations_satisfied) = residuals(
            snapshot,
            &values,
            &ids,
            &offsets,
            scale,
            options.residual_tolerance,
        )?;
        let final_analysis = analysis(
            snapshot,
            &values,
            &ids,
            &offsets,
            scale,
            options.residual_tolerance,
            relation_equations,
            relations_satisfied,
            rank,
            condition,
        )?;
        Ok(ConstraintSolveResult {
            converged: false,
            reason: SolveReason::MaxIterations,
            iterations: options.max_iterations,
            initial_residual_norm,
            final_residual_norm: stable_norm(&final_raw),
            initial_scaled_residual_norm,
            final_scaled_residual_norm: stable_norm(&final_scaled),
            final_step_norm: last_step_norm,
            analysis: final_analysis,
            geometry: materialize(snapshot, &values, &ids, &offsets)?,
        })
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn row_scaled_solver_preserves_uniform_geometry_scale_behavior() {
            let make_snapshot = |s: f64| SemanticSnapshot {
                parameters: Vec::new(),
                geometry: vec![GeometryItem {
                    id: "l".into(),
                    geometry: Geometry::Line(Line {
                        start: Point { x: 0.0, y: s },
                        end: Point { x: 2.0 * s, y: 2.0 * s },
                    }),
                    parameter_dependencies: Vec::new(),
                }],
                constraints: vec![(
                    "horizontal".into(),
                    Constraint::Horizontal { entity_id: "l".into() },
                )],
                relations: vec![],
            }
            .deterministic();

            let small = solve_snapshot_row_scaled(&make_snapshot(1.0), SolveOptions::default()).unwrap();
            let large = solve_snapshot_row_scaled(&make_snapshot(1.0e9), SolveOptions::default()).unwrap();
            assert_eq!(small.converged, large.converged);
            assert_eq!(small.reason, large.reason);
            assert_eq!(small.iterations, large.iterations);
            assert!((small.final_scaled_residual_norm - large.final_scaled_residual_norm).abs() <= 1.0e-12);
            assert!((small.final_step_norm - large.final_step_norm).abs() <= 1.0e-12);
        }
    }
}

pub use production::solve_snapshot_row_scaled;
