use umlcad_v6_geometry_api::{GeometryBackend, GeometryError, GeometryKind, GeometryStatus, ToleranceContext};
use umlcad_v6_occt_backend::OcctBackend;
use umlcad_v6_shell_api::{ClosedBoxThickness, ShellBackend};

const T: ToleranceContext = ToleranceContext { modeling: 1e-9, validation: 1e-9 };

fn definition() -> ClosedBoxThickness {
    ClosedBoxThickness { width: 20.0, depth: 30.0, height: 40.0, thickness: 2.0 }
}

#[test]
fn closed_box_thickness_produces_valid_hollow_solid() {
    let backend = OcctBackend::new();
    let d = definition();
    let result = backend.make_closed_box_thickness(d, T).unwrap();
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
    assert!(topology.shells >= 1);
    assert!(topology.faces > 6);
    assert!(topology.edges > 12);
    assert!(topology.vertices > 8);
}

#[test]
fn material_volume_formula_is_exact_and_has_positive_wall_volume() {
    let d = definition();
    assert_eq!(d.inner_dimensions(T).unwrap(), (16.0, 26.0, 36.0));
    assert_eq!(d.outer_volume(T).unwrap(), 24000.0);
    assert_eq!(d.inner_volume(T).unwrap(), 14976.0);
    assert_eq!(d.material_volume(T).unwrap(), 9024.0);
    assert!(d.material_volume(T).unwrap() > 0.0);
}

#[test]
fn closed_box_thickness_is_deterministic_and_source_immutable() {
    let backend = OcctBackend::new();
    let d = definition();
    let outer = backend.box_solid(d.width, d.depth, d.height, T).unwrap().shape;
    let first = backend.make_closed_box_thickness(d, T).unwrap();
    let second = backend.make_closed_box_thickness(d, T).unwrap();
    assert_eq!(backend.topology_counts(&first.shape, T).unwrap(), backend.topology_counts(&second.shape, T).unwrap());
    assert_eq!(backend.bounding_box(&first.shape, T).unwrap(), backend.bounding_box(&second.shape, T).unwrap());
    let outer_topology = backend.topology_counts(&outer, T).unwrap();
    assert_eq!(outer_topology.solids, 1);
    assert_eq!(outer_topology.shells, 1);
    assert_eq!(outer_topology.faces, 6);
    assert_eq!(outer_topology.edges, 12);
    assert_eq!(outer_topology.vertices, 8);
}

#[test]
fn semantic_contract_rejects_thin_or_degenerate_inputs() {
    let backend = OcctBackend::new();
    let mut d = definition();
    d.thickness = 10.0;
    assert_eq!(d.validate(T), Err(GeometryError::InvalidInput("box thickness must be strictly less than half the minimum outer dimension")));
    assert!(backend.make_closed_box_thickness(d, T).is_err());
    let mut d = definition();
    d.height = 0.0;
    assert!(backend.make_closed_box_thickness(d, T).is_err());
    let mut d = definition();
    d.thickness = f64::NAN;
    assert!(backend.make_closed_box_thickness(d, T).is_err());
}
