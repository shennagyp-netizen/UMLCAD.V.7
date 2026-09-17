use umlcad_v6_geometry_api::{GeometryBackend, GeometryKind, ToleranceContext};
use umlcad_v6_occt_backend::OcctBackend;

const TOLERANCE: ToleranceContext = ToleranceContext { modeling: 1e-9, validation: 1e-9 };

#[test]
fn overlapping_box_fuse_produces_valid_solid_with_expected_bounds() {
    let backend = OcctBackend::new();
    let left = backend.box_solid(10.0, 10.0, 10.0, TOLERANCE).unwrap().shape;
    let right_base = backend.box_solid(10.0, 10.0, 10.0, TOLERANCE).unwrap().shape;
    let right = backend.translate(&right_base, 5.0, 0.0, 0.0, TOLERANCE).unwrap().shape;

    let fused = backend.fuse(&left, &right, TOLERANCE).unwrap();
    assert_eq!(fused.kind, GeometryKind::Solid);
    assert!(backend.validate(&fused.shape, TOLERANCE).unwrap().valid);

    let bounds = backend.bounding_box(&fused.shape, TOLERANCE).unwrap();
    assert!((bounds.min_x - 0.0).abs() <= 1e-9);
    assert!((bounds.max_x - 15.0).abs() <= 1e-9);
    assert!((bounds.min_y - 0.0).abs() <= 1e-9);
    assert!((bounds.max_y - 10.0).abs() <= 1e-9);
    assert!((bounds.min_z - 0.0).abs() <= 1e-9);
    assert!((bounds.max_z - 10.0).abs() <= 1e-9);
}

#[test]
fn fuse_result_is_deterministic_for_identical_inputs() {
    let backend = OcctBackend::new();
    let left = backend.box_solid(10.0, 10.0, 10.0, TOLERANCE).unwrap().shape;
    let right = backend.translate(
        &backend.box_solid(10.0, 10.0, 10.0, TOLERANCE).unwrap().shape,
        5.0,
        0.0,
        0.0,
        TOLERANCE,
    ).unwrap().shape;

    let first = backend.fuse(&left, &right, TOLERANCE).unwrap().shape;
    let second = backend.fuse(&left, &right, TOLERANCE).unwrap().shape;

    assert_eq!(backend.bounding_box(&first, TOLERANCE).unwrap(), backend.bounding_box(&second, TOLERANCE).unwrap());
    assert_eq!(backend.topology_counts(&first, TOLERANCE).unwrap(), backend.topology_counts(&second, TOLERANCE).unwrap());
}
