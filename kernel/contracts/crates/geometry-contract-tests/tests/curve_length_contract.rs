use std::f64::consts::PI;
use umlcad_v6_geometry_api::{GeometryBackend, GeometryError, GeometryKind, ToleranceContext};
use umlcad_v6_occt_backend::OcctBackend;

const T: ToleranceContext = ToleranceContext { modeling: 1e-9, validation: 1e-6 };

#[test]
fn line_length_matches_euclidean_distance() {
    let backend = OcctBackend::new();
    let line = backend.line_curve(0.0, 0.0, 0.0, 3.0, 4.0, 12.0, T).unwrap();
    assert_eq!(line.kind, GeometryKind::Curve);
    assert!((backend.curve_length(&line.shape, T).unwrap() - 13.0).abs() <= T.validation);
}

#[test]
fn circle_length_matches_circumference() {
    let backend = OcctBackend::new();
    let circle = backend.circle_curve(10.0, T).unwrap();
    let expected = 20.0 * PI;
    assert!((backend.curve_length(&circle.shape, T).unwrap() - expected).abs() <= T.validation);
}

#[test]
fn curve_length_rejects_non_curve_shapes() {
    let backend = OcctBackend::new();
    let solid = backend.box_solid(10.0, 20.0, 30.0, T).unwrap().shape;
    assert!(matches!(backend.curve_length(&solid, T), Err(GeometryError::Unsupported(_))));
}

#[test]
fn curve_length_is_immutable_under_source_transform() {
    let backend = OcctBackend::new();
    let source = backend.circle_curve(10.0, T).unwrap().shape;
    let before = backend.curve_length(&source, T).unwrap();
    let translated = backend.translate(&source, 100.0, -200.0, 300.0, T).unwrap().shape;
    assert!((backend.curve_length(&translated, T).unwrap() - before).abs() <= T.validation);
    assert!((backend.curve_length(&source, T).unwrap() - before).abs() <= T.validation);
}
