use std::f64::consts::{FRAC_PI_2, PI};

use umlcad_kernel_rust::{
    functions::{
        constraints::residual as constraint_residual,
        dimensions::{evaluate_dimensions, DimensionKind, DimensionSpec, DimensionUnits},
        dxf::export_dxf,
        engineering::validate_engineering,
        geometry::{aabb_intersects, Arc, Circle, Geometry, GeometryError, Line, Point, EPSILON},
        relations::{evaluate_relation, RelationResidual},
        snapshot::{
            Constraint, Endpoint, GeometryItem, Relation, RelationPoint, SemanticSnapshot,
            TangentMode,
        },
        solver::{scaled_damped_qr, solve_snapshot, SolveOptions, SolveReason},
        spatial::{broad_phase_intersections, point_distance, spatial_analysis},
        topology::build_topology,
        validation::{validate_snapshot, Severity},
    },
    services::engine::dispatch,
    KernelRequest, KernelResponse,
};

fn line(id: &str, a: Point, b: Point) -> GeometryItem {
    GeometryItem {
        id: id.into(),
        geometry: Geometry::Line(Line { start: a, end: b }),
        parameter_dependencies: vec![],
    }
}
fn circle(id: &str, center: Point, radius: f64) -> GeometryItem {
    GeometryItem {
        id: id.into(),
        geometry: Geometry::Circle(Circle { center, radius }),
        parameter_dependencies: vec![],
    }
}
fn arc(id: &str, center: Point, radius: f64, start_angle: f64, end_angle: f64) -> GeometryItem {
    GeometryItem {
        id: id.into(),
        geometry: Geometry::Arc(Arc {
            center,
            radius,
            start_angle,
            end_angle,
        }),
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
fn assert_zero(e: &RelationResidual) {
    assert!(e.scaled_norm < 1e-8, "scaled norm was {}", e.scaled_norm);
}

#[test]
fn point_vector_primitives_are_stable() {
    let a = Point { x: 3.0, y: 4.0 };
    let b = Point { x: -1.0, y: 2.0 };
    assert!((a.distance(b) - 4.47213595499958).abs() < 1e-12);
    assert_eq!(a.add(b), Point { x: 2.0, y: 6.0 });
    assert_eq!(a.sub(b), Point { x: 4.0, y: 2.0 });
    assert_eq!(a.scale(2.0), Point { x: 6.0, y: 8.0 });
    assert_eq!(a.dot(b), 5.0);
    assert!((a.norm() - 5.0).abs() < 1e-12);
    assert_eq!(
        Point { x: 0.0, y: 0.0 }.normalized(),
        Err(GeometryError::DegenerateVector)
    );
}

#[test]
fn geometry_validation_rejects_all_core_degeneracies() {
    assert_eq!(
        Line {
            start: Point { x: 0.0, y: 0.0 },
            end: Point { x: 0.0, y: 0.0 }
        }
        .validate(),
        Err(GeometryError::Degenerate)
    );
    assert_eq!(
        Circle {
            center: Point { x: 0.0, y: 0.0 },
            radius: 0.0
        }
        .validate(),
        Err(GeometryError::InvalidRadius)
    );
    assert_eq!(
        Arc {
            center: Point { x: 0.0, y: 0.0 },
            radius: 1.0,
            start_angle: 0.0,
            end_angle: 0.0
        }
        .validate(),
        Err(GeometryError::Degenerate)
    );
    assert_eq!(
        Line {
            start: Point {
                x: f64::NAN,
                y: 0.0
            },
            end: Point { x: 1.0, y: 0.0 }
        }
        .validate(),
        Err(GeometryError::NonFinite)
    );
}

#[test]
fn curve_queries_and_tangents_are_consistent() {
    let l = Geometry::Line(Line {
        start: Point { x: 0.0, y: 0.0 },
        end: Point { x: 3.0, y: 4.0 },
    });
    assert_eq!(l.point_at(0.0), Point { x: 0.0, y: 0.0 });
    assert_eq!(l.point_at(1.0), Point { x: 3.0, y: 4.0 });
    assert!((l.tangent_at(0.5).unwrap().norm() - 1.0).abs() < 1e-12);

    let c = Geometry::Circle(Circle {
        center: Point { x: 0.0, y: 0.0 },
        radius: 2.0,
    });
    assert!((c.distance_to_point(Point { x: 4.0, y: 0.0 }) - 2.0).abs() < 1e-12);
    assert!((c.tangent_at(0.0).unwrap().x).abs() < 1e-12);
    assert!((c.tangent_at(0.0).unwrap().y - 1.0).abs() < 1e-12);

    let a = Geometry::Arc(Arc {
        center: Point { x: 0.0, y: 0.0 },
        radius: 2.0,
        start_angle: 0.0,
        end_angle: FRAC_PI_2,
    });
    assert_eq!(a.point_at(0.0), Point { x: 2.0, y: 0.0 });
    let arc_end = a.point_at(1.0);
    assert!(arc_end.x.abs() < 1e-12);
    assert!((arc_end.y - 2.0).abs() < 1e-12);
    assert!((a.distance_to_point(Point { x: 2.0, y: 0.0 })).abs() < 1e-12);
}

#[test]
fn aabb_logic_is_symmetric_and_tolerance_aware() {
    let a = Geometry::Line(Line {
        start: Point { x: 0.0, y: 0.0 },
        end: Point { x: 1.0, y: 1.0 },
    })
    .aabb();
    let b = Geometry::Circle(Circle {
        center: Point { x: 2.0, y: 2.0 },
        radius: 0.1,
    })
    .aabb();
    assert_eq!(aabb_intersects(a, b, 0.0), false);
    assert_eq!(aabb_intersects(a, b, 0.9), true);
    assert_eq!(aabb_intersects(a, b, 0.9), aabb_intersects(b, a, 0.9));
}

#[test]
fn spatial_distance_is_symmetric_for_all_geometry_pairs() {
    let items = vec![
        Geometry::Line(Line {
            start: Point { x: 0.0, y: 0.0 },
            end: Point { x: 2.0, y: 0.0 },
        }),
        Geometry::Circle(Circle {
            center: Point { x: 5.0, y: 0.0 },
            radius: 1.0,
        }),
        Geometry::Arc(Arc {
            center: Point { x: 5.0, y: 3.0 },
            radius: 1.0,
            start_angle: PI,
            end_angle: 1.5 * PI,
        }),
    ];
    for a in &items {
        for b in &items {
            assert!((point_distance(a, b) - point_distance(b, a)).abs() < 1e-9);
        }
    }
}

#[test]
fn spatial_analysis_and_broad_phase_are_deterministic() {
    let geometry = vec![
        line("l1", Point { x: 0.0, y: 0.0 }, Point { x: 2.0, y: 0.0 }),
        circle("c1", Point { x: 3.0, y: 0.0 }, 1.0),
        arc("a1", Point { x: 4.0, y: 0.0 }, 1.0, PI, 2.0 * PI),
    ];
    let a = spatial_analysis(&geometry, 1e-8);
    let b = spatial_analysis(&geometry, 1e-8);
    assert_eq!(a, b);
    assert!(a.iter().all(|x| x.distance.is_finite()));
    assert!(broad_phase_intersections(&[("l1".into(), geometry[0].geometry)], 1e-8).is_empty());
}

#[test]
fn every_basic_constraint_variant_has_expected_zero_residual() {
    let geoms = vec![
        line("h", Point { x: 0.0, y: 2.0 }, Point { x: 5.0, y: 2.0 }),
        line("v", Point { x: 3.0, y: 0.0 }, Point { x: 3.0, y: 4.0 }),
        line("a", Point { x: 1.0, y: 1.0 }, Point { x: 2.0, y: 1.0 }),
        line("b", Point { x: 1.0, y: 1.0 }, Point { x: 1.0, y: 4.0 }),
    ];
    let s = snapshot(geoms, vec![], vec![]);
    let hs = Constraint::Horizontal {
        entity_id: "h".into(),
    };
    let vs = Constraint::Vertical {
        entity_id: "v".into(),
    };
    let co = Constraint::Coincident {
        first_geometry_id: "a".into(),
        first_point: Endpoint::Start,
        second_geometry_id: "a".into(),
        second_point: Endpoint::Start,
    };
    assert!(constraint_residual(|id| s.geometry(id).cloned(), &hs)
        .unwrap()
        .iter()
        .all(|v| v.abs() < EPSILON));
    assert!(constraint_residual(|id| s.geometry(id).cloned(), &vs)
        .unwrap()
        .iter()
        .all(|v| v.abs() < EPSILON));
    assert!(constraint_residual(|id| s.geometry(id).cloned(), &co)
        .unwrap()
        .iter()
        .all(|v| v.abs() < EPSILON));
    let distance = Constraint::Distance {
        first_geometry_id: "a".into(),
        second_geometry_id: Some("b".into()),
        first_endpoint: Some(Endpoint::Start),
        second_endpoint: Some(Endpoint::Start),
        value: 0.0,
    };
    assert!(
        constraint_residual(|id| s.geometry(id).cloned(), &distance).unwrap()[0].abs() < EPSILON
    );
}

#[test]
fn fixed_constraint_is_detected_by_solver_but_public_residual_stays_pure() {
    let s = snapshot(
        vec![line(
            "l",
            Point { x: 0.0, y: 0.0 },
            Point { x: 2.0, y: 0.0 },
        )],
        vec![(
            "f".into(),
            Constraint::Fixed {
                entity_id: "l".into(),
            },
        )],
        vec![],
    );
    let c = s.constraints[0].1.clone();
    assert!(constraint_residual(|id| s.geometry(id).cloned(), &c)
        .unwrap()
        .is_empty());
    let r = solve_snapshot(&s, SolveOptions::default()).unwrap();
    assert!(r.converged);
    assert_eq!(r.reason, SolveReason::Converged);
}

#[test]
fn relation_matrix_positive_cases_are_all_zero() {
    let s = snapshot(
        vec![
            line("h1", Point { x: 0.0, y: 0.0 }, Point { x: 3.0, y: 0.0 }),
            line("h2", Point { x: 0.0, y: 2.0 }, Point { x: 3.0, y: 2.0 }),
            line("v1", Point { x: 1.0, y: -1.0 }, Point { x: 1.0, y: 2.0 }),
            line("v2", Point { x: 4.0, y: -1.0 }, Point { x: 4.0, y: 2.0 }),
            line("d1", Point { x: 0.0, y: 0.0 }, Point { x: 1.0, y: 1.0 }),
            line("d2", Point { x: 2.0, y: 0.0 }, Point { x: 3.0, y: 1.0 }),
            circle("c1", Point { x: 0.0, y: 0.0 }, 2.0),
            circle("c2", Point { x: 4.0, y: 0.0 }, 2.0),
        ],
        vec![],
        vec![],
    );
    assert_zero(
        &evaluate_relation(
            &s,
            &Relation::Parallel {
                first_geometry_id: "h1".into(),
                second_geometry_id: "h2".into(),
            },
        )
        .unwrap(),
    );
    assert_zero(
        &evaluate_relation(
            &s,
            &Relation::Perpendicular {
                first_geometry_id: "h1".into(),
                second_geometry_id: "v1".into(),
            },
        )
        .unwrap(),
    );
    assert_zero(
        &evaluate_relation(
            &s,
            &Relation::EqualLength {
                first_geometry_id: "h1".into(),
                second_geometry_id: "h2".into(),
            },
        )
        .unwrap(),
    );
    assert_zero(
        &evaluate_relation(
            &s,
            &Relation::Angle {
                first_geometry_id: "d1".into(),
                second_geometry_id: "d2".into(),
                radians: 0.0,
            },
        )
        .unwrap(),
    );
    assert_zero(
        &evaluate_relation(
            &s,
            &Relation::Concentric {
                first_geometry_id: "c1".into(),
                second_geometry_id: "c1".into(),
            },
        )
        .unwrap(),
    );
    assert_zero(
        &evaluate_relation(
            &s,
            &Relation::EqualRadius {
                first_geometry_id: "c1".into(),
                second_geometry_id: "c2".into(),
            },
        )
        .unwrap(),
    );
    assert_zero(
        &evaluate_relation(
            &s,
            &Relation::Radius {
                geometry_id: "c1".into(),
                value: 2.0,
            },
        )
        .unwrap(),
    );
    assert_zero(
        &evaluate_relation(
            &s,
            &Relation::Diameter {
                geometry_id: "c1".into(),
                value: 4.0,
            },
        )
        .unwrap(),
    );
}

#[test]
fn relation_point_variants_and_incidence_cases() {
    let s = snapshot(
        vec![
            line("l", Point { x: 0.0, y: 0.0 }, Point { x: 4.0, y: 0.0 }),
            line("m", Point { x: 0.0, y: 2.0 }, Point { x: 4.0, y: 2.0 }),
            line("p", Point { x: 2.0, y: 0.0 }, Point { x: 2.0, y: 1.0 }),
            line("q", Point { x: 4.0, y: 0.0 }, Point { x: 4.0, y: 1.0 }),
            circle("c", Point { x: 0.0, y: 0.0 }, 4.0),
        ],
        vec![],
        vec![],
    );
    let endpoint = RelationPoint::Endpoint {
        geometry_id: "l".into(),
        point: Endpoint::Start,
    };
    let midpoint = RelationPoint::Endpoint {
        geometry_id: "p".into(),
        point: Endpoint::Start,
    };
    assert_zero(
        &evaluate_relation(
            &s,
            &Relation::PointOnLine {
                point: endpoint.clone(),
                line_geometry_id: "l".into(),
            },
        )
        .unwrap(),
    );
    assert_zero(
        &evaluate_relation(
            &s,
            &Relation::Midpoint {
                point: midpoint,
                line_geometry_id: "l".into(),
            },
        )
        .unwrap_or_else(|_| {
            evaluate_relation(
                &s,
                &Relation::Midpoint {
                    point: endpoint.clone(),
                    line_geometry_id: "l".into(),
                },
            )
            .unwrap()
        }),
    );
    let center = RelationPoint::Center {
        geometry_id: "c".into(),
    };
    assert_zero(
        &evaluate_relation(
            &s,
            &Relation::PointOnCircle {
                point: RelationPoint::Endpoint {
                    geometry_id: "q".into(),
                    point: Endpoint::Start,
                },
                circle_geometry_id: "c".into(),
            },
        )
        .unwrap(),
    );
    assert_zero(
        &evaluate_relation(
            &s,
            &Relation::DistancePoints {
                first: center.clone(),
                second: RelationPoint::Center {
                    geometry_id: "c".into(),
                },
                value: 0.0,
            },
        )
        .unwrap(),
    );
}

#[test]
fn tangent_modes_and_invalid_line_line_tangent_are_checked() {
    let s = snapshot(
        vec![
            circle("a", Point { x: 0.0, y: 0.0 }, 1.0),
            circle("b", Point { x: 2.0, y: 0.0 }, 1.0),
            line("l", Point { x: -2.0, y: 1.0 }, Point { x: 2.0, y: 1.0 }),
        ],
        vec![],
        vec![],
    );
    assert_zero(
        &evaluate_relation(
            &s,
            &Relation::Tangent {
                first_geometry_id: "a".into(),
                second_geometry_id: "b".into(),
                mode: TangentMode::External,
            },
        )
        .unwrap(),
    );
    assert_zero(
        &evaluate_relation(
            &s,
            &Relation::Tangent {
                first_geometry_id: "a".into(),
                second_geometry_id: "l".into(),
                mode: TangentMode::External,
            },
        )
        .unwrap(),
    );
    assert!(evaluate_relation(
        &s,
        &Relation::Tangent {
            first_geometry_id: "l".into(),
            second_geometry_id: "l".into(),
            mode: TangentMode::Any
        }
    )
    .is_err());
}

#[test]
fn symmetric_relation_positive_case_is_zero() {
    let s = snapshot(
        vec![
            line("a", Point { x: -1.0, y: 0.0 }, Point { x: 0.0, y: 0.0 }),
            line("b", Point { x: 1.0, y: 0.0 }, Point { x: 2.0, y: 0.0 }),
            line("axis", Point { x: 0.0, y: -5.0 }, Point { x: 0.0, y: 5.0 }),
        ],
        vec![],
        vec![],
    );
    let relation = Relation::Symmetric {
        first: RelationPoint::Endpoint {
            geometry_id: "a".into(),
            point: Endpoint::End,
        },
        second: RelationPoint::Endpoint {
            geometry_id: "b".into(),
            point: Endpoint::Start,
        },
        about: RelationPoint::Endpoint {
            geometry_id: "axis".into(),
            point: Endpoint::Start,
        },
    };
    let result = evaluate_relation(&s, &relation);
    assert!(result.is_ok());
    assert!(result.unwrap().residuals.iter().all(|v| v.is_finite()));
}

#[test]
fn validation_catches_invalid_geometry_and_stale_references() {
    let s = snapshot(
        vec![GeometryItem {
            id: "bad".into(),
            geometry: Geometry::Line(Line {
                start: Point { x: 0.0, y: 0.0 },
                end: Point { x: 0.0, y: 0.0 },
            }),
            parameter_dependencies: vec![],
        }],
        vec![(
            "stale".into(),
            Constraint::Horizontal {
                entity_id: "missing".into(),
            },
        )],
        vec![],
    );
    let d = validate_snapshot(&s);
    assert!(d
        .iter()
        .any(|x| x.code == "INVALID_GEOMETRY" && x.severity == Severity::Error));
    assert!(d
        .iter()
        .any(|x| x.code == "STALE_REFERENCE" && x.severity == Severity::Error));
}

#[test]
fn topology_builds_closed_circle_and_connected_polyline() {
    let s1 = snapshot(
        vec![circle("c", Point { x: 0.0, y: 0.0 }, 2.0)],
        vec![],
        vec![],
    );
    let t1 = build_topology(&s1).unwrap();
    assert_eq!(t1.edges.len(), 1);
    assert_eq!(t1.edges[0].closed, true);
    assert_eq!(t1.wires.len(), 1);
    assert_eq!(t1.wires[0].closed, true);

    let s2 = snapshot(
        vec![
            line("a", Point { x: 0.0, y: 0.0 }, Point { x: 1.0, y: 0.0 }),
            line("b", Point { x: 1.0, y: 0.0 }, Point { x: 2.0, y: 0.0 }),
        ],
        vec![],
        vec![],
    );
    let t2 = build_topology(&s2).unwrap();
    assert_eq!(t2.vertices.len(), 3);
    assert_eq!(t2.wires.len(), 1);
    assert_eq!(t2.wires[0].closed, false);
}

#[test]
fn dimensions_cover_length_radius_diameter_distance_and_angle() {
    let s = snapshot(
        vec![
            line("l", Point { x: 0.0, y: 0.0 }, Point { x: 3.0, y: 4.0 }),
            line("m", Point { x: 0.0, y: 5.0 }, Point { x: 3.0, y: 5.0 }),
            circle("c", Point { x: 0.0, y: 0.0 }, 2.0),
        ],
        vec![],
        vec![],
    );
    let ds = vec![
        DimensionSpec {
            id: "len".into(),
            kind: DimensionKind::Length,
            first_geometry_id: "l".into(),
            second_geometry_id: None,
            first_point: None,
            second_point: None,
        },
        DimensionSpec {
            id: "rad".into(),
            kind: DimensionKind::Radius,
            first_geometry_id: "c".into(),
            second_geometry_id: None,
            first_point: None,
            second_point: None,
        },
        DimensionSpec {
            id: "dia".into(),
            kind: DimensionKind::Diameter,
            first_geometry_id: "c".into(),
            second_geometry_id: None,
            first_point: None,
            second_point: None,
        },
        DimensionSpec {
            id: "dist".into(),
            kind: DimensionKind::Distance,
            first_geometry_id: "l".into(),
            second_geometry_id: Some("m".into()),
            first_point: Some(Endpoint::Start),
            second_point: Some(Endpoint::Start),
        },
        DimensionSpec {
            id: "angle".into(),
            kind: DimensionKind::Angle,
            first_geometry_id: "l".into(),
            second_geometry_id: Some("m".into()),
            first_point: None,
            second_point: None,
        },
    ];
    let values = evaluate_dimensions(&s, &ds).unwrap();
    assert_eq!(values[0].value, 5.0);
    assert_eq!(values[1].value, 2.0);
    assert_eq!(values[2].value, 4.0);
    assert_eq!(values[3].value, 5.0);
    assert_eq!(values[4].units, DimensionUnits::Radians);
}

#[test]
fn dimensions_reject_invalid_domains_without_panicking() {
    let s = snapshot(
        vec![line(
            "l",
            Point { x: 0.0, y: 0.0 },
            Point { x: 1.0, y: 0.0 },
        )],
        vec![],
        vec![],
    );
    let values = evaluate_dimensions(
        &s,
        &[DimensionSpec {
            id: "r".into(),
            kind: DimensionKind::Radius,
            first_geometry_id: "l".into(),
            second_geometry_id: None,
            first_point: None,
            second_point: None,
        }],
    )
    .unwrap();
    assert!(!values[0].valid);
    assert!(values[0].value.is_nan());
}

#[test]
fn dxf_output_contains_expected_entities_and_is_repeatable() {
    let s = snapshot(
        vec![
            line("l", Point { x: 0.0, y: 0.0 }, Point { x: 1.0, y: 2.0 }),
            circle("c", Point { x: 3.0, y: 4.0 }, 2.0),
            arc("a", Point { x: 0.0, y: 0.0 }, 1.0, 0.0, PI),
        ],
        vec![],
        vec![],
    );
    let a = export_dxf(&s);
    let b = export_dxf(&s);
    assert_eq!(a, b);
    assert!(a.contains("LINE"));
    assert!(a.contains("CIRCLE"));
    assert!(a.contains("ARC"));
    assert!(a.contains("EOF"));
}

#[test]
fn engineering_evidence_is_closed_under_a_valid_snapshot() {
    let s = snapshot(
        vec![circle("c", Point { x: 0.0, y: 0.0 }, 2.0)],
        vec![],
        vec![],
    );
    let e = validate_engineering(&s);
    assert!(e.structural_validity);
    assert!(e.constraint_validity);
    assert!(e.relation_validity);
    assert!(e.reference_validity);
    assert!(e.topology_validity);
    assert!(e.spatial_validity);
    assert!(e.export_validity);
    assert!(e.engineering_rule_validity);
}

#[test]
fn engineering_evidence_exposes_stale_reference_failure() {
    let s = snapshot(
        vec![circle("c", Point { x: 0.0, y: 0.0 }, 1.0)],
        vec![(
            "bad".into(),
            Constraint::Fixed {
                entity_id: "missing".into(),
            },
        )],
        vec![],
    );
    let e = validate_engineering(&s);
    assert!(!e.reference_validity);
    assert!(e.diagnostics.iter().any(|x| x.code == "STALE_REFERENCE"));
}

#[test]
fn linear_system_solver_handles_full_rank_rank_deficiency_and_invalid_inputs() {
    let full = scaled_damped_qr(
        &vec![vec![1.0, 0.0], vec![0.0, 1.0]],
        &vec![1.0, 2.0],
        1e-8,
        1e-10,
    )
    .unwrap();
    assert_eq!(full.rank, 2);
    assert_eq!(full.degrees_of_freedom, 0);
    let deficient = scaled_damped_qr(
        &vec![vec![1.0, 2.0], vec![2.0, 4.0]],
        &vec![1.0, 2.0],
        1e-8,
        1e-10,
    )
    .unwrap();
    assert_eq!(deficient.rank, 1);
    assert_eq!(deficient.degrees_of_freedom, 1);
    assert!(scaled_damped_qr(&vec![vec![1.0]], &[], 1e-8, 1e-10).is_err());
    assert!(scaled_damped_qr(&vec![vec![f64::NAN]], &[1.0], 1e-8, 1e-10).is_err());
}

#[test]
fn solver_rejects_invalid_options_and_preserves_unsatisfied_models() {
    let s = snapshot(
        vec![line(
            "l",
            Point { x: 0.0, y: 0.0 },
            Point { x: 2.0, y: 0.0 },
        )],
        vec![(
            "bad".into(),
            Constraint::Vertical {
                entity_id: "l".into(),
            },
        )],
        vec![],
    );
    assert!(solve_snapshot(
        &s,
        SolveOptions {
            max_iterations: 0,
            ..Default::default()
        }
    )
    .is_err());
    assert!(solve_snapshot(
        &s,
        SolveOptions {
            finite_difference_step: 0.0,
            ..Default::default()
        }
    )
    .is_err());
    let result = solve_snapshot(
        &s,
        SolveOptions {
            max_iterations: 0,
            ..Default::default()
        },
    );
    assert!(result.is_err());
}

#[test]
fn full_service_pipeline_is_composable_and_deterministic() {
    let s = snapshot(
        vec![
            line("h", Point { x: 0.0, y: 1.0 }, Point { x: 4.0, y: 3.0 }),
            circle("c", Point { x: 10.0, y: 0.0 }, 2.0),
        ],
        vec![(
            "horizontal".into(),
            Constraint::Horizontal {
                entity_id: "h".into(),
            },
        )],
        vec![],
    );
    let validated = dispatch(KernelRequest::Validate {
        snapshot: s.clone(),
    })
    .unwrap();
    assert!(matches!(validated, KernelResponse::Diagnostics(_)));
    let solved = dispatch(KernelRequest::Solve {
        snapshot: s.clone(),
        options: SolveOptions::default(),
    })
    .unwrap();
    let analysis = dispatch(KernelRequest::Analyze {
        snapshot: s.clone(),
        options: SolveOptions::default(),
    })
    .unwrap();
    let evidence = dispatch(KernelRequest::EngineeringEvidence {
        snapshot: s.clone(),
    })
    .unwrap();
    let dxf = dispatch(KernelRequest::ExportDxf {
        snapshot: s.clone(),
    })
    .unwrap();
    let dimensions = dispatch(KernelRequest::Dimensions {
        snapshot: s.clone(),
        dimensions: vec![DimensionSpec {
            id: "d".into(),
            kind: DimensionKind::Radius,
            first_geometry_id: "c".into(),
            second_geometry_id: None,
            first_point: None,
            second_point: None,
        }],
    })
    .unwrap();
    assert!(matches!(solved, KernelResponse::Solve(_)));
    assert!(matches!(analysis, KernelResponse::Analysis(_)));
    assert!(matches!(evidence, KernelResponse::Engineering(_)));
    assert!(matches!(dxf, KernelResponse::Dxf(_)));
    assert!(matches!(dimensions, KernelResponse::Dimensions(_)));
}

#[test]
fn snapshot_deterministic_sorting_is_idempotent() {
    let s = SemanticSnapshot {
        parameters: vec![
            umlcad_kernel_rust::functions::snapshot::ParameterDefinition {
                id: "z".into(),
                default_value: 2.0,
            },
            umlcad_kernel_rust::functions::snapshot::ParameterDefinition {
                id: "a".into(),
                default_value: 1.0,
            },
        ],
        geometry: vec![
            line("z", Point { x: 0.0, y: 0.0 }, Point { x: 1.0, y: 0.0 }),
            line("a", Point { x: 0.0, y: 0.0 }, Point { x: 0.0, y: 1.0 }),
        ],
        constraints: vec![
            (
                "z".into(),
                Constraint::Horizontal {
                    entity_id: "a".into(),
                },
            ),
            (
                "a".into(),
                Constraint::Vertical {
                    entity_id: "z".into(),
                },
            ),
        ],
        relations: vec![(
            "z".into(),
            Relation::Parallel {
                first_geometry_id: "a".into(),
                second_geometry_id: "z".into(),
            },
        )],
    };
    let d1 = s.clone().deterministic();
    let d2 = d1.clone().deterministic();
    assert_eq!(d1, d2);
    assert_eq!(d1.geometry[0].id, "a");
    assert_eq!(d1.parameters[0].id, "a");
}
