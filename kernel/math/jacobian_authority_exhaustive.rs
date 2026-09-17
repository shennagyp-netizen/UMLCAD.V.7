//! Exhaustive independent regression coverage for the current analytic Jacobian authority.
//!
//! The central-difference oracle is intentionally independent of the production
//! Jacobian implementation. CPU `f64` remains normative; finite differences only
//! verify the analytic equations.

use super::{
    geometry::{Arc, Circle, Geometry, Line, Point},
    relation_jacobian::{analytic_relation_jacobian, RelationJacobianError},
    relations::evaluate_relation,
    snapshot::{Endpoint, GeometryItem, Relation, RelationPoint, SemanticSnapshot, TangentMode},
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

fn arc(
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

fn snapshot(geometry: Vec<GeometryItem>, relation: Relation) -> SemanticSnapshot {
    SemanticSnapshot {
        parameters: vec![],
        geometry,
        constraints: vec![],
        relations: vec![("r".into(), relation)],
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

fn relation_residual_rows(snapshot: &SemanticSnapshot) -> Vec<f64> {
    snapshot
        .relations
        .iter()
        .flat_map(|(_, relation)| evaluate_relation(snapshot, relation).unwrap().residuals)
        .collect()
}

fn assert_matches_central_difference(snapshot: &SemanticSnapshot) {
    let analytic = analytic_relation_jacobian(snapshot).unwrap();
    let base = relation_residual_rows(snapshot);
    assert_eq!(analytic.len(), base.len(), "analytic/residual row-count mismatch");
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
        let plus_values = relation_residual_rows(&plus);
        let minus_values = relation_residual_rows(&minus);
        assert_eq!(plus_values.len(), base.len());
        assert_eq!(minus_values.len(), base.len());
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
fn analytic_jacobian_covers_core_line_relations() {
    assert_matches_central_difference(&snapshot(
        vec![line("a", 0.0, 0.0, 3.0, 1.0), line("b", 2.0, 4.0, 5.0, 7.0)],
        Relation::Parallel {
            first_geometry_id: "a".into(),
            second_geometry_id: "b".into(),
        },
    ));
    assert_matches_central_difference(&snapshot(
        vec![line("a", 0.0, 0.0, 3.0, 1.0), line("b", 2.0, 4.0, 5.0, 7.0)],
        Relation::Perpendicular {
            first_geometry_id: "a".into(),
            second_geometry_id: "b".into(),
        },
    ));
    assert_matches_central_difference(&snapshot(
        vec![line("a", 0.0, 0.0, 3.0, 1.0), line("b", 2.0, 4.0, 5.0, 7.0)],
        Relation::EqualLength {
            first_geometry_id: "a".into(),
            second_geometry_id: "b".into(),
        },
    ));
    assert_matches_central_difference(&snapshot(
        vec![line("a", 0.0, 0.0, 3.0, 1.0), line("b", 2.0, 4.0, 5.0, 7.0)],
        Relation::Angle {
            first_geometry_id: "a".into(),
            second_geometry_id: "b".into(),
            radians: 0.35,
        },
    ));
    assert_matches_central_difference(&snapshot(
        vec![line("a", 0.0, 0.0, 3.0, 1.0), line("b", 2.0, 4.0, 5.0, 7.0)],
        Relation::Collinear {
            first_geometry_id: "a".into(),
            second_geometry_id: "b".into(),
        },
    ));
}

#[test]
fn analytic_jacobian_covers_circular_radius_and_center_relations() {
    assert_matches_central_difference(&snapshot(
        vec![circle("c", 1.0, 2.0, 3.0), arc("a", 7.0, -1.0, 1.5, 0.2, 1.1)],
        Relation::Concentric {
            first_geometry_id: "c".into(),
            second_geometry_id: "a".into(),
        },
    ));
    assert_matches_central_difference(&snapshot(
        vec![circle("c", 1.0, 2.0, 3.0), arc("a", 7.0, -1.0, 1.5, 0.2, 1.1)],
        Relation::EqualRadius {
            first_geometry_id: "c".into(),
            second_geometry_id: "a".into(),
        },
    ));
    assert_matches_central_difference(&snapshot(
        vec![arc("a", 1.0, 2.0, 3.0, 0.3, 1.4)],
        Relation::Radius {
            geometry_id: "a".into(),
            value: 2.0,
        },
    ));
    assert_matches_central_difference(&snapshot(
        vec![arc("a", 1.0, 2.0, 3.0, 0.3, 1.4)],
        Relation::Diameter {
            geometry_id: "a".into(),
            value: 6.0,
        },
    ));
}

#[test]
fn analytic_jacobian_covers_all_tangent_parameterizations_and_modes() {
    for mode in [TangentMode::External, TangentMode::Internal, TangentMode::Any] {
        assert_matches_central_difference(&snapshot(
            vec![circle("a", 0.0, 0.0, 3.0), circle("b", 8.0, 1.0, 1.0)],
            Relation::Tangent {
                first_geometry_id: "a".into(),
                second_geometry_id: "b".into(),
                mode,
            },
        ));
    }

    assert_matches_central_difference(&snapshot(
        vec![circle("a", 0.0, 0.0, 5.0), circle("b", 2.0, 0.5, 1.0)],
        Relation::Tangent {
            first_geometry_id: "a".into(),
            second_geometry_id: "b".into(),
            mode: TangentMode::Any,
        },
    ));

    let tangent_variants = [
        (
            vec![line("l", 0.0, 0.0, 3.0, 0.2), circle("c", 1.0, 2.0, 0.75)],
            "l",
            "c",
        ),
        (
            vec![line("l", 0.0, 0.0, 3.0, 0.2), arc("a", 1.0, 2.0, 0.75, 0.3, 1.2)],
            "l",
            "a",
        ),
        (
            vec![circle("c", 1.0, 2.0, 0.75), line("l", 0.0, 0.0, 3.0, 0.2)],
            "c",
            "l",
        ),
        (
            vec![arc("a", 1.0, 2.0, 0.75, 0.3, 1.2), line("l", 0.0, 0.0, 3.0, 0.2)],
            "a",
            "l",
        ),
        (
            vec![circle("a", 0.0, 0.0, 3.0), circle("b", 5.0, 1.0, 1.0)],
            "a",
            "b",
        ),
        (
            vec![circle("a", 0.0, 0.0, 3.0), arc("b", 5.0, 1.0, 1.0, 0.2, 1.0)],
            "a",
            "b",
        ),
        (
            vec![arc("a", 0.0, 0.0, 3.0, 0.2, 1.2), circle("b", 5.0, 1.0, 1.0)],
            "a",
            "b",
        ),
        (
            vec![arc("a", 0.0, 0.0, 3.0, 0.2, 1.2), arc("b", 5.0, 1.0, 1.0, 0.1, 1.1)],
            "a",
            "b",
        ),
    ];

    for (geometry, first_id, second_id) in tangent_variants {
        assert_matches_central_difference(&snapshot(
            geometry,
            Relation::Tangent {
                first_geometry_id: first_id.into(),
                second_geometry_id: second_id.into(),
                mode: TangentMode::External,
            },
        ));
    }
}

#[test]
fn analytic_jacobian_covers_relation_point_endpoint_and_center_variants() {
    assert_matches_central_difference(&snapshot(
        vec![
            line("l", 0.0, 0.0, 4.0, 2.0),
            circle("c", 3.0, 5.0, 1.0),
        ],
        Relation::Midpoint {
            point: RelationPoint::Center {
                geometry_id: "c".into(),
            },
            line_geometry_id: "l".into(),
        },
    ));
    assert_matches_central_difference(&snapshot(
        vec![
            line("l", 0.0, 0.0, 4.0, 2.0),
            circle("c", 3.0, 5.0, 1.0),
        ],
        Relation::PointOnLine {
            point: RelationPoint::Center {
                geometry_id: "c".into(),
            },
            line_geometry_id: "l".into(),
        },
    ));
    assert_matches_central_difference(&snapshot(
        vec![
            line("l", 0.0, 0.0, 4.0, 2.0),
            arc("a", 3.0, 5.0, 1.0, 0.3, 1.2),
        ],
        Relation::PointOnCircle {
            point: RelationPoint::Endpoint {
                geometry_id: "l".into(),
                point: Endpoint::Start,
            },
            circle_geometry_id: "a".into(),
        },
    ));
    assert_matches_central_difference(&snapshot(
        vec![
            arc("a", 0.0, 0.0, 2.0, 0.2, 1.4),
            circle("c", 5.0, 3.0, 1.0),
        ],
        Relation::DistancePoints {
            first: RelationPoint::Endpoint {
                geometry_id: "a".into(),
                point: Endpoint::End,
            },
            second: RelationPoint::Center {
                geometry_id: "c".into(),
            },
            value: 4.0,
        },
    ));
    assert_matches_central_difference(&snapshot(
        vec![
            arc("a", 0.0, 0.0, 2.0, 0.2, 1.4),
            circle("c", 5.0, 3.0, 1.0),
            line("l", 4.0, 4.0, 6.0, 4.0),
        ],
        Relation::Symmetric {
            first: RelationPoint::Endpoint {
                geometry_id: "a".into(),
                point: Endpoint::Start,
            },
            second: RelationPoint::Center {
                geometry_id: "c".into(),
            },
            about: RelationPoint::Center {
                geometry_id: "l".into(),
            },
        },
    ));
}

#[test]
fn zero_residual_point_on_line_remains_fail_closed() {
    let s = snapshot(
        vec![line("a", 0.0, 0.0, 4.0, 0.0), circle("c", 2.0, 0.0, 1.0)],
        Relation::PointOnLine {
            point: RelationPoint::Center {
                geometry_id: "c".into(),
            },
            line_geometry_id: "a".into(),
        },
    );
    assert_eq!(
        analytic_relation_jacobian(&s),
        Err(RelationJacobianError::Indeterminate)
    );
}

#[test]
fn zero_center_distance_tangent_remains_fail_closed() {
    let s = snapshot(
        vec![circle("a", 0.0, 0.0, 3.0), arc("b", 0.0, 0.0, 1.0, 0.2, 1.0)],
        Relation::Tangent {
            first_geometry_id: "a".into(),
            second_geometry_id: "b".into(),
            mode: TangentMode::External,
        },
    );
    assert_eq!(
        analytic_relation_jacobian(&s),
        Err(RelationJacobianError::Indeterminate)
    );
}
