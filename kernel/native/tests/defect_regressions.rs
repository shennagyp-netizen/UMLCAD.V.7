use umlcad_kernel_rust::functions::{
    curves3d::{Circle3D, Point3, Vec3},
    geometry::{Geometry, Line, Point},
    nurbs_surface::{NurbsSurface2D, Point3 as SurfacePoint3},
    nurbs_surface_differential::NurbsSurfaceDifferential,
    snapshot::{Constraint, Endpoint, GeometryItem, SemanticSnapshot},
    solver::{scaled_damped_qr, solve_snapshot, SolveOptions},
    spatial::point_distance,
    topology::build_topology,
};

fn snapshot(geometry: Vec<GeometryItem>, constraints: Vec<(String, Constraint)>) -> SemanticSnapshot {
    SemanticSnapshot {
        parameters: vec![],
        geometry,
        constraints,
        relations: vec![],
    }
}

#[test]
fn circle3d_distance_uses_a_unit_normal_for_axial_projection() {
    let epsilon = 5.0e-10;
    let circle = Circle3D {
        center: Point3 { x: 0.0, y: 0.0, z: 0.0 },
        radius: 1.0,
        normal: Vec3 { x: 0.0, y: 0.0, z: 1.0 + epsilon },
    };

    assert!(circle.validate().is_ok());
    let distance = circle
        .distance_to_point(Point3 { x: 0.5, y: 0.0, z: 1.0 })
        .unwrap();
    assert!((distance - 1.25_f64.sqrt()).abs() < 1.0e-12, "distance={distance}");
}

#[test]
fn topology_vertex_deduplication_scales_with_model_extent() {
    // The gap is 5e-9 while the model extent is about 2e-6. A fixed 1e-8
    // tolerance incorrectly merged these two distinct vertices.
    let gap = 5.0e-9;
    let a = GeometryItem {
        id: "a".into(),
        geometry: Geometry::Line(Line {
            start: Point { x: 0.0, y: 0.0 },
            end: Point { x: 1.0e-6, y: 0.0 },
        }),
        parameter_dependencies: vec![],
    };
    let b = GeometryItem {
        id: "b".into(),
        geometry: Geometry::Line(Line {
            start: Point { x: 1.0e-6 + gap, y: 0.0 },
            end: Point { x: 2.0e-6 + gap, y: 0.0 },
        }),
        parameter_dependencies: vec![],
    };

    let topology = build_topology(&snapshot(vec![a, b], vec![])).unwrap();
    assert_eq!(topology.vertices.len(), 4);
}

#[test]
fn solver_rejects_a_nonfinite_finite_difference_probe() {
    let near_max = f64::MAX * 0.99999995;
    assert!((near_max + 1.0e-7 * near_max).is_infinite());
    let geometry = vec![
        GeometryItem {
            id: "a".into(),
            geometry: Geometry::Line(Line {
                start: Point { x: near_max, y: 0.0 },
                end: Point { x: near_max * 0.5, y: 1.0 },
            }),
            parameter_dependencies: vec![],
        },
        GeometryItem {
            id: "b".into(),
            geometry: Geometry::Line(Line {
                start: Point { x: 0.0, y: 0.0 },
                end: Point { x: 1.0, y: 1.0 },
            }),
            parameter_dependencies: vec![],
        },
    ];
    let constraints = vec![(
        "coincident".into(),
        Constraint::Coincident {
            first_geometry_id: "a".into(),
            first_point: Endpoint::Start,
            second_geometry_id: "b".into(),
            second_point: Endpoint::Start,
        },
    )];

    let result = solve_snapshot(
        &snapshot(geometry, constraints),
        SolveOptions {
            max_iterations: 1,
            ..Default::default()
        },
    );

    let error = result.expect_err("non-finite finite-difference probe must be rejected");
    assert!(error.contains("finite-difference probe") || error.contains("solver parameter"));
}

#[test]
fn normalized_solver_condition_estimate_is_invariant_to_column_units() {
    let base = vec![vec![1.0, 2.0], vec![3.0, 4.0]];
    let rescaled = vec![vec![1.0e9, 2.0e-9], vec![3.0e9, 4.0e-9]];

    let a = scaled_damped_qr(&base, &[1.0, -2.0], 1.0e-3, 1.0e-12).unwrap();
    let b = scaled_damped_qr(&rescaled, &[1.0, -2.0], 1.0e-3, 1.0e-12).unwrap();

    assert!((a.condition_number - b.condition_number).abs() < 1.0e-12);
    assert_eq!(a.rank, b.rank);
}

#[test]
fn repeated_knot_surface_u_derivative_remains_finite_at_c0_join() {
    let surface = NurbsSurface2D::new(
        2,
        1,
        vec![
            SurfacePoint3 { x: 0.0, y: 0.0, z: 0.0 },
            SurfacePoint3 { x: 0.0, y: 1.0, z: 0.0 },
            SurfacePoint3 { x: 0.5, y: 0.0, z: 0.0 },
            SurfacePoint3 { x: 0.5, y: 1.0, z: 0.0 },
            SurfacePoint3 { x: 1.0, y: 0.0, z: 0.0 },
            SurfacePoint3 { x: 1.0, y: 1.0, z: 0.0 },
        ],
        vec![1.0; 6],
        vec![0.0, 0.0, 0.0, 0.5, 0.5, 1.0, 1.0, 1.0],
        vec![0.0, 0.0, 1.0, 1.0],
    );

    let derivative = surface.derivative_u_at(0.5, 0.5).unwrap();
    assert!(derivative.x.is_finite() && derivative.y.is_finite() && derivative.z.is_finite());
}

#[test]
fn full_circle_arc_and_arc_arc_queries_remain_geometric() {
    let full = Geometry::Arc(umlcad_kernel_rust::functions::geometry::Arc {
        center: Point { x: 0.0, y: 0.0 },
        radius: 1.0,
        start_angle: 0.0,
        end_angle: std::f64::consts::TAU,
    });
    let circle = Geometry::Circle(umlcad_kernel_rust::functions::geometry::Circle {
        center: Point { x: 3.0, y: 0.0 },
        radius: 1.0,
    });
    let full_same = full;

    assert!((point_distance(&full, &circle) - 1.0).abs() < 1.0e-12);
    assert!(point_distance(&full_same, &full).abs() < 1.0e-12);
}
