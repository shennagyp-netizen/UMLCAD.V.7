use umlcad_kernel_rust::functions::nurbs3d::{Nurbs3DError, NurbsCurve3D, Point3};

fn p(x: f64, y: f64, z: f64) -> Point3 {
    Point3 { x, y, z }
}

fn assert_point_close(actual: Point3, expected: Point3, tolerance: f64) {
    assert!((actual.x - expected.x).abs() <= tolerance);
    assert!((actual.y - expected.y).abs() <= tolerance);
    assert!((actual.z - expected.z).abs() <= tolerance);
}

#[test]
fn clamped_linear_nurbs_matches_affine_3d_oracle() {
    let curve = NurbsCurve3D::new(
        1,
        vec![p(0.0, 0.0, 0.0), p(10.0, 20.0, 30.0)],
        vec![1.0, 1.0],
        vec![0.0, 0.0, 1.0, 1.0],
    );

    for parameter in [0.0, 0.25, 0.5, 0.75, 1.0] {
        let point = curve.point_at(parameter).unwrap();
        assert!((point.x - 10.0 * parameter).abs() < 1e-12);
        assert!((point.y - 20.0 * parameter).abs() < 1e-12);
        assert!((point.z - 30.0 * parameter).abs() < 1e-12);
    }
}

#[test]
fn linear_nurbs_has_exact_constant_derivative_and_unit_tangent() {
    let curve = NurbsCurve3D::new(
        1,
        vec![p(0.0, 0.0, 0.0), p(10.0, 20.0, 30.0)],
        vec![1.0, 1.0],
        vec![0.0, 0.0, 1.0, 1.0],
    );
    let derivative = curve.derivative_at(0.37).unwrap();
    assert_eq!(derivative, p(10.0, 20.0, 30.0));
    let tangent = curve.tangent_at(0.37).unwrap();
    let magnitude = 1400.0_f64.sqrt();
    assert!((tangent.x - 10.0 / magnitude).abs() < 1e-12);
    assert!((tangent.y - 20.0 / magnitude).abs() < 1e-12);
    assert!((tangent.z - 30.0 / magnitude).abs() < 1e-12);
    assert!((tangent.x.hypot(tangent.y.hypot(tangent.z)) - 1.0).abs() < 1e-12);
}

#[test]
fn quadratic_nurbs_derivative_matches_exact_bezier_oracle() {
    let curve = NurbsCurve3D::new(
        2,
        vec![p(0.0, 0.0, 0.0), p(1.0, 2.0, 3.0), p(4.0, 1.0, 5.0)],
        vec![1.0, 1.0, 1.0],
        vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
    );
    let derivative = curve.derivative_at(0.5).unwrap();
    assert_point_close(derivative, p(4.0, 1.0, 5.0), 1e-12);
}

#[test]
fn positive_weights_preserve_the_control_point_box_property() {
    let curve = NurbsCurve3D::new(
        2,
        vec![p(-5.0, -10.0, 2.0), p(4.0, 20.0, 15.0), p(30.0, 3.0, -5.0)],
        vec![0.5, 2.0, 3.0],
        vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
    );
    let bounds = curve.control_hull_bounds().unwrap();

    for parameter in [0.0, 0.1, 0.25, 0.5, 0.75, 0.9, 1.0] {
        let point = curve.point_at(parameter).unwrap();
        assert!(point.x >= bounds.min.x - 1e-12 && point.x <= bounds.max.x + 1e-12);
        assert!(point.y >= bounds.min.y - 1e-12 && point.y <= bounds.max.y + 1e-12);
        assert!(point.z >= bounds.min.z - 1e-12 && point.z <= bounds.max.z + 1e-12);
    }
}

#[test]
fn translation_preserves_point_derivative_and_tangent_without_mutating_source() {
    let curve = NurbsCurve3D::new(
        2,
        vec![p(0.0, 0.0, 0.0), p(1.0, 2.0, 3.0), p(4.0, 1.0, 5.0)],
        vec![1.0, 2.0, 1.0],
        vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
    );
    let moved = curve.translated(100.0, -50.0, 7.0).unwrap();
    let original_point = curve.point_at(0.4).unwrap();
    let moved_point = moved.point_at(0.4).unwrap();
    let original_derivative = curve.derivative_at(0.4).unwrap();
    let moved_derivative = moved.derivative_at(0.4).unwrap();
    assert!((moved_point.x - original_point.x - 100.0).abs() < 1e-12);
    assert!((moved_point.y - original_point.y + 50.0).abs() < 1e-12);
    assert!((moved_point.z - original_point.z - 7.0).abs() < 1e-12);
    assert_point_close(moved_derivative, original_derivative, 1e-12);
    assert_point_close(
        moved.tangent_at(0.4).unwrap(),
        curve.tangent_at(0.4).unwrap(),
        1e-12,
    );
    assert_eq!(curve.point_at(0.4).unwrap(), original_point);
}

#[test]
fn endpoint_interpolation_remains_stable_with_small_positive_weight() {
    let curve = NurbsCurve3D::new(
        2,
        vec![p(3.0, 4.0, 5.0), p(9.0, 8.0, 7.0), p(10.0, 12.0, 14.0)],
        vec![1.0e-12, 3.0, 2.0],
        vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
    );
    assert_eq!(curve.point_at(0.0).unwrap(), p(3.0, 4.0, 5.0));
    assert_eq!(curve.point_at(1.0).unwrap(), p(10.0, 12.0, 14.0));
}

#[test]
fn invalid_projective_and_parameter_inputs_fail_closed() {
    let points = vec![p(0.0, 0.0, 0.0), p(1.0, 1.0, 1.0)];
    let knots = vec![0.0, 0.0, 1.0, 1.0];
    assert_eq!(
        NurbsCurve3D::new(0, points.clone(), vec![1.0, 1.0], knots.clone()).validate(),
        Err(Nurbs3DError::InvalidDegree)
    );
    assert_eq!(
        NurbsCurve3D::new(1, points.clone(), vec![1.0], knots.clone()).validate(),
        Err(Nurbs3DError::InvalidWeightCount)
    );
    assert_eq!(
        NurbsCurve3D::new(1, points.clone(), vec![1.0, 0.0], knots.clone()).validate(),
        Err(Nurbs3DError::InvalidWeight)
    );
    assert!(NurbsCurve3D::new(
        1,
        vec![p(f64::NAN, 0.0, 0.0), p(1.0, 1.0, 1.0)],
        vec![1.0, 1.0],
        knots.clone()
    )
    .validate()
    .is_err());
    assert_eq!(
        NurbsCurve3D::new(1, points, vec![1.0, 1.0], knots).point_at(f64::NAN),
        Err(Nurbs3DError::NonFinite)
    );
}
