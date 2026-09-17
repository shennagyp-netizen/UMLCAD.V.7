use umlcad_v6_geometry_api::{GeometryBackend, GeometryError, ToleranceContext};
use umlcad_v6_occt_backend::OcctBackend;

const TOLERANCE: ToleranceContext = ToleranceContext { modeling: 1e-9, validation: 1e-9 };

#[test]
fn triangle_extrusion_produces_expected_prism() {
    let backend = OcctBackend::new();
    let points = [(0.0, 0.0), (10.0, 0.0), (0.0, 20.0)];
    let result = backend.extrude_polygon(&points, 30.0, TOLERANCE).unwrap();

    assert!(result.evidence.status == umlcad_v6_geometry_api::GeometryStatus::Success);
    assert!(backend.validate(&result.shape, TOLERANCE).unwrap().valid);

    let bounds = backend.bounding_box(&result.shape, TOLERANCE).unwrap();
    assert!((bounds.min_x - 0.0).abs() <= 1e-9);
    assert!((bounds.min_y - 0.0).abs() <= 1e-9);
    assert!((bounds.min_z - 0.0).abs() <= 1e-9);
    assert!((bounds.max_x - 10.0).abs() <= 1e-9);
    assert!((bounds.max_y - 20.0).abs() <= 1e-9);
    assert!((bounds.max_z - 30.0).abs() <= 1e-9);

    let topology = backend.topology_counts(&result.shape, TOLERANCE).unwrap();
    assert_eq!(topology.solids, 1);
    assert_eq!(topology.shells, 1);
    assert_eq!(topology.faces, 5);
    assert_eq!(topology.edges, 9);
    assert_eq!(topology.vertices, 6);
}

#[test]
fn rectangle_extrusion_has_expected_immutable_transform_behavior() {
    let backend = OcctBackend::new();
    let points = [(0.0, 0.0), (10.0, 0.0), (10.0, 20.0), (0.0, 20.0)];
    let original = backend.extrude_polygon(&points, 30.0, TOLERANCE).unwrap().shape;
    let moved = backend.translate(&original, 5.0, -2.0, 7.0, TOLERANCE).unwrap().shape;

    let original_bounds = backend.bounding_box(&original, TOLERANCE).unwrap();
    let moved_bounds = backend.bounding_box(&moved, TOLERANCE).unwrap();

    assert_eq!(original_bounds.min_x, 0.0);
    assert_eq!(original_bounds.min_y, 0.0);
    assert_eq!(original_bounds.min_z, 0.0);
    assert_eq!(moved_bounds.min_x, 5.0);
    assert_eq!(moved_bounds.min_y, -2.0);
    assert_eq!(moved_bounds.min_z, 7.0);
    assert!(backend.validate(&original, TOLERANCE).unwrap().valid);
    assert!(backend.validate(&moved, TOLERANCE).unwrap().valid);
}

#[test]
fn extrusion_rejects_invalid_profiles_and_height() {
    let backend = OcctBackend::new();
    let open = [(0.0, 0.0), (10.0, 0.0)];
    let duplicate = [(0.0, 0.0), (10.0, 0.0), (10.0, 0.0), (0.0, 10.0)];
    let bow_tie = [(0.0, 0.0), (10.0, 10.0), (0.0, 10.0), (10.0, 0.0)];
    let nan_profile = [(0.0, 0.0), (f64::NAN, 1.0), (1.0, 0.0)];

    for points in [&open[..], &duplicate[..], &bow_tie[..], &nan_profile[..]] {
        assert!(matches!(
            backend.extrude_polygon(points, 10.0, TOLERANCE),
            Err(GeometryError::InvalidInput(_))
        ));
    }

    let triangle = [(0.0, 0.0), (10.0, 0.0), (0.0, 10.0)];
    assert!(matches!(
        backend.extrude_polygon(&triangle, 0.0, TOLERANCE),
        Err(GeometryError::InvalidInput(_))
    ));
    assert!(matches!(
        backend.extrude_polygon(&triangle, f64::INFINITY, TOLERANCE),
        Err(GeometryError::InvalidInput(_))
    ));
}

#[test]
fn extrusion_is_deterministic() {
    let backend = OcctBackend::new();
    let points = [(0.0, 0.0), (10.0, 0.0), (10.0, 20.0), (0.0, 20.0)];
    let first = backend.extrude_polygon(&points, 30.0, TOLERANCE).unwrap().shape;
    let second = backend.extrude_polygon(&points, 30.0, TOLERANCE).unwrap().shape;

    assert_eq!(
        backend.bounding_box(&first, TOLERANCE).unwrap(),
        backend.bounding_box(&second, TOLERANCE).unwrap()
    );
    assert_eq!(
        backend.topology_counts(&first, TOLERANCE).unwrap(),
        backend.topology_counts(&second, TOLERANCE).unwrap()
    );
}
