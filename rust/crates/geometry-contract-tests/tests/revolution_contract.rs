use std::f64::consts::PI;

use umlcad_v6_geometry_api::{GeometryBackend, GeometryError, ToleranceContext};
use umlcad_v6_occt_backend::OcctBackend;

const T: ToleranceContext = ToleranceContext { modeling: 1e-9, validation: 1e-9 };

#[test]
fn full_revolution_of_annular_profile_produces_valid_solid() {
    let backend = OcctBackend::new();
    let profile = [(5.0, 0.0), (10.0, 0.0), (10.0, 20.0), (5.0, 20.0)];
    let result = backend.revolve_polygon(&profile, 2.0 * PI, T).unwrap();

    assert!(backend.validate(&result.shape, T).unwrap().valid);
    let bounds = backend.bounding_box(&result.shape, T).unwrap();
    assert!((bounds.min_x + 10.0).abs() <= 1e-9);
    assert!((bounds.min_y + 10.0).abs() <= 1e-9);
    assert!((bounds.min_z - 0.0).abs() <= 1e-9);
    assert!((bounds.max_x - 10.0).abs() <= 1e-9);
    assert!((bounds.max_y - 10.0).abs() <= 1e-9);
    assert!((bounds.max_z - 20.0).abs() <= 1e-9);

    let topology = backend.topology_counts(&result.shape, T).unwrap();
    assert_eq!(topology.solids, 1);
    assert_eq!(topology.shells, 1);
    assert_eq!(topology.faces, 4);
}

#[test]
fn partial_revolution_is_still_immutable_and_valid() {
    let backend = OcctBackend::new();
    let profile = [(5.0, 0.0), (10.0, 0.0), (10.0, 20.0), (5.0, 20.0)];
    let original = backend.revolve_polygon(&profile, PI, T).unwrap().shape;
    let moved = backend.translate(&original, 3.0, 4.0, 2.0, T).unwrap().shape;

    assert!(backend.validate(&original, T).unwrap().valid);
    assert!(backend.validate(&moved, T).unwrap().valid);
}

#[test]
fn revolution_rejects_axis_crossing_degenerate_and_non_finite_profiles() {
    let backend = OcctBackend::new();
    let axis_crossing = [(-1.0, 0.0), (1.0, 0.0), (1.0, 10.0), (-1.0, 10.0)];
    let zero_radius = [(0.0, 0.0), (10.0, 0.0), (10.0, 10.0)];
    let nan_profile = [(5.0, 0.0), (f64::NAN, 1.0), (10.0, 10.0)];
    let normal = [(5.0, 0.0), (10.0, 0.0), (10.0, 10.0)];

    for profile in [&axis_crossing[..], &zero_radius[..], &nan_profile[..]] {
        assert!(matches!(backend.revolve_polygon(profile, 2.0 * PI, T), Err(GeometryError::InvalidInput(_))));
    }
    assert!(matches!(backend.revolve_polygon(&normal, 0.0, T), Err(GeometryError::InvalidInput(_))));
    assert!(matches!(backend.revolve_polygon(&normal, f64::NAN, T), Err(GeometryError::InvalidInput(_))));
}

#[test]
fn revolution_is_deterministic() {
    let backend = OcctBackend::new();
    let profile = [(5.0, 0.0), (10.0, 0.0), (10.0, 20.0), (5.0, 20.0)];
    let a = backend.revolve_polygon(&profile, 2.0 * PI, T).unwrap().shape;
    let b = backend.revolve_polygon(&profile, 2.0 * PI, T).unwrap().shape;
    assert_eq!(backend.bounding_box(&a, T).unwrap(), backend.bounding_box(&b, T).unwrap());
    assert_eq!(backend.topology_counts(&a, T).unwrap(), backend.topology_counts(&b, T).unwrap());
}
