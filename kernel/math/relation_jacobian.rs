//! Analytic Jacobians for the current geometric relation equations.
//!
//! These derivatives are authoritative for the relation residuals defined in
//! `relations.rs`. Numerical finite differences belong only in independent
//! tests and are never used to define the production Jacobian.

use super::{
    constraints::endpoint,
    geometry::{Geometry, Line, Point},
    relations::evaluate_relation,
    snapshot::{Endpoint, Relation, RelationPoint, SemanticSnapshot, TangentMode},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RelationJacobianError {
    UnknownGeometry,
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

fn index(ids: &[String], id: &str) -> Result<usize, RelationJacobianError> {
    ids.iter()
        .position(|value| value == id)
        .ok_or(RelationJacobianError::UnknownGeometry)
}

fn geometry<'a>(snapshot: &'a SemanticSnapshot, id: &str) -> Result<&'a Geometry, RelationJacobianError> {
    snapshot.geometry(id).ok_or(RelationJacobianError::UnknownGeometry)
}

fn add(row: &mut [f64], offset: usize, local: &[f64]) {
    for (i, value) in local.iter().copied().enumerate() {
        row[offset + i] += value;
    }
}

fn finite(row: &[f64]) -> bool {
    row.iter().all(|value| value.is_finite())
}

fn endpoint_data(
    g: &Geometry,
    e: Endpoint,
) -> Result<(Point, Vec<f64>, Vec<f64>), RelationJacobianError> {
    match (g, e) {
        (Geometry::Line(line), Endpoint::Start) => Ok((
            line.start,
            vec![1.0, 0.0, 0.0, 0.0],
            vec![0.0, 1.0, 0.0, 0.0],
        )),
        (Geometry::Line(line), Endpoint::End) => Ok((
            line.end,
            vec![0.0, 0.0, 1.0, 0.0],
            vec![0.0, 0.0, 0.0, 1.0],
        )),
        (Geometry::Arc(arc), Endpoint::Start) => Ok((
            endpoint(g, Endpoint::Start).ok_or(RelationJacobianError::InvalidDomain)?,
            vec![1.0, 0.0, arc.start_angle.cos(), -arc.radius * arc.start_angle.sin(), 0.0],
            vec![0.0, 1.0, arc.start_angle.sin(), arc.radius * arc.start_angle.cos(), 0.0],
        )),
        (Geometry::Arc(arc), Endpoint::End) => Ok((
            endpoint(g, Endpoint::End).ok_or(RelationJacobianError::InvalidDomain)?,
            vec![1.0, 0.0, arc.end_angle.cos(), 0.0, -arc.radius * arc.end_angle.sin()],
            vec![0.0, 1.0, arc.end_angle.sin(), 0.0, arc.radius * arc.end_angle.cos()],
        )),
        (Geometry::Circle(_), _) | (Geometry::Line(_), _) => Err(RelationJacobianError::InvalidDomain),
    }
}

fn relation_point(
    snapshot: &SemanticSnapshot,
    point: &RelationPoint,
) -> Result<(usize, Point, Vec<f64>, Vec<f64>), RelationJacobianError> {
    let (id, endpoint_kind) = match point {
        RelationPoint::Endpoint { geometry_id, point } => (geometry_id.as_str(), Some(*point)),
        RelationPoint::Center { geometry_id } => (geometry_id.as_str(), None),
    };
    let (ids, _) = layout(snapshot);
    let index = index(&ids, id)?;
    let g = geometry(snapshot, id)?;
    if let Some(endpoint_kind) = endpoint_kind {
        let (point, gx, gy) = endpoint_data(g, endpoint_kind)?;
        return Ok((index, point, gx, gy));
    }
    match g {
        Geometry::Circle(circle) => {
            let mut gx = vec![0.0; 3];
            let mut gy = vec![0.0; 3];
            gx[0] = 1.0;
            gy[1] = 1.0;
            Ok((index, circle.center, gx, gy))
        }
        Geometry::Arc(arc) => {
            let mut gx = vec![0.0; 5];
            let mut gy = vec![0.0; 5];
            gx[0] = 1.0;
            gy[1] = 1.0;
            Ok((index, arc.center, gx, gy))
        }
        Geometry::Line(_) => Err(RelationJacobianError::InvalidDomain),
    }
}

