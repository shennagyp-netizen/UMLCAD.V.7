use std::f64::consts::PI;

use approx::assert_relative_eq;
use umlcad_kernel_rust::api::{dispatch, KernelRequest, KernelResponse};
use umlcad_kernel_rust::functions::dimensions::{DimensionKind, DimensionSpec, DimensionUnits};
use umlcad_kernel_rust::functions::snapshot::{
    Constraint, Endpoint, GeometryItem, Relation, SemanticSnapshot,
};
use umlcad_kernel_rust::functions::solver::{SolveOptions, SolveReason};
use umlcad_kernel_rust::{Arc, Circle, Geometry, Line, Point};

fn line(id: &str, start: Point, end: Point) -> GeometryItem {
    GeometryItem {
        id: id.to_string(),
        geometry: Geometry::Line(Line { start, end }),
        parameter_dependencies: vec![],
    }
}

fn circle(id: &str, center: Point, radius: f64) -> GeometryItem {
    GeometryItem {
        id: id.to_string(),
        geometry: Geometry::Circle(Circle { center, radius }),
        parameter_dependencies: vec![],
    }
}

fn arc(id: &str, center: Point, radius: f64, start_angle: f64, end_angle: f64) -> GeometryItem {
    GeometryItem {
        id: id.to_string(),
        geometry: Geometry::Arc(Arc {
            center,
            radius,
            start_angle,
            end_angle,
        }),
        parameter_dependencies: vec![],
    }
}

fn valid_snapshot() -> SemanticSnapshot {
    SemanticSnapshot {
        parameters: vec![],
        geometry: vec![
            line("horizontal", Point { x: 0.0, y: 0.0 }, Point { x: 100.0, y: 0.0 }),
            line("horizontal-2", Point { x: 0.0, y: 20.0 }, Point { x: 100.0, y: 20.0 }),
            line("vertical", Point { x: 0.0, y: 0.0 }, Point { x: 0.0, y: 50.0 }),
            circle("hole", Point { x: 30.0, y: 20.0 }, 5.0),
            arc("fillet", Point { x: 20.0, y: 20.0 }, 10.0, 0.0, PI / 2.0),
        ],
        constraints: vec![],
        relations: vec![(
            "parallel".into(),
            Relation::Parallel {
                first_geometry_id: "horizontal".into(),
                second_geometry_id: "horizontal-2".into(),
            },
        )],
    }
}

#[test]
fn geometry_primitives_have_stable_numeric_contracts() {
    let l = Geometry::Line(Line {
        start: Point { x: 0.0, y: 0.0 },
        end: Point { x: 3.0, y: 4.0 },
    });
    l.validate().unwrap();
    assert_relative_eq!(l.aabb().min.x, 0.0);
    assert_relative_eq!(l.aabb().max.y, 4.0);
    assert_relative_eq!(l.point_at(0.5).x, 1.5);
    assert_relative_eq!(l.point_at(0.5).y, 2.0);
    assert_relative_eq!(l.tangent_at(0.5).unwrap().x, 0.6);
    assert_relative_eq!(l.tangent_at(0.5).unwrap().y, 0.8);
    assert_relative_eq!(l.distance_to_point(Point { x: 1.5, y: 3.0 }), 0.6);

    let c = Circle {
        center: Point { x: 10.0, y: 20.0 },
        radius: 5.0,
    };
    c.validate().unwrap();
    assert_relative_eq!(c.circumference(), 2.0 * PI * 5.0, epsilon = 1e-10);
    let cg = Geometry::Circle(c);
    assert_relative_eq!(cg.point_at(0.0).x, 15.0);
    assert_relative_eq!(cg.point_at(0.25).y, 25.0, epsilon = 1e-10);
    assert_relative_eq!(cg.tangent_at(0.0).unwrap().y, 1.0);
    assert_relative_eq!(cg.distance_to_point(Point { x: 10.0, y: 20.0 }), 5.0);

    let a = Arc {
        center: Point { x: 0.0, y: 0.0 },
        radius: 10.0,
        start_angle: 0.0,
        end_angle: PI / 2.0,
    };
    a.validate().unwrap();
    let ag = Geometry::Arc(a);
    assert_relative_eq!(ag.point_at(0.0).x, 10.0);
    assert_relative_eq!(ag.point_at(1.0).y, 10.0, epsilon = 1e-10);
    assert!(a.contains_point(Point { x: 0.0, y: 10.0 }));
    assert!(a.contains_angle(PI / 4.0));
}

