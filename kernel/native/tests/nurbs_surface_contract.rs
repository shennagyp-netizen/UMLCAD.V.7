use umlcad_kernel_rust::functions::nurbs_surface::{NurbsSurface2D, NurbsSurfaceError, Point3};

fn p(x: f64, y: f64, z: f64) -> Point3 {
    Point3 { x, y, z }
}

fn assert_point_close(actual: Point3, expected: Point3, tolerance: f64) {
    assert!((actual.x - expected.x).abs() <= tolerance);
    assert!((actual.y - expected.y).abs() <= tolerance);
    assert!((actual.z - expected.z).abs() <= tolerance);
}

#[test]
fn bilinear_surface_matches_plane_oracle() {
    let surface = NurbsSurface2D::new(
        1,
        1,
        vec![
            p(0.0, 0.0, 0.0),
            p(0.0, 1.0, 1.0),
            p(1.0, 0.0, 2.0),
            p(1.0, 1.0, 3.0),
        ],
        vec![1.0; 4],
        vec![0.0, 0.0, 1.0, 1.0],
        vec![0.0, 0.0, 1.0, 1.0],
    );
    let point = surface.point_at(0.25, 0.75).unwrap();
    assert_point_close(point, p(0.25, 0.75, 1.25), 1e-12);
}

#[test]
fn rational_weight_changes_surface_geometry() {
    let surface = NurbsSurface2D::new(
        1,
        1,
        vec![
            p(0.0, 0.0, 0.0),
            p(0.0, 1.0, 0.0),
            p(1.0, 0.0, 0.0),
            p(1.0, 1.0, 1.0),
        ],
        vec![1.0, 1.0, 1.0, 2.0],
        vec![0.0, 0.0, 1.0, 1.0],
        vec![0.0, 0.0, 1.0, 1.0],
    );
    let point = surface.point_at(0.5, 0.5).unwrap();
    assert_point_close(point, p(0.6, 0.6, 0.4), 1e-12);
}

#[test]
fn positive_weights_preserve_control_box_property() {
    let surface = NurbsSurface2D::new(
        2,
        2,
        vec![
            p(0.0, 0.0, -1.0),
            p(0.0, 1.0, 2.0),
            p(0.0, 2.0, 0.0),
            p(1.0, 0.0, 3.0),
            p(1.0, 1.0, 5.0),
            p(1.0, 2.0, 1.0),
            p(2.0, 0.0, 0.0),
            p(2.0, 1.0, 4.0),
            p(2.0, 2.0, -2.0),
        ],
        vec![1.0; 9],
        vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
        vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
    );
    let bounds = surface.control_hull_bounds().unwrap();

    for (u, v) in [
        (0.0, 0.0),
        (0.2, 0.3),
        (0.5, 0.5),
        (0.8, 0.7),
        (1.0, 1.0),
    ] {
        let point = surface.point_at(u, v).unwrap();
        assert!(point.x >= bounds.min.x - 1e-12 && point.x <= bounds.max.x + 1e-12);
        assert!(point.y >= bounds.min.y - 1e-12 && point.y <= bounds.max.y + 1e-12);
        assert!(point.z >= bounds.min.z - 1e-12 && point.z <= bounds.max.z + 1e-12);
    }
}

#[test]
fn translation_is_immutable() {
    let surface = NurbsSurface2D::new(
        1,
        1,
        vec![
            p(0.0, 0.0, 0.0),
            p(0.0, 1.0, 1.0),
            p(1.0, 0.0, 2.0),
            p(1.0, 1.0, 3.0),
        ],
        vec![1.0; 4],
        vec![0.0, 0.0, 1.0, 1.0],
        vec![0.0, 0.0, 1.0, 1.0],
    );
    let moved = surface.translated(100.0, -20.0, 7.0).unwrap();
    let original = surface.point_at(0.3, 0.7).unwrap();
    let translated = moved.point_at(0.3, 0.7).unwrap();
    assert_point_close(
        translated,
        p(original.x + 100.0, original.y - 20.0, original.z + 7.0),
        1e-12,
    );
    assert_eq!(surface.point_at(0.3, 0.7).unwrap(), original);
}

#[test]
fn invalid_inputs_fail_closed() {
    let points = vec![
        p(0.0, 0.0, 0.0),
        p(0.0, 1.0, 0.0),
        p(1.0, 0.0, 0.0),
        p(1.0, 1.0, 0.0),
    ];
    let weights = vec![1.0; 4];
    let knots = vec![0.0, 0.0, 1.0, 1.0];

    assert_eq!(
        NurbsSurface2D::new(
            0,
            1,
            points.clone(),
            weights.clone(),
            knots.clone(),
            knots.clone(),
        )
        .validate(),
        Err(NurbsSurfaceError::InvalidDegree)
    );
    assert_eq!(
        NurbsSurface2D::new(
            1,
            1,
            points.clone(),
            vec![1.0, 1.0],
            knots.clone(),
            knots.clone(),
        )
        .validate(),
        Err(NurbsSurfaceError::InvalidWeightCount)
    );
    assert_eq!(
        NurbsSurface2D::new(
            1,
            1,
            points,
            weights,
            vec![0.0, 0.5, 0.25, 1.0],
            knots,
        )
        .validate(),
        Err(NurbsSurfaceError::KnotsMustBeNondecreasing)
    );
}
