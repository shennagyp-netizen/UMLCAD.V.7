use std::panic::{catch_unwind, AssertUnwindSafe};
use std::thread;

use umlcad_kernel_rust::{
    functions::{
        dimensions::{evaluate_dimensions, DimensionKind, DimensionSpec},
        dxf::export_dxf,
        engineering::validate_engineering,
        geometry::{Arc, Circle, Geometry, Line, Point},
        relations::{evaluate_relation, RelationResidual},
        snapshot::{Constraint, GeometryItem, ParameterDefinition, Relation, SemanticSnapshot},
        solver::{scaled_damped_qr, solve_snapshot, SolveOptions},
        spatial::{point_distance, spatial_analysis},
        topology::build_topology,
        validation::validate_snapshot,
    },
    services::engine::dispatch,
    KernelRequest,
};

fn line(id: &str, a: Point, b: Point) -> GeometryItem {
    GeometryItem {
        id: id.into(),
        geometry: Geometry::Line(Line { start: a, end: b }),
        parameter_dependencies: vec![],
    }
}

fn snapshot(
    geometry: Vec<GeometryItem>,
    constraints: Vec<(String, Constraint)>,
    relations: Vec<(String, Relation)>,
) -> SemanticSnapshot {
    SemanticSnapshot {
        parameters: vec![],
        geometry,
        constraints,
        relations,
    }
}

fn assert_zero(r: &RelationResidual) {
    assert!(r.scaled_norm < 1e-8, "scaled norm={}", r.scaled_norm);
}

#[test]
fn rejects_duplicate_and_empty_semantic_ids() {
    let duplicate = snapshot(
        vec![
            line("g", Point { x: 0.0, y: 0.0 }, Point { x: 1.0, y: 0.0 }),
            line("g", Point { x: 0.0, y: 1.0 }, Point { x: 1.0, y: 1.0 }),
        ],
        vec![],
        vec![],
    );
    assert!(validate_snapshot(&duplicate)
        .iter()
        .any(|d| d.code == "DUPLICATE_ID"));

    let empty = snapshot(
        vec![line("", Point { x: 0.0, y: 0.0 }, Point { x: 1.0, y: 0.0 })],
        vec![],
        vec![],
    );
    assert!(validate_snapshot(&empty)
        .iter()
        .any(|d| d.code == "INVALID_ID"));
}

#[test]
fn rejects_duplicate_parameters_constraints_and_relations() {
    let s = SemanticSnapshot {
        parameters: vec![
            ParameterDefinition {
                id: "p".into(),
                default_value: 1.0,
            },
            ParameterDefinition {
                id: "p".into(),
                default_value: 2.0,
            },
        ],
        geometry: vec![line(
            "g",
            Point { x: 0.0, y: 0.0 },
            Point { x: 1.0, y: 0.0 },
        )],
        constraints: vec![
            (
                "c".into(),
                Constraint::Horizontal {
                    entity_id: "g".into(),
                },
            ),
            (
                "c".into(),
                Constraint::Vertical {
                    entity_id: "g".into(),
                },
            ),
        ],
        relations: vec![
            (
                "r".into(),
                Relation::Radius {
                    geometry_id: "g".into(),
                    value: 1.0,
                },
            ),
            (
                "r".into(),
                Relation::Diameter {
                    geometry_id: "g".into(),
                    value: 2.0,
                },
            ),
        ],
    };
    let d = validate_snapshot(&s);
    assert!(d.iter().filter(|x| x.code == "DUPLICATE_ID").count() >= 3);
}

#[test]
fn rejects_bad_parameter_dependencies_and_nonfinite_values() {
    let s = SemanticSnapshot {
        parameters: vec![ParameterDefinition {
            id: "p".into(),
            default_value: f64::NAN,
        }],
        geometry: vec![GeometryItem {
            id: "g".into(),
            geometry: Geometry::Line(Line {
                start: Point { x: 0.0, y: 0.0 },
                end: Point { x: 1.0, y: 0.0 },
            }),
            parameter_dependencies: vec!["missing".into()],
        }],
        constraints: vec![(
            "d".into(),
            Constraint::Distance {
                first_geometry_id: "g".into(),
                second_geometry_id: None,
                first_endpoint: None,
                second_endpoint: None,
                value: f64::NAN,
            },
        )],
        relations: vec![(
            "a".into(),
            Relation::Angle {
                first_geometry_id: "g".into(),
                second_geometry_id: "g".into(),
                radians: f64::INFINITY,
            },
        )],
    };
    let d = validate_snapshot(&s);
    assert!(d.iter().any(|x| x.code == "STALE_PARAMETER_REFERENCE"));
    assert!(d.iter().any(|x| x.code == "INVALID_PARAMETER"));
    assert!(d.iter().any(|x| x.code == "INVALID_CONSTRAINT"));
    assert!(d.iter().any(|x| x.code == "INVALID_RELATION"));
}

