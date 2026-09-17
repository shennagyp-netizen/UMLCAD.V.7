//! Analytic Jacobians for the current geometric relation equations.
//!
//! These rows are derived directly from the relation residual definitions in
//! [`super::relations`]. Finite differences may be used as an independent test
//! oracle, but never as the mathematical implementation authority.

use super::{
    constraints::endpoint,
    geometry::{Arc, Circle, Geometry, Line, Point},
    snapshot::{Endpoint, Relation, RelationPoint, SemanticSnapshot, TangentMode},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RelationJacobianError {
    UnknownGeometry,
    UnsupportedRelation,
    InvalidDomain,
    NonFinite,
    Indeterminate,
}

fn width(g: &Geometry) -> usize {
    match g {
        Geometry::Line(_) => 4,
        Geometry::Circle(_) => 3,
        Geometry::Arc(_) => 5,
    }
}

fn layout(snapshot: &SemanticSnapshot) -> (Vec<String>, Vec<usize>) {
    let mut ids = Vec::with_capacity(snapshot.geometry.len());
    let mut offsets = vec![0usize];
    for item in &snapshot.geometry {
        ids.push(item.id.clone());
        offsets.push(offsets.last().copied().unwrap_or(0) + width(&item.geometry));
    }
    (ids, offsets)
}

fn find(ids: &[String], id: &str) -> Result<usize, RelationJacobianError> {
    ids.iter()
        .position(|value| value == id)
        .ok_or(RelationJacobianError::UnknownGeometry)
}

fn geometry<'a>(snapshot: &'a SemanticSnapshot, id: &str) -> Result<&'a Geometry, RelationJacobianError> {
    snapshot.geometry(id).ok_or(RelationJacobianError::UnknownGeometry)
}

fn finite(values: &[f64]) -> bool {
    values.iter().all(|value| value.is_finite())
}

fn add_block(row: &mut [f64], offset: usize, values: &[f64]) {
    for (index, value) in values.iter().copied().enumerate() {
        row[offset + index] += value;
    }
}

fn endpoint_gradients(
    g: &Geometry,
    endpoint_kind: Endpoint,
) -> Result<(Vec<f64>, Vec<f64>, Point), RelationJacobianError> {
    match (g, endpoint_kind) {
        (Geometry::Line(Line { start, .. }), Endpoint::Start) => Ok((
            vec![1.0, 0.0, 0.0, 0.0],
            vec![0.0, 1.0, 0.0, 0.0],
            *start,
        )),
        (Geometry::Line(Line { end, .. }), Endpoint::End) => Ok((
            vec![0.0, 0.0, 1.0, 0.0],
            vec![0.0, 0.0, 0.0, 1.0],
            *end,
        )),
        (Geometry::Arc(a), Endpoint::Start) => Ok((
            vec![1.0, 0.0, a.start_angle.cos(), -a.radius * a.start_angle.sin(), 0.0],
            vec![0.0, 1.0, a.start_angle.sin(), a.radius * a.start_angle.cos(), 0.0],
            endpoint(g, Endpoint::Start).ok_or(RelationJacobianError::InvalidDomain)?,
        )),
        (Geometry::Arc(a), Endpoint::End) => Ok((
            vec![1.0, 0.0, a.end_angle.cos(), 0.0, -a.radius * a.end_angle.sin()],
            vec![0.0, 1.0, a.end_angle.sin(), 0.0, a.radius * a.end_angle.cos()],
            endpoint(g, Endpoint::End).ok_or(RelationJacobianError::InvalidDomain)?,
        )),
        (Geometry::Circle(_), _) => Err(RelationJacobianError::InvalidDomain),
    }
}

fn relation_point(
    snapshot: &SemanticSnapshot,
    point: &RelationPoint,
) -> Result<(usize, Vec<f64>, Vec<f64>, Point), RelationJacobianError> {
    let (id, endpoint_kind) = match point {
        RelationPoint::Endpoint { geometry_id, point } => (geometry_id.as_str(), Some(*point)),
        RelationPoint::Center { geometry_id } => (geometry_id.as_str(), None),
    };
    let (ids, offsets) = layout(snapshot);
    let index = find(&ids, id)?;
    let g = geometry(snapshot, id)?;
    let mut gx = vec![0.0; width(g)];
    let mut gy = vec![0.0; width(g)];
    let p = if let Some(endpoint_kind) = endpoint_kind {
        let (local_x, local_y, value) = endpoint_gradients(g, endpoint_kind)?;
        gx = local_x;
        gy = local_y;
        value
    } else {
        match g {
            Geometry::Circle(c) => {
                gx[0] = 1.0;
                gy[1] = 1.0;
                c.center
            }
            Geometry::Arc(a) => {
                gx[0] = 1.0;
                gy[1] = 1.0;
                a.center
            }
            Geometry::Line(_) => return Err(RelationJacobianError::InvalidDomain),
        }
    };
    let _ = offsets;
    Ok((index, gx, gy, p))
}

