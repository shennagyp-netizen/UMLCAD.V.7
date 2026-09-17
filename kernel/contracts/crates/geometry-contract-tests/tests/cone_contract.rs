use umlcad_v6_geometry_api::{GeometryBackend, GeometryError, GeometryKind, ToleranceContext, TopologyCounts, ValidationResult};
use umlcad_v6_occt_backend::OcctBackend;

const TOLERANCE: ToleranceContext = ToleranceContext { modeling: 1e-9, validation: 1e-9 };

#[test]
fn cone_is_a_valid_manifold_solid() {
    let backend = OcctBackend::new();
    let result = backend.cone_solid(10.0, 5.0, 20.0, TOLERANCE).unwrap();
    assert_eq!(result.kind, GeometryKind::Solid);
    assert_eq!(backend.validate(&result.shape, TOLERANCE).unwrap(), ValidationResult { valid: true, manifold: true, message: None });
}

#[test]
fn cone_has_canonical_frustum_topology() {
    let backend = OcctBackend::new();
    let shape = backend.cone_solid(10.0, 5.0, 20.0, TOLERANCE).unwrap().shape;
    assert_eq!(backend.topology_counts(&shape, TOLERANCE).unwrap(), TopologyCounts { solids: 1, shells: 1, faces: 3, edges: 3, vertices: 2 });
}

#[test]
fn cone_bounds_are_exact_for_axis_aligned_reference() {
    let backend = OcctBackend::new();
    let shape = backend.cone_solid(10.0, 5.0, 20.0, TOLERANCE).unwrap().shape;
    let bounds = backend.bounding_box(&shape, TOLERANCE).unwrap();
    assert!((bounds.min_x + 10.0).abs() <= 1e-9);
    assert!((bounds.max_x - 10.0).abs() <= 1e-9);
    assert!((bounds.min_y + 10.0).abs() <= 1e-9);
    assert!((bounds.max_y - 10.0).abs() <= 1e-9);
    assert!((bounds.min_z - 0.0).abs() <= 1e-9);
    assert!((bounds.max_z - 20.0).abs() <= 1e-9);
}

#[test]
fn cone_face_evidence_matches_analytic_frustum_area() {
    let backend = OcctBackend::new();
    let shape = backend.cone_solid(10.0, 5.0, 20.0, TOLERANCE).unwrap().shape;
    let faces = backend.face_descriptors(&shape, TOLERANCE).unwrap();
    assert_eq!(faces.len(), 3);
    let slant = ((10.0_f64 - 5.0_f64).powi(2) + 20.0_f64.powi(2)).sqrt();
    let expected = std::f64::consts::PI * (10.0_f64 + 5.0_f64) * slant + std::f64::consts::PI * 10.0_f64.powi(2) + std::f64::consts::PI * 5.0_f64.powi(2);
    let total_area: f64 = faces.iter().map(|face| face.area).sum();
    assert!((total_area - expected).abs() <= 1e-9, "expected total area {expected:.17e}, got {total_area:.17e}");
}

#[test]
fn cone_rejects_degenerate_and_non_finite_parameters() {
    let backend = OcctBackend::new();
    for input in [(0.0, 5.0, 20.0), (10.0, 0.0, 20.0), (10.0, 5.0, 0.0), (-1.0, 5.0, 20.0), (10.0, -1.0, 20.0), (10.0, 5.0, -1.0), (f64::NAN, 5.0, 20.0), (10.0, f64::NAN, 20.0), (10.0, 5.0, f64::INFINITY)] {
        match backend.cone_solid(input.0, input.1, input.2, TOLERANCE) {
            Err(GeometryError::InvalidInput(_)) => {}
            Err(error) => panic!("unexpected cone error: {error:?}"),
            Ok(_) => panic!("invalid cone parameters unexpectedly succeeded"),
        }
    }
}

#[test]
fn cone_is_immutable_through_transforms() {
    let backend = OcctBackend::new();
    let source = backend.cone_solid(10.0, 5.0, 20.0, TOLERANCE).unwrap().shape;
    let translated = backend.translate(&source, 100.0, -200.0, 300.0, TOLERANCE).unwrap().shape;
    let rotated = backend.rotate(&source, 0.0, 0.0, 1.0, std::f64::consts::FRAC_PI_2, TOLERANCE).unwrap().shape;
    for shape in [&source, &translated, &rotated] {
        assert_eq!(backend.validate(shape, TOLERANCE).unwrap(), ValidationResult { valid: true, manifold: true, message: None });
    }
}