#[test]
fn angle_dimension_fails_closed_on_missing_second_geometry() {
    let s = snapshot(
        vec![line(
            "a",
            Point { x: 0.0, y: 0.0 },
            Point { x: 1.0, y: 0.0 },
        )],
        vec![],
        vec![],
    );
    let result = evaluate_dimensions(
        &s,
        &[DimensionSpec {
            id: "a".into(),
            kind: DimensionKind::Angle,
            first_geometry_id: "a".into(),
            second_geometry_id: None,
            first_point: None,
            second_point: None,
        }],
    );
    assert!(result.is_err());
}

#[test]
fn large_arc_aabb_contains_sampled_curve() {
    let g = Geometry::Arc(Arc {
        center: Point { x: 10.0, y: -4.0 },
        radius: 3.0,
        start_angle: -0.25,
        end_angle: 6.0 * std::f64::consts::PI + 0.4,
    });
    assert!(g.validate().is_ok());
    let b = g.aabb();
    for i in 0..257 {
        let p = g.point_at(i as f64 / 256.0);
        assert!(p.x >= b.min.x - 1e-9 && p.x <= b.max.x + 1e-9);
        assert!(p.y >= b.min.y - 1e-9 && p.y <= b.max.y + 1e-9);
    }
}

#[test]
fn spatial_distance_is_symmetric_and_finite_across_scales() {
    for scale in [1e-9, 1e-3, 1.0, 1e3, 1e9, 1e100] {
        let a = Geometry::Line(Line {
            start: Point { x: 0.0, y: 0.0 },
            end: Point { x: scale, y: 0.0 },
        });
        let b = Geometry::Circle(Circle {
            center: Point {
                x: 2.0 * scale,
                y: scale,
            },
            radius: scale,
        });
        let x = point_distance(&a, &b);
        let y = point_distance(&b, &a);
        assert!(x.is_finite() && y.is_finite());
        assert!((x - y).abs() <= 1e-9 * x.abs().max(1.0));
    }
}

#[test]
fn obvious_intersection_is_reported_and_not_pruned() {
    let g = vec![
        line("a", Point { x: 0.0, y: 0.0 }, Point { x: 10.0, y: 0.0 }),
        line("b", Point { x: 5.0, y: -1.0 }, Point { x: 5.0, y: 1.0 }),
    ];
    let r = spatial_analysis(&g, 0.0);
    assert_eq!(r.len(), 1);
    assert!(r[0].intersects);
}

#[test]
fn solver_is_deterministic_and_input_is_immutable() {
    let s = snapshot(
        vec![line(
            "l",
            Point { x: 0.0, y: 0.0 },
            Point { x: 10.0, y: 2.0 },
        )],
        vec![(
            "h".into(),
            Constraint::Horizontal {
                entity_id: "l".into(),
            },
        )],
        vec![],
    );
    let original = s.clone();
    let a = solve_snapshot(&s, SolveOptions::default()).unwrap();
    let b = solve_snapshot(&s, SolveOptions::default()).unwrap();
    assert_eq!(a, b);
    assert_eq!(s, original);
}

#[test]
fn linear_solver_handles_near_singularity_without_nonfinite_delta() {
    let r = scaled_damped_qr(
        &vec![vec![1.0, 1.0], vec![1.0, 1.0 + 1e-14]],
        &vec![1.0, 1.0],
        1e-12,
        1e-12,
    )
    .unwrap();
    assert!(r.delta.iter().all(|x| x.is_finite()));
}

#[test]
fn linear_solver_rejects_nonfinite_inputs() {
    assert!(scaled_damped_qr(&vec![vec![f64::NAN]], [0.0].as_slice(), 1e-3, 1e-8).is_err());
    assert!(scaled_damped_qr(
        &vec![vec![1.0]],
        [f64::INFINITY].as_slice(),
        1e-3,
        1e-8
    )
    .is_err());
    assert!(
        scaled_damped_qr(&vec![vec![1.0]], [0.0].as_slice(), f64::NAN, 1e-8).is_err()
    );
}

