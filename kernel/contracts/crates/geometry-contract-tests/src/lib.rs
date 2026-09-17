//! Contract tests shared by every V6 geometry backend.
//! These tests define behavior independently of backend implementation details.

use umlcad_v6_geometry_api::{
    BoundingBox, GeometryBackend, GeometryError, GeometryKind, ToleranceContext, TopologyCounts,
    ValidationResult,
};

const TOLERANCE: ToleranceContext = ToleranceContext {
    modeling: 1e-9,
    validation: 1e-9,
};

fn assert_close(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "expected {expected:.17e}, got {actual:.17e}"
    );
}

fn assert_box(actual: BoundingBox, expected: BoundingBox) {
    const EPSILON: f64 = 1e-9;
    assert_close(actual.min_x, expected.min_x, EPSILON);
    assert_close(actual.min_y, expected.min_y, EPSILON);
    assert_close(actual.min_z, expected.min_z, EPSILON);
    assert_close(actual.max_x, expected.max_x, EPSILON);
    assert_close(actual.max_y, expected.max_y, EPSILON);
    assert_close(actual.max_z, expected.max_z, EPSILON);
}

pub fn assert_backend_identity<B: GeometryBackend>(backend: &B, expected: &'static str) {
    assert_eq!(backend.backend_name(), expected);
}

pub fn assert_box_requires_positive_dimensions<B: GeometryBackend>(backend: &B) {
    for dimensions in [
        (0.0, 20.0, 30.0),
        (-1.0, 20.0, 30.0),
        (20.0, 0.0, 30.0),
        (20.0, 30.0, -1.0),
        (f64::NAN, 20.0, 30.0),
        (20.0, f64::INFINITY, 30.0),
        (20.0, 30.0, f64::NEG_INFINITY),
    ] {
        match backend.box_solid(dimensions.0, dimensions.1, dimensions.2, TOLERANCE) {
            Err(GeometryError::InvalidInput(_)) => {}
            Err(GeometryError::Unsupported(_)) => {
                panic!("backend does not yet implement this contract")
            }
            Err(other) => panic!("unexpected geometry error: {other}"),
            Ok(_) => panic!("invalid box dimensions unexpectedly succeeded"),
        }
    }
}

pub fn assert_reference_backend_resolution_is_explicit<B: GeometryBackend>(backend: &B) {
    for edge in [1e-8, 1e-7, 1e-6] {
        match backend.box_solid(edge, edge * 2.0, edge * 3.0, TOLERANCE) {
            Err(GeometryError::InvalidInput(_)) => {}
            Err(other) => panic!("unexpected resolution error at {edge:e}: {other}"),
            Ok(_) => panic!("reference backend accepted unsupported box scale {edge:e}"),
        }
    }
}

pub fn assert_invalid_tolerance_is_rejected<B: GeometryBackend>(backend: &B) {
    let invalid = [
        ToleranceContext {
            modeling: -1e-9,
            validation: 1e-9,
        },
        ToleranceContext {
            modeling: f64::NAN,
            validation: 1e-9,
        },
        ToleranceContext {
            modeling: 1e-9,
            validation: f64::INFINITY,
        },
    ];

    for tolerance in invalid {
        match backend.box_solid(10.0, 20.0, 30.0, tolerance) {
            Err(GeometryError::InvalidTolerance) => {}
            Err(other) => panic!("unexpected error for invalid tolerance: {other}"),
            Ok(_) => panic!("invalid tolerance unexpectedly succeeded"),
        }
    }
}

pub fn assert_numeric_scale_survives_validation<B: GeometryBackend>(backend: &B) {
    for edge in [2e-6, 1e-5, 1e-3, 1e3, 1e6] {
        let result = backend
            .box_solid(edge, edge * 2.0, edge * 3.0, TOLERANCE)
            .unwrap_or_else(|error| panic!("failed supported scale {edge:e}: {error}"));
        let validation = backend
            .validate(&result.shape, TOLERANCE)
            .unwrap_or_else(|error| panic!("validation failed at supported scale {edge:e}: {error}"));
        assert_eq!(validation, ValidationResult {
            valid: true,
            manifold: true,
            message: None,
        });
    }
}

