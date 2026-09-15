use umlcad_kernel_rust::{
    functions::{
        constraints::residual as constraint_residual,
        dimensions::{DimensionKind, DimensionSpec},
        dxf::export_dxf,
        geometry::{Arc, Circle, Geometry, Line, Point},
        relations::evaluate_relation,
        snapshot::{
            Constraint, Endpoint, GeometryItem, Relation, RelationPoint, SemanticSnapshot,
            TangentMode,
        },
        solver::{scaled_damped_qr, solve_snapshot, SolveOptions},
        validation::validate_snapshot,
    },
    services::engine::dispatch,
    KernelRequest, KernelResponse,
};

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

#[test]
fn line_geometry_is_exact() {
    let l = Line {
        start: Point { x: 0.0, y: 0.0 },
        end: Point { x: 3.0, y: 4.0 },
    };
    assert_eq!(l.length(), 5.0);
    assert_eq!(Geometry::Line(l).point_at(0.5), Point { x: 1.5, y: 2.0 });
}

#[test]
fn circle_is_closed_and_exact() {
    let c = Circle {
        center: Point { x: 0.0, y: 0.0 },
        radius: 2.0,
    };
    let g = Geometry::Circle(c);
    assert_eq!(g.query().start, g.query().end);
    assert!((c.circumference() - 4.0 * std::f64::consts::PI).abs() < 1e-12);
}

#[test]
fn arc_has_analytic_length_and_domain_validation() {
    let a = Arc {
        center: Point { x: 0.0, y: 0.0 },
        radius: 2.0,
        start_angle: 0.0,
        end_angle: std::f64::consts::PI,
    };
    assert!((a.length() - 2.0 * std::f64::consts::PI).abs() < 1e-12);
    assert!(a.validate().is_ok());
    let invalid = Arc {
        end_angle: 0.0,
        ..a
    };
    assert!(invalid.validate().is_err());
}

#[test]
fn circle_has_no_endpoint_semantics() {
    let c = Geometry::Circle(Circle {
        center: Point { x: 0.0, y: 0.0 },
        radius: 1.0,
    });
    assert!(c.query().start.distance(Point { x: 1.0, y: 0.0 }) < 1e-12);
    assert!(c.query().start.distance(c.query().end) < 1e-12);
}

#[test]
fn tangent_external_mode_is_explicit() {
    let geometry = vec![
        GeometryItem {
            id: "a".into(),
            geometry: Geometry::Circle(Circle {
                center: Point { x: 0.0, y: 0.0 },
                radius: 1.0,
            }),
            parameter_dependencies: vec![],
        },
        GeometryItem {
            id: "b".into(),
            geometry: Geometry::Circle(Circle {
                center: Point { x: 2.0, y: 0.0 },
                radius: 1.0,
            }),
            parameter_dependencies: vec![],
        },
    ];
    let v = evaluate_relation(
        &snapshot(geometry, vec![], vec![]),
        &Relation::Tangent {
            first_geometry_id: "a".into(),
            second_geometry_id: "b".into(),
            mode: TangentMode::External,
        },
    )
    .unwrap();
    assert!(v.scaled_norm < 1e-12);
}

#[test]
fn point_distance_relation_uses_explicit_endpoints() {
    let geometry = vec![
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
                start: Point { x: 0.0, y: 2.0 },
                end: Point { x: 1.0, y: 2.0 },
            }),
            parameter_dependencies: vec![],
        },
    ];
    let r = Relation::DistancePoints {
        first: RelationPoint::Endpoint {
            geometry_id: "a".into(),
            point: Endpoint::Start,
        },
        second: RelationPoint::Endpoint {
            geometry_id: "b".into(),
            point: Endpoint::Start,
        },
        value: 2.0,
    };
    assert!(
        evaluate_relation(&snapshot(geometry, vec![], vec![]), &r)
            .unwrap()
            .scaled_norm
            < 1e-12
    );
}