#[test]
fn topology_is_deterministic_under_input_permutation() {
    let a = line("a", Point { x: 0.0, y: 0.0 }, Point { x: 1.0, y: 0.0 });
    let b = line("b", Point { x: 1.0, y: 0.0 }, Point { x: 2.0, y: 0.0 });
    let x = build_topology(&snapshot(vec![a.clone(), b.clone()], vec![], vec![])).unwrap();
    let y = build_topology(&snapshot(vec![b, a], vec![], vec![])).unwrap();
    assert_eq!(x, y);
}

#[test]
fn malformed_dispatch_does_not_panic() {
    let s = snapshot(
        vec![line("", Point { x: 0.0, y: 0.0 }, Point { x: 1.0, y: 0.0 })],
        vec![],
        vec![],
    );
    let result = catch_unwind(AssertUnwindSafe(|| {
        dispatch(KernelRequest::EngineeringEvidence {
            snapshot: s.clone(),
        })
    }));
    assert!(result.is_ok());
    assert!(catch_unwind(AssertUnwindSafe(|| export_dxf(&s))).is_ok());
}

#[test]
fn engineering_rejects_nonfinite_relation_semantically() {
    let s = snapshot(
        vec![
            line("a", Point { x: 0.0, y: 0.0 }, Point { x: 1.0, y: 0.0 }),
            line("b", Point { x: 0.0, y: 1.0 }, Point { x: 1.0, y: 1.0 }),
        ],
        vec![],
        vec![(
            "r".into(),
            Relation::Angle {
                first_geometry_id: "a".into(),
                second_geometry_id: "b".into(),
                radians: f64::NAN,
            },
        )],
    );
    let e = validate_engineering(&s);
    assert!(!e.engineering_rule_validity);
    assert!(!e.relation_validity);
}

#[test]
fn same_snapshot_is_safe_under_concurrent_solves() {
    let s = snapshot(
        vec![line(
            "l",
            Point { x: 0.0, y: 0.0 },
            Point { x: 10.0, y: 2.0 },
        )],
        vec![(
            "h".into(),
            Constraint::Horizontal {
                entity_id: "l".into(),
            },
        )],
        vec![],
    );
    let mut handles = Vec::new();
    for _ in 0..8 {
        let copy = s.clone();
        handles.push(thread::spawn(move || {
            solve_snapshot(&copy, SolveOptions::default()).unwrap()
        }));
    }
    let results = handles
        .into_iter()
        .map(|h| h.join().unwrap())
        .collect::<Vec<_>>();
    assert!(results.windows(2).all(|w| w[0] == w[1]));
}

#[test]
fn dispatch_dimensions_and_dxf_have_no_unchecked_panic_path() {
    let s = snapshot(
        vec![line(
            "l",
            Point { x: 0.0, y: 0.0 },
            Point { x: 3.0, y: 4.0 },
        )],
        vec![],
        vec![],
    );
    let r = dispatch(KernelRequest::Dimensions {
        snapshot: s.clone(),
        dimensions: vec![DimensionSpec {
            id: "d".into(),
            kind: DimensionKind::Length,
            first_geometry_id: "l".into(),
            second_geometry_id: None,
            first_point: None,
            second_point: None,
        }],
    })
    .unwrap();
    assert!(matches!(
        r,
        umlcad_kernel_rust::KernelResponse::Dimensions(_)
    ));
    assert!(catch_unwind(AssertUnwindSafe(|| export_dxf(&s))).is_ok());
}

#[test]
fn relation_zero_case_remains_finite() {
    let s = snapshot(
        vec![
            line("a", Point { x: 0.0, y: 0.0 }, Point { x: 1.0, y: 0.0 }),
            line("b", Point { x: 0.0, y: 1.0 }, Point { x: 1.0, y: 1.0 }),
        ],
        vec![],
        vec![],
    );
    let r = evaluate_relation(
        &s,
        &Relation::Parallel {
            first_geometry_id: "a".into(),
            second_geometry_id: "b".into(),
        },
    )
    .unwrap();
    assert_zero(&r);
    assert!(r.residuals.iter().all(|x| x.is_finite()));
}
