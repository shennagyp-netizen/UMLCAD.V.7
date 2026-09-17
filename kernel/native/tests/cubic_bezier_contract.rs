use umlcad_kernel_rust::functions::bezier::{CubicBezier, Point2};

fn p(x: f64, y: f64) -> Point2 {
    Point2 { x, y }
}

#[test]
fn cubic_bezier_interpolates_exact_endpoints_and_midpoint() {
    let curve = CubicBezier::new(p(0.0, 0.0), p(3.0, 0.0), p(3.0, 3.0), p(0.0, 3.0));
    assert_eq!(curve.point_at(0.0).unwrap(), p(0.0, 0.0));
    assert_eq!(curve.point_at(1.0).unwrap(), p(0.0, 3.0));
    let midpoint = curve.point_at(0.5).unwrap();
    assert!((midpoint.x - 2.25).abs() < 1e-12);
    assert!((midpoint.y - 1.5).abs() < 1e-12);
}

#[test]
fn cubic_bezier_derivative_and_unit_tangent_are_exact() {
    let curve = CubicBezier::new(p(0.0, 0.0), p(2.0, 0.0), p(2.0, 2.0), p(0.0, 2.0));
    let derivative = curve.derivative_at(0.0).unwrap();
    assert_eq!(derivative, p(6.0, 0.0));
    let tangent = curve.tangent_at(0.0).unwrap();
    assert_eq!(tangent, p(1.0, 0.0));
}

#[test]
fn cubic_bezier_bounding_box_uses_analytic_derivative_extrema_not_samples() {
    let curve = CubicBezier::new(p(0.0, 0.0), p(10.0, 10.0), p(-10.0, 10.0), p(0.0, 0.0));
    let bounds = curve.bounding_box().unwrap();
    assert!((bounds.min.x + 2.886751345948129).abs() < 1e-12);
    assert!(bounds.max.x > 2.886751345948129 - 1e-12);
    assert_eq!(bounds.min.y, 0.0);
    assert_eq!(bounds.max.y, 7.5);
}

#[test]
fn cubic_bezier_parameter_domain_and_input_validation_are_fail_closed() {
    let curve = CubicBezier::new(p(0.0, 0.0), p(1.0, 0.0), p(1.0, 1.0), p(0.0, 1.0));
    assert!(curve.validate().is_ok());
    assert!(curve.point_at(-1e-12).is_err());
    assert!(curve.point_at(1.0 + 1e-12).is_err());
    assert!(
        CubicBezier::new(p(f64::NAN, 0.0), p(1.0, 0.0), p(1.0, 1.0), p(0.0, 1.0))
            .validate()
            .is_err()
    );
}

#[test]
fn cubic_bezier_rejects_identically_collapsed_geometry_and_zero_tangent() {
    let collapsed = CubicBezier::new(p(1.0, 1.0), p(1.0, 1.0), p(1.0, 1.0), p(1.0, 1.0));
    assert!(collapsed.validate().is_err());
    let stationary = CubicBezier::new(p(0.0, 0.0), p(0.0, 0.0), p(1.0, 1.0), p(1.0, 1.0));
    assert!(stationary.tangent_at(0.0).is_err());
}

#[test]
fn cubic_bezier_is_immutable_and_translation_commutes_with_evaluation() {
    let curve = CubicBezier::new(p(1.0, 2.0), p(4.0, 2.0), p(4.0, 5.0), p(1.0, 5.0));
    let moved = curve.translated(100.0, -50.0).unwrap();
    let source_mid = curve.point_at(0.4).unwrap();
    let moved_mid = moved.point_at(0.4).unwrap();
    assert_eq!(curve.point_at(0.4).unwrap(), source_mid);
    assert!((moved_mid.x - source_mid.x - 100.0).abs() < 1e-12);
    assert!((moved_mid.y - source_mid.y + 50.0).abs() < 1e-12);
}
