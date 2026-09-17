use std::f64::consts::PI;

use umlcad_kernel_rust::functions::surfaces::Point3;
use umlcad_kernel_rust::functions::sweeps::{CircularSweep, SweepError};

fn point(x: f64, y: f64, z: f64) -> Point3 {
    Point3 { x, y, z }
}

#[test]
fn circular_sweep_matches_exact_cylinder_measurements() {
    let s = CircularSweep::new(point(0.0, 0.0, 0.0), point(0.0, 0.0, 10.0), 2.0);
    assert_eq!(s.length(), 10.0);
    assert!((s.volume() - 40.0 * PI).abs() < 1e-12);
    assert!((s.lateral_area() - 40.0 * PI).abs() < 1e-12);
    assert!((s.total_surface_area() - 48.0 * PI).abs() < 1e-12);
}

#[test]
fn circular_sweep_has_exact_axis_aligned_bounds() {
    let s = CircularSweep::new(point(0.0, 0.0, 0.0), point(0.0, 0.0, 10.0), 2.0);
    let b = s.bounding_box().unwrap();
    assert_eq!(b.min, point(-2.0, -2.0, 0.0));
    assert_eq!(b.max, point(2.0, 2.0, 10.0));
}

#[test]
fn circular_sweep_parameterization_is_on_the_circular_envelope() {
    let s = CircularSweep::new(point(0.0, 0.0, 0.0), point(0.0, 0.0, 10.0), 2.0);
    let p0 = s.surface_point_at(0.0, 0.0).unwrap();
    let p1 = s.surface_point_at(1.0, PI / 2.0).unwrap();
    assert!(((p0.x * p0.x + p0.y * p0.y).sqrt() - 2.0).abs() < 1e-12);
    assert!(p0.z.abs() < 1e-12);
    assert!(((p1.x * p1.x + p1.y * p1.y).sqrt() - 2.0).abs() < 1e-12);
    assert!((p1.z - 10.0).abs() < 1e-12);
}

#[test]
fn circular_sweep_is_orientation_independent() {
    let s = CircularSweep::new(point(-1.0, 2.0, 3.0), point(7.0, 8.0, 15.0), 2.0);
    let b = s.bounding_box().unwrap();
    let dx = 8.0_f64;
    let dy = 6.0_f64;
    let dz = 12.0_f64;
    let length = (dx * dx + dy * dy + dz * dz).sqrt();
    let ax = dx / length;
    let ay = dy / length;
    let az = dz / length;
    let ex = 2.0 * (1.0 - ax * ax).sqrt();
    let ey = 2.0 * (1.0 - ay * ay).sqrt();
    let ez = 2.0 * (1.0 - az * az).sqrt();
    assert!((b.min.x - (-1.0 - ex)).abs() < 1e-12);
    assert!((b.max.x - (7.0 + ex)).abs() < 1e-12);
    assert!((b.min.y - (2.0 - ey)).abs() < 1e-12);
    assert!((b.max.y - (8.0 + ey)).abs() < 1e-12);
    assert!((b.min.z - (3.0 - ez)).abs() < 1e-12);
    assert!((b.max.z - (15.0 + ez)).abs() < 1e-12);
}

#[test]
fn circular_sweep_rejects_invalid_geometry_and_parameter_domains() {
    assert_eq!(
        CircularSweep::new(point(0.0, 0.0, 0.0), point(0.0, 0.0, 0.0), 1.0).validate(),
        Err(SweepError::DegeneratePath)
    );
    assert_eq!(
        CircularSweep::new(point(0.0, 0.0, 0.0), point(0.0, 0.0, 1.0), 0.0).validate(),
        Err(SweepError::InvalidRadius)
    );
    assert_eq!(
        CircularSweep::new(point(0.0, 0.0, 0.0), point(0.0, 0.0, 1.0), f64::NAN).validate(),
        Err(SweepError::NonFinite)
    );
    let s = CircularSweep::new(point(0.0, 0.0, 0.0), point(0.0, 0.0, 1.0), 1.0);
    assert_eq!(s.surface_point_at(-1e-12, 0.0), Err(SweepError::OutOfDomain));
    assert_eq!(s.surface_point_at(1.0 + 1e-12, 0.0), Err(SweepError::OutOfDomain));
    assert_eq!(s.surface_point_at(0.5, f64::NAN), Err(SweepError::NonFinite));
    assert_eq!(s.surface_point_at(0.5, f64::INFINITY), Err(SweepError::NonFinite));
}

#[test]
fn circular_sweep_translation_is_immutable_and_exact() {
    let source = CircularSweep::new(point(1.0, 2.0, 3.0), point(4.0, 5.0, 9.0), 2.5);
    let moved = source.translated(100.0, -200.0, 300.0).unwrap();
    assert_eq!(source.path_start, point(1.0, 2.0, 3.0));
    assert_eq!(source.path_end, point(4.0, 5.0, 9.0));
    assert_eq!(moved.path_start, point(101.0, -198.0, 303.0));
    assert_eq!(moved.path_end, point(104.0, -195.0, 309.0));
    assert_eq!(moved.radius, source.radius);
}

#[test]
fn circular_sweep_rejects_nonrepresentable_derived_values() {
    let s = CircularSweep::new(point(0.0, 0.0, 0.0), point(f64::MAX, 0.0, 0.0), f64::MAX);
    assert_eq!(s.validate(), Err(SweepError::NonFinite));
    let near_overflow = CircularSweep::new(
        point(f64::MAX * 0.75, 0.0, 0.0),
        point(f64::MAX * 0.75 + 1.0, 0.0, 0.0),
        1.0,
    );
    assert!(near_overflow.translated(f64::MAX * 0.5, 0.0, 0.0).is_err());
}
