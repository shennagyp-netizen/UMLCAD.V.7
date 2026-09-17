use umlcad_kernel_rust::math::{
    constraints::{geometry_difference, residual as constraint_residual},
    geometry::{Arc, Circle, Geometry, Line, Point},
    jacobian::analytic_constraint_jacobian,
    snapshot::{Constraint, Endpoint, GeometryItem, SemanticSnapshot},
};

fn line(id: &str, sx: f64, sy: f64, ex: f64, ey: f64) -> GeometryItem {
    GeometryItem {
        id: id.into(),
        geometry: Geometry::Line(Line {
            start: Point { x: sx, y: sy },
            end: Point { x: ex, y: ey },
        }),
        parameter_dependencies: vec![],
    }
}

fn circle(id: &str, x: f64, y: f64, radius: f64) -> GeometryItem {
    GeometryItem {
        id: id.into(),
        geometry: Geometry::Circle(Circle {
            center: Point { x, y },
            radius,
        }),
        parameter_dependencies: vec![],
    }
}

fn arc(id: &str, x: f64, y: f64, radius: f64, start: f64, end: f64) -> GeometryItem {
    GeometryItem {
        id: id.into(),
        geometry: Geometry::Arc(Arc {
            center: Point { x, y },
            radius,
            start_angle: start,
            end_angle: end,
        }),
        parameter_dependencies: vec![],
    }
}

