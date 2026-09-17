use std::collections::BTreeSet;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use umlcad_v6_draft_api::{DraftBackend, DraftedRectangularSolid};
use umlcad_v6_exchange_api::{ExchangeBackend, ExchangeFormat, ExchangeStatus};
use umlcad_v6_fillet_api::{BoxAllEdgesFillet, FilletBackend};
use umlcad_v6_geometry_api::{GeometryBackend, GeometryKind, ToleranceContext};
use umlcad_v6_kernel_integration_api::{
    IntegrationSnapshot, OperationDescriptor, OperationId, OperationKind, OperationProvenance,
};
use umlcad_v6_mesh_api::{Mesh, TessellationBackend, TessellationOptions};
use umlcad_v6_nurbs_surface_api::{NurbsSurface3DDefinition, NurbsSurfaceBackend};
use umlcad_v6_offset_api::{OffsetBackend, PlanarSurfacePatch3D};
use umlcad_v6_occt_backend::OcctBackend;
use umlcad_v6_reference_evolution_api::ReferenceEvolution;
use umlcad_v6_reference_migration_api::{
    classify_reference_migration, ReferenceMigrationKind,
};
use umlcad_v6_shell_api::{ClosedBoxThickness, ShellBackend};
use umlcad_v6_sweep_api::{CircularProfile, LinearCircularSweep, LinearPath};
use umlcad_v6_topology_api::{EdgeId, EdgeUse, FaceId, ShellOrientationGraph};

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

fn valid_planar_surface() -> NurbsSurface3DDefinition {
    NurbsSurface3DDefinition::new(
        (1, 1),
        vec![
            umlcad_v6_nurbs_surface_api::Point3 { x: 0.0, y: 0.0, z: 0.0 },
            umlcad_v6_nurbs_surface_api::Point3 { x: 0.0, y: 10.0, z: 0.0 },
            umlcad_v6_nurbs_surface_api::Point3 { x: 20.0, y: 0.0, z: 0.0 },
            umlcad_v6_nurbs_surface_api::Point3 { x: 20.0, y: 10.0, z: 0.0 },
        ],
        vec![1.0; 4],
        (2, 2),
        vec![0.0, 0.0, 1.0, 1.0],
        vec![0.0, 0.0, 1.0, 1.0],
    )
}

#[test]
fn solid_pipeline_crosses_geometry_feature_shell_fillet_mesh_and_exchange_layers() {
    let backend = OcctBackend::new();

    let base = backend.box_solid(20.0, 30.0, 40.0, TOLERANCE).unwrap();
    assert_eq!(base.kind, GeometryKind::Solid);
    assert!(backend.validate(&base.shape, TOLERANCE).unwrap().valid);

    let rounded = backend
        .fillet_box_all_edges(
            BoxAllEdgesFillet {
                width: 20.0,
                depth: 30.0,
                height: 40.0,
                radius: 2.0,
            },
            TOLERANCE,
        )
        .unwrap();
    assert!(backend.validate(&rounded.shape, TOLERANCE).unwrap().valid);

    let mesh = backend
        .tessellate(
            &base.shape,
            TessellationOptions {
                linear_deflection: 0.1,
                angular_deflection_radians: 0.2,
            },
        )
        .unwrap();
    assert!(mesh.validate().is_ok());
    assert!(!mesh.vertices.is_empty());
    assert!(!mesh.triangles.is_empty());

    let path = temp_path("step", "solid_roundtrip");
    let evidence = backend
        .export_file(&rounded.shape, ExchangeFormat::Step, &path, TOLERANCE)
        .unwrap();
    assert_eq!(evidence.status, ExchangeStatus::Success);
    assert!(evidence.bytes > 0);

    let imported = backend
        .import_file(ExchangeFormat::Step, Path::new(&path), TOLERANCE)
        .unwrap();
    assert_eq!(imported.kind, GeometryKind::Solid);
    assert!(backend.validate(&imported.shape, TOLERANCE).unwrap().valid);
    assert_eq!(
        backend.topology_counts(&rounded.shape, TOLERANCE).unwrap(),
        backend.topology_counts(&imported.shape, TOLERANCE).unwrap()
    );

    std::fs::remove_file(path).unwrap();
}

