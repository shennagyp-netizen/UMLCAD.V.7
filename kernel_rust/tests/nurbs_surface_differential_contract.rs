use umlcad_kernel_rust::functions::nurbs_surface::{NurbsSurface2D, NurbsSurfaceError, Point3};
use umlcad_kernel_rust::functions::nurbs_surface_differential::NurbsSurfaceDifferential;

fn p(x: f64, y: f64, z: f64) -> Point3 { Point3 { x, y, z } }

fn plane() -> NurbsSurface2D {
    NurbsSurface2D::new(
        1,
        1,
        vec![
            p(0.0, 0.0, 0.0),
            p(0.0, 1.0, 1.0),
            p(1.0, 0.0, 2.0),
            p(1.0, 1.0, 3.0),
        ],
        vec![1.0; 4],
        vec![0.0, 0.0, 1.0, 1.0],
        vec![0.0, 0.0, 1.0, 1.0],
    )
}

#[test]
fn bilinear_plane_derivatives_match_independent_oracle() {
    let surface = plane();
    let du = surface.derivative_u_at(0.25, 0.75).unwrap();
    let dv = surface.derivative_v_at(0.25, 0.75).unwrap();
    assert!((du.x - 1.0).abs() < 1e-12);
    assert!(du.y.abs() < 1e-12);
    assert!((du.z - 2.0).abs() < 1e-12);
    assert!(dv.x.abs() < 1e-12);
    assert!((dv.y - 1.0).abs() < 1e-12);
    assert!((dv.z - 1.0).abs() < 1e-12);
}

#[test]
fn planar_normal_is_unit_and_deterministic() {
    let surface = plane();
    let normal = surface.normal_at(0.3, 0.8).unwrap();
    let expected = p(-2.0, -1.0, 1.0);
    let magnitude = expected.x.hypot(expected.y.hypot(expected.z));
    assert!((normal.x - expected.x / magnitude).abs() < 1e-12);
    assert!((normal.y - expected.y / magnitude).abs() < 1e-12);
    assert!((normal.z - expected.z / magnitude).abs() < 1e-12);
    assert!((normal.x.hypot(normal.y.hypot(normal.z)) - 1.0).abs() < 1e-12);
}

#[test]
fn translation_preserves_derivatives_and_normal() {
    let surface = plane();
    let moved = surface.translated(100.0, -50.0, 7.0).unwrap();
    assert_eq!(moved.derivative_u_at(0.4, 0.6).unwrap(), surface.derivative_u_at(0.4, 0.6).unwrap());
    assert_eq!(moved.derivative_v_at(0.4, 0.6).unwrap(), surface.derivative_v_at(0.4, 0.6).unwrap());
    assert_eq!(moved.normal_at(0.4, 0.6).unwrap(), surface.normal_at(0.4, 0.6).unwrap());
}

#[test]
fn degenerate_surface_normal_is_rejected() {
    let surface = NurbsSurface2D::new(
        1,
        1,
        vec![
            p(0.0, 0.0, 0.0),
            p(0.0, 1.0, 0.0),
            p(0.0, 0.0, 0.0),
            p(0.0, 1.0, 0.0),
        ],
        vec![1.0; 4],
        vec![0.0, 0.0, 1.0, 1.0],
        vec![0.0, 0.0, 1.0, 1.0],
    );
    assert_eq!(surface.normal_at(0.5, 0.5), Err(NurbsSurfaceError::Degenerate));
}

#[test]
fn derivative_domain_is_fail_closed() {
    let surface = plane();
    assert_eq!(surface.derivative_u_at(-1e-12, 0.5), Err(NurbsSurfaceError::OutOfDomain));
    assert_eq!(surface.derivative_v_at(0.5, f64::NAN), Err(NurbsSurfaceError::NonFinite));
}