#[test]
fn validate_dispatch_accepts_valid_and_rejects_degenerate_geometry() {
    match dispatch(KernelRequest::Validate {
        snapshot: valid_snapshot(),
    })
    .unwrap()
    {
        KernelResponse::Diagnostics(diagnostics) => {
            assert!(diagnostics.iter().all(|d| !matches!(
                d.severity,
                umlcad_kernel_rust::functions::validation::Severity::Error
            )));
        }
        other => panic!("unexpected response: {other:?}"),
    }

    let invalid = SemanticSnapshot {
        parameters: vec![],
        geometry: vec![line(
            "degenerate",
            Point { x: 1.0, y: 1.0 },
            Point { x: 1.0, y: 1.0 },
        )],
        constraints: vec![],
        relations: vec![],
    };
    match dispatch(KernelRequest::Validate { snapshot: invalid }).unwrap() {
        KernelResponse::Diagnostics(diagnostics) => {
            assert!(diagnostics.iter().any(|d| matches!(
                d.severity,
                umlcad_kernel_rust::functions::validation::Severity::Error
            )));
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn linear_solver_reports_rank_and_exact_delta() {
    let response = dispatch(KernelRequest::AnalyzeLinearSystem {
        jacobian: vec![vec![2.0, 0.0], vec![0.0, 4.0]],
        residuals: vec![-6.0, 8.0],
        damping: 0.0,
        rank_tolerance: 1e-12,
    })
    .unwrap();
    let KernelResponse::Linear(report) = response else {
        panic!("unexpected linear response")
    };
    assert_eq!(report.rank, 2);
    assert_eq!(report.degrees_of_freedom, 0);
    assert_relative_eq!(report.delta[0], 3.0, epsilon = 1e-10);
    assert_relative_eq!(report.delta[1], -2.0, epsilon = 1e-10);
    assert!(report.condition_number.is_finite());
}

#[test]
fn solve_and_analyze_preserve_constraints_and_return_geometry() {
    let snapshot = SemanticSnapshot {
        parameters: vec![],
        geometry: vec![line(
            "fixed-line",
            Point { x: 0.0, y: 0.0 },
            Point { x: 100.0, y: 0.0 },
        )],
        constraints: vec![(
            "fixed".into(),
            Constraint::Fixed {
                entity_id: "fixed-line".into(),
            },
        )],
        relations: vec![],
    };

    let response = dispatch(KernelRequest::Solve {
        snapshot: snapshot.clone(),
        options: SolveOptions::default(),
    })
    .unwrap();
    let KernelResponse::Solve(result) = response else {
        panic!("unexpected solve response")
    };
    assert!(result.converged);
    assert_eq!(result.reason, SolveReason::Converged);
    assert!(result.final_residual_norm <= 1e-8);
    assert_eq!(result.geometry.len(), 1);

    let response = dispatch(KernelRequest::Analyze {
        snapshot,
        options: SolveOptions::default(),
    })
    .unwrap();
    let KernelResponse::Analysis(analysis) = response else {
        panic!("unexpected analysis response")
    };
    assert!(analysis.valid);
    assert!(analysis.satisfied);
    assert_eq!(analysis.variable_count, 4);
    assert_eq!(analysis.equation_count, 4);
}

#[test]
fn dimensions_and_export_return_verifiable_results() {
    let snapshot = valid_snapshot();
    let dimensions = vec![
        DimensionSpec {
            id: "line-length".into(),
            kind: DimensionKind::Length,
            first_geometry_id: "horizontal".into(),
            second_geometry_id: None,
            first_point: None,
            second_point: None,
        },
        DimensionSpec {
            id: "hole-radius".into(),
            kind: DimensionKind::Radius,
            first_geometry_id: "hole".into(),
            second_geometry_id: None,
            first_point: None,
            second_point: None,
        },
        DimensionSpec {
            id: "hole-diameter".into(),
            kind: DimensionKind::Diameter,
            first_geometry_id: "hole".into(),
            second_geometry_id: None,
            first_point: None,
            second_point: None,
        },
        DimensionSpec {
            id: "endpoint-distance".into(),
            kind: DimensionKind::Distance,
            first_geometry_id: "horizontal".into(),
            second_geometry_id: Some("vertical".into()),
            first_point: Some(Endpoint::End),
            second_point: Some(Endpoint::End),
        },
        DimensionSpec {
            id: "right-angle".into(),
            kind: DimensionKind::Angle,
            first_geometry_id: "horizontal".into(),
            second_geometry_id: Some("vertical".into()),
            first_point: None,
            second_point: None,
        },
    ];

    match dispatch(KernelRequest::Dimensions {
        snapshot: snapshot.clone(),
        dimensions,
    })
    .unwrap()
    {
        KernelResponse::Dimensions(values) => {
            assert_eq!(values.len(), 5);
            assert_relative_eq!(values[0].value, 100.0);
            assert_eq!(values[0].units, DimensionUnits::Model);
            assert_relative_eq!(values[1].value, 5.0);
            assert_relative_eq!(values[2].value, 10.0);
            assert_relative_eq!(values[3].value, 100.0_f64.hypot(50.0));
            assert_relative_eq!(values[4].value.abs(), PI / 2.0, epsilon = 1e-10);
            assert!(values.iter().all(|x| x.valid));
        }
        other => panic!("unexpected dimension response: {other:?}"),
    }

    match dispatch(KernelRequest::EngineeringEvidence { snapshot: snapshot.clone() }).unwrap() {
        KernelResponse::Engineering(evidence) => {
            assert!(evidence.structural_validity);
            assert!(evidence.reference_validity);
            assert!(evidence.export_validity);
            assert!(evidence.spatial_validity);
        }
        other => panic!("unexpected engineering response: {other:?}"),
    }

    match dispatch(KernelRequest::ExportDxf { snapshot }).unwrap() {
        KernelResponse::Dxf(text) => {
            assert!(text.contains("SECTION"));
            assert!(text.contains("LINE"));
            assert!(text.contains("CIRCLE"));
            assert!(text.contains("ARC"));
            assert!(text.ends_with("EOF\n"));
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[test]
fn adversarial_requests_fail_closed_without_panicking() {
    let cases = [
        dispatch(KernelRequest::AnalyzeLinearSystem {
            jacobian: vec![vec![1.0]],
            residuals: vec![],
            damping: 0.0,
            rank_tolerance: 1e-12,
        }),
        dispatch(KernelRequest::AnalyzeLinearSystem {
            jacobian: vec![vec![f64::NAN]],
            residuals: vec![1.0],
            damping: 0.0,
            rank_tolerance: 1e-12,
        }),
        dispatch(KernelRequest::AnalyzeLinearSystem {
            jacobian: vec![vec![1.0]],
            residuals: vec![1.0],
            damping: -1.0,
            rank_tolerance: 1e-12,
        }),
        dispatch(KernelRequest::AnalyzeLinearSystem {
            jacobian: vec![vec![1.0]],
            residuals: vec![1.0],
            damping: 0.0,
            rank_tolerance: -1.0,
        }),
    ];
    assert!(cases.iter().all(Result::is_err));
}
