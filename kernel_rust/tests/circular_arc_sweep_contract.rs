use umlcad_kernel_rust::functions::surfaces::Point3;
use umlcad_kernel_rust::functions::sweeps::{CircularArcSweep, SweepArcError};

fn p(x: f64, y: f64, z: f64) -> Point3 { Point3 { x, y, z } }

fn quarter_tube() -> CircularArcSweep {
    CircularArcSweep::new(p(0.0, 0.0, 0.0), 10.0, 2.0, 0.0, core::f64::consts::FRAC_PI_2)
}

#[test]
fn circular_arc_sweep_has_exact_centerline_length_and_volume() {
    let sweep = quarter_tube();
    assert!((sweep.path_length() - 10.0 * core::f64::consts::FRAC_PI_2).abs() < 1e-12);
    let expected_volume = core::f64::consts::PI * 4.0 * 10.0 * core::f64::consts::FRAC_PI_2;
    assert!((sweep.volume() - expected_volume).abs() < 1e-12);
}

#[test]
fn circular_arc_sweep_surface_parameterization_matches_torus_geometry() {
    let sweep = quarter_tube();
    let q = sweep.surface_point_at(0.5, 0.0).unwrap();
    let expected_outer = 12.0 / core::f64::consts::SQRT_2;
    assert!((q.x - expected_outer).abs() < 1e-12);
    assert!((q.y - expected_outer).abs() < 1e-12);
    assert!(q.z.abs() < 1e-12);

    let top = sweep.surface_point_at(0.5, core::f64::consts::FRAC_PI_2).unwrap();
    let expected_mid = 10.0 / core::f64::consts::SQRT_2;
    assert!((top.x - expected_mid).abs() < 1e-12);
    assert!((top.y - expected_mid).abs() < 1e-12);
    assert!((top.z - 2.0).abs() < 1e-12);
}

#[test]
fn circular_arc_sweep_bounding_box_is_analytically_bounded() {
    let bounds = quarter_tube().bounding_box().unwrap();
    assert!(bounds.min.x.abs() < 1e-12);
    assert!((bounds.max.x - 12.0).abs() < 1e-12);
    assert!(bounds.min.y.abs() < 1e-12);
    assert!((bounds.max.y - 12.0).abs() < 1e-12);
    assert!((bounds.min.z + 2.0).abs() < 1e-12);
    assert!((bounds.max.z - 2.0).abs() < 1e-12);
}

#[test]
fn circular_arc_sweep_is_immutable_under_translation() {
    let sweep = quarter_tube();
    let moved = sweep.translated(100.0, -20.0, 7.0).unwrap();
    let a = sweep.surface_point_at(0.3, 1.1).unwrap();
    let b = moved.surface_point_at(0.3, 1.1).unwrap();
    assert!((b.x - (a.x + 100.0)).abs() < 1e-12);
    assert!((b.y - (a.y - 20.0)).abs() < 1e-12);
    assert!((b.z - (a.z + 7.0)).abs() < 1e-12);
    assert_eq!(sweep.surface_point_at(0.3, 1.1).unwrap(), a);
}

#[test]
fn circular_arc_sweep_rejects_invalid_geometry_and_parameters() {
    assert_eq!(CircularArcSweep::new(p(0.0, 0.0, 0.0), 1.0, 1.0, 0.0, 1.0).validate(), Err(SweepArcError::InvalidGeometry));
    assert!(CircularArcSweep::new(p(0.0, 0.0, 0.0), 10.0, -1.0, 0.0, 1.0).validate().is_err());
    assert!(CircularArcSweep::new(p(0.0, 0.0, 0.0), 10.0, 2.0, 1.0, 1.0).validate().is_err());
    assert!(CircularArcSweep::new(p(f64::NAN, 0.0, 0.0), 10.0, 2.0, 0.0, 1.0).validate().is_err());
    assert_eq!(quarter_tube().surface_point_at(f64::NAN, 0.0), Err(SweepArcError::NonFinite));
}
