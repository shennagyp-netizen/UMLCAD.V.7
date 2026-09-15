use umlcad_v6_geometry_api::{GeometryBackend, ToleranceContext, TopologyCounts, ValidationResult};
use umlcad_v6_occt_backend::OcctBackend;

const T: ToleranceContext = ToleranceContext { modeling: 1e-9, validation: 1e-6 };

fn valid() -> ValidationResult { ValidationResult { valid: true, manifold: true, message: None } }

fn assert_valid_solid(backend: &OcctBackend, shape: &umlcad_v6_occt_backend::OcctShape) {
    assert_eq!(backend.validate(shape, T).unwrap(), valid());
}

#[test]
fn canonical_primitives_have_explicit_valid_manifold_status() {
    let backend = OcctBackend::new();
    let box_shape = backend.box_solid(10., 20., 30., T).unwrap().shape;
    let cylinder = backend.cylinder_solid(5., 20., T).unwrap().shape;
    let sphere = backend.sphere_solid(5., T).unwrap().shape;
    let cone = backend.cone_solid(10., 5., 20., T).unwrap().shape;
    for shape in [&box_shape, &cylinder, &sphere, &cone] { assert_valid_solid(&backend, shape); }
}

#[test]
fn constructed_features_have_explicit_valid_manifold_status() {
    let backend = OcctBackend::new();
    let extrusion = backend.extrude_polygon(&[(0., 0.), (10., 0.), (10., 20.), (0., 20.)], 30., T).unwrap().shape;
    let revolution = backend.revolve_polygon(&[(5., 0.), (10., 0.), (10., 20.), (5., 20.)], std::f64::consts::TAU, T).unwrap().shape;
    let loft = backend.loft_between_polygons(&[(0., 0.), (20., 0.), (20., 10.), (0., 10.)], 0., &[(2., 1.), (18., 1.), (18., 9.), (2., 9.)], 30., T).unwrap().shape;
    let source = backend.box_solid(20., 20., 20., T).unwrap().shape;
    let fillet = backend.fillet_all_edges(&source, 2., T).unwrap().shape;
    let chamfer = backend.chamfer_all_edges(&source, 2., T).unwrap().shape;
    for shape in [&extrusion, &revolution, &loft, &fillet, &chamfer] { assert_valid_solid(&backend, shape); }
}

#[test]
fn boolean_results_preserve_explicit_manifold_status() {
    let backend = OcctBackend::new();
    let left = backend.box_solid(10., 10., 10., T).unwrap().shape;
    let right_base = backend.box_solid(10., 10., 10., T).unwrap().shape;
    let right = backend.translate(&right_base, 5., 0., 0., T).unwrap().shape;
    let fuse = backend.fuse(&left, &right, T).unwrap().shape;
    let common = backend.common(&left, &right, T).unwrap().shape;
    let cut = backend.cut(&left, &right, T).unwrap().shape;
    for shape in [&fuse, &common, &cut] { assert_valid_solid(&backend, shape); }
}

#[test]
fn disjoint_multi_solid_fuse_reports_two_solids_without_losing_manifold_status() {
    let backend = OcctBackend::new();
    let left = backend.box_solid(10., 10., 10., T).unwrap().shape;
    let right_base = backend.box_solid(10., 10., 10., T).unwrap().shape;
    let right = backend.translate(&right_base, 20., 0., 0., T).unwrap().shape;
    let fused = backend.fuse(&left, &right, T).unwrap().shape;
    assert_eq!(backend.topology_counts(&fused, T).unwrap().solids, 2);
    assert_valid_solid(&backend, &fused);
}

#[test]
fn topology_counts_are_compatible_with_closed_box_boundary() {
    let backend = OcctBackend::new();
    let shape = backend.box_solid(10., 20., 30., T).unwrap().shape;
    let counts = backend.topology_counts(&shape, T).unwrap();
    assert_eq!(counts, TopologyCounts { solids: 1, shells: 1, faces: 6, edges: 12, vertices: 8 });
    assert_eq!(counts.faces * 4, counts.edges * 2);
    assert_eq!(counts.edges * 2, counts.faces * 4);
}

#[test]
fn polyhedral_box_satisfies_euler_characteristic() {
    let backend = OcctBackend::new();
    let counts = backend.topology_counts(&backend.box_solid(10., 20., 30., T).unwrap().shape, T).unwrap();
    assert_eq!(counts.vertices as i64 - counts.edges as i64 + counts.faces as i64, 2);
}