pub fn assert_bounding_box_contract<B: GeometryBackend>(backend: &B) {
    let solid = backend
        .box_solid(10.0, 20.0, 30.0, TOLERANCE)
        .expect("box construction should succeed")
        .shape;
    let bounds = backend
        .bounding_box(&solid, TOLERANCE)
        .expect("bounding-box measurement should succeed");

    assert_box(
        bounds,
        BoundingBox {
            min_x: 0.0,
            min_y: 0.0,
            min_z: 0.0,
            max_x: 10.0,
            max_y: 20.0,
            max_z: 30.0,
        },
    );
}

pub fn assert_topology_counts_contract<B: GeometryBackend>(backend: &B) {
    let solid = backend
        .box_solid(10.0, 20.0, 30.0, TOLERANCE)
        .expect("box construction should succeed")
        .shape;
    let counts = backend
        .topology_counts(&solid, TOLERANCE)
        .expect("topology count query should succeed");
    assert_eq!(counts, TopologyCounts {
        solids: 1,
        shells: 1,
        faces: 6,
        edges: 12,
        vertices: 8,
    });
}

pub fn assert_translation_and_rotation_change_bounds_predictably<B: GeometryBackend>(backend: &B) {
    let solid = backend
        .box_solid(10.0, 20.0, 30.0, TOLERANCE)
        .expect("box construction should succeed")
        .shape;

    let translated = backend
        .translate(&solid, 100.0, -200.0, 300.0, TOLERANCE)
        .expect("translation should succeed")
        .shape;
    assert_box(
        backend.bounding_box(&translated, TOLERANCE).unwrap(),
        BoundingBox {
            min_x: 100.0,
            min_y: -200.0,
            min_z: 300.0,
            max_x: 110.0,
            max_y: -180.0,
            max_z: 330.0,
        },
    );

    let rotated = backend
        .rotate(
            &solid,
            0.0,
            0.0,
            1.0,
            core::f64::consts::FRAC_PI_2,
            TOLERANCE,
        )
        .expect("rotation should succeed")
        .shape;
    assert_box(
        backend.bounding_box(&rotated, TOLERANCE).unwrap(),
        BoundingBox {
            min_x: -20.0,
            min_y: 0.0,
            min_z: 0.0,
            max_x: 0.0,
            max_y: 10.0,
            max_z: 30.0,
        },
    );
}

pub fn assert_transform_algebra<B: GeometryBackend>(backend: &B) {
    let solid = backend
        .box_solid(10.0, 20.0, 30.0, TOLERANCE)
        .expect("box construction should succeed")
        .shape;

    let identity_translation = backend
        .translate(&solid, 0.0, 0.0, 0.0, TOLERANCE)
        .expect("zero translation should succeed")
        .shape;
    assert_box(
        backend.bounding_box(&identity_translation, TOLERANCE).unwrap(),
        backend.bounding_box(&solid, TOLERANCE).unwrap(),
    );

    let first = backend
        .translate(&solid, 11.0, -7.0, 3.0, TOLERANCE)
        .expect("first translation should succeed")
        .shape;
    let sequential = backend
        .translate(&first, -4.0, 9.0, 8.0, TOLERANCE)
        .expect("second translation should succeed")
        .shape;
    let direct = backend
        .translate(&solid, 7.0, 2.0, 11.0, TOLERANCE)
        .expect("composed translation should succeed")
        .shape;
    assert_box(
        backend.bounding_box(&sequential, TOLERANCE).unwrap(),
        backend.bounding_box(&direct, TOLERANCE).unwrap(),
    );

    let identity_rotation = backend
        .rotate(&solid, 0.0, 0.0, 1.0, 0.0, TOLERANCE)
        .expect("zero-angle rotation should succeed")
        .shape;
    assert_box(
        backend.bounding_box(&identity_rotation, TOLERANCE).unwrap(),
        backend.bounding_box(&solid, TOLERANCE).unwrap(),
    );

    let full_rotation = backend
        .rotate(
            &solid,
            0.0,
            0.0,
            1.0,
            2.0 * core::f64::consts::PI,
            TOLERANCE,
        )
        .expect("full rotation should succeed")
        .shape;
    assert_box(
        backend.bounding_box(&full_rotation, TOLERANCE).unwrap(),
        backend.bounding_box(&solid, TOLERANCE).unwrap(),
    );

    assert_eq!(backend.validate(&sequential, TOLERANCE).unwrap(), ValidationResult {
        valid: true,
        manifold: true,
        message: None,
    });
}

