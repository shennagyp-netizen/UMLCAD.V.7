use umlcad_kernel_rust::functions::nurbs3d::{NurbsCurve3D, Point3};

fn basis(i: usize, degree: usize, u: f64, knots: &[f64]) -> f64 {
    if degree == 0 {
        let last = *knots.last().unwrap();
        let in_span = (knots[i] <= u && u < knots[i + 1]) || (u == last && knots[i + 1] == last);
        return if in_span { 1.0 } else { 0.0 };
    }
    let left_denominator = knots[i + degree] - knots[i];
    let right_denominator = knots[i + degree + 1] - knots[i + 1];
    let left = if left_denominator == 0.0 { 0.0 } else { (u - knots[i]) / left_denominator * basis(i, degree - 1, u, knots) };
    let right = if right_denominator == 0.0 { 0.0 } else { (knots[i + degree + 1] - u) / right_denominator * basis(i + 1, degree - 1, u, knots) };
    left + right
}

fn independent_rational_point(curve: &NurbsCurve3D, u: f64) -> Point3 {
    let mut numerator = Point3 { x: 0.0, y: 0.0, z: 0.0 };
    let mut denominator = 0.0;
    for i in 0..curve.control_points.len() {
        let weighted_basis = basis(i, curve.degree, u, &curve.knots) * curve.weights[i];
        numerator.x += weighted_basis * curve.control_points[i].x;
        numerator.y += weighted_basis * curve.control_points[i].y;
        numerator.z += weighted_basis * curve.control_points[i].z;
        denominator += weighted_basis;
    }
    Point3 { x: numerator.x / denominator, y: numerator.y / denominator, z: numerator.z / denominator }
}

fn assert_point_close(a: Point3, b: Point3, tolerance: f64) {
    assert!((a.x - b.x).abs() <= tolerance, "x mismatch: {:?} vs {:?}", a, b);
    assert!((a.y - b.y).abs() <= tolerance, "y mismatch: {:?} vs {:?}", a, b);
    assert!((a.z - b.z).abs() <= tolerance, "z mismatch: {:?} vs {:?}", a, b);
}

#[test]
fn rational_nurbs_matches_independent_cox_de_boor_oracle() {
    let curve = NurbsCurve3D::new(
        3,
        vec![
            Point3 { x: 0.0, y: 0.0, z: 0.0 },
            Point3 { x: 2.0, y: 3.0, z: -1.0 },
            Point3 { x: 5.0, y: -2.0, z: 4.0 },
            Point3 { x: 8.0, y: 6.0, z: 2.0 },
            Point3 { x: 10.0, y: 0.0, z: 5.0 },
        ],
        vec![1.0, 0.7, 1.8, 0.9, 1.2],
        vec![0.0, 0.0, 0.0, 0.0, 0.5, 1.0, 1.0, 1.0, 1.0],
    );
    for u in [0.0, 0.05, 0.17, 0.31, 0.49, 0.63, 0.79, 0.93, 1.0] {
        assert_point_close(curve.point_at(u).unwrap(), independent_rational_point(&curve, u), 1e-12);
    }
}

#[test]
fn quarter_circle_matches_closed_form_and_independent_oracle() {
    let curve = NurbsCurve3D::new(
        2,
        vec![
            Point3 { x: 1.0, y: 0.0, z: 0.0 },
            Point3 { x: 1.0, y: 1.0, z: 0.0 },
            Point3 { x: 0.0, y: 1.0, z: 0.0 },
        ],
        vec![1.0, std::f64::consts::FRAC_1_SQRT_2, 1.0],
        vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
    );
    let midpoint = curve.point_at(0.5).unwrap();
    let expected = 1.0 / std::f64::consts::SQRT_2;
    assert_point_close(midpoint, Point3 { x: expected, y: expected, z: 0.0 }, 1e-12);
    assert_point_close(midpoint, independent_rational_point(&curve, 0.5), 1e-12);
}
