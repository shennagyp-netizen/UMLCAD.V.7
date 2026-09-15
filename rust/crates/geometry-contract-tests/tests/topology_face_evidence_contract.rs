use umlcad_v6_geometry_api::{FaceDescriptor, GeometryBackend, ToleranceContext};
use umlcad_v6_occt_backend::OcctBackend;

const T: ToleranceContext = ToleranceContext { modeling: 1e-9, validation: 1e-9 };

fn sorted_descriptors(mut values: Vec<FaceDescriptor>) -> Vec<FaceDescriptor> {
    values.sort_by(|a, b| {
        a.area
            .total_cmp(&b.area)
            .then_with(|| a.bounds.min_x.total_cmp(&b.bounds.min_x))
            .then_with(|| a.bounds.min_y.total_cmp(&b.bounds.min_y))
            .then_with(|| a.bounds.min_z.total_cmp(&b.bounds.min_z))
            .then_with(|| a.bounds.max_x.total_cmp(&b.bounds.max_x))
            .then_with(|| a.bounds.max_y.total_cmp(&b.bounds.max_y))
            .then_with(|| a.bounds.max_z.total_cmp(&b.bounds.max_z))
            .then_with(|| a.boundary_edge_count.cmp(&b.boundary_edge_count))
    });
    values
}

fn has_planar_z_face(values: &[FaceDescriptor], z: f64) -> bool {
    values.iter().any(|d| (d.bounds.min_z - z).abs() <= 1e-9 && (d.bounds.max_z - z).abs() <= 1e-9)
}

#[test]
fn box_face_evidence_has_six_deterministic_planar_faces() {
    let backend = OcctBackend::new();
    let box_shape = backend.box_solid(20.0, 20.0, 20.0, T).unwrap().shape;
    let descriptors = sorted_descriptors(backend.face_descriptors(&box_shape, T).unwrap());

    assert_eq!(descriptors.len(), 6);
    for descriptor in &descriptors {
        assert!(descriptor.area.is_finite());
        assert!((descriptor.area - 400.0).abs() <= 1e-9);
        assert_eq!(descriptor.boundary_edge_count, 4);
        assert!(descriptor.bounds.validate().is_ok());
    }

    assert!(has_planar_z_face(&descriptors, 0.0));
    assert!(has_planar_z_face(&descriptors, 20.0));
}

#[test]
fn repeated_face_evidence_is_deterministic() {
    let backend = OcctBackend::new();
    let box_shape = backend.box_solid(20.0, 20.0, 20.0, T).unwrap().shape;
    assert_eq!(
        sorted_descriptors(backend.face_descriptors(&box_shape, T).unwrap()),
        sorted_descriptors(backend.face_descriptors(&box_shape, T).unwrap())
    );
}

#[test]
fn duplicate_geometric_descriptors_remain_ambiguous_not_semantic_ids() {
    let backend = OcctBackend::new();
    let source = backend.box_solid(20.0, 20.0, 20.0, T).unwrap().shape;
    let filleted = backend.fillet_all_edges(&source, 2.0, T).unwrap().shape;
    let descriptors = sorted_descriptors(backend.face_descriptors(&filleted, T).unwrap());

    let duplicate_count = descriptors
        .windows(2)
        .filter(|pair| pair[0] == pair[1])
        .count();
    assert!(duplicate_count > 0 || descriptors.len() > 6);
}
