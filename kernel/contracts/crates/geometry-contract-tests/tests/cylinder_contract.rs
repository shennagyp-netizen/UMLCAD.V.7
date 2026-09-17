use umlcad_v6_geometry_api::{BoundingBox, GeometryBackend, GeometryError, GeometryKind, ToleranceContext, TopologyCounts, ValidationResult};
use umlcad_v6_occt_backend::OcctBackend;

const TOLERANCE: ToleranceContext = ToleranceContext {
    modeling: 1e-9,
    validation: 1e-9,
};

fn assert_close(actual: f64, expected: f64) {
    assert!((actual - expected).abs() <= 1e-9, "expected {expected:.17e}, got {actual:.17e}");
}

#[test]
fn cylinder_contract_is_analytic_solid() {
    let backend = OcctBackend::new();
    let result = backend.cylinder_solid(5.0, 20.0, TOLERANCE).unwrap();
    assert_eq!(result.kind, GeometryKind::Solid);
    assert_eq!(result.evidence.backend, "occt");
    assert_eq!(result.evidence.tolerance, TOLERANCE);
    assert_eq!(backend.validate(&result.shape, TOLERANCE).unwrap(), ValidationResult { valid: true, manifold: true, message: None });
}

#[test]
fn cylinder_contract_exposes_canonical_topology() {
    let backend = OcctBackend::new();
    let shape = backend.cylinder_solid(5.0, 20.0, TOLERANCE).unwrap().shape;
    assert_eq!(backend.topology_counts(&shape, TOLERANCE).unwrap(), TopologyCounts { solids: 1, shells: 1, faces: 3, edges: 3, vertices: 2 });
}

#[test]
fn cylinder_contract_has_exact_axis_aligned_bounds() {
    let backend = OcctBackend::new();
    let shape = backend.cylinder_solid(5.0, 20.0, TOLERANCE).unwrap().shape;
    let bounds = backend.bounding_box(&shape, TOLERANCE).unwrap();
    let expected = BoundingBox { min_x: -5.0, min_y: -5.0, min_z: 0.0, max_x: 5.0, max_y: 5.0, max_z: 20.0 };
    assert_close(bounds.min_x, expected.min_x);
    assert_close(bounds.min_y, expected.min_y);
    assert_close(bounds.min_z, expected.min_z);
    assert_close(bounds.max_x, expected.max_x);
    assert_close(bounds.max_y, expected.max_y);
    assert_close(bounds.max_z, expected.max_z);
}

#[test]
fn cylinder_face_evidence_preserves_analytic_surface_area() {
    let backend = OcctBackend::new();
    let shape = backend.cylinder_solid(5.0, 20.0, TOLERANCE).unwrap().shape;
    let faces = backend.face_descriptors(&shape, TOLERANCE).unwrap();
    assert_eq!(faces.len(), 3);
    let total_area: f64 = faces.iter().map(|face| face.area).sum();
    let expected = 2.0 * std::f64::consts::PI * 5.0 * 20.0 + 2.0 * std::f64::consts::PI * 25.0;
    assert!((total_area - expected).abs() <= 1e-9, "expected total area {expected:.17e}, got {total_area:.17e}");
    assert_eq!(faces.iter().filter(|face| (face.area - std::f64::consts::PI * 25.0).abs() <= 1e-9).count(), 2);
    assert!(faces.iter().any(|face| (face.area - 2.0 * std::f64::consts::PI * 5.0 * 20.0).abs() <= 1e-9));
}

#[test]
fn cylinder_contract_rejects_degenerate_or_non_finite_inputs() {
    let backend = OcctBackend::new();
    for input in [(0.0, 20.0), (-1.0, 20.0), (5.0, 0.0), (5.0, -1.0), (f64::NAN, 20.0), (5.0, f64::NAN), (f64::INFINITY, 20.0), (5.0, f64::INFINITY)] {
        match backend.cylinder_solid(input.0, input.1, TOLERANCE) {
            Err(GeometryError::InvalidInput(_)) => {}
            Err(error) => panic!("unexpected cylinder error: {error:?}"),
            Ok(_) => panic!("invalid cylinder input unexpectedly succeeded"),
        }
    }
}

#[test]
fn cylinder_contract_survives_immutable_transforms() {
    let backend = OcctBackend::new();
    let source = backend.cylinder_solid(5.0, 20.0, TOLERANCE).unwrap().shape;
    let translated = backend.translate(&source, 100.0, -200.0, 300.0, TOLERANCE).unwrap().shape;
    let rotated = backend.rotate(&source, 0.0, 0.0, 1.0, std::f64::consts::FRAC_PI_2, TOLERANCE).unwrap().shape;
    assert_eq!(backend.validate(&source, TOLERANCE).unwrap(), ValidationResult { valid: true, manifold: true, message: None });
    assert_eq!(backend.validate(&translated, TOLERANCE).unwrap(), ValidationResult { valid: true, manifold: true, message: None });
    assert_eq!(backend.validate(&rotated, TOLERANCE).unwrap(), ValidationResult { valid: true, manifold: true, message: None });
}
