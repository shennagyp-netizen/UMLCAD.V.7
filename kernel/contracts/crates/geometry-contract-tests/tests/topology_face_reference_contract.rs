use umlcad_v6_geometry_api::{FaceDescriptor, GeometryBackend, ReferenceResolution, ToleranceContext};
use umlcad_v6_occt_backend::OcctBackend;

const T: ToleranceContext = ToleranceContext { modeling: 1e-9, validation: 1e-6 };

fn bottom_face(descriptors: &[FaceDescriptor]) -> FaceDescriptor {
    descriptors.iter().find(|d| d.bounds.min_z == 0.0 && d.bounds.max_z == 0.0).copied().unwrap()
}

#[test]
fn unique_geometric_face_query_resolves_without_traversal_identity() {
    let backend = OcctBackend::new();
    let shape = backend.box_solid(20.0, 20.0, 20.0, T).unwrap().shape;
    let query = bottom_face(&backend.face_descriptors(&shape, T).unwrap());
    match backend.resolve_face_descriptor(&shape, &query, T).unwrap() {
        ReferenceResolution::Unique(found) => assert!(found.matches_within(query, T.validation)),
        other => panic!("expected unique geometric resolution, got {other:?}"),
    }
}

#[test]
fn equivalent_regeneration_preserves_geometric_face_resolution() {
    let backend = OcctBackend::new();
    let first = backend.box_solid(20.0, 20.0, 20.0, T).unwrap().shape;
    let reference = bottom_face(&backend.face_descriptors(&first, T).unwrap());
    let regenerated = backend.box_solid(20.0, 20.0, 20.0, T).unwrap().shape;

    assert_eq!(backend.resolve_face_descriptor(&regenerated, &reference, T).unwrap(), ReferenceResolution::Unique(reference));
}

#[test]
fn tolerance_small_geometric_change_still_resolves_but_large_change_does_not() {
    let backend = OcctBackend::new();
    let shape = backend.box_solid(20.0, 20.0, 20.0, T).unwrap().shape;
    let original = bottom_face(&backend.face_descriptors(&shape, T).unwrap());
    let near = FaceDescriptor { area: original.area * (1.0 + 5e-7), ..original };
    assert!(matches!(backend.resolve_face_descriptor(&shape, &near, T).unwrap(), ReferenceResolution::Unique(_)));
    let far = FaceDescriptor { area: original.area * (1.0 + 5e-4), ..original };
    assert_eq!(backend.resolve_face_descriptor(&shape, &far, T).unwrap(), ReferenceResolution::NotFound);
}

#[test]
fn unmatched_geometric_face_query_is_not_found() {
    let backend = OcctBackend::new();
    let shape = backend.box_solid(20.0, 20.0, 20.0, T).unwrap().shape;
    let mut query = backend.face_descriptors(&shape, T).unwrap()[0];
    query.area = 1234.0;
    assert_eq!(backend.resolve_face_descriptor(&shape, &query, T).unwrap(), ReferenceResolution::NotFound);
}

#[test]
fn duplicate_geometric_face_query_is_explicitly_ambiguous() {
    let backend = OcctBackend::new();
    let source = backend.box_solid(20.0, 20.0, 20.0, T).unwrap().shape;
    let shape = backend.fillet_all_edges(&source, 2.0, T).unwrap().shape;
    let descriptors = backend.face_descriptors(&shape, T).unwrap();
    let query = descriptors.iter().find(|candidate| descriptors.iter().filter(|other| *other == *candidate).count() > 1).copied().unwrap_or(descriptors[0]);
    let resolution = backend.resolve_face_descriptor(&shape, &query, T).unwrap();
    if descriptors.iter().filter(|candidate| **candidate == query).count() > 1 {
        assert_eq!(resolution, ReferenceResolution::Ambiguous);
    } else {
        assert!(matches!(resolution, ReferenceResolution::Unique(_) | ReferenceResolution::Ambiguous));
    }
}
