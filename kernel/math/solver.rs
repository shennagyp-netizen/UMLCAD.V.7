#![allow(clippy::should_implement_trait)]

use super::{
    constraints::{geometry_difference, residual as constraint_residual},
    convergence::{evaluate_accepted_history, ConvergenceStatus},
    geometry::{Arc, Circle, Geometry, Line, Point},
    jacobian::analytic_constraint_jacobian,
    relations::evaluate_relation,
    snapshot::{Constraint, SemanticSnapshot},
};
use nalgebra::{DMatrix, DVector};

#[derive(Clone, Debug, PartialEq)]
pub struct LinearSolveReport {
    pub delta: Vec<f64>,
    pub rank: usize,
    pub degrees_of_freedom: usize,
    /// 2-norm condition estimate of the internally column-normalized Jacobian.
    /// It describes the normalized linear system, not the raw input Jacobian.
    /// Uniform changes of parameter units leave this diagnostic invariant because
    /// each nonzero column is normalized before the singular-value decomposition.
    pub condition_number: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SolveOptions {
    pub max_iterations: usize,
    pub residual_tolerance: f64,
    pub step_tolerance: f64,
    pub initial_damping: f64,
    /// Retained for API compatibility. Supported production equations use the
    /// complete analytic Jacobian authority rather than finite-difference probes.
    pub finite_difference_step: f64,
}

impl Default for SolveOptions {
    fn default() -> Self {
        Self {
            max_iterations: 100,
            residual_tolerance: 1.0e-8,
            step_tolerance: 1.0e-10,
            initial_damping: 1.0e-3,
            finite_difference_step: 1.0e-7,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum SolveReason {
    Converged,
    MaxIterations,
    Singular,
    InvalidDomain,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ConstraintResidualReport {
    pub residuals: Vec<f64>,
    pub norm: f64,
    pub kind: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ConstraintAnalysis {
    pub residuals: Vec<ConstraintResidualReport>,
    pub residual_norm: f64,
    pub scaled_residual_norm: f64,
    pub max_residual: f64,
    pub satisfied: bool,
    pub relation_satisfied: bool,
    pub valid: bool,
    pub degrees_of_freedom: usize,
    pub variable_count: usize,
    pub equation_count: usize,
    pub rank: usize,
    pub condition_estimate: f64,
    pub well_conditioned: bool,
    pub relation_count: usize,
    pub relation_equation_count: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ConstraintSolveResult {
    pub converged: bool,
    pub reason: SolveReason,
    pub iterations: usize,
    pub initial_residual_norm: f64,
    pub final_residual_norm: f64,
    pub initial_scaled_residual_norm: f64,
    pub final_scaled_residual_norm: f64,
    /// Dimensionless 2-norm of the last accepted solver step. Geometric
    /// translations and radii are normalized by the initial model extent; angular
    /// parameters remain in radians. Zero means that no step was accepted, as in
    /// a zero-iteration convergence or a failure before the first accepted proposal.
    pub final_step_norm: f64,
    pub analysis: ConstraintAnalysis,
    pub geometry: Vec<(String, Geometry)>,
}

fn stable_norm(values: &[f64]) -> f64 {
    if values.iter().any(|value| !value.is_finite()) {
        return f64::NAN;
    }
    let scale = values.iter().map(|value| value.abs()).fold(0.0, f64::max);
    if scale == 0.0 {
        return 0000.0;
    }
    let sum = values
        .iter()
        .map(|value| {
            let normalized = *value / scale;
            normalized * normalized
        })
        .sum::<f64>();
    if !sum.is_finite() {
        return f64::INFINITY;
    }
    scale * sum.sqrt()
}

fn model_scale(snapshot: &SemanticSnapshot) -> Result<f64, String> {
    let mut min_x = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_y = f64::NEG_INFINITY;

    let mut include = |x: f64, y: f64| -> Result<(), String> {
        if !x.is_finite() || !y.is_finite() {
            return Err("non-finite model geometry".into());
        }
        min_x = min_x.min(x);
        max_x = max_x.max(x);
        min_y = min_y.min(y);
        max_y = max_y.max(y);
        Ok(())
    };

    for item in &snapshot.geometry {
        match item.geometry {
            Geometry::Line(line) => {
                include(line.start.x, line.start.y)?;
                include(line.end.x, line.end.y)?;
            }
            Geometry::Circle(circle) => {
                let r = circle.radius;
                include(circle.center.x - r, circle.center.y - r)?;
                include(circle.center.x + r, circle.center.y + r)?;
            }
            Geometry::Arc(arc) => {
                let r = arc.radius;
                include(arc.center.x - r, arc.center.y - r)?;
                include(arc.center.x + r, arc.center.y + r)?;
            }
        }
    }

    let dx = max_x - min_x;
    let dy = max_y - min_y;
    if !dx.is_finite() || !dy.is_finite() {
        return Err("non-finite model scale".into());
    }
    let scale = dx.hypot(dy);
    if scale.is_finite() && scale > 0.0 {
        Ok(scale)
    } else {
        Err("degenerate model scale".into())
    }
}

fn normalized_step_norm(
    snapshot: &SemanticSnapshot,
    delta: &[f64],
    scale: f64,
) -> Result<f64, String> {
    if !scale.is_finite() || scale <= 0.0 {
        return Err("invalid model scale for step normalization".into());
    }

    let expected = snapshot
        .geometry
        .iter()
        .map(|item| enc(&item.geometry).len())
        .sum::<usize>();
    if delta.len() != expected {
        return Err("solver step/model variable mismatch".into());
    }

    let mut normalized = Vec::with_capacity(delta.len());
    let mut offset = 0usize;
    for item in &snapshot.geometry {
        let width = enc(&item.geometry).len();
        let values = &delta[offset..offset + width];
        match &item.geometry {
            Geometry::Line(_) | Geometry::Circle(_) => {
                normalized.extend(values.iter().map(|value| *value / scale));
            }
            Geometry::Arc(_) => {
                normalized.extend(values[..3].iter().map(|value| *value / scale));
                normalized.extend(values[3..].iter().copied());
            }
        }
        offset += width;
    }

    if normalized.iter().any(|value| !value.is_finite()) {
        return Err("non-finite normalized solver step".into());
    }
    Ok(stable_norm(&normalized))
}

fn rank_condition(
    singular_values: &DVector<f64>,
    tol: f64,
) -> Result<(usize, f64), String> {
    let max = singular_values
        .iter()
        .copied()
        .filter(|value| value.is_finite())
        .fold(0.0, f64::max);
    if max <= 0.0 {
        return Ok((0, f64::INFINITY));
    }
    let threshold = tol * max;
    if !threshold.is_finite() {
        return Err("non-finite rank threshold".into());
    }
    let mut rank = 0usize;
    let mut min_nonzero = f64::INFINITY;
    for value in singular_values.iter().copied() {
        if !value.is_finite() {
            return Err("non-finite singular value".into());
        }
        if value > threshold {
            rank += 1;
            min_nonzero = min_nonzero.min(value);
        }
    }
    if rank == 0 {
        Ok((0, f64::INFINITY))
    } else {
        let condition = max / min_nonzero;
        if condition.is_finite() {
            Ok((rank, condition))
        } else {
            Ok((rank, f64::INFINITY))
        }
    }
}

pub fn scaled_damped_qr(
    j: &[Vec<f64>],
    r: &[f64],
    damping: f64,
    tolerance: f64,
) -> Result<LinearSolveReport, String> {
    let rows = j.len();
    let cols = j.first().map_or(0, Vec::len);
    if rows != r.len() {
        return Err("jacobian/residual row mismatch".into());
    }
    if j.iter().any(|row| row.len() != cols) {
        return Err("ragged jacobian".into());
    }
    if !damping.is_finite() || damping < 0.0 {
        return Err("invalid damping".into());
    }
    if !tolerance.is_finite() || tolerance < 0.0 {
        return Err("invalid rank tolerance".into());
    }
    if r.iter().any(|value| !value.is_finite()) {
        return Err("non-finite residual".into());
    }
    if cols == 0 {
        return Ok(LinearSolveReport {
            delta: Vec::new(),
            rank: 0,
            degrees_of_freedom: 0,
            condition_number: 1.0,
        });
    }

    let mut a = DMatrix::<f64>::zeros(rows, cols);
    for i in 0..rows {
        for k in 0..cols {
            if !j[i][k].is_finite() {
                return Err("non-finite jacobian".into());
            }
            a[(i, k)] = j[i][k];
        }
    }

    let mut column_scales = vec![1.0; cols];
    for column in 0..cols {
        let values = (0..rows).map(|row| a[(row, column)]).collect::<Vec<_>>();
        let norm = stable_norm(&values);
        if norm.is_nan() {
            return Err("non-finite jacobian column norm".into());
        }
        if norm > 0.0 {
            column_scales[column] = norm;
            for row in 0..rows {
                a[(row, column)] /= norm;
            }
        }
    }

    let svd = a.svd(true, true);
    let (rank, condition_number) = rank_condition(&svd.singular_values, tolerance)?;
    let max_singular = svd
        .singular_values
        .iter()
        .copied()
        .fold(0.0, f64::max);
    let rank_threshold = tolerance * max_singular;
    if !rank_threshold.is_finite() {
        return Err("non-finite rank threshold".into());
    }
    let u = svd.u.ok_or("SVD left vectors unavailable")?;
    let vt = svd.v_t.ok_or("SVD right vectors unavailable")?;
    let b = DVector::from_iterator(rows, r.iter().map(|value| -*value));
    let ub = u.transpose() * b;
    if ub.iter().any(|value| !value.is_finite()) {
        return Err("non-finite projected residual".into());
    }
    let mut z = DVector::<f64>::zeros(cols);
    for k in 0..svd.singular_values.len().min(cols) {
        let singular = svd.singular_values[k];
        if !singular.is_finite() {
            return Err("non-finite singular value".into());
        }
        if damping == 0.0 && singular <= rank_threshold {
            continue;
        }
        let denominator = singular * singular + damping;
        if !denominator.is_finite() || denominator == 0.0 {
            return Err("invalid damped singular denominator".into());
        }
        let coefficient = singular / denominator * ub[k];
        if !coefficient.is_finite() {
            return Err("non-finite damped solve coefficient".into());
        }
        for n in 0..cols {
            z[n] += vt[(k, n)] * coefficient;
            if !z[n].is_finite() {
                return Err("non-finite normalized solver step".into());
            }
        }
    }

    let delta = z
        .iter()
        .enumerate()
        .map(|(column, value)| *value / column_scales[column])
        .collect::<Vec<_>>();
    if delta.iter().any(|value| !value.is_finite()) {
        return Err("non-finite solver step".into());
    }

    Ok(LinearSolveReport {
        delta,
        rank,
        degrees_of_freedom: cols.saturating_sub(rank),
        condition_number,
    })
}

fn enc(g: &Geometry) -> Vec<f64> {
    match g {
        Geometry::Line(x) => vec![x.start.x, x.start.y, x.end.x, x.end.y],
        Geometry::Circle(x) => vec![x.center.x, x.center.y, x.radius],
        Geometry::Arc(x) => vec![x.center.x, x.center.y, x.radius, x.start_angle, x.end_angle],
    }
}

fn dec(template: &Geometry, v: &[f64]) -> Geometry {
    match template {
        Geometry::Line(_) => Geometry::Line(Line {
            start: Point { x: v[0], y: v[1] },
            end: Point { x: v[2], y: v[3] },
        }),
        Geometry::Circle(_) => Geometry::Circle(Circle {
            center: Point { x: v[0], y: v[1] },
            radius: v[2],
        }),
        Geometry::Arc(_) => Geometry::Arc(Arc {
            center: Point { x: v[0], y: v[1] },
            radius: v[2],
            start_angle: v[3],
            end_angle: v[4],
        }),
    }
}

fn vars(snapshot: &SemanticSnapshot) -> (Vec<String>, Vec<usize>) {
    let mut ids = Vec::with_capacity(snapshot.geometry.len());
    let mut offsets = vec![0usize];
    for g in &snapshot.geometry {
        ids.push(g.id.clone());
        let width = enc(&g.geometry).len();
        offsets.push(offsets.last().copied().unwrap_or(0) + width);
    }
    (ids, offsets)
}

fn candidate(
    snapshot: &SemanticSnapshot,
    values: &[f64],
    ids: &[String],
    offsets: &[usize],
) -> Result<SemanticSnapshot, String> {
    let mut result = snapshot.clone();
    for (index, geometry) in result.geometry.iter_mut().enumerate() {
        let template = snapshot
            .geometry(&ids[index])
            .ok_or_else(|| format!("Unknown geometry: {}", ids[index]))?;
        let decoded = dec(template, &values[offsets[index]..offsets[index + 1]]);
        decoded
            .validate()
            .map_err(|error| format!("Candidate geometry invalid for {}: {error:?}", ids[index]))?;
        geometry.geometry = decoded;
    }
    Ok(result)
}

fn constraint_scales(
    snapshot: &SemanticSnapshot,
    constraint: &Constraint,
    scale: f64,
) -> Result<Vec<f64>, String> {
    match constraint {
        Constraint::Horizontal { .. } | Constraint::Vertical { .. } => Ok(vec![scale]),
        Constraint::Coincident { .. } => Ok(vec![scale, scale]),
        Constraint::Distance { .. } => Ok(vec![scale]),
        Constraint::Fixed { entity_id } => {
            let geometry = snapshot
                .geometry(entity_id)
                .ok_or_else(|| format!("Unknown geometry: {entity_id}"))?;
            Ok(match geometry {
                Geometry::Line(_) => vec![scale; 4],
                Geometry::Circle(_) => vec![scale; 3],
                Geometry::Arc(_) => vec![scale, scale, scale, 1.0, 1.0],
            })
        }
    }
}

fn residuals(
    snapshot: &SemanticSnapshot,
    values: &[f64],
    ids: &[String],
    offsets: &[usize],
    scale: f64,
    satisfaction_tolerance: f64,
) -> Result<(Vec<f64>, Vec<f64>, Vec<ConstraintResidualReport>, usize, bool), String> {
    let current = candidate(snapshot, values, ids, offsets)?;
    let mut raw = Vec::new();
    let mut scaled = Vec::new();
    let mut reports = Vec::new();
    let mut relation_equations = 0usize;
    let mut relations_satisfied = true;

    for (id, constraint) in &snapshot.constraints {
        let residual = match constraint {
            Constraint::Fixed { entity_id } => {
                let actual = current
                    .geometry(entity_id)
                    .ok_or_else(|| format!("Unknown geometry: {entity_id}"))?;
                let reference = snapshot
                    .geometry(entity_id)
                    .ok_or_else(|| format!("Unknown geometry: {entity_id}"))?;
                geometry_difference(actual, reference).ok_or("Fixed geometry type mismatch")?
            }
            _ => constraint_residual(
                |geometry_id| current.geometry(geometry_id).cloned(),
                constraint,
            )
            .ok_or_else(|| format!("Invalid constraint domain: {id}"))?,
        };
        if residual.iter().any(|value| !value.is_finite()) {
            return Err(format!("non-finite constraint residual: {id}"));
        }
        let scales = constraint_scales(snapshot, constraint, scale)?;
        if scales.len() != residual.len() || scales.iter().any(|value| !value.is_finite() || *value <= 0.0) {
            return Err(format!("invalid constraint residual scale: {id}"));
        }
        raw.extend(residual.iter().copied());
        scaled.extend(
            residual
                .iter()
                .zip(scales.iter())
                .map(|(value, scale)| value / scale),
        );
        reports.push(ConstraintResidualReport {
            norm: stable_norm(&residual),
            residuals: residual,
            kind: format!("constraint:{id}"),
        });
    }

    for (index, (_, relation)) in snapshot.relations.iter().enumerate() {
        let evaluation = evaluate_relation(&current, relation)?;
        if evaluation.residuals.iter().any(|value| !value.is_finite())
            || evaluation
                .scales
                .iter()
                .any(|value| !value.is_finite() || *value <= 0.0)
            || evaluation.scales.len() != evaluation.residuals.len()
        {
            return Err(format!("non-finite relation evaluation: {}", index + 1));
        }
        let scaled_relation = evaluation
            .residuals
            .iter()
            .zip(evaluation.scales.iter())
            .map(|(value, relation_scale)| *value / *relation_scale)
            .collect::<Vec<_>>();
        if scaled_relation.iter().any(|value| !value.is_finite()) {
            return Err(format!("non-finite scaled relation residual: {}", index + 1));
        }
        relation_equations += evaluation.residuals.len();
        relations_satisfied &= stable_norm(&scaled_relation) <= satisfaction_tolerance;
        raw.extend(evaluation.residuals.iter().copied());
        scaled.extend(scaled_relation);
        reports.push(ConstraintResidualReport {
            norm: evaluation.norm,
            residuals: evaluation.residuals,
            kind: format!("relation:{}", index + 1),
        });
    }

    Ok((raw, scaled, reports, relation_equations, relations_satisfied))
}

fn jac(
    snapshot: &SemanticSnapshot,
    values: &[f64],
    ids: &[String],
    offsets: &[usize],
) -> Result<Vec<Vec<f64>>, String> {
    let current = candidate(snapshot, values, ids, offsets)?;
    analytic_constraint_jacobian(&current)
        .map_err(|error| format!("analytic constraint jacobian: {error:?}"))
}

fn finite(values: &[f64]) -> bool {
    values.iter().all(|value| value.is_finite())
}

fn materialize(
    snapshot: &SemanticSnapshot,
    values: &[f64],
    ids: &[String],
    offsets: &[usize],
) -> Result<Vec<(String, Geometry)>, String> {
    let mut out = Vec::with_capacity(ids.len());
    for (index, id) in ids.iter().enumerate() {
        let geometry = snapshot
            .geometry(id)
            .ok_or_else(|| format!("Unknown geometry: {id}"))?;
        let decoded = dec(geometry, &values[offsets[index]..offsets[index + 1]]);
        if !decoded.validate().is_ok() {
            return Err(format!("Invalid solved geometry: {id}"));
        }
        out.push((id.clone(), decoded));
    }
    Ok(out)
}

fn analysis(
    snapshot: &SemanticSnapshot,
    values: &[f64],
    ids: &[String],
    offsets: &[usize],
    scale: f64,
    satisfaction_tolerance: f64,
    relation_equation_count: usize,
    relations_satisfied: bool,
    rank: usize,
    condition: f64,
) -> Result<ConstraintAnalysis, String> {
    let (raw, scaled, reports, _, _) = residuals(
        snapshot,
        values,
        ids,
        offsets,
        scale,
        satisfaction_tolerance,
    )?;
    let residual_norm = stable_norm(&raw);
    let scaled_residual_norm = stable_norm(&scaled);
    let max_residual = raw.iter().map(|value| value.abs()).fold(0.0, f64::max);
    Ok(ConstraintAnalysis {
        residuals: reports,
        residual_norm,
        scaled_residual_norm,
        max_residual,
        satisfied: scaled_residual_norm.is_finite()
            && scaled_residual_norm <= satisfaction_tolerance
            && relations_satisfied,
        relation_satisfied: relations_satisfied,
        valid: finite(&raw) && finite(&scaled),
        degrees_of_freedom: values.len().saturating_sub(rank),
        variable_count: values.len(),
        equation_count: raw.len(),
        rank,
        condition_estimate: condition,
        well_conditioned: condition.is_finite() && condition < 1.0e10,
        relation_count: snapshot.relations.len(),
        relation_equation_count,
    })
}

pub fn solve_snapshot(
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
    let (initial_raw, initial_scaled, _, initial_relation_equations, initial_relations_satisfied) = residuals(
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
    if initial_scaled_residual_norm <= options.residual_tolerance && initial_relations_satisfied {
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
    let mut accepted_scaled_residual_history = vec![initial_scaled_residual_norm];

    for iteration in 1..=options.max_iterations {
        let (base_raw, _, _, relation_equations, relations_satisfied) = residuals(
            snapshot,
            &values,
            &ids,
            &offsets,
            scale,
            options.residual_tolerance,
        )?;
        let jacobian = jac(snapshot, &values, &ids, &offsets)?;
        let linear = match scaled_damped_qr(&jacobian, &base_raw, damping, 1.0e-10) {
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
        let candidate_step_norm =
            normalized_step_norm(snapshot, &linear.delta, scale)?;

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
        let (_, current_scaled, _, _, _) = residuals(
            snapshot,
            &values,
            &ids,
            &offsets,
            scale,
            options.residual_tolerance,
        )?;
        let current_scaled_norm = stable_norm(&current_scaled);
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
            accepted_scaled_residual_history.push(proposed_scaled_norm);
            last_step_norm = candidate_step_norm;
            damping = (damping * 0.3).max(1.0e-12);
            let convergence = evaluate_accepted_history(
                &accepted_scaled_residual_history,
                candidate_step_norm,
                options.residual_tolerance,
                options.step_tolerance,
                accepted_scaled_residual_history.len().saturating_sub(1),
                iteration,
                options.max_iterations,
            );
            if convergence.status == ConvergenceStatus::Converged
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
mod dimensionless_step_metric_tests {
    use super::*;

    fn snapshot_with_geometry(scale: f64) -> SemanticSnapshot {
        SemanticSnapshot {
            parameters: Vec::new(),
            geometry: vec![
                super::super::snapshot::GeometryItem {
                    id: "line".into(),
                    geometry: Geometry::Line(Line {
                        start: Point { x: 0.0, y: 0.0 },
                        end: Point { x: scale, y: 0.0 },
                    }),
                    parameter_dependencies: Vec::new(),
                },
                super::super::snapshot::GeometryItem {
                    id: "circle".into(),
                    geometry: Geometry::Circle(Circle {
                        center: Point { x: 2.0 * scale, y: scale },
                        radius: 0.5 * scale,
                    }),
                    parameter_dependencies: Vec::new(),
                },
                super::super::snapshot::GeometryItem {
                    id: "arc".into(),
                    geometry: Geometry::Arc(Arc {
                        center: Point { x: 4.0 * scale, y: 0.0 },
                        radius: 0.75 * scale,
                        start_angle: 0.25,
                        end_angle: 1.25,
                    }),
                    parameter_dependencies: Vec::new(),
                },
            ],
            constraints: Vec::new(),
            relations: Vec::new(),
        }
    }

    #[test]
    fn normalized_step_norm_is_uniform_scale_invariant() {
        let small = snapshot_with_geometry(10.0);
        let large = snapshot_with_geometry(1000.0);
        let small_scale = model_scale(&small).unwrap();
        let large_scale = model_scale(&large).unwrap();
        let base = vec![
            0.4, -0.2, -0.3, 0.1,
            0.5, -0.25, 0.125,
            0.75, -0.5, 0.2, 0.04, -0.06,
        ];
        let scaled = vec![
            40.0, -20.0, -30.0, 10.0,
            50.0, -25.0, 12.5,
            75.0, -50.0, 20.0, 0.04, -0.06,
        ];

        let small_norm = normalized_step_norm(&small, &base, small_scale).unwrap();
        let large_norm = normalized_step_norm(&large, &scaled, large_scale).unwrap();
        assert!((small_norm - large_norm).abs() <= 1.0e-14);
    }

    #[test]
    fn normalized_step_norm_keeps_angles_scale_independent() {
        let small = snapshot_with_geometry(10.0);
        let large = snapshot_with_geometry(1000.0);
        let small_scale = model_scale(&small).unwrap();
        let large_scale = model_scale(&large).unwrap();
        let angular = vec![
            0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.3, -0.4,
        ];

        let small_norm = normalized_step_norm(&small, &angular, small_scale).unwrap();
        let large_norm = normalized_step_norm(&large, &angular, large_scale).unwrap();
        assert!((small_norm - 0.5).abs() <= 1.0e-15);
        assert!((small_norm - large_norm).abs() <= 1.0e-15);
    }

    #[test]
    fn normalized_step_norm_fails_closed_on_bad_scale_or_length() {
        let snapshot = snapshot_with_geometry(10.0);
        let delta = vec![0.0; 12];
        assert!(normalized_step_norm(&snapshot, &delta, 0.0).is_err());
        assert!(normalized_step_norm(&snapshot, &delta[..11], 10.0).is_err());
    }
}


#[cfg(test)]
mod mixed_unit_solver_tests {
    use super::*;
    use super::super::snapshot::{Endpoint, GeometryItem, Relation, RelationPoint};

    fn mixed_unit_snapshot(scale: f64) -> SemanticSnapshot {
        SemanticSnapshot {
            parameters: Vec::new(),
            geometry: vec![
                GeometryItem {
                    id: "base".into(),
                    geometry: Geometry::Line(Line {
                        start: Point { x: 0.0, y: 0.0 },
                        end: Point { x: 4.0 * scale, y: 0.0 },
                    }),
                    parameter_dependencies: Vec::new(),
                },
                GeometryItem {
                    id: "moving".into(),
                    geometry: Geometry::Line(Line {
                        start: Point { x: 4.2 * scale, y: 0.3 * scale },
                        end: Point { x: 7.2 * scale, y: 4.3 * scale },
                    }),
                    parameter_dependencies: Vec::new(),
                },
            ],
            constraints: vec![
                (
                    "base-fixed".into(),
                    Constraint::Fixed {
                        entity_id: "base".into(),
                    },
                ),
                (
                    "moving-start-coincident".into(),
                    Constraint::Coincident {
                        first_geometry_id: "base".into(),
                        first_point: Endpoint::End,
                        second_geometry_id: "moving".into(),
                        second_point: Endpoint::Start,
                    },
                ),
            ],
            relations: vec![
                (
                    "moving-length".into(),
                    Relation::DistancePoints {
                        first: RelationPoint::Endpoint {
                            geometry_id: "moving".into(),
                            point: Endpoint::Start,
                        },
                        second: RelationPoint::Endpoint {
                            geometry_id: "moving".into(),
                            point: Endpoint::End,
                        },
                        value: 5.0 * scale,
                    },
                ),
                (
                    "moving-angle".into(),
                    Relation::Angle {
                        first_geometry_id: "base".into(),
                        second_geometry_id: "moving".into(),
                        radians: std::f64::consts::FRAC_PI_2,
                    },
                ),
            ],
        }
        .deterministic()
    }

    fn line(result: &ConstraintSolveResult) -> Line {
        result
            .geometry
            .iter()
            .find_map(|(id, geometry)| {
                if id == "moving" {
                    match geometry {
                        Geometry::Line(line) => Some(*line),
                        _ => None,
                    }
                } else {
                    None
                }
            })
            .expect("moving line")
    }

    #[test]
    fn mixed_unit_nonlinear_fixture_is_scale_consistent() {
        let small = solve_snapshot(&mixed_unit_snapshot(1.0), SolveOptions::default()).unwrap();
        let large = solve_snapshot(&mixed_unit_snapshot(1.0e9), SolveOptions::default()).unwrap();

        assert!(small.converged);
        assert!(large.converged);
        assert_eq!(small.reason, SolveReason::Converged);
        assert_eq!(large.reason, SolveReason::Converged);
        assert!(small.final_scaled_residual_norm <= 1.0e-8);
        assert!(large.final_scaled_residual_norm <= 1.0e-8);
        assert!(small.final_step_norm <= 1.0e-10);
        assert!(large.final_step_norm <= 1.0e-10);

        let a = line(&small);
        let b = line(&large);
        for (x_small, x_large) in [
            (a.start.x, b.start.x / 1.0e9),
            (a.start.y, b.start.y / 1.0e9),
            (a.end.x, b.end.x / 1.0e9),
            (a.end.y, b.end.y / 1.0e9),
        ] {
            assert!(
                (x_small - x_large).abs() <= 1.0e-8,
                "scale-inconsistent coordinate: small={x_small}, large={x_large}"
            );
        }
    }

    #[test]
    fn mixed_unit_fixture_retains_dimensionless_terminal_evidence() {
        for scale in [1.0, 1.0e6, 1.0e9] {
            let result = solve_snapshot(&mixed_unit_snapshot(scale), SolveOptions::default()).unwrap();
            assert!(result.converged, "scale {scale}");
            assert!(result.final_scaled_residual_norm.is_finite());
            assert!(result.final_step_norm.is_finite());
            assert!(result.final_scaled_residual_norm <= 1.0e-8);
            assert!(result.final_step_norm <= 1.0e-10);
        }
    }
}
