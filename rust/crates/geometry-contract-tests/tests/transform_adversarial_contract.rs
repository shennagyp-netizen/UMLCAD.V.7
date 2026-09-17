use umlcad_v6_geometry_api::{GeometryBackend, GeometryError, ToleranceContext};
use umlcad_v6_occt_backend::OcctBackend;

const T: ToleranceContext = ToleranceContext { modeling: 1e-9, validation: 1e-6 };

fn assert_invalid<TShape: Clone>(result: Result<umlcad_v6_geometry_api::GeometryResult<TShape>, GeometryError>, expected: GeometryError) {
    match result {
        Err(error) => assert_eq!(error, expected),
        Ok(_) => panic!("expected operation to fail"),
    }
}

#[test]
fn non_finite_translation_is_rejected_without_mutating_source() {
    let backend = OcctBackend::new();
    let source = backend.box_solid(12.0, 20.0, 30.0, T).unwrap().shape;
    let before = backend.bounding_box(&source, T).unwrap();

    for (x, y, z) in [
        (f64::NAN, 0.0, 0.0),
        (f64::INFINITY, 0.0, 0.0),
        (0.0, f64::NEG_INFINITY, 0.0),
    ] {
        assert_invalid(backend.translate(&source, x, y, z, T), GeometryError::InvalidInput("translation must be finite"));
        assert_eq!(before, backend.bounding_box(&source, T).unwrap());
    }
}

#[test]
fn zero_rotation_axis_is_rejected() {
    let backend = OcctBackend::new();
    let source = backend.box_solid(12.0, 20.0, 30.0, T).unwrap().shape;

    assert_invalid(backend.rotate(&source, 0.0, 0.0, 0.0, 1.0, T), GeometryError::InvalidInput("rotation axis must be non-zero"));
}

#[test]
fn non_finite_rotation_is_rejected_without_mutating_source() {
    let backend = OcctBackend::new();
    let source = backend.box_solid(12.0, 20.0, 30.0, T).unwrap().shape;
    let before = backend.bounding_box(&source, T).unwrap();

    for (x, y, z, angle) in [
        (f64::NAN, 0.0, 1.0, 0.5),
        (0.0, f64::INFINITY, 1.0, 0.5),
        (0.0, 0.0, f64::NEG_INFINITY, 0.5),
        (0.0, 0.0, 1.0, f64::NAN),
        (0.0, 0.0, 1.0, f64::INFINITY),
    ] {
        assert_invalid(backend.rotate(&source, x, y, z, angle, T), GeometryError::InvalidInput("rotation must be finite"));
        assert_eq!(before, backend.bounding_box(&source, T).unwrap());
    }
}

#[test]
fn invalid_tolerance_is_rejected_by_transform_operations() {
    let backend = OcctBackend::new();
    let source = backend.box_solid(12.0, 20.0, 30.0, T).unwrap().shape;
    let invalid = ToleranceContext { modeling: -1.0, validation: 1e-6 };

    assert_invalid(backend.translate(&source, 1.0, 2.0, 3.0, invalid), GeometryError::InvalidTolerance);
    assert_invalid(backend.rotate(&source, 0.0, 0.0, 1.0, 0.5, invalid), GeometryError::InvalidTolerance);
}

#[test]
fn failed_transform_leaves_source_geometry_and_topology_unchanged() {
    let backend = OcctBackend::new();
    let source = backend.box_solid(12.0, 20.0, 30.0, T).unwrap().shape;
    let before_box = backend.bounding_box(&source, T).unwrap();
    let before_topology = backend.topology_counts(&source, T).unwrap();

    assert_invalid(backend.rotate(&source, 0.0, 0.0, 0.0, 0.5, T), GeometryError::InvalidInput("rotation axis must be non-zero"));
    assert_invalid(backend.translate(&source, f64::NAN, 0.0, 0.0, T), GeometryError::InvalidInput("translation must be finite"));

    assert_eq!(before_box, backend.bounding_box(&source, T).unwrap());
    assert_eq!(before_topology, backend.topology_counts(&source, T).unwrap());
    assert!(backend.validate(&source, T).unwrap().valid);
}
