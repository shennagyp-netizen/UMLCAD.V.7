use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use umlcad_v6_exchange_api::{ExchangeBackend, ExchangeFormat, ExchangeStatus};
use umlcad_v6_geometry_api::{GeometryBackend, GeometryKind, ToleranceContext};
use umlcad_v6_mesh_api::{TessellationBackend, TessellationOptions};
use umlcad_v6_nurbs_surface_api::{NurbsSurface3DDefinition, NurbsSurfaceBackend, Point3};
use umlcad_v6_occt_backend::OcctBackend;
use umlcad_v6_surface_operations_api::{
    closest_point_on_planar_nurbs_surface, intersect_line_segment_nurbs_surface,
    IntersectionStatus, LineSegment3D,
};
use umlcad_v6_sweep_api::{
    CircularProfile, LinearVariableRadiusSweep, Point3 as SweepPoint3, TwoSegmentCircularSweep,
};

const TOLERANCE: ToleranceContext = ToleranceContext {
    modeling: 1e-9,
    validation: 1e-9,
};

fn temp_path(extension: &str, label: &str) -> std::path::PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "umlcad_v6_s18_{label}_{}_{}.{}",
        std::process::id(),
        stamp,
        extension
    ))
}

fn planar_patch() -> NurbsSurface3DDefinition {
    NurbsSurface3DDefinition::new(
        (1, 1),
        vec![
            Point3 { x: 0.0, y: 0.0, z: 0.0 },
            Point3 { x: 0.0, y: 10.0, z: 0.0 },
            Point3 { x: 20.0, y: 0.0, z: 0.0 },
            Point3 { x: 20.0, y: 10.0, z: 0.0 },
        ],
        vec![1.0; 4],
        (2, 2),
        vec![0.0, 0.0, 1.0, 1.0],
        vec![0.0, 0.0, 1.0, 1.0],
    )
}

#[test]
fn analytic_primitive_family_crosses_transform_boolean_and_descriptor_layers() {
    let backend = OcctBackend::new();

    let box_shape = backend.box_solid(10.0, 20.0, 30.0, TOLERANCE).unwrap();
    let cylinder = backend.cylinder_solid(3.0, 15.0, TOLERANCE).unwrap();
    let sphere = backend.sphere_solid(4.0, TOLERANCE).unwrap();
    let cone = backend.cone_solid(5.0, 2.0, 12.0, TOLERANCE).unwrap();
    let torus = backend.torus_solid(8.0, 2.0, TOLERANCE).unwrap();
    let circle = backend.circle_curve(2.0, TOLERANCE).unwrap();
    let line = backend
        .line_curve(0.0, 0.0, 0.0, 3.0, 4.0, 0.0, TOLERANCE)
        .unwrap();

    for shape in [&box_shape.shape, &cylinder.shape, &sphere.shape, &cone.shape, &torus.shape] {
        assert!(backend.validate(shape, TOLERANCE).unwrap().valid);
    }
    assert!((backend.curve_length(&circle.shape, TOLERANCE).unwrap() - 4.0 * std::f64::consts::PI).abs() < 1e-12);
    assert!((backend.curve_length(&line.shape, TOLERANCE).unwrap() - 5.0).abs() < 1e-12);

    let moved = backend.translate(&box_shape.shape, 5.0, -2.0, 3.0, TOLERANCE).unwrap();
    let rotated = backend.rotate(&moved.shape, 0.0, 0.0, 1.0, 0.5, TOLERANCE).unwrap();
    let fused = backend.fuse(&rotated.shape, &sphere.shape, TOLERANCE).unwrap();
    assert!(backend.validate(&fused.shape, TOLERANCE).unwrap().valid);
    assert!(!backend.face_descriptors(&fused.shape, TOLERANCE).unwrap().is_empty());
    assert!(!backend.edge_descriptors(&fused.shape, TOLERANCE).unwrap().is_empty());
    assert!(!backend.vertex_descriptors(&fused.shape, TOLERANCE).unwrap().is_empty());
}