fn line_data(g: &Geometry) -> Result<(Line, Point), RelationJacobianError> {
    let Geometry::Line(line) = g else {
        return Err(RelationJacobianError::InvalidDomain);
    };
    Ok((*line, line.end.sub(line.start)))
}

fn angle_gradient(v: Point) -> Result<(f64, f64), RelationJacobianError> {
    let denominator = v.x * v.x + v.y * v.y;
    if !denominator.is_finite() || denominator == 0.0 {
        return Err(RelationJacobianError::Indeterminate);
    }
    Ok((-v.y / denominator, v.x / denominator))
}

fn length_gradient(g: &Geometry) -> Result<Vec<f64>, RelationJacobianError> {
    match g {
        Geometry::Line(line) => {
            let d = line.end.sub(line.start);
            let length = d.length();
            if !length.is_finite() || length == 0.0 {
                return Err(RelationJacobianError::Indeterminate);
            }
            Ok(vec![-d.x / length, -d.y / length, d.x / length, d.y / length])
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
            let mut gradient = vec![0.0; width(g)];
            gradient[2] = 1.0;
            Ok(gradient)
        }
        Geometry::Line(_) => Err(RelationJacobianError::InvalidDomain),
    }
}

fn line_point_distance_gradient(
    point: Point,
    line: &Line,
) -> Result<(Vec<f64>, Vec<f64>), RelationJacobianError> {
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
    let inv_length_squared = inv_length * inv_length;
    let point_gradient = vec![sign * dy * inv_length, -sign * dx * inv_length];
    let dq = [qy - dy, dx - qx, -qy, qx];
    let dlength = [-dx * inv_length, -dy * inv_length, dx * inv_length, dy * inv_length];
    let mut line_gradient = vec![0.0; 4];
    for i in 0..4 {
        line_gradient[i] = sign * dq[i] * inv_length
            - abs_cross * dlength[i] * inv_length_squared;
    }
    if !finite(&line_gradient) || !finite(&point_gradient) {
        return Err(RelationJacobianError::NonFinite);
    }
    Ok((line_gradient, point_gradient))
}

