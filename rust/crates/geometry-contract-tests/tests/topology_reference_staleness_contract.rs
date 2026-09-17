use umlcad_v6_geometry_api::{EdgeDescriptor, FaceDescriptor, GeometryBackend, ReferenceResolution, ToleranceContext, VertexDescriptor};
use umlcad_v6_occt_backend::OcctBackend;

const T: ToleranceContext = ToleranceContext { modeling: 1e-9, validation: 1e-6 };

fn bottom_face(values: &[FaceDescriptor]) -> FaceDescriptor { values.iter().find(|d| d.bounds.min_z == 0.0 && d.bounds.max_z == 0.0).copied().unwrap() }
fn unique_edge(values: &[EdgeDescriptor]) -> EdgeDescriptor { values.iter().copied().find(|v| values.iter().filter(|o| **o == *v).count() == 1).unwrap() }
fn top_vertex(values: &[VertexDescriptor]) -> VertexDescriptor { values.iter().find(|v| v.x == 10.0 && v.y == 20.0 && v.z == 30.0).copied().unwrap() }

#[test]
fn face_reference_must_not_be_guessed_after_topology_change() {
    let backend = OcctBackend::new();
    let source = backend.box_solid(20., 20., 20., T).unwrap().shape;
    let reference = bottom_face(&backend.face_descriptors(&source, T).unwrap());
    let changed = backend.fillet_all_edges(&source, 2., T).unwrap().shape;

    let resolution = backend.resolve_face_descriptor(&changed, &reference, T).unwrap();
    assert!(!matches!(resolution, ReferenceResolution::Unique(_)));
}

#[test]
fn edge_reference_must_not_be_guessed_after_topology_change() {
    let backend = OcctBackend::new();
    let source = backend.box_solid(10., 20., 30., T).unwrap().shape;
    let reference = unique_edge(&backend.edge_descriptors(&source, T).unwrap());
    let changed = backend.chamfer_all_edges(&source, 2., T).unwrap().shape;

    let resolution = backend.resolve_edge_descriptor(&changed, &reference, T).unwrap();
    assert!(!matches!(resolution, ReferenceResolution::Unique(_)));
}

#[test]
fn vertex_reference_must_not_be_guessed_after_geometric_change() {
    let backend = OcctBackend::new();
    let source = backend.box_solid(10., 20., 30., T).unwrap().shape;
    let reference = top_vertex(&backend.vertex_descriptors(&source, T).unwrap());
    let changed = backend.translate(&source, 100., 0., 0., T).unwrap().shape;

    let resolution = backend.resolve_vertex_descriptor(&changed, &reference, T).unwrap();
    assert_eq!(resolution, ReferenceResolution::NotFound);
}

#[test]
fn topology_change_is_explicit_even_when_result_remains_valid() {
    let backend = OcctBackend::new();
    let source = backend.box_solid(20., 20., 20., T).unwrap().shape;
    let changed = backend.fillet_all_edges(&source, 2., T).unwrap().shape;

    assert!(backend.validate(&source, T).unwrap().valid);
    assert!(backend.validate(&changed, T).unwrap().valid);
    assert_ne!(backend.topology_counts(&source, T).unwrap(), backend.topology_counts(&changed, T).unwrap());
}
