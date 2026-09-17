use std::f64::consts::PI;

use umlcad_kernel_rust::functions::surfaces::{Point3, SphereSurface, SurfaceError};

#[test]
fn sphere_surface_has_exact_area_parameterization_normal_and_distance() {
    let s = SphereSurface::new(Point3 { x: 1.0, y: -2.0, z: 3.0 }, 5.0);
    assert!(s.validate().is_ok());
    assert!((s.area() - 4.0 * PI * 25.0).abs() < 1e-12);

    // u is azimuth in [0,1], v is polar angle in [0,1].
    // u=0.5, v=0.5 maps to the negative Y axis.
    let p = s.point_at(0.5, 0.5).unwrap();
    assert!((p.x - 1.0).abs() < 1e-12);
    assert!((p.y + 7.0).abs() < 1e-12);
    assert!((p.z - 3.0).abs() < 1e-12);

    let n = s.normal_at(0.5, 0.5).unwrap();
    assert!((n.norm() - 1.0).abs() < 1e-12);
    let center_to_point = s.center().vector_to(p);
    assert!((center_to_point.dot(n) - 5.0).abs() < 1e-12);

    assert!(s.distance_to_point(Point3 { x: 1.0, y: -2.0, z: 8.0 }).unwrap().abs() < 1e-12);
    assert!((s.distance_to_point(Point3 { x: 1.0, y: -2.0, z: 13.0 }).unwrap() - 5.0).abs() < 1e-12);
}

#[test]
fn sphere_surface_has_exact_axis_aligned_bounds() {
    let s = SphereSurface::new(Point3 { x: 2.0, y: -3.0, z: 4.0 }, 5.0);
    let b = s.bounding_box().unwrap();
    assert_eq!(b.min, Point3 { x: -3.0, y: -8.0, z: -1.0 });
    assert_eq!(b.max, Point3 { x: 7.0, y: 2.0, z: 9.0 });
}

#[test]
fn sphere_surface_rejects_invalid_inputs_and_out_of_domain_parameters() {
    assert_eq!(
        SphereSurface::new(Point3 { x: 0.0, y: 0.0, z: 0.0 }, 0.0).validate(),
        Err(SurfaceError::InvalidRadius)
    );
    assert_eq!(
        SphereSurface::new(Point3 { x: f64::NAN, y: 0.0, z: 0.0 }, 1.0).validate(),
        Err(SurfaceError::NonFinite)
    );
    assert_eq!(
        SphereSurface::new(Point3 { x: 0.0, y: 0.0, z: 0.0 }, f64::INFINITY).validate(),
        Err(SurfaceError::NonFinite)
    );
    let s = SphereSurface::new(Point3 { x: 0.0, y: 0.0, z: 0.0 }, 1.0);
    assert_eq!(s.point_at(-1e-6, 0.5), Err(SurfaceError::OutOfDomain));
    assert_eq!(s.point_at(0.5, 1.000001), Err(SurfaceError::OutOfDomain));
    assert_eq!(s.normal_at(f64::NAN, 0.5), Err(SurfaceError::NonFinite));
}

#[test]
fn sphere_surface_translation_preserves_radius_and_area() {
    let s = SphereSurface::new(Point3 { x: 0.0, y: 0.0, z: 0.0 }, 3.0);
    let moved = s.translated(4.0, -3.0, 2.0).unwrap();
    assert_eq!(moved.radius(), s.radius());
    assert!((moved.area() - s.area()).abs() < 1e-12);
    assert_eq!(moved.center(), Point3 { x: 4.0, y: -3.0, z: 2.0 });
    assert_eq!(s.center(), Point3 { x: 0.0, y: 0.0, z: 0.0 });
}

#[test]
fn sphere_surface_distance_is_radial_and_parameterization_invariant() {
    let s = SphereSurface::new(Point3 { x: 0.0, y: 0.0, z: 0.0 }, 2.0);
    for uv in [(0.0, 0.0), (0.0, 0.25), (0.5, 0.5), (1.0, 0.75), (0.37, 0.81)] {
        let p = s.point_at(uv.0, uv.1).unwrap();
        assert!(s.distance_to_point(p).unwrap() < 1e-12);
        let n = s.normal_at(uv.0, uv.1).unwrap();
        assert!((n.norm() - 1.0).abs() < 1e-12);
        assert!((s.center().vector_to(p).dot(n) - 2.0).abs() < 1e-12);
    }
}

#[test]
fn sphere_surface_translation_rejects_overflow() {
    let s = SphereSurface::new(Point3 { x: f64::MAX, y: 0.0, z: 0.0 }, 1.0);
    assert_eq!(
        s.translated(f64::MAX, 0.0, 0.0),
        Err(SurfaceError::NonFinite)
    );
}
