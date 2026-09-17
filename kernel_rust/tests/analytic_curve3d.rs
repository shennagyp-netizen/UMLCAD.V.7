use std::f64::consts::PI;

use umlcad_kernel_rust::functions::curves3d::{Circle3D, Curve3D, Curve3DError, Line3D, Point3, Vec3};

#[test]
fn line3d_has_exact_length_parameterization_tangent_and_distance() {
    let line = Line3D { start: Point3 { x: -1.0, y: 2.0, z: 3.0 }, end: Point3 { x: 3.0, y: 5.0, z: 15.0 } };
    assert!(line.validate().is_ok());
    assert!((line.length() - 13.0).abs() < 1e-12);
    let p = line.point_at(0.25).unwrap();
    assert!((p.x - 0.0).abs() < 1e-12);
    assert!((p.y - 2.75).abs() < 1e-12);
    assert!((p.z - 6.0).abs() < 1e-12);
    let t = line.tangent().unwrap();
    assert!((t.norm() - 1.0).abs() < 1e-12);
    assert!((line.distance_to_point(Point3 { x: 0.0, y: 2.0, z: 3.0 }).unwrap() - (153.0_f64 / 169.0).sqrt()).abs() < 1e-12);
}

#[test]
fn line3d_rejects_nonfinite_degenerate_and_out_of_domain_inputs() {
    let bad = Line3D { start: Point3 { x: 0.0, y: 0.0, z: f64::NAN }, end: Point3 { x: 1.0, y: 0.0, z: 0.0 } };
    assert_eq!(bad.validate(), Err(Curve3DError::NonFinite));
    let degenerate = Line3D { start: Point3 { x: 0.0, y: 0.0, z: 0.0 }, end: Point3 { x: 0.0, y: 0.0, z: 0.0 } };
    assert_eq!(degenerate.validate(), Err(Curve3DError::Degenerate));
    let line = Line3D { start: Point3 { x: 0.0, y: 0.0, z: 0.0 }, end: Point3 { x: 1.0, y: 0.0, z: 0.0 } };
    assert_eq!(line.point_at(-1e-6), Err(Curve3DError::OutOfDomain));
    assert_eq!(line.point_at(1.000001), Err(Curve3DError::OutOfDomain));
}

#[test]
fn circle3d_has_exact_circumference_unit_normal_frame_and_parameterization() {
    let c = Circle3D { center: Point3 { x: 1.0, y: -2.0, z: 4.0 }, radius: 3.0, normal: Vec3 { x: 0.0, y: 0.0, z: 1.0 } };
    assert!(c.validate().is_ok());
    assert!((c.circumference() - 6.0 * PI).abs() < 1e-12);
    let p0 = c.point_at(0.0).unwrap();
    let p25 = c.point_at(0.25).unwrap();
    assert!((p0.x - 4.0).abs() < 1e-12);
    assert!((p0.y + 2.0).abs() < 1e-12);
    assert!((p0.z - 4.0).abs() < 1e-12);
    assert!((p25.x - 1.0).abs() < 1e-12);
    assert!((p25.y - 1.0).abs() < 1e-12);
    assert!((p25.z - 4.0).abs() < 1e-12);
    let tangent = c.tangent_at(0.0).unwrap();
    assert!((tangent.norm() - 1.0).abs() < 1e-12);
    assert!(tangent.dot(c.normal).abs() < 1e-12);
}

#[test]
fn circle3d_distance_separates_axial_and_radial_error() {
    let c = Circle3D { center: Point3 { x: 0.0, y: 0.0, z: 0.0 }, radius: 5.0, normal: Vec3 { x: 0.0, y: 0.0, z: 1.0 } };
    assert!(c.distance_to_point(Point3 { x: 5.0, y: 0.0, z: 0.0 }).unwrap().abs() < 1e-12);
    assert!((c.distance_to_point(Point3 { x: 7.0, y: 0.0, z: 0.0 }).unwrap() - 2.0).abs() < 1e-12);
    assert!((c.distance_to_point(Point3 { x: 5.0, y: 0.0, z: 12.0 }).unwrap() - 12.0).abs() < 1e-12);
}

#[test]
fn circle3d_rejects_nonunit_zero_nonfinite_normal_and_invalid_radius() {
    let zero = Circle3D { center: Point3 { x: 0.0, y: 0.0, z: 0.0 }, radius: 1.0, normal: Vec3 { x: 0.0, y: 0.0, z: 0.0 } };
    assert_eq!(zero.validate(), Err(Curve3DError::Degenerate));
    let nonunit = Circle3D { center: Point3 { x: 0.0, y: 0.0, z: 0.0 }, radius: 1.0, normal: Vec3 { x: 0.0, y: 0.0, z: 2.0 } };
    assert_eq!(nonunit.validate(), Err(Curve3DError::Degenerate));
    let invalid = Circle3D { center: Point3 { x: 0.0, y: 0.0, z: 0.0 }, radius: 0.0, normal: Vec3 { x: 0.0, y: 0.0, z: 1.0 } };
    assert_eq!(invalid.validate(), Err(Curve3DError::InvalidRadius));
}

#[test]
fn curve3d_dispatch_preserves_exact_curve_semantics() {
    let line = Curve3D::Line(Line3D { start: Point3 { x: 0.0, y: 0.0, z: 0.0 }, end: Point3 { x: 0.0, y: 0.0, z: 2.0 } });
    let circle = Curve3D::Circle(Circle3D { center: Point3 { x: 0.0, y: 0.0, z: 0.0 }, radius: 2.0, normal: Vec3 { x: 0.0, y: 1.0, z: 0.0 } });
    assert!(line.validate().is_ok());
    assert!(circle.validate().is_ok());
    assert!((line.point_at(0.5).unwrap().z - 1.0).abs() < 1e-12);
    assert!(circle.distance_to_point(Point3 { x: 0.0, y: 0.0, z: 2.0 }).unwrap().abs() < 1e-12);
}
