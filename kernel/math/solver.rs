#![allow(clippy::type_complexity, clippy::too_many_arguments)]

use super::{
    constraints::{geometry_difference, residual as constraint_residual},
    geometry::{Arc, Circle, Geometry, Line, Point},
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
    pub finite_difference_step: f64,
}

impl Default for SolveOptions {
    fn default() -> Self {
        Self {
            max_iterations: 100,
            residual_tolerance: 1e-8,
            step_tolerance: 1e-10,
            initial_damping: 1e-3,
            finite_difference_step: 1e-7,
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
    /// Condition estimate of the internally column-normalized Jacobian.
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

fn rank_condition(s: &DVector<f64>, tol: f64) -> (usize, f64) {
    let max = s
        .iter()
        .copied()
        .filter(|v| v.is_finite())
        .fold(0.0, f64::max);
    if max <= 0.0 {
        return (0, f64::INFINITY);
    }
    let th = tol.max(0.0) * max.max(1.0);
    let mut rank = 0;
    let mut min = f64::INFINITY;
    for v in s.iter().copied() {
        if v.is_finite() && v > th {
            rank += 1;
            min = min.min(v)
        }
    }
    (rank, if rank == 0 { f64::INFINITY } else { max / min })
}

pub fn scaled_damped_qr(
    j: &[Vec<f64>],
    r: &[f64],
    d: f64,
    tol: f64,
) -> Result<LinearSolveReport, String> {
    let rows = j.len();
    let cols = j.first().map_or(0, Vec::len);
    if rows != r.len() {
        return Err("jacobian/residual row mismatch".into());
    }
    if j.iter().any(|x| x.len() != cols) {
        return Err("ragged jacobian".into());
    }
    if !d.is_finite() || d < 0.0 {
        return Err("invalid damping".into());
    }
    if !tol.is_finite() || tol < 0.0 {
        return Err("invalid rank tolerance".into());
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
            a[(i, k)] = j[i][k]
        }
    }
    let mut cs = vec![1.0; cols];
    for k in 0..cols {
        let n = a.column(k).norm();
        if n.is_finite() && n > 0.0 {
            cs[k] = n;
            for i in 0..rows {
                a[(i, k)] /= n
            }
        }
    }
    let svd = a.svd(true, true);
    let (rank, cond) = rank_condition(&svd.singular_values, tol);
    let u = svd.u.ok_or("SVD left vectors unavailable")?;
    let vt = svd.v_t.ok_or("SVD right vectors unavailable")?;
    if r.iter().any(|v| !v.is_finite()) {
        return Err("non-finite residual".into());
    }
    let b = DVector::from_iterator(rows, r.iter().map(|v| -v));
    let ub = u.transpose() * b;
    let mut z = DVector::<f64>::zeros(cols);
    for k in 0..svd.singular_values.len().min(cols) {
        let s = svd.singular_values[k];
        let c = s / (s * s + d) * ub[k];
        for n in 0..cols {
            z[n] += vt[(k, n)] * c
        }
    }
    let delta = z.iter().enumerate().map(|(k, v)| v / cs[k]).collect();
    Ok(LinearSolveReport {
        delta,
        rank,
        degrees_of_freedom: cols.saturating_sub(rank),
        condition_number: cond,
    })
}

fn enc(g: &Geometry) -> Vec<f64> {
    match g {
        Geometry::Line(x) => vec![x.start.x, x.start.y, x.end.x, x.end.y],
        Geometry::Circle(x) => vec![x.center.x, x.center.y, x.radius],
        Geometry::Arc(x) => vec![x.center.x, x.center.y, x.radius, x.start_angle, x.end_angle],
    }
}

fn dec(t: &Geometry, v: &[f64]) -> Geometry {
    match t {
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

fn vars(s: &SemanticSnapshot) -> (Vec<String>, Vec<usize>) {
    let mut ids = Vec::with_capacity(s.geometry.len());
    let mut o = vec![0];
    for g in &s.geometry {
        ids.push(g.id.clone());
        o.push(o.last().copied().unwrap() + enc(&g.geometry).len())
    }
    (ids, o)
}

fn candidate(
    s: &SemanticSnapshot,
    v: &[f64],
    ids: &[String],
    o: &[usize],
) -> Result<SemanticSnapshot, String> {
    let mut x = s.clone();
    for (i, g) in x.geometry.iter_mut().enumerate() {
        let t = s
            .geometry(&ids[i])
            .ok_or_else(|| format!("Unknown geometry: {}", ids[i]))?;
        g.geometry = dec(t, &v[o[i]..o[i + 1]])
    }
    Ok(x)
}

fn residuals(
    s: &SemanticSnapshot,
    v: &[f64],
    ids: &[String],
    o: &[usize],
) -> Result<
    (
        Vec<f64>,
        Vec<f64>,
        Vec<ConstraintResidualReport>,
        usize,
        bool,
    ),
    String,
> {
    let x = candidate(s, v, ids, o)?;
    let mut raw = Vec::new();
    let mut scaled = Vec::new();
    let mut reports = Vec::new();
    let mut req = 0;
    let mut rok = true;
    for (id, c) in &s.constraints {
        let rr = match c {
            Constraint::Fixed { entity_id } => {
                let a = x
                    .geometry(entity_id)
                    .ok_or_else(|| format!("Unknown geometry: {entity_id}"))?;
                let b = s
                    .geometry(entity_id)
                    .ok_or_else(|| format!("Unknown geometry: {entity_id}"))?;
                geometry_difference(a, b).ok_or("Fixed geometry type mismatch")?
            }
            _ => constraint_residual(|gid| x.geometry(gid).cloned(), c)
                .ok_or_else(|| format!("Invalid constraint domain: {id}"))?,
        };
        raw.extend(rr.iter().copied());
        scaled.extend(rr.iter().copied());
        reports.push(ConstraintResidualReport {
            norm: rr.iter().map(|v| v * v).sum::<f64>().sqrt(),
            residuals: rr,
            kind: format!("constraint:{id}"),
        });
    }
    for (i, (_, r)) in s.relations.iter().enumerate() {
        let e = evaluate_relation(&x, r)?;
        raw.extend(e.residuals.iter().copied());
        scaled.extend(e.residuals.iter().zip(e.scales.iter()).map(|(v, z)| {
            if *z > 0.0 {
                *v / *z
            } else {
                *v
            }
        }));
        req += e.residuals.len();
        rok &= e.scaled_norm <= 1e-8;
        reports.push(ConstraintResidualReport {
            norm: e.norm,
            residuals: e.residuals,
            kind: format!("relation:{}", i + 1),
        });
    }
    Ok((raw, scaled, reports, req, rok))
}

fn jac(
    s: &SemanticSnapshot,
    v: &[f64],
    ids: &[String],
    o: &[usize],
    step: f64,
    base: &[f64],
) -> Result<Vec<Vec<f64>>, String> {
    let mut j = vec![vec![0.0; v.len()]; base.len()];
    for c in 0..v.len() {
        if !v[c].is_finite() {
            return Err(format!("non-finite solver parameter at column {c}"));
        }
        let mut p = v.to_vec();
        let h = step.max(1e-12) * v[c].abs().max(1.0);
        if !h.is_finite() || h <= 0.0 {
            return Err(format!("non-finite finite-difference step at column {c}"));
        }
        p[c] += h;
        if !p[c].is_finite() {
            return Err(format!("finite-difference probe overflow at column {c}"));
        }
        let (n, _, _, _, _) = residuals(s, &p, ids, o)?;
        if n.iter().any(|x| !x.is_finite()) {
            return Err(format!("non-finite residual at finite-difference probe column {c}"));
        }
        for r in 0..base.len() {
            j[r][c] = (n[r] - base[r]) / h;
            if !j[r][c].is_finite() {
                return Err(format!("non-finite jacobian entry at row {r}, column {c}"));
            }
        }
    }
    Ok(j)
}

fn finite(v: &[f64]) -> bool {
    v.iter().all(|x| x.is_finite())
}

fn materialize(
    s: &SemanticSnapshot,
    v: &[f64],
    ids: &[String],
    o: &[usize],
) -> Result<Vec<(String, Geometry)>, String> {
    let mut out = Vec::with_capacity(ids.len());
    for (i, id) in ids.iter().enumerate() {
        let g = s
            .geometry(id)
            .ok_or_else(|| format!("Unknown geometry: {id}"))?;
        out.push((id.clone(), dec(g, &v[o[i]..o[i + 1]])))
    }
    Ok(out)
}

fn analysis(
    s: &SemanticSnapshot,
    v: &[f64],
    ids: &[String],
    o: &[usize],
    req: usize,
    rok: bool,
    rank: usize,
    cond: f64,
) -> Result<ConstraintAnalysis, String> {
    let (raw, scaled, reps, _, _) = residuals(s, v, ids, o)?;
    let rn = raw.iter().map(|x| x * x).sum::<f64>().sqrt();
    let sn = scaled.iter().map(|x| x * x).sum::<f64>().sqrt();
    Ok(ConstraintAnalysis {
        residuals: reps,
        residual_norm: rn,
        scaled_residual_norm: sn,
        max_residual: raw.iter().map(|x| x.abs()).fold(0.0, f64::max),
        satisfied: sn <= 1e-8 && rok,
        relation_satisfied: rok,
        valid: raw.iter().all(|x| x.is_finite()),
        degrees_of_freedom: v.len().saturating_sub(rank),
        variable_count: v.len(),
        equation_count: raw.len(),
        rank,
        condition_estimate: cond,
        well_conditioned: cond.is_finite() && cond < 1e10,
        relation_count: s.relations.len(),
        relation_equation_count: req,
    })
}

pub fn solve_snapshot(
    s: &SemanticSnapshot,
    o: SolveOptions,
) -> Result<ConstraintSolveResult, String> {
    if o.max_iterations == 0
        || !o.residual_tolerance.is_finite()
        || o.residual_tolerance < 0.0
        || !o.step_tolerance.is_finite()
        || o.step_tolerance < 0.0
        || !o.initial_damping.is_finite()
        || o.initial_damping < 0.0
        || !o.finite_difference_step.is_finite()
        || o.finite_difference_step <= 0.0
    {
        return Err("Invalid solver options".into());
    }
    let (ids, ofs) = vars(s);
    let mut v = Vec::new();
    for g in &s.geometry {
        v.extend(enc(&g.geometry))
    }
    let (ir, is, _, ireq, irok) = residuals(s, &v, &ids, &ofs)?;
    let irn = ir.iter().map(|x| x * x).sum::<f64>().sqrt();
    let isn = is.iter().map(|x| x * x).sum::<f64>().sqrt();
    if isn <= o.residual_tolerance {
        let a = analysis(s, &v, &ids, &ofs, ireq, irok, 0, 1.0)?;
        return Ok(ConstraintSolveResult {
            converged: a.satisfied,
            reason: if a.satisfied {
                SolveReason::Converged
            } else {
                SolveReason::MaxIterations
            },
            iterations: 0,
            initial_residual_norm: irn,
            final_residual_norm: irn,
            initial_scaled_residual_norm: isn,
            final_scaled_residual_norm: isn,
            analysis: a,
            geometry: materialize(s, &v, &ids, &ofs)?,
        });
    }
    let mut damp = o.initial_damping;
    let mut rank = 0;
    let mut cond = f64::INFINITY;
    for it in 1..=o.max_iterations {
        let (br, _, _, req, rok) = residuals(s, &v, &ids, &ofs)?;
        let j = jac(s, &v, &ids, &ofs, o.finite_difference_step, &br)?;
        let lin = match scaled_damped_qr(&j, &br, damp, 1e-10) {
            Ok(x) => x,
            Err(_) => {
                let a = analysis(s, &v, &ids, &ofs, req, rok, rank, cond)?;
                return Ok(ConstraintSolveResult {
                    converged: false,
                    reason: SolveReason::Singular,
                    iterations: it,
                    initial_residual_norm: irn,
                    final_residual_norm: a.residual_norm,
                    initial_scaled_residual_norm: isn,
                    final_scaled_residual_norm: a.scaled_residual_norm,
                    analysis: a,
                    geometry: materialize(s, &v, &ids, &ofs)?,
                });
            }
        };
        rank = lin.rank;
        cond = lin.condition_number;
        let mut p = v.clone();
        for (x, d) in p.iter_mut().zip(lin.delta.iter()) {
            *x += *d
        }
        if !finite(&p) {
            damp = (damp * 10.0).max(1e-12);
            continue;
        }
        let (nr, ns, _, nreq, nrok) = residuals(s, &p, &ids, &ofs)?;
        let cur = residuals(s, &v, &ids, &ofs)?.1;
        let cn = cur.iter().map(|x| x * x).sum::<f64>().sqrt();
        let nn = ns.iter().map(|x| x * x).sum::<f64>().sqrt();
        if nn < cn {
            v = p;
            damp = (damp * 0.3).max(1e-12);
            if nn <= o.residual_tolerance {
                let a = analysis(s, &v, &ids, &ofs, nreq, nrok, rank, cond)?;
                return Ok(ConstraintSolveResult {
                    converged: a.satisfied,
                    reason: if a.satisfied {
                        SolveReason::Converged
                    } else {
                        SolveReason::MaxIterations
                    },
                    iterations: it,
                    initial_residual_norm: irn,
                    final_residual_norm: nr.iter().map(|x| x * x).sum::<f64>().sqrt(),
                    initial_scaled_residual_norm: isn,
                    final_scaled_residual_norm: nn,
                    analysis: a,
                    geometry: materialize(s, &v, &ids, &ofs)?,
                });
            }
        } else {
            damp = (damp * 10.0).min(1e12)
        }
    }
    let fr = residuals(s, &v, &ids, &ofs)?;
    let a = analysis(s, &v, &ids, &ofs, fr.3, fr.4, rank, cond)?;
    Ok(ConstraintSolveResult {
        converged: false,
        reason: SolveReason::MaxIterations,
        iterations: o.max_iterations,
        initial_residual_norm: irn,
        final_residual_norm: fr.0.iter().map(|x| x * x).sum::<f64>().sqrt(),
        initial_scaled_residual_norm: isn,
        final_scaled_residual_norm: fr.1.iter().map(|x| x * x).sum::<f64>().sqrt(),
        analysis: a,
        geometry: materialize(s, &v, &ids, &ofs)?,
    })
}