#[test]
fn surface_operation_semantics_are_consumable_before_native_freeform_realization() {
    let surface = planar_patch();
    assert!(surface.validate().is_ok());

    let line = LineSegment3D {
        start: Point3 { x: 10.0, y: 5.0, z: -2.0 },
        end: Point3 { x: 10.0, y: 5.0, z: 2.0 },
    };
    let intersection = intersect_line_segment_nurbs_surface(line, &surface, 1e-9).unwrap();
    assert_eq!(intersection.status, IntersectionStatus::Unique);
    assert!((intersection.points[0].point.z).abs() < 1e-12);

    let closest = closest_point_on_planar_nurbs_surface(
        Point3 { x: 10.0, y: 5.0, z: 3.0 },
        &surface,
        1e-9,
    )
    .unwrap();
    assert_eq!(closest.status, IntersectionStatus::Unique);
    let closest = closest.closest.unwrap();
    assert!((closest.point.z).abs() < 1e-12);
    assert!((closest.distance - 3.0).abs() < 1e-12);
}

#[test]
fn native_freeform_and_exchange_consume_the_same_semantic_geometry() {
    let backend = OcctBackend::new();
    let surface = planar_patch();
    let native_surface = backend.nurbs_surface3d(&surface, TOLERANCE).unwrap();
    assert_eq!(native_surface.kind, GeometryKind::Surface);
    assert!(backend.validate(&native_surface.shape, TOLERANCE).unwrap().valid);

    let path = temp_path("iges", "surface_iges");
    let evidence = backend
        .export_file(&native_surface.shape, ExchangeFormat::Iges, &path, TOLERANCE)
        .unwrap();
    assert_eq!(evidence.status, ExchangeStatus::Success);
    assert!(evidence.bytes > 0);
    let imported = backend
        .import_file(ExchangeFormat::Iges, Path::new(&path), TOLERANCE)
        .unwrap();
    assert_eq!(imported.kind, GeometryKind::Surface);
    assert!(backend.validate(&imported.shape, TOLERANCE).unwrap().valid);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn advanced_sweep_families_cross_into_native_boolean_and_mesh_layers() {
    let backend = OcctBackend::new();

    let two = TwoSegmentCircularSweep {
        profile: CircularProfile {
            center: SweepPoint3 { x: 0.0, y: 0.0, z: 0.0 },
            normal: SweepPoint3 { x: 1.0, y: 0.0, z: 0.0 },
            radius: 1.5,
        },
        corner: SweepPoint3 { x: 10.0, y: 0.0, z: 0.0 },
        end: SweepPoint3 { x: 10.0, y: 10.0, z: 0.0 },
    };
    let two_shape = two.realize_with(&backend, TOLERANCE).unwrap();
    assert!(backend.validate(&two_shape.shape, TOLERANCE).unwrap().valid);

    let variable = LinearVariableRadiusSweep {
        start: SweepPoint3 { x: 0.0, y: 0.0, z: 0.0 },
        end: SweepPoint3 { x: 0.0, y: 0.0, z: 15.0 },
        normal: SweepPoint3 { x: 0.0, y: 0.0, z: 1.0 },
        start_radius: 1.0,
        end_radius: 2.0,
    };
    let variable_shape = variable.realize_with(&backend, TOLERANCE).unwrap();
    assert!(backend.validate(&variable_shape.shape, TOLERANCE).unwrap().valid);

    let merged = backend
        .fuse(&two_shape.shape, &variable_shape.shape, TOLERANCE)
        .unwrap();
    assert!(backend.validate(&merged.shape, TOLERANCE).unwrap().valid);

    let mesh = backend
        .tessellate(
            &variable_shape.shape,
            TessellationOptions {
                linear_deflection: 0.2,
                angular_deflection_radians: 0.25,
            },
        )
        .unwrap();
    assert!(mesh.validate().is_ok());
    assert!(!mesh.triangles.is_empty());
}
