use umlcad_kernel_rust::functions::{bspline, nurbs};
use nurbs::{NurbsCurve2D, NurbsError, Point2};

fn p(x: f64, y: f64) -> Point2 { Point2 { x, y } }

fn quarter_circle() -> NurbsCurve2D {
    NurbsCurve2D::new(2, vec![p(1.0, 0.0), p(1.0, 1.0), p(0.0, 1.0)], vec![1.0, 2.0_f64.sqrt() / 2.0, 1.0], vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0])
}

#[test]
fn rational_quadratic_matches_exact_quarter_circle_points() {
    let curve = quarter_circle();
    assert_eq!(curve.point_at(0.0).unwrap(), p(1.0, 0.0));
    assert_eq!(curve.point_at(1.0).unwrap(), p(0.0, 1.0));
    let midpoint = curve.point_at(0.5).unwrap();
    let expected = 2.0_f64.sqrt() / 2.0;
    assert!((midpoint.x - expected).abs() < 1e-12);
    assert!((midpoint.y - expected).abs() < 1e-12);
    assert!((midpoint.x.hypot(midpoint.y) - 1.0).abs() < 1e-12);
}

#[test]
fn rational_curve_reduces_to_bspline_when_all_weights_are_one() {
    let rational = NurbsCurve2D::new(2, vec![p(0.0, 0.0), p(1.0, 2.0), p(3.0, 0.0)], vec![1.0, 1.0, 1.0], vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0]);
    let ordinary = bspline::BSplineCurve2D::new(2, vec![bspline::Point2 { x: 0.0, y: 0.0 }, bspline::Point2 { x: 1.0, y: 2.0 }, bspline::Point2 { x: 3.0, y: 0.0 }], vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0]);
    for t in [0.0, 0.1, 0.25, 0.5, 0.75, 0.9, 1.0] {
        let a = rational.point_at(t).unwrap();
        let b = ordinary.point_at(t).unwrap();
        assert!((a.x - b.x).abs() < 1e-12);
        assert!((a.y - b.y).abs() < 1e-12);
    }
}

#[test]
fn positive_weights_keep_points_inside_the_control_hull() {
    let curve = NurbsCurve2D::new(2, vec![p(-10.0, -3.0), p(4.0, 20.0), p(30.0, 5.0), p(50.0, -4.0)], vec![0.5, 2.0, 1.5, 3.0], vec![0.0, 0.0, 0.0, 0.5, 1.0, 1.0, 1.0]);
    let bounds = curve.control_hull_bounds().unwrap();
    for t in [0.0, 0.1, 0.25, 0.5, 0.75, 0.9, 1.0] {
        let q = curve.point_at(t).unwrap();
        assert!(q.x >= bounds.min.x - 1e-12 && q.x <= bounds.max.x + 1e-12);
        assert!(q.y >= bounds.min.y - 1e-12 && q.y <= bounds.max.y + 1e-12);
    }
}

#[test]
fn translation_is_immutable_and_commutes_with_rational_evaluation() {
    let curve = quarter_circle();
    let moved = curve.translated(100.0, -50.0).unwrap();
    let original = curve.point_at(0.25).unwrap();
    let translated = moved.point_at(0.25).unwrap();
    assert_eq!(curve.point_at(0.25).unwrap(), original);
    assert!((translated.x - (original.x + 100.0)).abs() < 1e-12);
    assert!((translated.y - (original.y - 50.0)).abs() < 1e-12);
}

#[test]
fn validation_rejects_invalid_weights_and_structure() {
    let points = vec![p(0.0, 0.0), p(1.0, 1.0), p(2.0, 0.0)];
    let knots = vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
    assert_eq!(NurbsCurve2D::new(2, points.clone(), vec![1.0, 0.0, 1.0], knots.clone()).validate(), Err(NurbsError::InvalidWeight));
    assert_eq!(NurbsCurve2D::new(2, points.clone(), vec![1.0, -1.0, 1.0], knots.clone()).validate(), Err(NurbsError::InvalidWeight));
    assert!(NurbsCurve2D::new(2, points.clone(), vec![1.0, f64::NAN, 1.0], knots.clone()).validate().is_err());
    assert_eq!(NurbsCurve2D::new(2, points.clone(), vec![1.0, 1.0], knots.clone()).validate(), Err(NurbsError::InvalidWeightCount));
    assert_eq!(NurbsCurve2D::new(2, points, vec![1.0, 1.0, 1.0], vec![0.0, 0.0, 0.5, 0.25, 1.0, 1.0]).validate(), Err(NurbsError::KnotsMustBeNondecreasing));
}

#[test]
fn parameter_domain_is_fail_closed() {
    let curve = quarter_circle();
    assert_eq!(curve.point_at(-1e-12), Err(NurbsError::OutOfDomain));
    assert_eq!(curve.point_at(1.0 + 1e-12), Err(NurbsError::OutOfDomain));
    assert_eq!(curve.point_at(f64::NAN), Err(NurbsError::NonFinite));
    assert_eq!(curve.point_at(f64::INFINITY), Err(NurbsError::NonFinite));
}

#[test]
fn zero_weight_is_never_treated_as_a_valid_projective_point() {
    let curve = NurbsCurve2D::new(2, vec![p(0.0, 0.0), p(1.0, 1.0), p(2.0, 0.0)], vec![1.0, 0.0, 1.0], vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0]);
    assert!(curve.validate().is_err());
}