fn line_vector(g: &Geometry) -> Result<Point, RelationJacobianError> {
    let Geometry::Line(line) = g else {
        return Err(RelationJacobianError::InvalidDomain);
    };
    Ok(line.end.sub(line.start))
}

fn angle_gradient(v: Point) -> Result<(f64, f64), RelationJacobianError> {
    let denom = v.x * v.x + v.y * v.y;
    if !denom.is_finite() || denom == 0.0 {
        return Err(RelationJacobianError::Indeterminate);
    }
    Ok((-v.y / denom, v.x / denom))
}

fn length_gradient(g: &Geometry) -> Result<Vec<f64>, RelationJacobianError> {
    match g {
        Geometry::Line(line) => {
            let d = line.end.sub(line.start);
            let l = d.length();
            if !l.is_finite() || l == 0.0 {
                return Err(RelationJacobianError::Indeterminate);
            }
            Ok(vec![-d.x / l, -d.y / l, d.x / l, d.y / l])
        }
        Geometry::Circle(_) => Ok(vec![0.0, 0.0, 2.0 * std::f64::consts::PI]),
        Geometry::Arc(arc) => {
            let delta = arc.end_angle - arc.start_angle;
            if !delta.is_finite() || delta == 0.0 {
                return Err(RelationJacobianError::Indeterminate);
            }
            let sign = delta.signum();
            Ok(vec![0.0, 0.0, delta.abs(), -arc.radius * sign, arc.radius * sign])
        }
    }
}

fn radius_gradient(g: &Geometry) -> Result<Vec<f64>, RelationJacobianError> {
    match g {
        Geometry::Circle(_) | Geometry::Arc(_) => {
            let mut row = vec![0.0; width(g)];
            row[2] = 1.0;
            Ok(row)
        }
        Geometry::Line(_) => Err(RelationJacobianError::InvalidDomain),
    }
}

fn line_point_distance_gradient(
    point: Point,
    line: &Line,
) -> Result<(f64, Vec<f64>, Vec<f64>), RelationJacobianError> {
    let dx = line.end.x - line.start.x;
    let dy = line.end.y - line.start.y;
    let length = dx.hypot(dy);
    if !length.is_finite() || length == 0.0 {
        return Err(RelationJacobianError::Indeterminate);
    }
    let qx = point.x - line.start.x;
    let qy = point.y - line.start.y;
    let cross = qx * dy - qy * dx;
    if !cross.is_finite() {
        return Err(RelationJacobianError::NonFinite);
    }
    if cross == 0.0 {
        return Err(RelationJacobianError::Indeterminate);
    }
    let sign = cross.signum();
    let abs_cross = cross.abs();
    let inv_length = 1.0 / length;
    let inv_length3 = inv_length / (length * length);
    let residual = abs_cross * inv_length;

    let q_grad = [dy, -dx];
    let mut point_gradient = vec![sign * q_grad[0] * inv_length, sign * q_grad[1] * inv_length];
    if point_gradient.iter().any(|value| !value.is_finite()) {
        return Err(RelationJacobianError::NonFinite);
    }

    let start_dq_x = qy - dy;
    let start_dq_y = dx - qx;
    let end_dq_x = -qy;
    let end_dq_y = qx;
    let dlength = [-dx * inv_length, -dy * inv_length, dx * inv_length, dy * inv_length];
    let dq = [start_dq_x, start_dq_y, end_dq_x, end_dq_y];
    let mut line_gradient = vec![0.0; 4];
    for index in 0..4 {
        line_gradient[index] = sign * dq[index] * inv_length
            - abs_cross * dlength[index] * inv_length * inv_length;
    }
    let _ = &mut point_gradient;
    if !finite(&line_gradient) || !finite(&point_gradient) || !residual.is_finite() {
        return Err(RelationJacobianError::NonFinite);
    }
    Ok((residual, line_gradient, point_gradient))
}

