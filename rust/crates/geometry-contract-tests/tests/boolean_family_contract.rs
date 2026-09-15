use umlcad_v6_geometry_api::{GeometryBackend, GeometryError, ToleranceContext};
use umlcad_v6_occt_backend::OcctBackend;

const TOLERANCE: ToleranceContext = ToleranceContext { modeling: 1e-9, validation: 1e-9 };

fn shifted_box(backend: &OcctBackend, x: f64) -> umlcad_v6_occt_backend::OcctShape {
    let base = backend.box_solid(10.0, 10.0, 10.0, TOLERANCE).unwrap().shape;
    backend.translate(&base, x, 0.0, 0.0, TOLERANCE).unwrap().shape
}

fn equivalent_geometry(backend: &OcctBackend, left: &umlcad_v6_occt_backend::OcctShape, right: &umlcad_v6_occt_backend::OcctShape) {
    assert_eq!(backend.bounding_box(left, TOLERANCE).unwrap(), backend.bounding_box(right, TOLERANCE).unwrap());
    assert_eq!(backend.topology_counts(left, TOLERANCE).unwrap(), backend.topology_counts(right, TOLERANCE).unwrap());
}

#[test]
fn common_and_cut_produce_valid_results() {
    let backend = OcctBackend::new();
    let left = backend.box_solid(10.0, 10.0, 10.0, TOLERANCE).unwrap().shape;
    let right = shifted_box(&backend, 5.0);
    let common = backend.common(&left, &right, TOLERANCE).unwrap().shape;
    let cut = backend.cut(&left, &right, TOLERANCE).unwrap().shape;
    assert!(backend.validate(&common, TOLERANCE).unwrap().valid);
    assert!(backend.validate(&cut, TOLERANCE).unwrap().valid);
    let common_bounds = backend.bounding_box(&common, TOLERANCE).unwrap();
    assert!((common_bounds.min_x - 5.0).abs() <= 1e-9);
    assert!((common_bounds.max_x - 10.0).abs() <= 1e-9);
    let cut_bounds = backend.bounding_box(&cut, TOLERANCE).unwrap();
    assert!((cut_bounds.min_x - 0.0).abs() <= 1e-9);
    assert!((cut_bounds.max_x - 5.0).abs() <= 1e-9);
}

#[test]
fn disjoint_fuse_remains_a_valid_multi_solid_result() {
    let backend = OcctBackend::new();
    let left = backend.box_solid(10.0, 10.0, 10.0, TOLERANCE).unwrap().shape;
    let right = shifted_box(&backend, 20.0);
    let fused = backend.fuse(&left, &right, TOLERANCE).unwrap().shape;
    assert!(backend.validate(&fused, TOLERANCE).unwrap().valid);
    assert_eq!(backend.topology_counts(&fused, TOLERANCE).unwrap().solids, 2);
    let bounds = backend.bounding_box(&fused, TOLERANCE).unwrap();
    assert!((bounds.min_x - 0.0).abs() <= 1e-9);
    assert!((bounds.max_x - 30.0).abs() <= 1e-9);
}

#[test]
fn disjoint_common_is_rejected_as_unrepresentable_empty_geometry() {
    let backend = OcctBackend::new();
    let left = backend.box_solid(10.0, 10.0, 10.0, TOLERANCE).unwrap().shape;
    let right = shifted_box(&backend, 20.0);
    assert!(matches!(backend.common(&left, &right, TOLERANCE), Err(GeometryError::Unsupported(_))));
}

#[test]
fn full_containment_cut_is_rejected_when_result_is_empty() {
    let backend = OcctBackend::new();
    let left = backend.box_solid(10.0, 10.0, 10.0, TOLERANCE).unwrap().shape;
    let inner = backend.box_solid(20.0, 20.0, 20.0, TOLERANCE).unwrap().shape;
    assert!(matches!(backend.cut(&left, &inner, TOLERANCE), Err(GeometryError::Unsupported(_))));
}

#[test]
fn face_touching_fuse_remains_one_valid_solid() {
    let backend = OcctBackend::new();
    let left = backend.box_solid(10.0, 10.0, 10.0, TOLERANCE).unwrap().shape;
    let right = shifted_box(&backend, 10.0);
    let fused = backend.fuse(&left, &right, TOLERANCE).unwrap().shape;
    assert!(backend.validate(&fused, TOLERANCE).unwrap().valid);
    assert_eq!(backend.topology_counts(&fused, TOLERANCE).unwrap().solids, 1);
    let bounds = backend.bounding_box(&fused, TOLERANCE).unwrap();
    assert!((bounds.min_x - 0.0).abs() <= 1e-9);
    assert!((bounds.max_x - 20.0).abs() <= 1e-9);
}