#[test]
fn feature_realization_crosses_shell_draft_sweep_and_offset_layers() {
    let backend = OcctBackend::new();

    let shell = backend
        .make_closed_box_thickness(
            ClosedBoxThickness {
                width: 20.0,
                depth: 30.0,
                height: 40.0,
                thickness: 2.0,
            },
            TOLERANCE,
        )
        .unwrap();
    assert!(backend.validate(&shell.shape, TOLERANCE).unwrap().valid);

    let drafted = backend
        .make_drafted_rectangular_solid(
            DraftedRectangularSolid {
                width: 20.0,
                depth: 30.0,
                height: 10.0,
                draft_angle_radians: 0.1,
            },
            TOLERANCE,
        )
        .unwrap();
    assert!(backend.validate(&drafted.shape, TOLERANCE).unwrap().valid);

    let sweep = LinearCircularSweep {
        profile: CircularProfile {
            center: umlcad_v6_sweep_api::Point3 { x: 0.0, y: 0.0, z: 0.0 },
            normal: umlcad_v6_sweep_api::Point3 { x: 0.0, y: 0.0, z: 1.0 },
            radius: 2.0,
        },
        path: LinearPath {
            start: umlcad_v6_sweep_api::Point3 { x: 0.0, y: 0.0, z: 0.0 },
            end: umlcad_v6_sweep_api::Point3 { x: 0.0, y: 0.0, z: 10.0 },
        },
    }
    .realize_with(&backend, TOLERANCE)
    .unwrap();
    assert!(backend.validate(&sweep.shape, TOLERANCE).unwrap().valid);

    let patch = PlanarSurfacePatch3D {
        origin: umlcad_v6_offset_api::Point3 { x: 0.0, y: 0.0, z: 0.0 },
        u_dir: umlcad_v6_offset_api::Point3 { x: 1.0, y: 0.0, z: 0.0 },
        v_dir: umlcad_v6_offset_api::Point3 { x: 0.0, y: 1.0, z: 0.0 },
        width: 20.0,
        height: 30.0,
    };
    let offset = backend
        .offset_planar_surface(patch, 2.0, TOLERANCE)
        .unwrap();
    assert!(backend.validate(&offset.shape, TOLERANCE).unwrap().valid);
}

#[test]
fn freeform_surface_crosses_definition_native_realization_and_differential_semantics() {
    let backend = OcctBackend::new();
    let definition = valid_planar_surface();

    assert!(definition.validate().is_ok());
    let ((u0, u1), (v0, v1)) = definition.parameter_domain().unwrap();
    assert_eq!((u0, u1), (0.0, 1.0));
    assert_eq!((v0, v1), (0.0, 1.0));

    let semantic = definition.differential_at(0.25, 0.75).unwrap();
    assert!((semantic.normal().unwrap().norm() - 1.0).abs() < 1e-14);

    let native = backend.nurbs_surface3d(&definition, TOLERANCE).unwrap();
    assert_eq!(native.kind, GeometryKind::Surface);
    assert!(backend.validate(&native.shape, TOLERANCE).unwrap().valid);
}

#[test]
fn topology_reference_and_immutable_integration_form_one_system_identity_path() {
    let graph = ShellOrientationGraph {
        faces: [FaceId(1), FaceId(2)].into_iter().collect::<BTreeSet<_>>(),
        edge_uses: vec![
            EdgeUse { edge: EdgeId(1), face: FaceId(1), forward: true },
            EdgeUse { edge: EdgeId(1), face: FaceId(2), forward: false },
        ],
    };
    graph.validate().unwrap();
    let initial_id = graph.snapshot_id();

    let second_graph = ShellOrientationGraph {
        faces: [FaceId(2), FaceId(3)].into_iter().collect::<BTreeSet<_>>(),
        edge_uses: vec![
            EdgeUse { edge: EdgeId(2), face: FaceId(2), forward: true },
            EdgeUse { edge: EdgeId(2), face: FaceId(3), forward: false },
        ],
    };
    let output_id = second_graph.snapshot_id();

    let initial = IntegrationSnapshot::new(OperationProvenance::new(), vec![initial_id]).unwrap();
    let operation = OperationDescriptor {
        id: OperationId(1),
        kind: OperationKind::Modify,
        inputs: vec![initial_id],
        output: output_id,
    };
    let before = initial.clone();
    let next = initial.apply_operation(operation).unwrap();

    assert_eq!(initial, before);
    assert_eq!(next.roots(), &[output_id]);
    assert_ne!(next.identity(), initial.identity());

    let preserved = classify_reference_migration(1, 1).unwrap();
    assert_eq!(preserved.migration, ReferenceMigrationKind::Preserved);
    assert!(ReferenceEvolution::OneToOne.preserves_a_unique_target());

    let ambiguous = classify_reference_migration(2, 3).unwrap();
    assert_eq!(ambiguous.migration, ReferenceMigrationKind::Ambiguous);
    assert!(!ambiguous.migration.is_automatic_identity_safe());
}

#[test]
fn mesh_contract_remains_independent_and_fail_closed_at_system_boundary() {
    let valid = Mesh {
        vertices: vec![
            umlcad_v6_mesh_api::Vertex { x: 0.0, y: 0.0, z: 0.0 },
            umlcad_v6_mesh_api::Vertex { x: 1.0, y: 0.0, z: 0.0 },
            umlcad_v6_mesh_api::Vertex { x: 0.0, y: 1.0, z: 0.0 },
        ],
        triangles: vec![umlcad_v6_mesh_api::Triangle { a: 0, b: 1, c: 2 }],
    };
    assert!(valid.validate().is_ok());

    let invalid = Mesh {
        vertices: valid.vertices.clone(),
        triangles: vec![umlcad_v6_mesh_api::Triangle { a: 0, b: 0, c: 2 }],
    };
    assert!(invalid.validate().is_err());
}
