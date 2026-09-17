use umlcad_v6_geometry_api::{GeometryBackend, GeometryError, GeometryKind, ToleranceContext, ValidationResult};
use umlcad_v6_occt_backend::OcctBackend;

const T: ToleranceContext = ToleranceContext { modeling: 1e-9, validation: 1e-6 };

#[test]
fn ring_torus_is_a_valid_manifold_solid() {
    let backend = OcctBackend::new();
    let result = backend.torus_solid(20.0, 5.0, T).unwrap();
    assert_eq!(result.kind, GeometryKind::Solid);
    assert_eq!(result.evidence.backend, "occt");
    assert_eq!(backend.validate(&result.shape, T).unwrap(), ValidationResult { valid: true, manifold: true, message: None });
}

#[test]
fn ring_torus_has_expected_axis_aligned_bounds_within_declared_validation_tolerance() {
    let backend = OcctBackend::new();
    let shape = backend.torus_solid(20.0, 5.0, T).unwrap().shape;
    let bounds = backend.bounding_box(&shape, T).unwrap();
    assert!((bounds.min_x + 25.0).abs() <= T.validation, "x bounds: {:?}", bounds);
    assert!((bounds.max_x - 25.0).abs() <= T.validation, "x bounds: {:?}", bounds);
    assert!((bounds.min_y + 25.0).abs() <= T.validation, "y bounds: {:?}", bounds);
    assert!((bounds.max_y - 25.0).abs() <= T.validation, "y bounds: {:?}", bounds);
    assert!((bounds.min_z + 5.0).abs() <= T.validation, "z bounds: {:?}", bounds);
    assert!((bounds.max_z - 5.0).abs() <= T.validation, "z bounds: {:?}", bounds);
}

#[test]
fn ring_torus_rejects_degenerate_self_intersecting_or_non_finite_parameters() {
    let backend = OcctBackend::new();
    for (major, minor) in [(0.0,5.0),(-1.0,5.0),(20.0,0.0),(20.0,-1.0),(5.0,5.0),(4.0,5.0),(f64::NAN,5.0),(20.0,f64::INFINITY)] {
        match backend.torus_solid(major, minor, T) {
            Err(GeometryError::InvalidInput(_)) => {}
            Err(error) => panic!("unexpected torus error: {error:?}"),
            Ok(_) => panic!("invalid torus parameters unexpectedly succeeded"),
        }
    }
}

#[test]
fn ring_torus_is_immutable_and_deterministic() {
    let backend = OcctBackend::new();
    let source = backend.torus_solid(20.0, 5.0, T).unwrap().shape;
    let first = backend.translate(&source, 100.0, -200.0, 300.0, T).unwrap().shape;
    let second = backend.translate(&source, 100.0, -200.0, 300.0, T).unwrap().shape;
    assert_eq!(backend.bounding_box(&first,T).unwrap(), backend.bounding_box(&second,T).unwrap());
    assert_eq!(backend.topology_counts(&first,T).unwrap(), backend.topology_counts(&second,T).unwrap());
    assert_eq!(backend.bounding_box(&source,T).unwrap(), backend.bounding_box(&backend.torus_solid(20.0,5.0,T).unwrap().shape,T).unwrap());
    assert!(backend.validate(&source,T).unwrap().valid);
    assert!(backend.validate(&first,T).unwrap().valid);
}

#[test]
fn ring_torus_periodic_topology_exposes_valid_geometric_descriptors() {
    let backend = OcctBackend::new();
    let shape = backend.torus_solid(20.0, 5.0, T).unwrap().shape;
    let counts = backend.topology_counts(&shape, T).unwrap();
    assert_eq!(counts.solids, 1);
    assert_eq!(counts.shells, 1);
    assert!(counts.faces >= 1);
    assert!(counts.edges >= 1);

    let faces = backend.face_descriptors(&shape, T).unwrap();
    assert!(!faces.is_empty());
    assert!(faces.iter().all(|face| face.area.is_finite() && face.area > 0.0 && face.boundary_edge_count >= 1));

    let edges = backend.edge_descriptors(&shape, T).unwrap();
    assert!(!edges.is_empty());
    assert!(edges.iter().all(|edge| edge.length.is_finite() && edge.length >= 0.0 && edge.face_use_count >= 1 && edge.vertex_use_count >= 1));

    let vertices = backend.vertex_descriptors(&shape, T).unwrap();
    assert!(vertices.iter().all(|vertex| vertex.x.is_finite() && vertex.y.is_finite() && vertex.z.is_finite() && vertex.edge_use_count >= 1 && vertex.face_use_count >= 1));

    assert_eq!(backend.face_descriptors(&shape, T).unwrap(), faces);
    assert_eq!(backend.edge_descriptors(&shape, T).unwrap(), edges);
    assert_eq!(backend.vertex_descriptors(&shape, T).unwrap(), vertices);
}
