use umlcad_v6_geometry_api::{GeometryBackend, GeometryError, GeometryKind, ToleranceContext, TopologyCounts, ValidationResult};
use umlcad_v6_occt_backend::OcctBackend;

const TOLERANCE: ToleranceContext = ToleranceContext { modeling: 1e-9, validation: 1e-9 };

#[test]
fn sphere_is_a_valid_manifold_solid() {
    let backend = OcctBackend::new();
    let result = backend.sphere_solid(5.0, TOLERANCE).unwrap();
    assert_eq!(result.kind, GeometryKind::Solid);
    assert_eq!(backend.validate(&result.shape, TOLERANCE).unwrap(), ValidationResult { valid: true, manifold: true, message: None });
}

#[test]
fn sphere_has_canonical_occt_topology() {
    let backend = OcctBackend::new();
    let shape = backend.sphere_solid(5.0, TOLERANCE).unwrap().shape;
    assert_eq!(backend.topology_counts(&shape, TOLERANCE).unwrap(), TopologyCounts { solids: 1, shells: 1, faces: 1, edges: 3, vertices: 2 });
}

#[test]
fn sphere_face_evidence_matches_analytic_surface_area() {
    let backend = OcctBackend::new();
    let shape = backend.sphere_solid(5.0, TOLERANCE).unwrap().shape;
    let faces = backend.face_descriptors(&shape, TOLERANCE).unwrap();
    assert_eq!(faces.len(), 1);
    let expected = 4.0 * std::f64::consts::PI * 25.0;
    assert!((faces[0].area - expected).abs() <= 1e-9, "expected sphere area {expected:.17e}, got {:.17e}", faces[0].area);
    assert!(faces[0].boundary_edge_count >= 1);
}

#[test]
fn sphere_rejects_invalid_radius() {
    let backend = OcctBackend::new();
    for radius in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        match backend.sphere_solid(radius, TOLERANCE) {
            Err(GeometryError::InvalidInput(_)) => {}
            Err(error) => panic!("unexpected sphere error: {error:?}"),
            Ok(_) => panic!("invalid sphere radius unexpectedly succeeded"),
        }
    }
}

#[test]
fn sphere_bounds_and_immutable_transforms_are_preserved() {
    let backend = OcctBackend::new();
    let source = backend.sphere_solid(5.0, TOLERANCE).unwrap().shape;
    let bounds = backend.bounding_box(&source, TOLERANCE).unwrap();
    assert_eq!((bounds.min_x, bounds.min_y, bounds.min_z), (-5.0, -5.0, -5.0));
    assert_eq!((bounds.max_x, bounds.max_y, bounds.max_z), (5.0, 5.0, 5.0));
    let translated = backend.translate(&source, 10.0, -20.0, 30.0, TOLERANCE).unwrap().shape;
    let rotated = backend.rotate(&source, 1.0, 2.0, 3.0, std::f64::consts::FRAC_PI_2, TOLERANCE).unwrap().shape;
    for shape in [&source, &translated, &rotated] {
        assert_eq!(backend.validate(shape, TOLERANCE).unwrap(), ValidationResult { valid: true, manifold: true, message: None });
    }
}