fn circle_tangent_gradient(
    ca: Point,
    ra: f64,
    cb: Point,
    rb: f64,
    mode: TangentMode,
) -> Result<(f64, Vec<f64>, Vec<f64>), RelationJacobianError> {
    let dx = ca.x - cb.x;
    let dy = ca.y - cb.y;
    let d = dx.hypot(dy);
    if !d.is_finite() || d == 0.0 {
        return Err(RelationJacobianError::Indeterminate);
    }
    let external = d - ra - rb;
    let radius_delta = ra - rb;
    let internal = d - radius_delta.abs();
    let (value, dra, drb) = match mode {
        TangentMode::External => (external, -1.0, -1.0),
        TangentMode::Internal => {
            if radius_delta == 0.0 {
                return Err(RelationJacobianError::Indeterminate);
            }
            let sign = radius_delta.signum();
            (internal, -sign, sign)
        }
        TangentMode::Any => {
            if external.abs() == internal.abs() {
                return Err(RelationJacobianError::Indeterminate);
            }
            if external.abs() < internal.abs() {
                (external, -1.0, -1.0)
            } else {
                if radius_delta == 0.0 {
                    return Err(RelationJacobianError::Indeterminate);
                }
                let sign = radius_delta.signum();
                (internal, -sign, sign)
            }
        }
    };
    let ux = dx / d;
    let uy = dy / d;
    Ok((value, vec![ux, uy, dra], vec![-ux, -uy, drb]))
}