fn circle_tangent_gradient(
    ca: Point,
    ra: f64,
    cb: Point,
    rb: f64,
    mode: TangentMode,
) -> Result<(Vec<f64>, Vec<f64>), RelationJacobianError> {
    let dx = ca.x - cb.x;
    let dy = ca.y - cb.y;
    let distance = dx.hypot(dy);
    if !distance.is_finite() || distance == 0.0 {
        return Err(RelationJacobianError::Indeterminate);
    }
    let external = distance - ra - rb;
    let radius_delta = ra - rb;
    let internal = distance - radius_delta.abs();
    let (dra, drb) = match mode {
        TangentMode::External => (-1.0, -1.0),
        TangentMode::Internal => {
            if radius_delta == 0.0 {
                return Err(RelationJacobianError::Indeterminate);
            }
            let sign = radius_delta.signum();
            (-sign, sign)
        }
        TangentMode::Any => {
            if external.abs() == internal.abs() {
                return Err(RelationJacobianError::Indeterminate);
            }
            if external.abs() < internal.abs() {
                (-1.0, -1.0)
            } else {
                if radius_delta == 0.0 {
                    return Err(RelationJacobianError::Indeterminate);
                }
                let sign = radius_delta.signum();
                (-sign, sign)
            }
        }
    };
    let ux = dx / distance;
    let uy = dy / distance;
    Ok((vec![ux, uy, dra], vec![-ux, -uy, drb]))
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
                let first = index(&ids, first_geometry_id)?;
                let second = index(&ids, second_geometry_id)?;
                let (_, a) = line_data(geometry(snapshot, first_geometry_id)?)?;
                let (_, b) = line_data(geometry(snapshot, second_geometry_id)?)?;
                let mut row = vec![0.0; total];
                add(&mut row, offsets[first], &[-b.y, b.x, b.y, -b.x]);
                add(&mut row, offsets[second], &[a.y, -a.x, -a.y, a.x]);
                rows.push(row);
            }
            Relation::Perpendicular { first_geometry_id, second_geometry_id } => {
                let first = index(&ids, first_geometry_id)?;
                let second = index(&ids, second_geometry_id)?;
                let (_, a) = line_data(geometry(snapshot, first_geometry_id)?)?;
                let (_, b) = line_data(geometry(snapshot, second_geometry_id)?)?;
                let mut row = vec![0.0; total];
                add(&mut row, offsets[first], &[-b.x, -b.y, b.x, b.y]);
                add(&mut row, offsets[second], &[-a.x, -a.y, a.x, a.y]);
                rows.push(row);
            }
            Relation::EqualLength { first_geometry_id, second_geometry_id } => {
                let first = index(&ids, first_geometry_id)?;
                let second = index(&ids, second_geometry_id)?;
                let ga = length_gradient(geometry(snapshot, first_geometry_id)?)?;
                let gb = length_gradient(geometry(snapshot, second_geometry_id)?)?;
                let mut row = vec![0.0; total];
                add(&mut row, offsets[first], &ga);
                add(&mut row, offsets[second], &gb.iter().map(|value| -*value).collect::<Vec<_>>());
                rows.push(row);
            }
            Relation::Angle { first_geometry_id, second_geometry_id, .. } => {
                let first = index(&ids, first_geometry_id)?;
                let second = index(&ids, second_geometry_id)?;
                let (_, a) = line_data(geometry(snapshot, first_geometry_id)?)?;
                let (_, b) = line_data(geometry(snapshot, second_geometry_id)?)?;
                let (a_tx, a_ty) = angle_gradient(a)?;
                let (b_tx, b_ty) = angle_gradient(b)?;
                let mut row = vec![0.0; total];
                add(&mut row, offsets[first], &[a_tx, a_ty, -a_tx, -a_ty]);
                add(&mut row, offsets[second], &[-b_tx, -b_ty, b_tx, b_ty]);
                rows.push(row);
            }
            Relation::Collinear { first_geometry_id, second_geometry_id } => {
                let first = index(&ids, first_geometry_id)?;
                let second = index(&ids, second_geometry_id)?;
                let (a_line, d) = line_data(geometry(snapshot, first_geometry_id)?)?;
                let (b_line, _) = line_data(geometry(snapshot, second_geometry_id)?)?;
                for (endpoint_index, b_point) in [(0usize, b_line.start), (1usize, b_line.end)] {
                    let q = b_point.sub(a_line.start);
                    let mut row = vec![0.0; total];
                    add(&mut row, offsets[first], &[d.y - q.y, q.x - d.x, q.y, -q.x]);
                    let block = if endpoint_index == 0 {
                        vec![-d.y, d.x, 0.0, 0.0]
                    } else {
                        vec![0.0, 0.0, -d.y, d.x]
                    };
                    add(&mut row, offsets[second], &block);
                    rows.push(row);
                }
            }
            Relation::Concentric { first_geometry_id, second_geometry_id } => {
                let first = index(&ids, first_geometry_id)?;
                let second = index(&ids, second_geometry_id)?;
                let a = geometry(snapshot, first_geometry_id)?;
                let b = geometry(snapshot, second_geometry_id)?;
                if !matches!(a, Geometry::Circle(_) | Geometry::Arc(_)) || !matches!(b, Geometry::Circle(_) | Geometry::Arc(_)) {
                    return Err(RelationJacobianError::InvalidDomain);
                }
                let mut x = vec![0.0; total];
                let mut y = vec![0.0; total];
                let mut ax = vec![0.0; width(a)];
                let mut ay = vec![0.0; width(a)];
                let mut bx = vec![0.0; width(b)];
                let mut by = vec![0.0; width(b)];
                ax[0] = 1.0; ay[1] = 1.0; bx[0] = -1.0; by[1] = -1.0;
                add(&mut x, offsets[first], &ax);
                add(&mut x, offsets[second], &bx);
                add(&mut y, offsets[first], &ay);
                add(&mut y, offsets[second], &by);
                rows.push(x);
                rows.push(y);
            }
            Relation::EqualRadius { first_geometry_id, second_geometry_id } => {
                let first = index(&ids, first_geometry_id)?;
                let second = index(&ids, second_geometry_id)?;
                let ga = radius_gradient(geometry(snapshot, first_geometry_id)?)?;
                let gb = radius_gradient(geometry(snapshot, second_geometry_id)?)?;
                let mut row = vec![0.0; total];
                add(&mut row, offsets[first], &ga);
                add(&mut row, offsets[second], &gb.iter().map(|value| -*value).collect::<Vec<_>>());
                rows.push(row);
            }
            Relation::Radius { geometry_id, .. } => {
                let index = index(&ids, geometry_id)?;
                let mut row = vec![0.0; total];
                add(&mut row, offsets[index], &radius_gradient(geometry(snapshot, geometry_id)?)?);
                rows.push(row);
            }
            Relation::Diameter { geometry_id, .. } => {
                let index = index(&ids, geometry_id)?;
                let mut row = vec![0.0; total];
                let mut gradient = radius_gradient(geometry(snapshot, geometry_id)?)?;
                gradient.iter_mut().for_each(|value| *value *= 2.0);
                add(&mut row, offsets[index], &gradient);
                rows.push(row);
            }
            Relation::Tangent { first_geometry_id, second_geometry_id, mode } => {
                let first = index(&ids, first_geometry_id)?;
                let second = index(&ids, second_geometry_id)?;
                let a = geometry(snapshot, first_geometry_id)?;
                let b = geometry(snapshot, second_geometry_id)?;
                let mut row = vec![0.0; total];
                match (a, b) {
                    (Geometry::Line(_), Geometry::Line(_)) => return Err(RelationJacobianError::InvalidDomain),
                    (Geometry::Line(line), Geometry::Circle(circle)) => {
                        let (line_gradient, point_gradient) = line_point_distance_gradient(circle.center, line)?;
                        add(&mut row, offsets[first], &line_gradient);
                        let circle_gradient = vec![point_gradient[0], point_gradient[1], -1.0];
                        add(&mut row, offsets[second], &circle_gradient);
                    }
                    (Geometry::Line(line), Geometry::Arc(arc)) => {
                        let (line_gradient, point_gradient) = line_point_distance_gradient(arc.center, line)?;
                        add(&mut row, offsets[first], &line_gradient);
                        let arc_gradient = vec![point_gradient[0], point_gradient[1], -1.0, 0.0, 0.0];
                        add(&mut row, offsets[second], &arc_gradient);
                    }
                    (Geometry::Circle(circle), Geometry::Line(line)) => {
                        let (line_gradient, point_gradient) = line_point_distance_gradient(circle.center, line)?;
                        let circle_gradient = vec![point_gradient[0], point_gradient[1], -1.0];
                        add(&mut row, offsets[first], &circle_gradient);
                        add(&mut row, offsets[second], &line_gradient);
                    }
                    (Geometry::Arc(arc), Geometry::Line(line)) => {
                        let (line_gradient, point_gradient) = line_point_distance_gradient(arc.center, line)?;
                        let arc_gradient = vec![point_gradient[0], point_gradient[1], -1.0, 0.0, 0.0];
                        add(&mut row, offsets[first], &arc_gradient);
                        add(&mut row, offsets[second], &line_gradient);
                    }
                    (Geometry::Circle(a_circle), Geometry::Circle(b_circle)) => {
                        let (ga, gb) = circle_tangent_gradient(a_circle.center, a_circle.radius, b_circle.center, b_circle.radius, *mode)?;
                        add(&mut row, offsets[first], &ga);
                        add(&mut row, offsets[second], &gb);
                    }
                    (Geometry::Circle(a_circle), Geometry::Arc(b_arc)) => {
                        let (ga, gb) = circle_tangent_gradient(a_circle.center, a_circle.radius, b_arc.center, b_arc.radius, *mode)?;
                        let vb = vec![gb[0], gb[1], gb[2], 0.0, 0.0];
                        add(&mut row, offsets[first], &ga);
                        add(&mut row, offsets[second], &vb);
                    }
                    (Geometry::Arc(a_arc), Geometry::Circle(b_circle)) => {
                        let (ga, gb) = circle_tangent_gradient(a_arc.center, a_arc.radius, b_circle.center, b_circle.radius, *mode)?;
                        let va = vec![ga[0], ga[1], ga[2], 0.0, 0.0];
                        add(&mut row, offsets[first], &va);
                        add(&mut row, offsets[second], &gb);
                    }
                    (Geometry::Arc(a_arc), Geometry::Arc(b_arc)) => {
                        let (ga, gb) = circle_tangent_gradient(a_arc.center, a_arc.radius, b_arc.center, b_arc.radius, *mode)?;
                        let va = vec![ga[0], ga[1], ga[2], 0.0, 0.0];
                        let vb = vec![gb[0], gb[1], gb[2], 0.0, 0.0];
                        add(&mut row, offsets[first], &va);
                        add(&mut row, offsets[second], &vb);
                    }
                }
                rows.push(row);
            }
            Relation::Midpoint { point, line_geometry_id } => {
                let (point_index, _, px, py) = relation_point(snapshot, point)?;
                let line_index = index(&ids, line_geometry_id)?;
                if !matches!(geometry(snapshot, line_geometry_id)?, Geometry::Line(_)) {
                    return Err(RelationJacobianError::InvalidDomain);
                }
                let mut row_x = vec![0.0; total];
                let mut row_y = vec![0.0; total];
                add(&mut row_x, offsets[point_index], &px);
                add(&mut row_y, offsets[point_index], &py);
                add(&mut row_x, offsets[line_index], &[-0.5, 0.0, -0.5, 0.0]);
                add(&mut row_y, offsets[line_index], &[0.0, -0.5, 0.0, -0.5]);
                rows.push(row_x);
                rows.push(row_y);
            }
            Relation::PointOnLine { point, line_geometry_id } => {
                let (point_index, point_value, px, py) = relation_point(snapshot, point)?;
                let line_index = index(&ids, line_geometry_id)?;
                let Geometry::Line(line) = geometry(snapshot, line_geometry_id)? else {
                    return Err(RelationJacobianError::InvalidDomain);
                };
                let (line_gradient, point_gradient) = line_point_distance_gradient(point_value, line)?;
                let mut row = vec![0.0; total];
                add(&mut row, offsets[line_index], &line_gradient);
                let mut point_local = vec![0.0; px.len()];
                for i in 0..point_local.len() {
                    point_local[i] = point_gradient[0] * px[i] + point_gradient[1] * py[i];
                }
                add(&mut row, offsets[point_index], &point_local);
                rows.push(row);
            }
            Relation::PointOnCircle { point, circle_geometry_id } => {
                let (point_index, point_value, px, py) = relation_point(snapshot, point)?;
                let circle_index = index(&ids, circle_geometry_id)?;
                let circle_geometry = geometry(snapshot, circle_geometry_id)?;
                let center = match circle_geometry {
                    Geometry::Circle(circle) => circle.center,
                    Geometry::Arc(arc) => arc.center,
                    Geometry::Line(_) => return Err(RelationJacobianError::InvalidDomain),
                };
                let dx = point_value.x - center.x;
                let dy = point_value.y - center.y;
                let distance = dx.hypot(dy);
                if !distance.is_finite() || distance == 0.0 {
                    return Err(RelationJacobianError::Indeterminate);
                }
                let ux = dx / distance;
                let uy = dy / distance;
                let mut row = vec![0.0; total];
                let mut point_local = vec![0.0; px.len()];
                for i in 0..point_local.len() {
                    point_local[i] = ux * px[i] + uy * py[i];
                }
                add(&mut row, offsets[point_index], &point_local);
                let mut circle_local = vec![0.0; width(circle_geometry)];
                circle_local[0] = -ux;
                circle_local[1] = -uy;
                circle_local[2] = -1.0;
                add(&mut row, offsets[circle_index], &circle_local);
                rows.push(row);
            }
            Relation::DistancePoints { first, second, .. } => {
                let (first_index, first_value, first_px, first_py) = relation_point(snapshot, first)?;
                let (second_index, second_value, second_px, second_py) = relation_point(snapshot, second)?;
                let dx = first_value.x - second_value.x;
                let dy = first_value.y - second_value.y;
                let distance = dx.hypot(dy);
                if !distance.is_finite() || distance == 0.0 {
                    return Err(RelationJacobianError::Indeterminate);
                }
                let ux = dx / distance;
                let uy = dy / distance;
                let mut row = vec![0.0; total];
                let mut first_local = vec![0.0; first_px.len()];
                let mut second_local = vec![0.0; second_px.len()];
                for i in 0..first_local.len() {
                    first_local[i] = ux * first_px[i] + uy * first_py[i];
                }
                for i in 0..second_local.len() {
                    second_local[i] = -(ux * second_px[i] + uy * second_py[i]);
                }
                add(&mut row, offsets[first_index], &first_local);
                add(&mut row, offsets[second_index], &second_local);
                rows.push(row);
            }
            Relation::Symmetric { first, second, about } => {
                let (first_index, _, first_px, first_py) = relation_point(snapshot, first)?;
                let (second_index, _, second_px, second_py) = relation_point(snapshot, second)?;
                let (about_index, _, about_px, about_py) = relation_point(snapshot, about)?;
                let mut x = vec![0.0; total];
                let mut y = vec![0.0; total];
                add(&mut x, offsets[first_index], &first_px.iter().map(|value| 0.5 * *value).collect::<Vec<_>>());
                add(&mut x, offsets[second_index], &second_px.iter().map(|value| 0.5 * *value).collect::<Vec<_>>());
                add(&mut x, offsets[about_index], &about_px.iter().map(|value| -*value).collect::<Vec<_>>());
                add(&mut y, offsets[first_index], &first_py.iter().map(|value| 0.5 * *value).collect::<Vec<_>>());
                add(&mut y, offsets[second_index], &second_py.iter().map(|value| 0.5 * *value).collect::<Vec<_>>());
                add(&mut y, offsets[about_index], &about_py.iter().map(|value| -*value).collect::<Vec<_>>());
                rows.push(x);
                rows.push(y);
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
    use super::super::geometry::{Circle, Geometry, Line};
    use super::super::snapshot::GeometryItem;

    fn line(id: &str, sx: f64, sy: f64, ex: f64, ey: f64) -> GeometryItem {
        GeometryItem {
            id: id.into(),
            geometry: Geometry::Line(Line { start: Point { x: sx, y: sy }, end: Point { x: ex, y: ey } }),
            parameter_dependencies: vec![],
        }
    }

    fn circle(id: &str, x: f64, y: f64, r: f64) -> GeometryItem {
        GeometryItem {
            id: id.into(),
            geometry: Geometry::Circle(Circle { center: Point { x, y }, radius: r }),
            parameter_dependencies: vec![],
        }
    }

    fn snapshot(geometry: Vec<GeometryItem>, relations: Vec<Relation>) -> SemanticSnapshot {
        SemanticSnapshot {
            parameters: vec![],
            geometry,
            constraints: vec![],
            relations: relations.into_iter().enumerate().map(|(i, relation)| (format!("r{i}"), relation)).collect(),
        }
    }

    fn set_parameter(snapshot: &mut SemanticSnapshot, column: usize, delta: f64) {
        let mut cursor = 0usize;
        for item in &mut snapshot.geometry {
            match &mut item.geometry {
                Geometry::Line(line) => {
                    let end = cursor + 4;
                    if column >= cursor && column < end {
                        match column - cursor {
                            0 => line.start.x += delta,
                            1 => line.start.y += delta,
                            2 => line.end.x += delta,
                            3 => line.end.y += delta,
                            _ => unreachable!(),
                        }
                        return;
                    }
                    cursor = end;
                }
                Geometry::Circle(circle) => {
                    let end = cursor + 3;
                    if column >= cursor && column < end {
                        match column - cursor {
                            0 => circle.center.x += delta,
                            1 => circle.center.y += delta,
                            2 => circle.radius += delta,
                            _ => unreachable!(),
                        }
                        return;
                    }
                    cursor = end;
                }
                Geometry::Arc(arc) => {
                    let end = cursor + 5;
                    if column >= cursor && column < end {
                        match column - cursor {
                            0 => arc.center.x += delta,
                            1 => arc.center.y += delta,
                            2 => arc.radius += delta,
                            3 => arc.start_angle += delta,
                            4 => arc.end_angle += delta,
                            _ => unreachable!(),
                        }
                        return;
                    }
                    cursor = end;
                }
            }
        }
        panic!("parameter column {column} not found");
    }

    fn residual_rows(snapshot: &SemanticSnapshot) -> Vec<f64> {
        snapshot
            .relations
            .iter()
            .flat_map(|(_, relation)| evaluate_relation(snapshot, relation).unwrap().residuals)
            .collect()
    }

    #[test]
    fn core_relation_rows_are_finite_and_counted() {
        let snapshot = snapshot(
            vec![line("a", 0.0, 0.0, 3.0, 0.0), line("b", 0.0, 1.0, 3.0, 2.0), circle("c", 10.0, 0.0, 2.0)],
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
    fn tangent_gradient_is_finite_for_line_circle_and_circle_circle() {
        let line_circle = snapshot(
            vec![line("l", 0.0, 0.0, 10.0, 0.0), circle("c", 5.0, 2.0, 2.0)],
            vec![Relation::Tangent { first_geometry_id: "l".into(), second_geometry_id: "c".into(), mode: TangentMode::External }],
        );
        assert!(analytic_relation_jacobian(&line_circle).unwrap()[0].iter().all(|value| value.is_finite()));

        let circles = snapshot(
            vec![circle("a", 0.0, 0.0, 2.0), circle("b", 5.0, 0.0, 3.0)],
            vec![Relation::Tangent { first_geometry_id: "a".into(), second_geometry_id: "b".into(), mode: TangentMode::External }],
        );
        assert!(analytic_relation_jacobian(&circles).unwrap()[0].iter().all(|value| value.is_finite()));
    }

    #[test]
    fn analytic_rows_match_central_difference_oracle() {
        let snapshot = snapshot(
            vec![
                line("a", 0.0, 0.0, 3.0, 0.0),
                line("b", 0.0, 2.0, 4.0, 3.0),
                circle("c", 8.0, 2.0, 1.5),
            ],
            vec![
                Relation::Parallel { first_geometry_id: "a".into(), second_geometry_id: "b".into() },
                Relation::Perpendicular { first_geometry_id: "a".into(), second_geometry_id: "b".into() },
                Relation::EqualLength { first_geometry_id: "a".into(), second_geometry_id: "b".into() },
                Relation::Angle { first_geometry_id: "a".into(), second_geometry_id: "b".into(), radians: 0.2 },
                Relation::Collinear { first_geometry_id: "a".into(), second_geometry_id: "b".into() },
                Relation::Concentric { first_geometry_id: "c".into(), second_geometry_id: "c".into() },
                Relation::EqualRadius { first_geometry_id: "c".into(), second_geometry_id: "c".into() },
                Relation::Radius { geometry_id: "c".into(), value: 2.0 },
                Relation::Diameter { geometry_id: "c".into(), value: 3.0 },
                Relation::Tangent { first_geometry_id: "a".into(), second_geometry_id: "c".into(), mode: TangentMode::External },
                Relation::Midpoint { point: RelationPoint::Endpoint { geometry_id: "b".into(), point: Endpoint::Start }, line_geometry_id: "a".into() },
                Relation::PointOnLine { point: RelationPoint::Endpoint { geometry_id: "b".into(), point: Endpoint::Start }, line_geometry_id: "a".into() },
                Relation::PointOnCircle { point: RelationPoint::Endpoint { geometry_id: "b".into(), point: Endpoint::Start }, circle_geometry_id: "c".into() },
                Relation::DistancePoints { first: RelationPoint::Endpoint { geometry_id: "a".into(), point: Endpoint::Start }, second: RelationPoint::Center { geometry_id: "c".into() }, value: 8.0 },
                Relation::Symmetric {
                    first: RelationPoint::Endpoint { geometry_id: "a".into(), point: Endpoint::Start },
                    second: RelationPoint::Endpoint { geometry_id: "b".into(), point: Endpoint::Start },
                    about: RelationPoint::Center { geometry_id: "c".into() },
                },
            ],
        );

        let analytic = analytic_relation_jacobian(&snapshot).unwrap();
        let base = residual_rows(&snapshot);
        assert_eq!(analytic.len(), base.len());

        let (_, offsets) = layout(&snapshot);
        let columns = offsets.last().copied().unwrap_or(0);
        let h = 1.0e-6;
        for column in 0..columns {
            let mut plus = snapshot.clone();
            let mut minus = snapshot.clone();
            set_parameter(&mut plus, column, h);
            set_parameter(&mut minus, column, -h);
            let plus_values = residual_rows(&plus);
            let minus_values = residual_rows(&minus);
            for row in 0..analytic.len() {
                let numerical = (plus_values[row] - minus_values[row]) / (2.0 * h);
                let expected = analytic[row][column];
                let scale = numerical.abs().max(expected.abs()).max(1.0);
                assert!((numerical - expected).abs() <= 5.0e-5 * scale,
                    "row {row}, col {column}: analytic={expected:.12e}, numerical={numerical:.12e}");
            }
        }
    }

    #[test]
    fn nondifferentiable_point_on_line_fails_closed() {
        let snapshot = snapshot(
            vec![line("a", 0.0, 0.0, 1.0, 0.0), line("b", 0.0, 1.0, 1.0, 1.0)],
            vec![Relation::PointOnLine {
                point: RelationPoint::Endpoint { geometry_id: "a".into(), point: Endpoint::Start },
                line_geometry_id: "b".into(),
            }],
        );
        assert_eq!(analytic_relation_jacobian(&snapshot), Err(RelationJacobianError::Indeterminate));
    }
}