#[test]
fn constraint_endpoint_distance_is_preserved() {
    let geometry = vec![
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
                start: Point { x: 0.0, y: 3.0 },
                end: Point { x: 1.0, y: 3.0 },
            }),
            parameter_dependencies: vec![],
        },
    ];
    let c = Constraint::Distance {
        first_geometry_id: "a".into(),
        second_geometry_id: Some("b".into()),
        first_endpoint: Some(Endpoint::Start),
        second_endpoint: Some(Endpoint::Start),
        value: 3.0,
    };
    let s = snapshot(geometry, vec![], vec![]);
    assert!(constraint_residual(|id| s.geometry(id).cloned(), &c)
        .unwrap()
        .iter()
        .all(|x| x.abs() < 1e-12));
}

#[test]
fn rank_and_dof_are_reported_on_rectangular_system() {
    let report = scaled_damped_qr(&vec![vec![1.0, 0.0]], &vec![0.0], 1e-8, 1e-10).unwrap();
    assert_eq!(report.rank, 1);
    assert_eq!(report.degrees_of_freedom, 1);
}

#[test]
fn invalid_geometry_is_rejected() {
    let s = snapshot(
        vec![GeometryItem {
            id: "bad".into(),
            geometry: Geometry::Line(Line {
                start: Point { x: 0.0, y: 0.0 },
                end: Point { x: 0.0, y: 0.0 },
            }),
            parameter_dependencies: vec![],
        }],
        vec![],
        vec![],
    );
    assert!(validate_snapshot(&s)
        .iter()
        .any(|x| x.code == "INVALID_GEOMETRY"));
}

#[test]
fn dxf_is_deterministic() {
    let s = snapshot(
        vec![GeometryItem {
            id: "c".into(),
            geometry: Geometry::Circle(Circle {
                center: Point { x: 0.0, y: 0.0 },
                radius: 1.0,
            }),
            parameter_dependencies: vec![],
        }],
        vec![],
        vec![],
    );
    assert_eq!(export_dxf(&s), export_dxf(&s));
}

#[test]
fn snapshot_solver_converges_horizontal_line() {
    let geometry = vec![GeometryItem {
        id: "l".into(),
        geometry: Geometry::Line(Line {
            start: Point { x: 0.0, y: 0.0 },
            end: Point { x: 10.0, y: 2.0 },
        }),
        parameter_dependencies: vec![],
    }];
    let s = snapshot(
        geometry,
        vec![(
            "h".into(),
            Constraint::Horizontal {
                entity_id: "l".into(),
            },
        )],
        vec![],
    );
    let result = solve_snapshot(
        &s,
        SolveOptions {
            max_iterations: 50,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(result.converged);
    assert!(result.analysis.satisfied);
    let Geometry::Line(line) = result.geometry.iter().find(|(id, _)| id == "l").unwrap().1 else {
        panic!("wrong geometry")
    };
    assert!((line.end.y - line.start.y).abs() < 1e-7);
}

#[test]
fn snapshot_solver_keeps_valid_underconstrained_model_valid() {
    let geometry = vec![GeometryItem {
        id: "l".into(),
        geometry: Geometry::Line(Line {
            start: Point { x: 0.0, y: 0.0 },
            end: Point { x: 10.0, y: 0.0 },
        }),
        parameter_dependencies: vec![],
    }];
    let s = snapshot(
        geometry,
        vec![(
            "h".into(),
            Constraint::Horizontal {
                entity_id: "l".into(),
            },
        )],
        vec![],
    );
    let result = solve_snapshot(&s, SolveOptions::default()).unwrap();
    assert!(result.converged);
    assert!(result.analysis.degrees_of_freedom > 0);
}

#[test]
fn api_dispatch_reaches_solver_and_dimensions() {
    let s = snapshot(
        vec![GeometryItem {
            id: "l".into(),
            geometry: Geometry::Line(Line {
                start: Point { x: 0.0, y: 0.0 },
                end: Point { x: 3.0, y: 4.0 },
            }),
            parameter_dependencies: vec![],
        }],
        vec![],
        vec![],
    );
    let response = dispatch(KernelRequest::Dimensions {
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
    match response {
        KernelResponse::Dimensions(values) => assert!((values[0].value - 5.0).abs() < 1e-12),
        _ => panic!("unexpected response"),
    }
    let response = dispatch(KernelRequest::Solve {
        snapshot: s,
        options: SolveOptions::default(),
    })
    .unwrap();
    assert!(matches!(response, KernelResponse::Solve(_)));
}
