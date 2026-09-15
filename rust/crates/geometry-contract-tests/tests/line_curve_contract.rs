use umlcad_v6_geometry_api::{GeometryBackend, GeometryError, GeometryKind, ToleranceContext, ValidationResult};
use umlcad_v6_occt_backend::OcctBackend;

const T: ToleranceContext = ToleranceContext { modeling: 1e-9, validation: 1e-9 };

#[test]
fn line_curve_is_a_curve_with_exact_endpoints() {
    let backend = OcctBackend::new();
    let result = backend.line_curve(0.0, 0.0, 0.0, 100.0, 0.0, 0.0, T).unwrap();
    assert_eq!(result.kind, GeometryKind::Curve);
    assert_eq!(backend.validate(&result.shape, T).unwrap(), ValidationResult { valid: true, manifold: false, message: None });
    let bounds = backend.bounding_box(&result.shape, T).unwrap();
    assert_eq!((bounds.min_x, bounds.min_y, bounds.min_z), (0.0, 0.0, 0.0));
    assert_eq!((bounds.max_x, bounds.max_y, bounds.max_z), (100.0, 0.0, 0.0));
}

#[test]
fn line_curve_rejects_coincident_or_non_finite_endpoints() {
    let backend = OcctBackend::new();
    for input in [
        (0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
        (f64::NAN, 0.0, 0.0, 1.0, 0.0, 0.0),
        (0.0, f64::INFINITY, 0.0, 1.0, 0.0, 0.0),
        (0.0, 0.0, 0.0, f64::NEG_INFINITY, 0.0, 0.0),
    ] {
        assert!(matches!(backend.line_curve(input.0, input.1, input.2, input.3, input.4, input.5, T), Err(GeometryError::InvalidInput(_))));
    }
}

#[test]
fn line_curve_transform_preserves_curve_kind_and_source() {
    let backend = OcctBackend::new();
    let source = backend.line_curve(0.0, 0.0, 0.0, 100.0, 0.0, 0.0, T).unwrap().shape;
    let before = backend.bounding_box(&source, T).unwrap();
    let translated = backend.translate(&source, 10.0, 20.0, 30.0, T).unwrap();
    assert_eq!(translated.kind, GeometryKind::Curve);
    assert_eq!(before, backend.bounding_box(&source, T).unwrap());
    let moved = backend.bounding_box(&translated.shape, T).unwrap();
    assert_eq!((moved.min_x, moved.min_y, moved.min_z), (10.0, 20.0, 30.0));
    assert_eq!((moved.max_x, moved.max_y, moved.max_z), (110.0, 20.0, 30.0));
}