pub fn assert_translation_preserves_validation<B: GeometryBackend>(backend: &B) {
    let solid = backend
        .box_solid(10.0, 20.0, 30.0, TOLERANCE)
        .expect("box construction should succeed")
        .shape;

    let translated = backend
        .translate(&solid, 1000.0, -2000.0, 3000.0, TOLERANCE)
        .expect("translation should succeed")
        .shape;

    let before = backend
        .validate(&solid, TOLERANCE)
        .expect("validation should succeed");
    let after = backend
        .validate(&translated, TOLERANCE)
        .expect("validation should succeed");

    assert_eq!(before, ValidationResult { valid: true, manifold: true, message: None });
    assert_eq!(after, before);
}

pub fn assert_rotation_preserves_validation<B: GeometryBackend>(backend: &B) {
    let solid = backend
        .box_solid(10.0, 20.0, 30.0, TOLERANCE)
        .expect("box construction should succeed")
        .shape;

    for angle in [0.0, core::f64::consts::FRAC_PI_2, core::f64::consts::PI] {
        let rotated = backend
            .rotate(&solid, 0.0, 0.0, 1.0, angle, TOLERANCE)
            .unwrap_or_else(|error| panic!("rotation failed for angle {angle}: {error}"));
        assert_eq!(rotated.kind, GeometryKind::Solid);
        assert_eq!(rotated.evidence.tolerance, TOLERANCE);
        assert_eq!(
            backend.validate(&rotated.shape, TOLERANCE).unwrap(),
            ValidationResult {
                valid: true,
                manifold: true,
                message: None,
            }
        );
    }
}

pub fn assert_invalid_rotation_is_rejected<B: GeometryBackend>(backend: &B) {
    let solid = backend
        .box_solid(10.0, 20.0, 30.0, TOLERANCE)
        .expect("box construction should succeed")
        .shape;

    for input in [
        (0.0, 0.0, 0.0, 0.0),
        (0.0, 0.0, 1.0, f64::NAN),
        (f64::INFINITY, 0.0, 1.0, 0.0),
    ] {
        match backend.rotate(&solid, input.0, input.1, input.2, input.3, TOLERANCE) {
            Err(GeometryError::InvalidInput(_)) => {}
            Err(other) => panic!("unexpected rotation error: {other}"),
            Ok(_) => panic!("invalid rotation unexpectedly succeeded"),
        }
    }
}

pub fn assert_deterministic_validation<B: GeometryBackend>(backend: &B) {
    let mut reference = None;
    for _ in 0..32 {
        let shape = backend
            .box_solid(37.0, 11.0, 5.0, TOLERANCE)
            .expect("deterministic box should construct")
            .shape;
        let current = backend
            .validate(&shape, TOLERANCE)
            .expect("deterministic validation should succeed");
        match &reference {
            Some(expected) => assert_eq!(&current, expected),
            None => reference = Some(current),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use umlcad_v6_occt_backend::OcctBackend;

    #[test]
    fn occt_identity_contract_is_stable() {
        assert_backend_identity(&OcctBackend::new(), "occt");
    }

    #[test]
    fn occt_box_contract() {
        assert_box_requires_positive_dimensions(&OcctBackend::new());
    }

    #[test]
    fn occt_reference_resolution_contract() {
        assert_reference_backend_resolution_is_explicit(&OcctBackend::new());
    }

    #[test]
    fn occt_invalid_tolerance_contract() {
        assert_invalid_tolerance_is_rejected(&OcctBackend::new());
    }

    #[test]
    fn occt_numeric_scale_contract() {
        assert_numeric_scale_survives_validation(&OcctBackend::new());
    }

    #[test]
    fn occt_bounding_box_contract() {
        assert_bounding_box_contract(&OcctBackend::new());
    }

    #[test]
    fn occt_topology_counts_contract() {
        assert_topology_counts_contract(&OcctBackend::new());
    }

    #[test]
    fn occt_transform_measurement_contract() {
        assert_translation_and_rotation_change_bounds_predictably(&OcctBackend::new());
    }

    #[test]
    fn occt_transform_algebra_contract() {
        assert_transform_algebra(&OcctBackend::new());
    }

    #[test]
    fn occt_translation_validation_contract() {
        assert_translation_preserves_validation(&OcctBackend::new());
    }

    #[test]
    fn occt_rotation_validation_contract() {
        assert_rotation_preserves_validation(&OcctBackend::new());
    }

    #[test]
    fn occt_invalid_rotation_contract() {
        assert_invalid_rotation_is_rejected(&OcctBackend::new());
    }

    #[test]
    fn occt_determinism_contract() {
        assert_deterministic_validation(&OcctBackend::new());
    }
}
