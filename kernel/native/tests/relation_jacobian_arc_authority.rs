use umlcad_kernel_rust::math::{
    geometry::{Arc, Circle, Geometry, Line, Point},
    jacobian::analytic_constraint_jacobian,
    relation_jacobian::analytic_relation_jacobian,
    relations::evaluate_relation,
    snapshot::{Endpoint, GeometryItem, Relation, RelationPoint, SemanticSnapshot, TangentMode},
};

fn arc(id: &str, cx: f64, cy: f64, radius: f64, start: f64, end: f64) -> GeometryItem {
    GeometryItem {
        id: id.into(),
        geometry: Geometry::Arc(Arc {
            center: Point { x: cx, y: cy },
            radius,
            start_angle: start,
            end_angle: end,
        }),
        parameter_dependencies: vec![],
    }
}

fn circle(id: &str, cx: f64, cy: f64, radius: f64) -> GeometryItem {
    GeometryItem {
        id: id.into(),
        geometry: Geometry::Circle(Circle {
            center: Point { x: cx, y: cy },
            radius,
        }),
        parameter_dependencies: vec![],
    }
}

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

fn snapshot(geometry: Vec<GeometryItem>, relation: Relation) -> SemanticSnapshot {
    SemanticSnapshot {
        parameters: vec![],
        geometry,
        constraints: vec![],
        relations: vec![("r".into(), relation)],
    }
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
        .relations
        .iter()
        .flat_map(|(_, relation)| evaluate_relation(snapshot, relation).unwrap().residuals)
        .collect()
}

fn assert_matches_central_difference(snapshot: &SemanticSnapshot) {
    let analytic = analytic_relation_jacobian(snapshot).unwrap();
    let base = residual_rows(snapshot);
    let total = analytic.first().map_or(0, Vec::len);
    assert_eq!(analytic.iter().map(Vec::len).collect::<Vec<_>>(), vec![total; analytic.len()]);
    assert_eq!(analytic.len(), base.len());

    let h = 1.0e-7;
    for column in 0..total {
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
fn arc_radius_and_diameter_derivatives_match_independent_oracle() {
    for relation in [
        Relation::Radius {
            geometry_id: "a".into(),
            value: 3.0,
        },
        Relation::Diameter {
            geometry_id: "a".into(),
            value: 6.0,
        },
    ] {
        assert_matches_central_difference(&snapshot(
            vec![arc("a", 1.0, -2.0, 2.5, 0.3, 1.7)],
            relation,
        ));
    }
}

#[test]
fn arc_circle_center_and_radius_derivatives_match_independent_oracle() {
    assert_matches_central_difference(&snapshot(
        vec![
            arc("a", 1.0, -2.0, 2.5, 0.3, 1.7),
            circle("c", 7.0, 3.0, 1.25),
        ],
        Relation::Concentric {
            first_geometry_id: "a".into(),
            second_geometry_id: "c".into(),
        },
    ));

    assert_matches_central_difference(&snapshot(
        vec![
            arc("a", 1.0, -2.0, 2.5, 0.3, 1.7),
            circle("c", 7.0, 3.0, 1.25),
        ],
        Relation::EqualRadius {
            first_geometry_id: "a".into(),
            second_geometry_id: "c".into(),
        },
    ));

    assert_matches_central_difference(&snapshot(
        vec![
            arc("a", 1.0, -2.0, 2.5, 0.3, 1.7),
            circle("c", 7.0, 3.0, 1.25),
        ],
        Relation::EqualLength {
            first_geometry_id: "a".into(),
            second_geometry_id: "c".into(),
        },
    ));
}

#[test]
fn arc_endpoint_relations_match_independent_oracle() {
    let point = RelationPoint::Endpoint {
        geometry_id: "a".into(),
        point: Endpoint::Start,
    };

    assert_matches_central_difference(&snapshot(
        vec![
            arc("a", 1.0, -2.0, 2.5, 0.3, 1.7),
            line("l", -4.0, 5.0, 3.0, 5.0),
        ],
        Relation::Midpoint {
            point: point.clone(),
            line_geometry_id: "l".into(),
        },
    ));

    assert_matches_central_difference(&snapshot(
        vec![
            arc("a", 1.0, -2.0, 2.5, 0.3, 1.7),
            line("l", -4.0, 5.0, 3.0, 5.0),
        ],
        Relation::PointOnLine {
            point: point.clone(),
            line_geometry_id: "l".into(),
        },
    ));

    assert_matches_central_difference(&snapshot(
        vec![
            arc("a", 1.0, -2.0, 2.5, 0.3, 1.7),
            circle("c", 7.0, 3.0, 1.25),
        ],
        Relation::PointOnCircle {
            point: point.clone(),
            circle_geometry_id: "c".into(),
        },
    ));

    assert_matches_central_difference(&snapshot(
        vec![
            arc("a", 1.0, -2.0, 2.5, 0.3, 1.7),
            circle("c", 7.0, 3.0, 1.25),
        ],
        Relation::DistancePoints {
            first: point,
            second: RelationPoint::Center {
                geometry_id: "c".into(),
            },
            value: 5.0,
        },
    ));
}

#[test]
fn arc_circle_tangent_modes_match_independent_oracle_away_from_branch_switches() {
    let geometries = vec![
        arc("a", 0.0, 0.0, 2.0, 0.2, 1.3),
        circle("c", 5.0, 1.0, 0.75),
    ];

    for mode in [TangentMode::External, TangentMode::Internal, TangentMode::Any] {
        assert_matches_central_difference(&snapshot(
            geometries.clone(),
            Relation::Tangent {
                first_geometry_id: "a".into(),
                second_geometry_id: "c".into(),
                mode,
            },
        ));
    }
}

#[test]
fn arc_constraints_and_relation_rows_share_the_same_complete_composer() {
    let snapshot = SemanticSnapshot {
        parameters: vec![],
        geometry: vec![
            arc("a", 1.0, -2.0, 2.5, 0.3, 1.7),
            circle("c", 7.0, 3.0, 1.25),
        ],
        constraints: vec![],
        relations: vec![
            (
                "r".into(),
                Relation::Radius {
                    geometry_id: "a".into(),
                    value: 2.5,
                },
            ),
        ],
    };
    let complete = analytic_constraint_jacobian(&snapshot).unwrap();
    let relations = analytic_relation_jacobian(&snapshot).unwrap();
    assert_eq!(complete, relations);
}
