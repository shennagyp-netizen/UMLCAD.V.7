#![allow(clippy::should_implement_trait)]

use super::{
    constraints::{geometry_difference, residual as constraint_residual},
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
            analysis,
            geometry: materialize(snapshot, &values, &ids, &offsets)?,
        });
    }

    let mut damping = options.initial_damping;
    let mut rank = 0usize;
    let mut condition = f64::INFINITY;

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
                    analysis: current_analysis,
                    geometry: materialize(snapshot, &values, &ids, &offsets)?,
                });
            }
        };
        rank = linear.rank;
        condition = linear.condition_number;

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
            damping = (damping * 0.3).max(1.0e-12);
            if proposed_scaled_norm <= options.residual_tolerance
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
        analysis: final_analysis,
        geometry: materialize(snapshot, &values, &ids, &offsets)?,
    })
}
