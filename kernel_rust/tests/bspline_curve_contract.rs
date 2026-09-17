use umlcad_kernel_rust::functions::bspline::{BSplineCurve2D, BSplineError, Point2};

fn p(x: f64, y: f64) -> Point2 {
    Point2 { x, y }
}

fn quadratic() -> BSplineCurve2D {
    BSplineCurve2D::new(
        2,
        vec![p(0.0, 0.0), p(1.0, 1.0), p(2.0, 0.0)],
        vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
    )
}

#[test]
fn clamped_quadratic_matches_bezier_endpoint_and_midpoint_oracle() {
    let curve = quadratic();
    assert_eq!(curve.point_at(0.0).unwrap(), p(0.0, 0.0));
    assert_eq!(curve.point_at(1.0).unwrap(), p(2.0, 0.0));
    let midpoint = curve.point_at(0.5).unwrap();
    assert!((midpoint.x - 1.0).abs() < 1e-12);
    assert!((midpoint.y - 0.5).abs() < 1e-12);
}

#[test]
fn linear_clamped_bspline_is_exact_linear_interpolation() {
    let curve = BSplineCurve2D::new(1, vec![p(-2.0, 3.0), p(6.0, 11.0)], vec![0.0, 0.0, 1.0, 1.0]);
    assert_eq!(curve.parameter_domain().unwrap(), (0.0, 1.0));
    let q = curve.point_at(0.25).unwrap();
    assert!((q.x - 0.0).abs() < 1e-12);
    assert!((q.y - 5.0).abs() < 1e-12);
}

#[test]
fn bspline_points_stay_inside_the_control_hull() {
    let curve = BSplineCurve2D::new(2, vec![p(0.0, 0.0), p(10.0, 20.0), p(20.0, -10.0), p(30.0, 5.0)], vec![0.0, 0.0, 0.0, 0.5, 1.0, 1.0, 1.0]);
    let bounds = curve.control_hull_bounds().unwrap();
    for t in [0.0, 0.1, 0.25, 0.5, 0.75, 0.9, 1.0] {
        let q = curve.point_at(t).unwrap();
        assert!(q.x >= bounds.min.x - 1e-12 && q.x <= bounds.max.x + 1e-12);
        assert!(q.y >= bounds.min.y - 1e-12 && q.y <= bounds.max.y + 1e-12);
    }
}

#[test]
fn translation_is_immutable_and_commutes_with_evaluation() {
    let curve = quadratic();
    let moved = curve.translated(100.0, -25.0).unwrap();
    let original = curve.point_at(0.25).unwrap();
    let translated = moved.point_at(0.25).unwrap();
    assert_eq!(curve.point_at(0.25).unwrap(), original);
    assert!((translated.x - (original.x + 100.0)).abs() < 1e-12);
    assert!((translated.y - (original.y - 25.0)).abs() < 1e-12);
}

#[test]
fn validation_rejects_invalid_degree_knot_structure_and_control_points() {
    let base_points = vec![p(0.0, 0.0), p(1.0, 1.0), p(2.0, 0.0)];
    let clamped_knots = vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0];

    assert_eq!(BSplineCurve2D::new(0, base_points.clone(), clamped_knots.clone()).validate(), Err(BSplineError::InvalidDegree));
    assert_eq!(BSplineCurve2D::new(3, base_points.clone(), clamped_knots.clone()).validate(), Err(BSplineError::InvalidControlPointCount));
    assert_eq!(BSplineCurve2D::new(2, base_points.clone(), vec![0.0, 0.0, 1.0]).validate(), Err(BSplineError::InvalidKnotCount));
    assert_eq!(BSplineCurve2D::new(2, base_points.clone(), vec![0.0, 0.0, 0.5, 0.25, 1.0, 1.0]).validate(), Err(BSplineError::KnotsMustBeNondecreasing));
    assert_eq!(BSplineCurve2D::new(2, base_points.clone(), vec![0.0, 0.0, 0.0, 1.0, 1.0, 2.0]).validate(), Err(BSplineError::NotClamped));
    assert!(BSplineCurve2D::new(2, vec![p(f64::NAN, 0.0), p(1.0, 1.0), p(2.0, 0.0)], clamped_knots.clone()).validate().is_err());
    assert!(BSplineCurve2D::new(2, base_points, vec![0.0, 0.0, 0.0, 1.0, f64::INFINITY, 1.0]).validate().is_err());
}

#[test]
fn parameter_domain_is_fail_closed() {
    let curve = quadratic();
    assert_eq!(curve.point_at(-1e-12), Err(BSplineError::OutOfDomain));
    assert_eq!(curve.point_at(1.0 + 1e-12), Err(BSplineError::OutOfDomain));
    assert_eq!(curve.point_at(f64::NAN), Err(BSplineError::NonFinite));
    assert_eq!(curve.point_at(f64::INFINITY), Err(BSplineError::NonFinite));
}

#[test]
fn repeated_internal_knot_is_supported_without_breaking_the_curve_domain() {
    let curve = BSplineCurve2D::new(
        2,
        vec![p(0.0, 0.0), p(1.0, 2.0), p(2.0, -1.0), p(3.0, 0.0), p(4.0, 1.0)],
        vec![0.0, 0.0, 0.0, 0.5, 0.5, 1.0, 1.0, 1.0],
    );
    curve.validate().unwrap();
    for t in [0.0, 0.25, 0.49, 0.5, 0.51, 0.75, 1.0] {
        let point = curve.point_at(t).unwrap();
        assert!(point.x.is_finite() && point.y.is_finite());
    }
}
