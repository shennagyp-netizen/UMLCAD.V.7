use umlcad_kernel_rust::functions::surfaces::{PlanarSurface, Point3, SurfaceError};

fn patch() -> PlanarSurface {
    PlanarSurface::new(
        Point3 { x: 5.0, y: -7.0, z: 3.0 },
        20.0,
        30.0,
    )
}

#[test]
fn planar_surface_has_exact_area_and_unit_normal() {
    let surface = patch();
    assert!(surface.validate().is_ok());
    assert_eq!(surface.area(), 600.0);
    assert_eq!(surface.normal(), Point3 { x: 0.0, y: 0.0, z: 1.0 });
}

#[test]
fn planar_surface_parameterization_maps_unit_square_to_patch() {
    let surface = patch();
    assert_eq!(surface.point_at(0.0, 0.0).unwrap(), Point3 { x: -5.0, y: -22.0, z: 3.0 });
    assert_eq!(surface.point_at(0.5, 0.5).unwrap(), Point3 { x: 5.0, y: -7.0, z: 3.0 });
    assert_eq!(surface.point_at(1.0, 1.0).unwrap(), Point3 { x: 15.0, y: 8.0, z: 3.0 });
    assert_eq!(surface.point_at(-f64::EPSILON, 0.5), Err(SurfaceError::OutOfDomain));
    assert_eq!(surface.point_at(0.5, 1.0 + f64::EPSILON), Err(SurfaceError::OutOfDomain));
}

#[test]
fn planar_surface_has_exact_bounding_box() {
    let bounds = patch().bounding_box().unwrap();
    assert_eq!(bounds.min, Point3 { x: -5.0, y: -22.0, z: 3.0 });
    assert_eq!(bounds.max, Point3 { x: 15.0, y: 8.0, z: 3.0 });
}

#[test]
fn planar_surface_point_distance_is_euclidean_to_the_bounded_patch() {
    let surface = patch();
    assert_eq!(surface.distance_to_point(Point3 { x: 5.0, y: -7.0, z: 13.0 }).unwrap(), 10.0);
    assert_eq!(surface.distance_to_point(Point3 { x: 25.0, y: -7.0, z: 3.0 }).unwrap(), 10.0);
    assert_eq!(surface.distance_to_point(Point3 { x: -5.0, y: -22.0, z: 3.0 }).unwrap(), 0.0);
}

#[test]
fn planar_surface_translation_is_immutable_and_exact() {
    let source = patch();
    let moved = source.translated(100.0, -200.0, 300.0).unwrap();
    assert_eq!(source.center, Point3 { x: 5.0, y: -7.0, z: 3.0 });
    assert_eq!(moved.center, Point3 { x: 105.0, y: -207.0, z: 303.0 });
    assert_eq!(moved.width, source.width);
    assert_eq!(moved.depth, source.depth);
}

#[test]
fn planar_surface_rejects_invalid_geometry_inputs() {
    for invalid in [
        PlanarSurface::new(Point3 { x: 0.0, y: 0.0, z: 0.0 }, 0.0, 1.0),
        PlanarSurface::new(Point3 { x: 0.0, y: 0.0, z: 0.0 }, -1.0, 1.0),
        PlanarSurface::new(Point3 { x: 0.0, y: 0.0, z: 0.0 }, 1.0, 0.0),
        PlanarSurface::new(Point3 { x: f64::NAN, y: 0.0, z: 0.0 }, 1.0, 1.0),
        PlanarSurface::new(Point3 { x: 0.0, y: f64::INFINITY, z: 0.0 }, 1.0, 1.0),
    ] {
        assert!(invalid.validate().is_err());
    }
    assert_eq!(patch().translated(f64::NAN, 0.0, 0.0), Err(SurfaceError::NonFinite));
}
