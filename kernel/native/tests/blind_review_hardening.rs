use std::f64::consts::PI;

use umlcad_kernel_rust::functions::{
    geometry::{Arc, Circle, Geometry, Line, Point},
    relations::evaluate_relation,
    snapshot::{Relation, RelationPoint, SemanticSnapshot},
    solver::scaled_damped_qr,
    spatial::point_distance,
    topology::build_topology,
};

fn snapshot_with_geometry(geometry: Vec<(&str, Geometry)>) -> SemanticSnapshot {
    SemanticSnapshot {
        parameters: Vec::new(),
        geometry: geometry
            .into_iter()
            .map(|(id, geometry)| umlcad_kernel_rust::GeometryItem {
                id: id.into(),
                geometry,
                parameter_dependencies: Vec::new(),
            })
            .collect(),
        constraints: Vec::new(),
        relations: Vec::new(),
    }
}

#[test]
fn symmetric_relation_has_only_independent_residuals() {
    let snapshot = snapshot_with_geometry(vec![
        (
            "a",
            Geometry::Line(Line {
                start: Point { x: -2.0, y: 1.0 },
                end: Point { x: -2.0, y: 3.0 },
            }),
        ),
        (
            "b",
            Geometry::Line(Line {
                start: Point { x: 2.0, y: 1.0 },
                end: Point { x: 2.0, y: 3.0 },
            }),
        ),
        (
            "o",
            Geometry::Line(Line {
                start: Point { x: 0.0, y: 2.0 },
                end: Point { x: 1.0, y: 2.0 },
            }),
        ),
    ]);
    let relation = Relation::Symmetric {
        first: RelationPoint::Endpoint {
            geometry_id: "a".into(),
            point: umlcad_kernel_rust::functions::snapshot::Endpoint::Start,
        },
        second: RelationPoint::Endpoint {
            geometry_id: "b".into(),
            point: umlcad_kernel_rust::functions::snapshot::Endpoint::Start,
        },
        about: RelationPoint::Endpoint {
            geometry_id: "o".into(),
            point: umlcad_kernel_rust::functions::snapshot::Endpoint::Start,
        },
    };
    let result = evaluate_relation(&snapshot, &relation).unwrap();
    assert_eq!(result.residuals.len(), 2);
    assert_eq!(result.scales.len(), 2);
}

#[test]
fn line_arc_detects_interior_intersection() {
    let line = Geometry::Line(Line {
        start: Point { x: 5.0, y: -20.0 },
        end: Point { x: 5.0, y: 20.0 },
    });
    let arc = Geometry::Arc(Arc {
        center: Point { x: 0.0, y: 0.0 },
        radius: 10.0,
        start_angle: 0.0,
        end_angle: PI,
    });
    assert!(point_distance(&line, &arc) <= 1.0e-9);
}

#[test]
fn arc_circle_detects_interior_tangency() {
    let arc = Geometry::Arc(Arc {
        center: Point { x: 0.0, y: 0.0 },
        radius: 10.0,
        start_angle: 0.0,
        end_angle: PI,
    });
    let circle = Geometry::Circle(Circle {
        center: Point { x: 0.0, y: 11.0 },
        radius: 1.0,
    });
    assert!(point_distance(&arc, &circle) <= 1.0e-9);
}

#[test]
fn arc_arc_detects_intersection_not_at_endpoints_or_midpoints() {
    let a = Geometry::Arc(Arc {
        center: Point { x: 0.0, y: 0.0 },
        radius: 5.0,
        start_angle: 0.0,
        end_angle: PI / 2.0,
    });
    let b = Geometry::Arc(Arc {
        center: Point { x: 5.0, y: 0.0 },
        radius: 5.0,
        start_angle: 2.0,
        end_angle: 4.0,
    });
    assert!(point_distance(&a, &b) <= 1.0e-9);
}

#[test]
fn multi_wrap_arc_is_length_and_set_membership_consistent() {
    let arc = Arc {
        center: Point { x: 0.0, y: 0.0 },
        radius: 2.0,
        start_angle: 7.0 * PI,
        end_angle: 10.0 * PI,
    };
    assert_eq!(arc.length(), 6.0 * PI);
    assert!(arc.contains_point(Point { x: 0.0, y: 2.0 }));
}

#[test]
fn multi_wrap_arc_spatial_membership_is_offset_invariant() {
    let arc = Geometry::Arc(Arc {
        center: Point { x: 0.0, y: 0.0 },
        radius: 2.0,
        start_angle: 7.0 * PI,
        end_angle: 10.0 * PI,
    });
    let line = Geometry::Line(Line {
        start: Point { x: 0.0, y: -3.0 },
        end: Point { x: 0.0, y: 3.0 },
    });
    assert!(point_distance(&arc, &line) <= 1.0e-9);
}

#[test]
fn self_loop_contributes_two_to_topology_degree() {
    let snapshot = snapshot_with_geometry(vec![
        (
            "loop",
            Geometry::Arc(Arc {
                center: Point { x: 0.0, y: 0.0 },
                radius: 1.0,
                start_angle: 0.0,
                end_angle: 2.0 * PI,
            }),
        ),
        (
            "tail",
            Geometry::Line(Line {
                start: Point { x: 1.0, y: 0.0 },
                end: Point { x: 2.0, y: 0.0 },
            }),
        ),
    ]);
    let error = build_topology(&snapshot).unwrap_err();
    assert!(error.contains("degree 3"), "unexpected topology error: {error}");
}

#[test]
fn line_to_circumference_distance_is_nonzero_for_segment_inside_circle() {
    let line = Geometry::Line(Line {
        start: Point { x: -1.0, y: 0.0 },
        end: Point { x: 1.0, y: 0.0 },
    });
    let circle = Geometry::Circle(Circle {
        center: Point { x: 0.0, y: 0.0 },
        radius: 5.0,
    });
    assert!((point_distance(&line, &circle) - 4.0).abs() <= 1.0e-12);
    assert!((point_distance(&circle, &line) - 4.0).abs() <= 1.0e-12);
}

#[test]
fn zero_damping_uses_pseudoinverse_for_underdetermined_system() {
    let report = scaled_damped_qr(&[vec![1.0, 0.0]], &[1.0], 0.0, 1.0e-10).unwrap();
    assert_eq!(report.rank, 1);
    assert_eq!(report.degrees_of_freedom, 1);
    assert!((report.delta[0] + 1.0).abs() <= 1.0e-10);
    assert!(report.delta[1].abs() <= 1.0e-10);
}

#[test]
fn rank_detection_is_invariant_to_uniform_column_scaling() {
    let small = scaled_damped_qr(
        &[vec![1.0e-12, 0.0], vec![0.0, 1.0e-12]],
        &[1.0, 1.0],
        0.0,
        1.0e-10,
    )
    .unwrap();
    let large = scaled_damped_qr(
        &[vec![1.0e6, 0.0], vec![0.0, 1.0e6]],
        &[1.0, 1.0],
        0.0,
        1.0e-10,
    )
    .unwrap();
    assert_eq!((small.rank, small.degrees_of_freedom), (2, 0));
    assert_eq!((large.rank, large.degrees_of_freedom), (2, 0));
}