fn snapshot(geometry: Vec<GeometryItem>, constraints: Vec<Constraint>) -> SemanticSnapshot {
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

fn parameter_count(snapshot: &SemanticSnapshot) -> usize {
    snapshot
        .geometry
        .iter()
        .map(|item| match item.geometry {
            Geometry::Line(_) => 4,
            Geometry::Circle(_) => 3,
            Geometry::Arc(_) => 5,
        })
        .sum()
}

fn mutate_parameter(snapshot: &mut SemanticSnapshot, column: usize, delta: f64) {
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

fn residual_rows(snapshot: &SemanticSnapshot) -> Vec<f64> {
    snapshot
        .constraints
        .iter()
        .flat_map(|(_, constraint)| {
            constraint_residual(
                |geometry_id| snapshot.geometry(geometry_id).cloned(),
                constraint,
            )
            .unwrap_or_default()
        })
        .collect()
}

fn assert_matches_oracle(snapshot: &SemanticSnapshot) {
    let analytic = analytic_constraint_jacobian(snapshot).unwrap();
    let base = residual_rows(snapshot);
    assert_eq!(analytic.len(), base.len());
    assert_eq!(analytic.iter().map(Vec::len).collect::<Vec<_>>(), vec![parameter_count(snapshot); analytic.len()]);

    let h = 1.0e-7;
    for column in 0..parameter_count(snapshot) {
        let mut plus = snapshot.clone();
        let mut minus = snapshot.clone();
        mutate_parameter(&mut plus, column, h);
        mutate_parameter(&mut minus, column, -h);
        let plus_values = residual_rows(&plus);
        let minus_values = residual_rows(&minus);
        assert_eq!(plus_values.len(), analytic.len());
        assert_eq!(minus_values.len(), analytic.len());

        for row in 0..analytic.len() {
            let numerical = (plus_values[row] - minus_values[row]) / (2.0 * h);
            let expected = analytic[row][column];
            let scale = numerical.abs().max(expected.abs()).max(1.0);
            assert!(
                (numerical - expected).abs() <= 2.0e-5 * scale,
                "row {row}, col {column}: analytic={expected:.12e}, numerical={numerical:.12e}"
            );
        }
    }
}

#[test]
fn line_constraint_jacobians_match_central_difference() {
    assert_matches_oracle(&snapshot(
        vec![line("a", 0.2, -0.7, 3.4, 1.9)],
        vec![
            Constraint::Horizontal { entity_id: "a".into() },
            Constraint::Vertical { entity_id: "a".into() },
        ],
    ));

    assert_matches_oracle(&snapshot(
        vec![line("a", 0.2, -0.7, 3.4, 1.9), line("b", -2.0, 4.0, 1.0, -3.0)],
        vec![Constraint::Coincident {
            first_geometry_id: "a".into(),
            first_point: Endpoint::End,
            second_geometry_id: "b".into(),
            second_point: Endpoint::Start,
        }],
    ));

    assert_matches_oracle(&snapshot(
        vec![line("a", 0.2, -0.7, 3.4, 1.9), line("b", -2.0, 4.0, 1.0, -3.0)],
        vec![Constraint::Distance {
            first_geometry_id: "a".into(),
            second_geometry_id: Some("b".into()),
            first_endpoint: Some(Endpoint::End),
            second_endpoint: Some(Endpoint::Start),
            value: 2.0,
        }],
    ));

    assert_matches_oracle(&snapshot(
        vec![line("a", 0.2, -0.7, 3.4, 1.9)],
        vec![Constraint::Distance {
            first_geometry_id: "a".into(),
            second_geometry_id: None,
            first_endpoint: None,
            second_endpoint: None,
            value: 4.0,
        }],
    ));
}

#[test]
fn circular_constraint_jacobians_match_central_difference() {
    assert_matches_oracle(&snapshot(
        vec![circle("c", 1.0, -2.0, 2.5)],
        vec![Constraint::Distance {
            first_geometry_id: "c".into(),
            second_geometry_id: None,
            first_endpoint: None,
            second_endpoint: None,
            value: 3.0,
        }],
    ));

    assert_matches_oracle(&snapshot(
        vec![arc("a", 1.0, -2.0, 2.5, 0.3, 1.7)],
        vec![Constraint::Distance {
            first_geometry_id: "a".into(),
            second_geometry_id: None,
            first_endpoint: None,
            second_endpoint: None,
            value: 3.0,
        }],
    ));

    assert_matches_oracle(&snapshot(
        vec![
            arc("a", 1.0, -2.0, 2.5, 0.3, 1.7),
            line("l", -4.0, 5.0, 3.0, 5.0),
        ],
        vec![Constraint::Coincident {
            first_geometry_id: "a".into(),
            first_point: Endpoint::Start,
            second_geometry_id: "l".into(),
            second_point: Endpoint::End,
        }],
    ));
}

#[test]
fn fixed_constraint_is_exact_identity_for_each_geometry_parameter() {
    for geometry in [
        line("l", 0.0, 1.0, 2.0, 3.0),
        circle("c", 1.0, -2.0, 2.0),
        arc("a", 1.0, -2.0, 2.0, 0.2, 1.4),
    ] {
        let id = geometry.id.clone();
        let width = match geometry.geometry {
            Geometry::Line(_) => 4,
            Geometry::Circle(_) => 3,
            Geometry::Arc(_) => 5,
        };
        let jacobian = analytic_constraint_jacobian(&snapshot(
            vec![geometry],
            vec![Constraint::Fixed { entity_id: id }],
        ))
        .unwrap();
        assert_eq!(jacobian.len(), width);
        for (row_index, row) in jacobian.iter().enumerate() {
            for (column, value) in row.iter().enumerate() {
                let expected = if row_index == column { 1.0 } else { 0.0 };
                assert_eq!(*value, expected);
            }
        }
    }
}

#[test]
fn complete_composer_preserves_constraint_then_relation_order() {
    let snapshot = SemanticSnapshot {
        parameters: vec![],
        geometry: vec![circle("c", 1.0, 2.0, 2.0)],
        constraints: vec![(
            "c".into(),
            Constraint::Distance {
                first_geometry_id: "c".into(),
                second_geometry_id: None,
                first_endpoint: None,
                second_endpoint: None,
                value: 2.5,
            },
        )],
        relations: vec![],
    };
    let jacobian = analytic_constraint_jacobian(&snapshot).unwrap();
    assert_eq!(jacobian.len(), 1);
    assert_eq!(jacobian[0], vec![0.0, 0.0, 1.0]);
}

#[test]
fn degenerate_distance_and_invalid_circle_endpoint_fail_closed() {
    let coincident = snapshot(
        vec![line("a", 0.0, 0.0, 1.0, 0.0), line("b", 0.0, 0.0, 0.0, 1.0)],
        vec![Constraint::Distance {
            first_geometry_id: "a".into(),
            second_geometry_id: Some("b".into()),
            first_endpoint: Some(Endpoint::Start),
            second_endpoint: Some(Endpoint::Start),
            value: 0.0,
        }],
    );
    assert!(analytic_constraint_jacobian(&coincident).is_err());

    let invalid_endpoint = snapshot(
        vec![circle("c", 0.0, 0.0, 1.0)],
        vec![Constraint::Coincident {
            first_geometry_id: "c".into(),
            first_point: Endpoint::Start,
            second_geometry_id: "c".into(),
            second_point: Endpoint::End,
        }],
    );
    assert!(analytic_constraint_jacobian(&invalid_endpoint).is_err());
}

#[test]
fn fixed_geometry_difference_has_expected_translation_and_shape_coordinates() {
    let a = Geometry::Circle(Circle {
        center: Point { x: 3.0, y: -1.0 },
        radius: 4.0,
    });
    let b = Geometry::Circle(Circle {
        center: Point { x: 1.0, y: 2.0 },
        radius: 1.5,
    });
    assert_eq!(
        geometry_difference(&a, &b),
        Some(vec![2.0, -3.0, 2.5])
    );
}
