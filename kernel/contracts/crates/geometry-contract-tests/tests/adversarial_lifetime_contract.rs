use umlcad_v6_geometry_api::{GeometryBackend, GeometryKind, ToleranceContext, ValidationResult};
use umlcad_v6_occt_backend::OcctBackend;

const TOLERANCE: ToleranceContext = ToleranceContext { modeling: 1e-9, validation: 1e-9 };

fn valid() -> ValidationResult {
    ValidationResult { valid: true, manifold: true, message: None }
}

#[test]
fn repeated_clone_transform_drop_cycles_remain_valid() {
    let backend = OcctBackend::new();
    for _ in 0..512 {
        let source = backend.box_solid(10.0, 20.0, 30.0, TOLERANCE).unwrap().shape;
        let clone = source.clone();
        let translated = backend.translate(&clone, 123.0, -456.0, 789.0, TOLERANCE).unwrap().shape;
        assert_eq!(backend.validate(&source, TOLERANCE).unwrap(), valid());
        assert_eq!(backend.validate(&clone, TOLERANCE).unwrap(), valid());
        assert_eq!(backend.validate(&translated, TOLERANCE).unwrap(), valid());
    }
}

#[test]
fn numeric_scales_remain_supported_above_backend_resolution() {
    let backend = OcctBackend::new();
    for edge in [2e-6, 1e-5, 1e-3, 1e3, 1e6] {
        let result = backend
            .box_solid(edge, edge * 2.0, edge * 3.0, TOLERANCE)
            .unwrap_or_else(|error| panic!("supported scale {edge:e} failed: {error}"));
        assert_eq!(result.kind, GeometryKind::Solid);
        assert_eq!(backend.validate(&result.shape, TOLERANCE).unwrap(), valid());
    }
}

#[test]
fn transform_results_are_immutable_and_source_is_unchanged() {
    let backend = OcctBackend::new();
    let source = backend.box_solid(10.0, 20.0, 30.0, TOLERANCE).unwrap().shape;
    let before = backend.bounding_box(&source, TOLERANCE).unwrap();

    let translated = backend.translate(&source, 100.0, -200.0, 300.0, TOLERANCE).unwrap().shape;
    let rotated = backend
        .rotate(&source, 0.0, 0.0, 1.0, std::f64::consts::FRAC_PI_2, TOLERANCE)
        .unwrap()
        .shape;

    assert_eq!(backend.bounding_box(&source, TOLERANCE).unwrap(), before);
    assert_eq!(backend.validate(&source, TOLERANCE).unwrap(), valid());
    assert_eq!(backend.validate(&translated, TOLERANCE).unwrap(), valid());
    assert_eq!(backend.validate(&rotated, TOLERANCE).unwrap(), valid());
}

#[test]
fn repeated_validation_is_deterministic() {
    let backend = OcctBackend::new();
    let source = backend.cylinder_solid(5.0, 20.0, TOLERANCE).unwrap().shape;
    let expected = backend.validate(&source, TOLERANCE).unwrap();
    for _ in 0..256 {
        assert_eq!(backend.validate(&source, TOLERANCE).unwrap(), expected);
    }
}
