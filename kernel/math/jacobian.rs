//! Analytic Jacobian authority for semantic constraints and relations.
//!
//! Rows are emitted in exactly the same order as the residual system:
//! constraints first, followed by relations. Finite differences are not used
//! by this production authority; they remain an independent verification oracle
//! in the dedicated relation-Jacobian tests.

use super::{
    constraints::endpoint,
    geometry::{Arc, Circle, Geometry, Line, Point},
    relation_jacobian::analytic_relation_jacobian,
    snapshot::{Constraint, Endpoint, SemanticSnapshot},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JacobianError {
    UnknownGeometry,
    UnsupportedConstraint,
    InvalidDomain,
    NonFinite,
    Indeterminate,
}

pub fn geometry_vector_layout(snapshot: &SemanticSnapshot) -> (Vec<String>, Vec<usize>) {
    let mut ids = Vec::with_capacity(snapshot.geometry.len());
    let mut offsets = vec![0usize];
    for item in &snapshot.geometry {
        ids.push(item.id.clone());
        let width = match item.geometry {
            Geometry::Line(_) => 4,
            Geometry::Circle(_) => 3,
            Geometry::Arc(_) => 5,
        };
        offsets.push(offsets.last().copied().unwrap_or(0) + width);
    }
    (ids, offsets)
}

fn index(ids: &[String], id: &str) -> Option<usize> {
    ids.iter().position(|value| value == id)
}

fn add_block(row: &mut [f64], offset: usize, values: &[f64]) {
    for (index, value) in values.iter().copied().enumerate() {
        row[offset + index] = value;
    }
}

fn finite_row(row: &[f64]) -> bool {
    row.iter().all(|value| value.is_finite())
}

fn endpoint_value(g: &Geometry, e: Endpoint) -> Result<Point, JacobianError> {
    endpoint(g, e).ok_or(JacobianError::InvalidDomain)
}

fn endpoint_derivative_x(g: &Geometry, e: Endpoint) -> Result<Vec<f64>, JacobianError> {
    match (g, e) {
        (Geometry::Line(_), Endpoint::Start) => Ok(vec![1.0, 0.0, 0.0, 0.0]),
        (Geometry::Line(_), Endpoint::End) => Ok(vec![0.0, 0.0, 1.0, 0.0]),
        (Geometry::Arc(a), Endpoint::Start) => Ok(vec![
            1.0,
            0.0,
            a.start_angle.cos(),
            -a.radius * a.start_angle.sin(),
            0.0,
        ]),
        (Geometry::Arc(a), Endpoint::End) => Ok(vec![
            1.0,
            0.0,
            a.end_angle.cos(),
            0.0,
            -a.radius * a.end_angle.sin(),
        ]),
        (Geometry::Circle(_), _) => Err(JacobianError::InvalidDomain),
    }
}

fn endpoint_derivative_y(g: &Geometry, e: Endpoint) -> Result<Vec<f64>, JacobianError> {
    match (g, e) {
        (Geometry::Line(_), Endpoint::Start) => Ok(vec![0.0, 1.0, 0.0, 0.0]),
        (Geometry::Line(_), Endpoint::End) => Ok(vec![0.0, 0.0, 0.0, 1.0]),
        (Geometry::Arc(a), Endpoint::Start) => Ok(vec![
            0.0,
            1.0,
            a.start_angle.sin(),
            a.radius * a.start_angle.cos(),
            0.0,
        ]),
        (Geometry::Arc(a), Endpoint::End) => Ok(vec![
            0.0,
            1.0,
            a.end_angle.sin(),
            0.0,
            a.radius * a.end_angle.cos(),
        ]),
        (Geometry::Circle(_), _) => Err(JacobianError::InvalidDomain),
    }
}

fn local_identity(width: usize) -> Vec<Vec<f64>> {
    (0..width)
        .map(|index| {
            let mut row = vec![0.0; width];
            row[index] = 1.0;
            row
        })
        .collect()
}

pub fn analytic_constraint_jacobian(
    snapshot: &SemanticSnapshot,
) -> Result<Vec<Vec<f64>>, JacobianError> {
    let (ids, offsets) = geometry_vector_layout(snapshot);
    let total = offsets.last().copied().unwrap_or(0);
    let mut rows = Vec::new();

    for (_, constraint) in &snapshot.constraints {
        match constraint {
            Constraint::Horizontal { entity_id } => {
                let index = index(&ids, entity_id).ok_or(JacobianError::UnknownGeometry)?;
                if offsets[index + 1] - offsets[index] != 4 {
                    return Err(JacobianError::InvalidDomain);
                }
                let mut row = vec![0.0; total];
                row[offsets[index] + 3] = 1.0;
                row[offsets[index] + 1] = -1.0;
                rows.push(row);
            }
            Constraint::Vertical { entity_id } => {
                let index = index(&ids, entity_id).ok_or(JacobianError::UnknownGeometry)?;
                if offsets[index + 1] - offsets[index] != 4 {
                    return Err(JacobianError::InvalidDomain);
                }
                let mut row = vec![0.0; total];
                row[offsets[index] + 2] = 1.0;
                row[offsets[index]] = -1.0;
                rows.push(row);
            }
            Constraint::Coincident {
                first_geometry_id,
                first_point,
                second_geometry_id,
                second_point,
            } => {
                let first_index = index(&ids, first_geometry_id).ok_or(JacobianError::UnknownGeometry)?;
                let second_index = index(&ids, second_geometry_id).ok_or(JacobianError::UnknownGeometry)?;
                let first_geometry = &snapshot.geometry[first_index].geometry;
                let second_geometry = &snapshot.geometry[second_index].geometry;
                let first_x = endpoint_derivative_x(first_geometry, *first_point)?;
                let first_y = endpoint_derivative_y(first_geometry, *first_point)?;
                let second_x = endpoint_derivative_x(second_geometry, *second_point)?;
                let second_y = endpoint_derivative_y(second_geometry, *second_point)?;
                let mut row_x = vec![0.0; total];
                let mut row_y = vec![0.0; total];
                add_block(&mut row_x, offsets[first_index], &first_x);
                add_block(&mut row_y, offsets[first_index], &first_y);
                let second_x = second_x.iter().map(|value| -*value).collect::<Vec<_>>();
                let second_y = second_y.iter().map(|value| -*value).collect::<Vec<_>>();
                add_block(&mut row_x, offsets[second_index], &second_x);
                add_block(&mut row_y, offsets[second_index], &second_y);
                rows.push(row_x);
                rows.push(row_y);
            }
            Constraint::Fixed { entity_id } => {
                let index = index(&ids, entity_id).ok_or(JacobianError::UnknownGeometry)?;
                let width = offsets[index + 1] - offsets[index];
                for local in local_identity(width) {
                    let mut row = vec![0.0; total];
                    add_block(&mut row, offsets[index], &local);
                    rows.push(row);
                }
            }
            Constraint::Distance {
                first_geometry_id,
                second_geometry_id,
                first_endpoint,
                second_endpoint,
                value: _,
            } => {
                let first_index = index(&ids, first_geometry_id).ok_or(JacobianError::UnknownGeometry)?;
                let first_geometry = &snapshot.geometry[first_index].geometry;
                let first_endpoint = first_endpoint.unwrap_or(Endpoint::Start);

                if let Some(second_geometry_id) = second_geometry_id {
                    let second_index = index(&ids, second_geometry_id).ok_or(JacobianError::UnknownGeometry)?;
                    let second_geometry = &snapshot.geometry[second_index].geometry;
                    let second_endpoint = second_endpoint.unwrap_or(Endpoint::Start);
                    let first_point = endpoint_value(first_geometry, first_endpoint)?;
                    let second_point = endpoint_value(second_geometry, second_endpoint)?;
                    let distance = first_point.distance(second_point);
                    if distance == 0.0 || !distance.is_finite() {
                        return Err(JacobianError::Indeterminate);
                    }
                    let ux = (first_point.x - second_point.x) / distance;
                    let uy = (first_point.y - second_point.y) / distance;
                    let first_x = endpoint_derivative_x(first_geometry, first_endpoint)?;
                    let first_y = endpoint_derivative_y(first_geometry, first_endpoint)?;
                    let second_x = endpoint_derivative_x(second_geometry, second_endpoint)?;
                    let second_y = endpoint_derivative_y(second_geometry, second_endpoint)?;
                    let mut row = vec![0.0; total];
                    let first_block = first_x
                        .iter()
                        .zip(first_y.iter())
                        .map(|(x, y)| ux * x + uy * y)
                        .collect::<Vec<_>>();
                    let second_block = second_x
                        .iter()
                        .zip(second_y.iter())
                        .map(|(x, y)| -(ux * x + uy * y))
                        .collect::<Vec<_>>();
                    add_block(&mut row, offsets[first_index], &first_block);
                    add_block(&mut row, offsets[second_index], &second_block);
                    rows.push(row);
                } else {
                    let mut row = vec![0.0; total];
                    match first_geometry {
                        Geometry::Line(Line { start, end }) => {
                            let dx = end.x - start.x;
                            let dy = end.y - start.y;
                            let length = dx.hypot(dy);
                            if length == 0.0 || !length.is_finite() {
                                return Err(JacobianError::Indeterminate);
                            }
                            add_block(
                                &mut row,
                                offsets[first_index],
                                &[-dx / length, -dy / length, dx / length, dy / length],
                            );
                        }
                        Geometry::Circle(Circle { .. }) => {
                            add_block(&mut row, offsets[first_index], &[0.0, 0.0, 1.0]);
                        }
                        Geometry::Arc(Arc {
                            radius,
                            start_angle,
                            end_angle,
                            ..
                        }) => {
                            let delta = end_angle - start_angle;
                            if !delta.is_finite() || delta == 0.0 {
                                return Err(JacobianError::Indeterminate);
                            }
                            let sign = delta.signum();
                            add_block(
                                &mut row,
                                offsets[first_index],
                                &[0.0, 0.0, delta.abs(), -radius * sign, radius * sign],
                            );
                        }
                    }
                    rows.push(row);
                }
            }
        }
    }

    let relation_rows = analytic_relation_jacobian(snapshot).map_err(|error| match error {
        super::relation_jacobian::RelationJacobianError::UnknownGeometry => JacobianError::UnknownGeometry,
        super::relation_jacobian::RelationJacobianError::InvalidDomain => JacobianError::InvalidDomain,
        super::relation_jacobian::RelationJacobianError::NonFinite => JacobianError::NonFinite,
        super::relation_jacobian::RelationJacobianError::Indeterminate => JacobianError::Indeterminate,
    })?;
    if relation_rows.iter().any(|row| row.len() != total || !finite_row(row)) {
        return Err(JacobianError::NonFinite);
    }
    rows.extend(relation_rows);

    if rows.iter().all(|row| row.len() == total && finite_row(row)) {
        Ok(rows)
    } else {
        Err(JacobianError::NonFinite)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::{
        geometry::{Arc, Circle, Geometry, Line, Point},
        snapshot::{Constraint, GeometryItem, Relation, SemanticSnapshot},
    };

    fn snap(
        geometry: Vec<GeometryItem>,
        constraints: Vec<Constraint>,
        relations: Vec<Relation>,
    ) -> SemanticSnapshot {
        SemanticSnapshot {
            parameters: vec![],
            geometry,
            constraints: constraints
                .into_iter()
                .enumerate()
                .map(|(i, constraint)| (format!("c{i}"), constraint))
                .collect(),
            relations: relations
                .into_iter()
                .enumerate()
                .map(|(i, relation)| (format!("r{i}"), relation))
                .collect(),
        }
    }

    #[test]
    fn horizontal_and_vertical_have_exact_sparse_rows() {
        let snapshot = snap(
            vec![GeometryItem {
                id: "l".into(),
                geometry: Geometry::Line(Line {
                    start: Point { x: 0.0, y: 0.0 },
                    end: Point { x: 1.0, y: 1.0 },
                }),
                parameter_dependencies: vec![],
            }],
            vec![
                Constraint::Horizontal { entity_id: "l".into() },
                Constraint::Vertical { entity_id: "l".into() },
            ],
            vec![],
        );
        let jacobian = analytic_constraint_jacobian(&snapshot).unwrap();
        assert_eq!(jacobian.len(), 2);
        assert_eq!(jacobian[0], vec![0.0, -1.0, 0.0, 1.0]);
        assert_eq!(jacobian[1], vec![-1.0, 0.0, 1.0, 0.0]);
    }

    #[test]
    fn coincident_arc_endpoint_derivative_matches_formula() {
        let snapshot = snap(
            vec![
                GeometryItem {
                    id: "a".into(),
                    geometry: Geometry::Arc(Arc {
                        center: Point { x: 0.0, y: 0.0 },
                        radius: 2.0,
                        start_angle: 0.3,
                        end_angle: 1.2,
                    }),
                    parameter_dependencies: vec![],
                },
                GeometryItem {
                    id: "l".into(),
                    geometry: Geometry::Line(Line {
                        start: Point { x: 3.0, y: 4.0 },
                        end: Point { x: 4.0, y: 4.0 },
                    }),
                    parameter_dependencies: vec![],
                },
            ],
            vec![Constraint::Coincident {
                first_geometry_id: "a".into(),
                first_point: Endpoint::Start,
                second_geometry_id: "l".into(),
                second_point: Endpoint::Start,
            }],
            vec![],
        );
        let jacobian = analytic_constraint_jacobian(&snapshot).unwrap();
        assert_eq!(jacobian.len(), 2);
        assert!((jacobian[0][2] - 0.3f64.cos()).abs() < 1.0e-15);
        assert!((jacobian[1][3] - 2.0 * 0.3f64.cos()).abs() < 1.0e-15);
    }

    #[test]
    fn line_length_jacobian_is_unit_endpoint_direction() {
        let snapshot = snap(
            vec![GeometryItem {
                id: "l".into(),
                geometry: Geometry::Line(Line {
                    start: Point { x: 0.0, y: 0.0 },
                    end: Point { x: 3.0, y: 4.0 },
                }),
                parameter_dependencies: vec![],
            }],
            vec![Constraint::Distance {
                first_geometry_id: "l".into(),
                second_geometry_id: None,
                first_endpoint: None,
                second_endpoint: None,
                value: 5.0,
            }],
            vec![],
        );
        let jacobian = analytic_constraint_jacobian(&snapshot).unwrap();
        assert_eq!(jacobian[0], vec![-0.6, -0.8, 0.6, 0.8]);
    }

    #[test]
    fn coincident_distance_has_no_finite_derivative() {
        let snapshot = snap(
            vec![
                GeometryItem {
                    id: "a".into(),
                    geometry: Geometry::Line(Line {
                        start: Point { x: 0.0, y: 0.0 },
                        end: Point { x: 1.0, y: 0.0 },
                    }),
                    parameter_dependencies: vec![],
                },
                GeometryItem {
                    id: "b".into(),
                    geometry: Geometry::Line(Line {
                        start: Point { x: 0.0, y: 0.0 },
                        end: Point { x: 0.0, y: 1.0 },
                    }),
                    parameter_dependencies: vec![],
                },
            ],
            vec![Constraint::Distance {
                first_geometry_id: "a".into(),
                second_geometry_id: Some("b".into()),
                first_endpoint: Some(Endpoint::Start),
                second_endpoint: Some(Endpoint::Start),
                value: 0.0,
            }],
            vec![],
        );
        assert_eq!(analytic_constraint_jacobian(&snapshot), Err(JacobianError::Indeterminate));
    }


    fn line_item(
        id: &str,
        sx: f64,
        sy: f64,
        ex: f64,
        ey: f64,
    ) -> GeometryItem {
        GeometryItem {
            id: id.into(),
            geometry: Geometry::Line(Line {
                start: Point { x: sx, y: sy },
                end: Point { x: ex, y: ey },
            }),
            parameter_dependencies: vec![],
        }
    }

    fn circle_item(id: &str, x: f64, y: f64, radius: f64) -> GeometryItem {
        GeometryItem {
            id: id.into(),
            geometry: Geometry::Circle(Circle {
                center: Point { x, y },
                radius,
            }),
            parameter_dependencies: vec![],
        }
    }

    fn arc_item(
        id: &str,
        x: f64,
        y: f64,
        radius: f64,
        start_angle: f64,
        end_angle: f64,
    ) -> GeometryItem {
        GeometryItem {
            id: id.into(),
            geometry: Geometry::Arc(Arc {
                center: Point { x, y },
                radius,
                start_angle,
                end_angle,
            }),
            parameter_dependencies: vec![],
        }
    }

    fn set_parameter(snapshot: &mut SemanticSnapshot, column: usize, delta: f64) {
        let mut cursor = 0usize;
        for item in &mut snapshot.geometry {
            match &mut item.geometry {
                Geometry::Line(line) => {
                    if column < cursor + 4 {
                        match column - cursor {
                            0 => line.start.x += delta,
                            1 => line.start.y += delta,
                            2 => line.end.x += delta,
                            3 => line.end.y += delta,
                            _ => unreachable!(),
                        }
                        return;
                    }
                    cursor += 4;
                }
                Geometry::Circle(circle) => {
                    if column < cursor + 3 {
                        match column - cursor {
                            0 => circle.center.x += delta,
                            1 => circle.center.y += delta,
                            2 => circle.radius += delta,
                            _ => unreachable!(),
                        }
                        return;
                    }
                    cursor += 3;
                }
                Geometry::Arc(arc) => {
                    if column < cursor + 5 {
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
                    cursor += 5;
                }
            }
        }
        panic!("parameter column {column} not found");
    }

    fn constraint_residual_rows(snapshot: &SemanticSnapshot) -> Vec<f64> {
        snapshot
            .constraints
            .iter()
            .flat_map(|(_, constraint)| {
                match constraint {
                    Constraint::Fixed { entity_id } => {
                        let current = snapshot.geometry(entity_id).unwrap();
                        let reference = snapshot.geometry(entity_id).unwrap();
                        geometry_difference(current, reference).unwrap()
                    }
                    _ => super::super::constraints::residual(
                        |id| snapshot.geometry(id).cloned(),
                        constraint,
                    )
                    .unwrap(),
                }
            })
            .collect()
    }

    fn constraint_only_snapshot(
        geometry: Vec<GeometryItem>,
        constraints: Vec<Constraint>,
    ) -> SemanticSnapshot {
        SemanticSnapshot {
            parameters: vec![],
            geometry,
            constraints: constraints
                .into_iter()
                .enumerate()
                .map(|(i, constraint)| (format!("c{i}"), constraint))
                .collect(),
            relations: vec![],
        }
    }

    fn assert_constraint_jacobian_matches_central_difference(snapshot: &SemanticSnapshot) {
        let analytic = analytic_constraint_jacobian(snapshot).unwrap();
        let base = constraint_residual_rows(snapshot);
        assert_eq!(analytic.len(), base.len());
        let total_columns = snapshot
            .geometry
            .iter()
            .map(|item| match item.geometry {
                Geometry::Line(_) => 4,
                Geometry::Circle(_) => 3,
                Geometry::Arc(_) => 5,
            })
            .sum::<usize>();
        let h = 1.0e-7;
        for column in 0..total_columns {
            let mut plus = snapshot.clone();
            let mut minus = snapshot.clone();
            set_parameter(&mut plus, column, h);
            set_parameter(&mut minus, column, -h);
            let plus_values = constraint_residual_rows(&plus);
            let minus_values = constraint_residual_rows(&minus);
            assert_eq!(plus_values.len(), base.len());
            assert_eq!(minus_values.len(), base.len());
            for row in 0..analytic.len() {
                let numerical = (plus_values[row] - minus_values[row]) / (2.0 * h);
                let expected = analytic[row][column];
                let tolerance = 5.0e-6 * numerical.abs().max(expected.abs()).max(1.0);
                assert!(
                    (numerical - expected).abs() <= tolerance,
                    "row {row}, col {column}: analytic={expected:.12e}, numerical={numerical:.12e}"
                );
            }
        }
    }

    #[test]
    fn analytic_jacobian_covers_horizontal_vertical_fixed_and_arc_fixed_constraints() {
        let snapshot = constraint_only_snapshot(
            vec![
                line_item("l", 0.0, 0.5, 3.0, 2.0),
                circle_item("c", 4.0, -1.0, 1.5),
                arc_item("a", -2.0, 3.0, 2.0, 0.25, 1.35),
            ],
            vec![
                Constraint::Horizontal { entity_id: "l".into() },
                Constraint::Vertical { entity_id: "l".into() },
                Constraint::Fixed { entity_id: "l".into() },
                Constraint::Fixed { entity_id: "c".into() },
                Constraint::Fixed { entity_id: "a".into() },
            ],
        );
        assert_constraint_jacobian_matches_central_difference(&snapshot);
    }

    #[test]
    fn analytic_jacobian_covers_coincident_line_and_arc_endpoints() {
        let snapshot = constraint_only_snapshot(
            vec![
                line_item("l", 0.0, 0.0, 3.0, 1.0),
                arc_item("a", 4.0, -1.0, 2.0, 0.3, 1.4),
            ],
            vec![
                Constraint::Coincident {
                    first_geometry_id: "a".into(),
                    first_point: Endpoint::Start,
                    second_geometry_id: "l".into(),
                    second_point: Endpoint::End,
                },
                Constraint::Coincident {
                    first_geometry_id: "l".into(),
                    first_point: Endpoint::Start,
                    second_geometry_id: "a".into(),
                    second_point: Endpoint::End,
                },
            ],
        );
        assert_constraint_jacobian_matches_central_difference(&snapshot);
    }

    #[test]
    fn analytic_jacobian_covers_all_distance_constraint_parameterizations() {
        let snapshot = constraint_only_snapshot(
            vec![
                line_item("l1", 0.0, 0.0, 3.0, 1.0),
                line_item("l2", 5.0, -2.0, 7.0, 4.0),
                circle_item("c", 11.0, 2.0, 2.5),
                arc_item("a", -4.0, 3.0, 1.5, 0.3, 1.4),
            ],
            vec![
                Constraint::Distance {
                    first_geometry_id: "l1".into(),
                    second_geometry_id: None,
                    first_endpoint: None,
                    second_endpoint: None,
                    value: 3.1622776601683795,
                },
                Constraint::Distance {
                    first_geometry_id: "c".into(),
                    second_geometry_id: None,
                    first_endpoint: None,
                    second_endpoint: None,
                    value: 2.5,
                },
                Constraint::Distance {
                    first_geometry_id: "a".into(),
                    second_geometry_id: None,
                    first_endpoint: None,
                    second_endpoint: None,
                    value: 1.65,
                },
                Constraint::Distance {
                    first_geometry_id: "l1".into(),
                    second_geometry_id: Some("l2".into()),
                    first_endpoint: Some(Endpoint::End),
                    second_endpoint: Some(Endpoint::Start),
                    value: 3.0,
                },
                Constraint::Distance {
                    first_geometry_id: "a".into(),
                    second_geometry_id: Some("l2".into()),
                    first_endpoint: Some(Endpoint::Start),
                    second_endpoint: Some(Endpoint::End),
                    value: 2.0,
                },
            ],
        );
        assert_constraint_jacobian_matches_central_difference(&snapshot);
    }

    #[test]
    fn relation_rows_are_appended_after_constraint_rows() {
        let snapshot = snap(
            vec![GeometryItem {
                id: "c".into(),
                geometry: Geometry::Circle(Circle {
                    center: Point { x: 0.0, y: 0.0 },
                    radius: 2.0,
                }),
                parameter_dependencies: vec![],
            }],
            vec![],
            vec![Relation::Radius {
                geometry_id: "c".into(),
                value: 1.0,
            }],
        );
        let jacobian = analytic_constraint_jacobian(&snapshot).unwrap();
        assert_eq!(jacobian, vec![vec![0.0, 0.0, 1.0]]);
    }
}