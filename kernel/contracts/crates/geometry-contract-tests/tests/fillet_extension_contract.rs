use umlcad_v6_fillet_api::{BoxAllEdgesFillet, FilletBackend};
use umlcad_v6_geometry_api::{GeometryBackend, GeometryError, GeometryKind, GeometryStatus, ToleranceContext};
use umlcad_v6_occt_backend::OcctBackend;

const T: ToleranceContext = ToleranceContext { modeling: 1e-9, validation: 1e-9 };

fn definition() -> BoxAllEdgesFillet {
    BoxAllEdgesFillet { width: 20.0, depth: 30.0, height: 40.0, radius: 2.0 }
}

#[test]
fn bounded_box_fillet_has_valid_solid_and_topology_change_evidence() {
    let backend = OcctBackend::new();
    let d = definition();
    let result = backend.fillet_box_all_edges(d, T).unwrap();

    assert_eq!(result.kind, GeometryKind::Solid);
    assert_eq!(result.evidence.status, GeometryStatus::Success);
    let validation = backend.validate(&result.shape, T).unwrap();
    assert!(validation.valid);
    assert!(validation.manifold);

    let bbox = backend.bounding_box(&result.shape, T).unwrap();
    assert!((bbox.min_x - 0.0).abs() <= 1e-9);
    assert!((bbox.max_x - 20.0).abs() <= 1e-9);
    assert!((bbox.min_y - 0.0).abs() <= 1e-9);
    assert!((bbox.max_y - 30.0).abs() <= 1e-9);
    assert!((bbox.min_z - 0.0).abs() <= 1e-9);
    assert!((bbox.max_z - 40.0).abs() <= 1e-9);

    let topology = backend.topology_counts(&result.shape, T).unwrap();
    assert_eq!(topology.solids, 1);
    assert_eq!(topology.shells, 1);
    assert!(topology.faces > 6);
    assert!(topology.edges > 12);
    assert!(topology.vertices > 8);
}

#[test]
fn bounded_box_fillet_is_deterministic_and_source_immutable() {
    let backend = OcctBackend::new();
    let d = definition();
    let source = backend.box_solid(d.width, d.depth, d.height, T).unwrap().shape;
    let first = backend.fillet_all_edges(&source, d.radius, T).unwrap();
    let second = backend.fillet_all_edges(&source, d.radius, T).unwrap();

    assert_eq!(backend.topology_counts(&first.shape, T).unwrap(), backend.topology_counts(&second.shape, T).unwrap());
    assert_eq!(backend.bounding_box(&first.shape, T).unwrap(), backend.bounding_box(&second.shape, T).unwrap());
    let source_topology = backend.topology_counts(&source, T).unwrap();
    assert_eq!(source_topology.faces, 6);
    assert_eq!(source_topology.edges, 12);
    assert_eq!(source_topology.vertices, 8);
}

#[test]
fn semantic_contract_rejects_impossible_radius_before_native_realization() {
    let backend = OcctBackend::new();
    let mut d = definition();
    d.radius = 10.0;
    assert_eq!(d.validate(T), Err(GeometryError::InvalidInput("box fillet radius must be strictly less than half the minimum box dimension")));
    assert!(backend.fillet_box_all_edges(d, T).is_err());
}

#[test]
fn semantic_contract_rejects_nonfinite_and_degenerate_dimensions() {
    let backend = OcctBackend::new();
    let mut d = definition();
    d.width = f64::NAN;
    assert!(matches!(backend.fillet_box_all_edges(d, T), Err(GeometryError::InvalidInput("box fillet dimensions and radius must be finite"))));
    let mut d = definition();
    d.height = 0.0;
    assert!(matches!(backend.fillet_box_all_edges(d, T), Err(GeometryError::InvalidInput("box fillet dimensions must exceed modeling tolerance"))));
    let mut d = definition();
    d.radius = f64::INFINITY;
    assert!(matches!(backend.fillet_box_all_edges(d, T), Err(GeometryError::InvalidInput("box fillet dimensions and radius must be finite"))));
}
