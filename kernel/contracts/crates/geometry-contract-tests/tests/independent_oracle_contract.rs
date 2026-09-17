use umlcad_v6_geometry_api::{GeometryBackend, GeometryKind, ToleranceContext, TopologyCounts};
use umlcad_v6_occt_backend::OcctBackend;

const T: ToleranceContext = ToleranceContext { modeling: 1e-9, validation: 1e-6 };

fn assert_close(actual: f64, expected: f64, tolerance: f64, label: &str) {
    assert!((actual - expected).abs() <= tolerance, "{label}: expected {expected}, got {actual}, tolerance {tolerance}");
}

fn assert_bounds(backend: &OcctBackend, shape: &<OcctBackend as GeometryBackend>::Shape, expected: [f64; 6]) {
    let actual = backend.bounding_box(shape, T).unwrap();
    let values = [actual.min_x, actual.min_y, actual.min_z, actual.max_x, actual.max_y, actual.max_z];
    for (index, (got, want)) in values.into_iter().zip(expected).enumerate() {
        assert_close(got, want, T.validation, &format!("bound[{index}]"));
    }
}

#[test]
fn primitive_bounds_match_independent_analytic_oracles() {
    let backend = OcctBackend::new();
    let box_shape = backend.box_solid(10.0, 20.0, 30.0, T).unwrap();
    assert_eq!(box_shape.kind, GeometryKind::Solid);
    assert_bounds(&backend, &box_shape.shape, [0.0, 0.0, 0.0, 10.0, 20.0, 30.0]);
    let cylinder = backend.cylinder_solid(5.0, 12.0, T).unwrap();
    assert_eq!(cylinder.kind, GeometryKind::Solid);
    assert_bounds(&backend, &cylinder.shape, [-5.0, -5.0, 0.0, 5.0, 5.0, 12.0]);
    let sphere = backend.sphere_solid(7.0, T).unwrap();
    assert_eq!(sphere.kind, GeometryKind::Solid);
    assert_bounds(&backend, &sphere.shape, [-7.0, -7.0, -7.0, 7.0, 7.0, 7.0]);
    let cone = backend.cone_solid(8.0, 3.0, 15.0, T).unwrap();
    assert_eq!(cone.kind, GeometryKind::Solid);
    assert_bounds(&backend, &cone.shape, [-8.0, -8.0, 0.0, 8.0, 8.0, 15.0]);
    let torus = backend.torus_solid(20.0, 5.0, T).unwrap();
    assert_eq!(torus.kind, GeometryKind::Solid);
    assert_bounds(&backend, &torus.shape, [-25.0, -25.0, -5.0, 25.0, 25.0, 5.0]);
}

#[test]
fn box_topology_matches_the_independent_euler_cube_expectation() {
    let backend = OcctBackend::new();
    let shape = backend.box_solid(10.0, 20.0, 30.0, T).unwrap().shape;
    let counts = backend.topology_counts(&shape, T).unwrap();
    let expected = TopologyCounts { solids: 1, shells: 1, faces: 6, edges: 12, vertices: 8 };
    assert_eq!(counts, expected);
    assert_eq!(counts.vertices as i64 - counts.edges as i64 + counts.faces as i64, 2);
}

#[test]
fn translation_matches_independent_affine_bound_transform() {
    let backend = OcctBackend::new();
    let source = backend.box_solid(10.0, 20.0, 30.0, T).unwrap().shape;
    let translated = backend.translate(&source, 100.0, -50.0, 7.5, T).unwrap().shape;
    assert_bounds(&backend, &translated, [100.0, -50.0, 7.5, 110.0, -30.0, 37.5]);
    assert_bounds(&backend, &source, [0.0, 0.0, 0.0, 10.0, 20.0, 30.0]);
}

#[test]
fn two_translations_equal_one_composed_translation() {
    let backend = OcctBackend::new();
    let source = backend.box_solid(10.0, 20.0, 30.0, T).unwrap().shape;
    let first = backend.translate(&source, 10.0, -20.0, 30.0, T).unwrap().shape;
    let twice = backend.translate(&first, 7.0, 5.0, -2.0, T).unwrap().shape;
    let once = backend.translate(&source, 17.0, -15.0, 28.0, T).unwrap().shape;
    let twice_box = backend.bounding_box(&twice, T).unwrap();
    let once_box = backend.bounding_box(&once, T).unwrap();
    for (index, (got, want)) in [twice_box.min_x, twice_box.min_y, twice_box.min_z, twice_box.max_x, twice_box.max_y, twice_box.max_z]
        .into_iter()
        .zip([once_box.min_x, once_box.min_y, once_box.min_z, once_box.max_x, once_box.max_y, once_box.max_z])
        .enumerate()
    {
        assert_close(got, want, T.validation, &format!("bound[{index}]"));
    }
}

#[test]
fn full_turn_rotation_matches_identity_within_validation_tolerance() {
    let backend = OcctBackend::new();
    let source = backend.box_solid(10.0, 20.0, 30.0, T).unwrap().shape;
    let rotated = backend.rotate(&source, 0.0, 0.0, 1.0, core::f64::consts::TAU, T).unwrap().shape;
    let source_box = backend.bounding_box(&source, T).unwrap();
    let rotated_box = backend.bounding_box(&rotated, T).unwrap();
    let source_values = [source_box.min_x, source_box.min_y, source_box.min_z, source_box.max_x, source_box.max_y, source_box.max_z];
    let rotated_values = [rotated_box.min_x, rotated_box.min_y, rotated_box.min_z, rotated_box.max_x, rotated_box.max_y, rotated_box.max_z];
    for (index, (got, want)) in rotated_values.into_iter().zip(source_values).enumerate() {
        assert_close(got, want, T.validation, &format!("bound[{index}]"));
    }
}