#[test]
fn face_touching_common_is_rejected_as_zero_volume_intersection() {
    let backend = OcctBackend::new();
    let left = backend.box_solid(10.0, 10.0, 10.0, TOLERANCE).unwrap().shape;
    let right = shifted_box(&backend, 10.0);
    assert!(matches!(backend.common(&left, &right, TOLERANCE), Err(GeometryError::Unsupported(_))));
}

#[test]
fn coincident_boxes_have_explicit_boolean_semantics() {
    let backend = OcctBackend::new();
    let left = backend.box_solid(10.0, 10.0, 10.0, TOLERANCE).unwrap().shape;
    let right = backend.box_solid(10.0, 10.0, 10.0, TOLERANCE).unwrap().shape;
    let fused = backend.fuse(&left, &right, TOLERANCE).unwrap().shape;
    assert_eq!(backend.topology_counts(&fused, TOLERANCE).unwrap().solids, 1);
    assert!(backend.validate(&fused, TOLERANCE).unwrap().valid);
    let common = backend.common(&left, &right, TOLERANCE).unwrap().shape;
    assert_eq!(backend.topology_counts(&common, TOLERANCE).unwrap().solids, 1);
    assert!(backend.validate(&common, TOLERANCE).unwrap().valid);
    assert!(matches!(backend.cut(&left, &right, TOLERANCE), Err(GeometryError::Unsupported(_))));
}

#[test]
fn boolean_union_and_common_are_commutative() {
    let backend = OcctBackend::new();
    let left = backend.box_solid(10.0, 10.0, 10.0, TOLERANCE).unwrap().shape;
    let right = shifted_box(&backend, 4.0);
    let union_lr = backend.fuse(&left, &right, TOLERANCE).unwrap().shape;
    let union_rl = backend.fuse(&right, &left, TOLERANCE).unwrap().shape;
    equivalent_geometry(&backend, &union_lr, &union_rl);
    let common_lr = backend.common(&left, &right, TOLERANCE).unwrap().shape;
    let common_rl = backend.common(&right, &left, TOLERANCE).unwrap().shape;
    equivalent_geometry(&backend, &common_lr, &common_rl);
}

#[test]
fn boolean_cut_is_explicitly_non_commutative() {
    let backend = OcctBackend::new();
    let left = backend.box_solid(10.0, 10.0, 10.0, TOLERANCE).unwrap().shape;
    let right = shifted_box(&backend, 4.0);
    let left_minus_right = backend.cut(&left, &right, TOLERANCE).unwrap().shape;
    let right_minus_left = backend.cut(&right, &left, TOLERANCE).unwrap().shape;
    let first = backend.bounding_box(&left_minus_right, TOLERANCE).unwrap();
    let second = backend.bounding_box(&right_minus_left, TOLERANCE).unwrap();
    assert!((first.min_x - second.min_x).abs() > 1e-9 || (first.max_x - second.max_x).abs() > 1e-9 || backend.topology_counts(&left_minus_right, TOLERANCE).unwrap() != backend.topology_counts(&right_minus_left, TOLERANCE).unwrap());
}

#[test]
fn near_degenerate_overlap_is_not_accepted_as_confident_common_geometry() {
    let backend = OcctBackend::new();
    let left = backend.box_solid(10.0, 10.0, 10.0, TOLERANCE).unwrap().shape;
    let right = shifted_box(&backend, 10.0 - 5e-10);
    assert!(matches!(backend.common(&left, &right, TOLERANCE), Err(GeometryError::Unsupported(_))));
}

#[test]
fn boolean_results_are_deterministic_for_identical_operands() {
    let backend = OcctBackend::new();
    let left = backend.box_solid(10.0, 10.0, 10.0, TOLERANCE).unwrap().shape;
    let right = shifted_box(&backend, 5.0);
    for operation in [0_u8, 1_u8, 2_u8] {
        let first = match operation { 0 => backend.fuse(&left,&right,TOLERANCE).unwrap().shape, 1 => backend.common(&left,&right,TOLERANCE).unwrap().shape, _ => backend.cut(&left,&right,TOLERANCE).unwrap().shape };
        let second = match operation { 0 => backend.fuse(&left,&right,TOLERANCE).unwrap().shape, 1 => backend.common(&left,&right,TOLERANCE).unwrap().shape, _ => backend.cut(&left,&right,TOLERANCE).unwrap().shape };
        equivalent_geometry(&backend, &first, &second);
    }
}