pub fn analytic_relation_jacobian(
    snapshot: &SemanticSnapshot,
) -> Result<Vec<Vec<f64>>, RelationJacobianError> {
    let (ids, offsets) = layout(snapshot);
    let total = offsets.last().copied().unwrap_or(0);
    let mut rows = Vec::new();

    for (_, relation) in &snapshot.relations {
        match relation {
            Relation::Parallel { first_geometry_id, second_geometry_id } => {
                let first = find(&ids, first_geometry_id)?;
                let second = find(&ids, second_geometry_id)?;
                let a = line_vector(geometry(snapshot, first_geometry_id)?)?;
                let b = line_vector(geometry(snapshot, second_geometry_id)?)?;
                let mut row = vec![0.0; total];
                add_block(&mut row, offsets[first], &[b.y, -b.x, -b.y, b.x]);
                add_block(&mut row, offsets[second], &[-a.y, a.x, a.y, -a.x]);
                rows.push(row);
            }
            Relation::Perpendicular { first_geometry_id, second_geometry_id } => {
                let first = find(&ids, first_geometry_id)?;
                let second = find(&ids, second_geometry_id)?;
                let a = line_vector(geometry(snapshot, first_geometry_id)?)?;
                let b = line_vector(geometry(snapshot, second_geometry_id)?)?;
                let mut row = vec![0.0; total];
                add_block(&mut row, offsets[first], &[-b.x, -b.y, b.x, b.y]);
                add_block(&mut row, offsets[second], &[-a.x, -a.y, a.x, a.y]);
                rows.push(row);
            }
            Relation::EqualLength { first_geometry_id, second_geometry_id } => {
                let first = find(&ids, first_geometry_id)?;
                let second = find(&ids, second_geometry_id)?;
                let ga = length_gradient(geometry(snapshot, first_geometry_id)?)?;
                let gb = length_gradient(geometry(snapshot, second_geometry_id)?)?;
                let mut row = vec![0.0; total];
                add_block(&mut row, offsets[first], &ga);
                add_block(&mut row, offsets[second], &gb.iter().map(|value| -*value).collect::<Vec<_>>());
                rows.push(row);
            }
            Relation::Angle { first_geometry_id, second_geometry_id, .. } => {
                let first = find(&ids, first_geometry_id)?;
                let second = find(&ids, second_geometry_id)?;
                let a = line_vector(geometry(snapshot, first_geometry_id)?)?;
                let b = line_vector(geometry(snapshot, second_geometry_id)?)?;
                let (a_tx, a_ty) = angle_gradient(a)?;
                let (b_tx, b_ty) = angle_gradient(b)?;
                let mut row = vec![0.0; total];
                // d(theta_b - theta_a): first-line vector derivatives contribute
                // -dtheta/da; second-line vector derivatives contribute +dtheta/db.
                add_block(&mut row, offsets[first], &[-a_tx, -a_ty, a_tx, a_ty]);
                add_block(&mut row, offsets[second], &[-b_tx, -b_ty, b_tx, b_ty]);
                rows.push(row);
            }
            Relation::Collinear { first_geometry_id, second_geometry_id } => {
                let first = find(&ids, first_geometry_id)?;
                let second = find(&ids, second_geometry_id)?;
                let Geometry::Line(a_line) = geometry(snapshot, first_geometry_id)? else {
                    return Err(RelationJacobianError::InvalidDomain);
                };
                let Geometry::Line(b_line) = geometry(snapshot, second_geometry_id)? else {
                    return Err(RelationJacobianError::InvalidDomain);
                };
                let d = a_line.end.sub(a_line.start);
                for (b_index, b_point) in [(0usize, b_line.start), (1usize, b_line.end)] {
                    let q = b_point.sub(a_line.start);
                    let mut row = vec![0.0; total];
                    add_block(&mut row, offsets[first], &[d.y - q.y, q.x - d.x, q.y, -q.x]);
                    let block = if b_index == 0 {
                        vec![-d.y, d.x, 0.0, 0.0]
                    } else {
                        vec![0.0, 0.0, -d.y, d.x]
                    };
                    add_block(&mut row, offsets[second], &block);
                    rows.push(row);
                }
            }
            Relation::Concentric { first_geometry_id, second_geometry_id } => {
                let first = find(&ids, first_geometry_id)?;
                let second = find(&ids, second_geometry_id)?;
                let a = geometry(snapshot, first_geometry_id)?;
                let b = geometry(snapshot, second_geometry_id)?;
                if !matches!(a, Geometry::Circle(_) | Geometry::Arc(_)) || !matches!(b, Geometry::Circle(_) | Geometry::Arc(_)) {
                    return Err(RelationJacobianError::InvalidDomain);
                }
                let mut row_x = vec![0.0; total];
                let mut row_y = vec![0.0; total];
                let mut ax = vec![0.0; width(a)];
                let mut ay = vec![0.0; width(a)];
                let mut bx = vec![0.0; width(b)];
                let mut by = vec![0.0; width(b)];
                ax[0] = 1.0; ay[1] = 1.0; bx[0] = -1.0; by[1] = -1.0;
                add_block(&mut row_x, offsets[first], &ax);
                add_block(&mut row_x, offsets[second], &bx);
                add_block(&mut row_y, offsets[first], &ay);
                add_block(&mut row_y, offsets[second], &by);
                rows.push(row_x);
                rows.push(row_y);
            }
            Relation::EqualRadius { first_geometry_id, second_geometry_id } => {
                let first = find(&ids, first_geometry_id)?;
                let second = find(&ids, second_geometry_id)?;
                let ga = radius_gradient(geometry(snapshot, first_geometry_id)?)?;
                let gb = radius_gradient(geometry(snapshot, second_geometry_id)?)?;
                let mut row = vec![0.0; total];
                add_block(&mut row, offsets[first], &ga);
                add_block(&mut row, offsets[second], &gb.iter().map(|value| -*value).collect::<Vec<_>>());
                rows.push(row);
            }
            Relation::Radius { geometry_id, .. } => {
                let index = find(&ids, geometry_id)?;
                let mut row = vec![0.0; total];
                add_block(&mut row, offsets[index], &radius_gradient(geometry(snapshot, geometry_id)?)?);
                rows.push(row);
            }
            Relation::Diameter { geometry_id, .. } => {
                let index = find(&ids, geometry_id)?;
                let mut gradient = radius_gradient(geometry(snapshot, geometry_id)?)?;
                gradient.iter_mut().for_each(|value| *value *= 2.0);
                let mut row = vec![0.0; total];
                add_block(&mut row, offsets[index], &gradient);
                rows.push(row);
            }
            Relation::Tangent { first_geometry_id, second_geometry_id, mode } => {
                let first = find(&ids, first_geometry_id)?;
                let second = find(&ids, second_geometry_id)?;
                let a = geometry(snapshot, first_geometry_id)?;
                let b = geometry(snapshot, second_geometry_id)?;
                let mut row = vec![0.0; total];
                match (a, b) {
                    (Geometry::Line(line), Geometry::Circle(circle)) | (Geometry::Line(line), Geometry::Arc(Arc { center: circle_center, radius: circle_radius, .. })) => {
                        let center = match b { Geometry::Circle(c) => c.center, Geometry::Arc(a) => a.center, _ => unreachable!() };
                        let (_, line_gradient, point_gradient) = line_point_distance_gradient(center, line)?;
                        add_block(&mut row, offsets[first], &line_gradient);
                        let mut second_gradient = vec![0.0; width(b)];
                        second_gradient[0] = point_gradient[0];
                        second_gradient[1] = point_gradient[1];
                        second_gradient[2] = -1.0;
                        add_block(&mut row, offsets[second], &second_gradient);
                        let _ = (circle, circle_center, circle_radius);
                    }
                    (Geometry::Circle(circle), Geometry::Line(line)) | (Geometry::Arc(Arc { center: circle_center, radius: circle_radius, .. }), Geometry::Line(line)) => {
                        let center = match a { Geometry::Circle(c) => c.center, Geometry::Arc(a) => a.center, _ => unreachable!() };
                        let radius = match a { Geometry::Circle(c) => c.radius, Geometry::Arc(a) => a.radius, _ => unreachable!() };
                        let (_, line_gradient, point_gradient) = line_point_distance_gradient(center, line)?;
                        let mut first_gradient = vec![0.0; width(a)];
                        first_gradient[0] = point_gradient[0];
                        first_gradient[1] = point_gradient[1];
                        first_gradient[2] = -1.0;
                        add_block(&mut row, offsets[first], &first_gradient);
                        add_block(&mut row, offsets[second], &line_gradient);
                        let _ = radius;
                    }
                    (circle_a, circle_b)
                        if matches!(circle_a, Geometry::Circle(_) | Geometry::Arc(_))
                            && matches!(circle_b, Geometry::Circle(_) | Geometry::Arc(_)) =>
                    {
                        let (ca, ra) = match circle_a {
                            Geometry::Circle(c) => (c.center, c.radius),
                            Geometry::Arc(c) => (c.center, c.radius),
                            Geometry::Line(_) => unreachable!(),
                        };
                        let (cb, rb) = match circle_b {
                            Geometry::Circle(c) => (c.center, c.radius),
                            Geometry::Arc(c) => (c.center, c.radius),
                            Geometry::Line(_) => unreachable!(),
                        };
                        let (_, ga, gb) = circle_tangent_gradient(ca, ra, cb, rb, *mode)?;
                        let mut va = vec![0.0; width(a)];
                        let mut vb = vec![0.0; width(b)];
                        va[0] = ga[0]; va[1] = ga[1]; va[2] = ga[2];
                        vb[0] = gb[0]; vb[1] = gb[1]; vb[2] = gb[2];
                        add_block(&mut row, offsets[first], &va);
                        add_block(&mut row, offsets[second], &vb);
                    }
                    _ => return Err(RelationJacobianError::InvalidDomain),
                }
                rows.push(row);
            }
            Relation::Midpoint { point, line_geometry_id } => {
                let (point_index, px, py, _) = relation_point(snapshot, point)?;
                let line_index = find(&ids, line_geometry_id)?;
                if !matches!(geometry(snapshot, line_geometry_id)?, Geometry::Line(_)) {
                    return Err(RelationJacobianError::InvalidDomain);
                }
                let mut row_x = vec![0.0; total];
                let mut row_y = vec![0.0; total];
                add_block(&mut row_x, offsets[point_index], &px);
                add_block(&mut row_y, offsets[point_index], &py);
                add_block(&mut row_x, offsets[line_index], &[-0.5, 0.0, -0.5, 0.0]);
                add_block(&mut row_y, offsets[line_index], &[0.0, -0.5, 0.0, -0.5]);
                rows.push(row_x);
                rows.push(row_y);
            }
            Relation::PointOnLine { point, line_geometry_id } => {
                let (point_index, px, py, point_value) = relation_point(snapshot, point)?;
                let line_index = find(&ids, line_geometry_id)?;
                let Geometry::Line(line) = geometry(snapshot, line_geometry_id)? else {
                    return Err(RelationJacobianError::InvalidDomain);
                };
                let (_, line_gradient, point_gradient) = line_point_distance_gradient(point_value, line)?;
                let mut row = vec![0.0; total];
                add_block(&mut row, offsets[line_index], &line_gradient);
                let mut point_local = vec![0.0; width(geometry(snapshot, match point { RelationPoint::Endpoint { geometry_id, .. } | RelationPoint::Center { geometry_id } => geometry_id })?)];
                for i in 0..point_local.len() {
                    point_local[i] = point_gradient[0] * px[i] + point_gradient[1] * py[i];
                }
                add_block(&mut row, offsets[point_index], &point_local);
                rows.push(row);
            }
            Relation::PointOnCircle { point, circle_geometry_id } => {
                let (point_index, px, py, point_value) = relation_point(snapshot, point)?;
                let circle_index = find(&ids, circle_geometry_id)?;
                let g = geometry(snapshot, circle_geometry_id)?;
                let (center, radius) = match g {
                    Geometry::Circle(c) => (c.center, c.radius),
                    Geometry::Arc(a) => (a.center, a.radius),
                    Geometry::Line(_) => return Err(RelationJacobianError::InvalidDomain),
                };
                let dx = point_value.x - center.x;
                let dy = point_value.y - center.y;
                let d = dx.hypot(dy);
                if !d.is_finite() || d == 0.0 || !radius.is_finite() {
                    return Err(RelationJacobianError::Indeterminate);
                }
                let ux = dx / d;
                let uy = dy / d;
                let mut row = vec![0.0; total];
                let mut point_local = vec![0.0; px.len()];
                for i in 0..point_local.len() {
                    point_local[i] = ux * px[i] + uy * py[i];
                }
                add_block(&mut row, offsets[point_index], &point_local);
                let mut circle_local = vec![0.0; width(g)];
                circle_local[0] = -ux;
                circle_local[1] = -uy;
                circle_local[2] = -1.0;
                add_block(&mut row, offsets[circle_index], &circle_local);
                rows.push(row);
            }
            Relation::DistancePoints { first, second, .. } => {
                let (first_index, first_px, first_py, first_value) = relation_point(snapshot, first)?;
                let (second_index, second_px, second_py, second_value) = relation_point(snapshot, second)?;
                let dx = first_value.x - second_value.x;
                let dy = first_value.y - second_value.y;
                let d = dx.hypot(dy);
                if !d.is_finite() || d == 0.0 {
                    return Err(RelationJacobianError::Indeterminate);
                }
                let ux = dx / d;
                let uy = dy / d;
                let mut row = vec![0.0; total];
                let first_geo = match first { RelationPoint::Endpoint { geometry_id, .. } | RelationPoint::Center { geometry_id } => geometry_id };
                let second_geo = match second { RelationPoint::Endpoint { geometry_id, .. } | RelationPoint::Center { geometry_id } => geometry_id };
                let mut local = vec![0.0; width(geometry(snapshot, first_geo)?)];
                for i in 0..local.len() { local[i] = ux * first_px[i] + uy * first_py[i]; }
                add_block(&mut row, offsets[first_index], &local);
                let mut local_second = vec![0.0; width(geometry(snapshot, second_geo)?)];
                for i in 0..local_second.len() { local_second[i] = -(ux * second_px[i] + uy * second_py[i]); }
                add_block(&mut row, offsets[second_index], &local_second);
                rows.push(row);
            }
            Relation::Symmetric { first, second, about } => {
                let (first_index, first_px, first_py, _) = relation_point(snapshot, first)?;
                let (second_index, second_px, second_py, _) = relation_point(snapshot, second)?;
                let (about_index, about_px, about_py, _) = relation_point(snapshot, about)?;
                let mut row_x = vec![0.0; total];
                let mut row_y = vec![0.0; total];
                add_block(&mut row_x, offsets[first_index], &first_px.iter().map(|value| 0.5 * *value).collect::<Vec<_>>());
                add_block(&mut row_x, offsets[second_index], &second_px.iter().map(|value| 0.5 * *value).collect::<Vec<_>>());
                add_block(&mut row_x, offsets[about_index], &about_px.iter().map(|value| -*value).collect::<Vec<_>>());
                add_block(&mut row_y, offsets[first_index], &first_py.iter().map(|value| 0.5 * *value).collect::<Vec<_>>());
                add_block(&mut row_y, offsets[second_index], &second_py.iter().map(|value| 0.5 * *value).collect::<Vec<_>>());
                add_block(&mut row_y, offsets[about_index], &about_py.iter().map(|value| -*value).collect::<Vec<_>>());
                rows.push(row_x);
                rows.push(row_y);
            }
        }
    }

    if rows.iter().all(|row| row.len() == total && finite(row)) {
        Ok(rows)
    } else {
        Err(RelationJacobianError::NonFinite)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::snapshot::{GeometryItem, Relation, RelationPoint, SemanticSnapshot};

    fn snapshot(geometry: Vec<GeometryItem>, relations: Vec<Relation>) -> SemanticSnapshot {
        SemanticSnapshot {
            parameters: vec![],
            geometry,
            constraints: vec![],
            relations: relations.into_iter().enumerate().map(|(i, relation)| (format!("r{i}"), relation)).collect(),
        }
    }

    fn line(id: &str, sx: f64, sy: f64, ex: f64, ey: f64) -> GeometryItem {
        GeometryItem { id: id.into(), geometry: Geometry::Line(Line { start: Point { x: sx, y: sy }, end: Point { x: ex, y: ey } }), parameter_dependencies: vec![] }
    }

    fn circle(id: &str, x: f64, y: f64, r: f64) -> GeometryItem {
        GeometryItem { id: id.into(), geometry: Geometry::Circle(Circle { center: Point { x, y }, radius: r }), parameter_dependencies: vec![] }
    }

    #[test]
    fn analytic_relation_jacobian_covers_core_relations() {
        let snapshot = snapshot(
            vec![line("a", 0.0, 0.0, 3.0, 0.0), line("b", 0.0, 1.0, 3.0, 1.0), circle("c", 10.0, 0.0, 2.0)],
            vec![
                Relation::Parallel { first_geometry_id: "a".into(), second_geometry_id: "b".into() },
                Relation::Perpendicular { first_geometry_id: "a".into(), second_geometry_id: "b".into() },
                Relation::EqualLength { first_geometry_id: "a".into(), second_geometry_id: "b".into() },
                Relation::Angle { first_geometry_id: "a".into(), second_geometry_id: "b".into(), radians: 0.0 },
                Relation::Collinear { first_geometry_id: "a".into(), second_geometry_id: "b".into() },
                Relation::Midpoint { point: RelationPoint::Endpoint { geometry_id: "a".into(), point: Endpoint::Start }, line_geometry_id: "b".into() },
                Relation::PointOnCircle { point: RelationPoint::Endpoint { geometry_id: "a".into(), point: Endpoint::End }, circle_geometry_id: "c".into() },
                Relation::DistancePoints { first: RelationPoint::Endpoint { geometry_id: "a".into(), point: Endpoint::Start }, second: RelationPoint::Center { geometry_id: "c".into() }, value: 10.0 },
            ],
        );
        let jacobian = analytic_relation_jacobian(&snapshot).unwrap();
        assert_eq!(jacobian.len(), 10);
        assert!(jacobian.iter().all(|row| row.iter().all(|value| value.is_finite())));
    }

    #[test]
    fn tangent_line_circle_has_finite_gradient() {
        let snapshot = snapshot(
            vec![line("l", 0.0, 0.0, 10.0, 0.0), circle("c", 5.0, 2.0, 2.0)],
            vec![Relation::Tangent { first_geometry_id: "l".into(), second_geometry_id: "c".into(), mode: TangentMode::External }],
        );
        let jacobian = analytic_relation_jacobian(&snapshot).unwrap();
        assert_eq!(jacobian.len(), 1);
        assert!(jacobian[0].iter().all(|value| value.is_finite()));
    }

    #[test]
    fn point_on_line_fails_closed_at_nondifferentiable_zero_distance() {
        let snapshot = snapshot(
            vec![line("a", 0.0, 0.0, 1.0, 0.0), line("b", 0.0, 1.0, 1.0, 1.0)],
            vec![Relation::PointOnLine { point: RelationPoint::Endpoint { geometry_id: "a".into(), point: Endpoint::Start }, line_geometry_id: "b".into() }],
        );
        assert_eq!(analytic_relation_jacobian(&snapshot), Err(RelationJacobianError::Indeterminate));
    }
}
