use umlcad_v6_geometry_api::{GeometryBackend, GeometryError, ToleranceContext};
use umlcad_v6_occt_backend::OcctBackend;

const T: ToleranceContext = ToleranceContext { modeling: 1e-9, validation: 1e-9 };

#[test]
fn equal_vertex_sections_loft_into_valid_solid() {
    let backend = OcctBackend::new();
    let lower = [(0.0, 0.0), (20.0, 0.0), (20.0, 10.0), (0.0, 10.0)];
    let upper = [(2.0, 1.0), (18.0, 1.0), (18.0, 9.0), (2.0, 9.0)];
    let result = backend.loft_between_polygons(&lower, 0.0, &upper, 30.0, T).unwrap();

    assert!(backend.validate(&result.shape, T).unwrap().valid);
    let bounds = backend.bounding_box(&result.shape, T).unwrap();
    assert!((bounds.min_x - 0.0).abs() <= 1e-9, "unexpected loft bounds: {:?}", bounds);
    assert!((bounds.max_x - 20.0).abs() <= 1e-9, "unexpected loft bounds: {:?}", bounds);
    assert!((bounds.min_y - 0.0).abs() <= 1e-9, "unexpected loft bounds: {:?}", bounds);
    assert!((bounds.max_y - 10.0).abs() <= 1e-9, "unexpected loft bounds: {:?}", bounds);
    assert!((bounds.min_z - 0.0).abs() <= 1e-9, "unexpected loft bounds: {:?}", bounds);
    assert!((bounds.max_z - 30.0).abs() <= 1e-9, "unexpected loft bounds: {:?}", bounds);
    assert_eq!(backend.topology_counts(&result.shape, T).unwrap().solids, 1);
}

#[test]
fn loft_rejects_invalid_sections() {
    let backend = OcctBackend::new();
    let triangle = [(0.0, 0.0), (10.0, 0.0), (0.0, 10.0)];
    let rectangle = [(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)];
    let nan_section = [(0.0, 0.0), (f64::NAN, 0.0), (0.0, 10.0)];

    assert!(matches!(
        backend.loft_between_polygons(&triangle, 0.0, &rectangle, 10.0, T),
        Err(GeometryError::InvalidInput(_))
    ));
    assert!(matches!(
        backend.loft_between_polygons(&nan_section, 0.0, &triangle, 10.0, T),
        Err(GeometryError::InvalidInput(_))
    ));
    assert!(matches!(
        backend.loft_between_polygons(&triangle, 5.0, &triangle, 5.0, T),
        Err(GeometryError::InvalidInput(_))
    ));
}

#[test]
fn loft_is_deterministic_and_immutable() {
    let backend = OcctBackend::new();
    let lower = [(0.0, 0.0), (20.0, 0.0), (20.0, 10.0), (0.0, 10.0)];
    let upper = [(2.0, 1.0), (18.0, 1.0), (18.0, 9.0), (2.0, 9.0)];
    let first = backend.loft_between_polygons(&lower, 0.0, &upper, 30.0, T).unwrap().shape;
    let second = backend.loft_between_polygons(&lower, 0.0, &upper, 30.0, T).unwrap().shape;
    assert_eq!(backend.bounding_box(&first, T).unwrap(), backend.bounding_box(&second, T).unwrap());
    assert_eq!(backend.topology_counts(&first, T).unwrap(), backend.topology_counts(&second, T).unwrap());
    let moved = backend.translate(&first, 3.0, 4.0, 5.0, T).unwrap().shape;
    assert!(backend.validate(&first, T).unwrap().valid);
    assert!(backend.validate(&moved, T).unwrap().valid);
}
