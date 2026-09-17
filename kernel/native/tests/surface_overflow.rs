use umlcad_kernel_rust::functions::surfaces::{PlanarSurface, Point3, SurfaceError};

#[test]
fn planar_surface_rejects_finite_inputs_with_nonrepresentable_derived_values() {
    let huge = PlanarSurface::new(
        Point3 { x: 0.0, y: 0.0, z: 0.0 },
        f64::MAX,
        f64::MAX,
    );
    assert_eq!(huge.validate(), Err(SurfaceError::NonFinite));
}

#[test]
fn planar_surface_translation_rejects_coordinate_overflow() {
    let source = PlanarSurface::new(
        Point3 { x: f64::MAX * 0.75, y: -2.0, z: 3.0 },
        4.0,
        5.0,
    );
    assert!(source.validate().is_ok());
    assert_eq!(
        source.translated(f64::MAX * 0.5, 0.0, 0.0),
        Err(SurfaceError::NonFinite)
    );
}
