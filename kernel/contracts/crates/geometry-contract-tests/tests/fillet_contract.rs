use umlcad_v6_geometry_api::{GeometryBackend, GeometryError, GeometryStatus, ToleranceContext};
use umlcad_v6_occt_backend::OcctBackend;

const T: ToleranceContext = ToleranceContext { modeling: 1e-9, validation: 1e-9 };

#[test]
fn fillet_all_edges_produces_valid_solid_with_preserved_envelope() {
    let backend = OcctBackend::new();
    let source = backend.box_solid(20.0, 20.0, 20.0, T).unwrap().shape;
    let result = backend.fillet_all_edges(&source, 2.0, T).unwrap();

    assert_eq!(result.evidence.status, GeometryStatus::Success);
    let validation = backend.validate(&result.shape, T).unwrap();
    assert!(validation.valid);
    assert!(validation.manifold);

    let bounds = backend.bounding_box(&result.shape, T).unwrap();
    assert!((bounds.min_x - 0.0).abs() <= 1e-9);
    assert!((bounds.max_x - 20.0).abs() <= 1e-9);
    assert!((bounds.min_y - 0.0).abs() <= 1e-9);
    assert!((bounds.max_y - 20.0).abs() <= 1e-9);
    assert!((bounds.min_z - 0.0).abs() <= 1e-9);
    assert!((bounds.max_z - 20.0).abs() <= 1e-9);

    let topology = backend.topology_counts(&result.shape, T).unwrap();
    assert_eq!(topology.solids, 1);
    assert_eq!(topology.shells, 1);
    assert!(topology.faces > 6);
    assert!(topology.edges > 12);
}

#[test]
fn fillet_is_deterministic_and_does_not_mutate_source() {
    let backend = OcctBackend::new();
    let source = backend.box_solid(20.0, 20.0, 20.0, T).unwrap().shape;
    let first = backend.fillet_all_edges(&source, 2.0, T).unwrap().shape;
    let second = backend.fillet_all_edges(&source, 2.0, T).unwrap().shape;

    assert_eq!(backend.bounding_box(&first, T).unwrap(), backend.bounding_box(&second, T).unwrap());
    assert_eq!(backend.topology_counts(&first, T).unwrap(), backend.topology_counts(&second, T).unwrap());
    assert!(backend.validate(&source, T).unwrap().valid);
    assert!((backend.bounding_box(&source, T).unwrap().max_x - 20.0).abs() <= 1e-9);
}

#[test]
fn fillet_rejects_invalid_or_geometrically_impossible_radii() {
    let backend = OcctBackend::new();
    let source = backend.box_solid(20.0, 20.0, 20.0, T).unwrap().shape;

    for radius in [0.0, -1.0, f64::NAN, f64::INFINITY, 11.0] {
        assert!(matches!(
            backend.fillet_all_edges(&source, radius, T),
            Err(GeometryError::InvalidInput(_)) | Err(GeometryError::Unsupported(_))
        ));
    }
}
