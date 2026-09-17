use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use umlcad_v6_exchange_api::{ExchangeBackend, ExchangeFormat};
use umlcad_v6_geometry_api::{GeometryBackend, ToleranceContext};
use umlcad_v6_mesh_api::{TessellationBackend, TessellationOptions};
use umlcad_v6_occt_backend::OcctBackend;

const TOLERANCE: ToleranceContext = ToleranceContext {
    modeling: 1e-9,
    validation: 1e-9,
};

const TESSELLATION: TessellationOptions = TessellationOptions {
    linear_deflection: 0.1,
    angular_deflection_radians: 0.2,
};

fn temp_path(extension: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock must be after UNIX epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "umlcad_v6_s17_{}_{}.{}",
        std::process::id(),
        stamp,
        extension
    ))
}

#[test]
fn repeated_native_clone_transform_and_boolean_cycles_preserve_source_semantics() {
    let backend = OcctBackend::new();
    let source = backend.box_solid(20.0, 30.0, 40.0, TOLERANCE).unwrap().shape;
    let tool = backend.box_solid(5.0, 6.0, 7.0, TOLERANCE).unwrap().shape;
    let baseline = backend.topology_counts(&source, TOLERANCE).unwrap();

    for i in 0..100 {
        let clone = source.clone();
        assert_eq!(backend.topology_counts(&clone, TOLERANCE).unwrap(), baseline);

        let moved = backend
            .translate(&clone, i as f64 * 0.001, 0.0, 0.0, TOLERANCE)
            .unwrap()
            .shape;
        assert!(backend.validate(&moved, TOLERANCE).unwrap().valid);

        let fused = backend.fuse(&source, &tool, TOLERANCE).unwrap().shape;
        let common = backend.common(&source, &tool, TOLERANCE).unwrap().shape;
        let cut = backend.cut(&source, &tool, TOLERANCE).unwrap().shape;
        assert!(backend.validate(&fused, TOLERANCE).unwrap().valid);
        assert!(backend.validate(&common, TOLERANCE).unwrap().valid);
        assert!(backend.validate(&cut, TOLERANCE).unwrap().valid);
    }

    assert_eq!(backend.topology_counts(&source, TOLERANCE).unwrap(), baseline);
    assert!(backend.validate(&source, TOLERANCE).unwrap().valid);
}

#[test]
fn repeated_tessellation_is_deterministic_and_valid() {
    let backend = OcctBackend::new();
    let source = backend.box_solid(10.0, 20.0, 30.0, TOLERANCE).unwrap().shape;
    let expected = backend.tessellate(&source, TESSELLATION).unwrap();

    for _ in 0..100 {
        let mesh = backend.tessellate(&source, TESSELLATION).unwrap();
        assert_eq!(mesh, expected);
        assert!(mesh.validate().is_ok());
    }
}

#[test]
fn repeated_step_exchange_round_trips_preserve_valid_topology() {
    let backend = OcctBackend::new();
    let source = backend.box_solid(11.0, 13.0, 17.0, TOLERANCE).unwrap().shape;
    let expected_counts = backend.topology_counts(&source, TOLERANCE).unwrap();
    let path = temp_path("step");

    for _ in 0..10 {
        let evidence = backend
            .export_file(&source, ExchangeFormat::Step, &path, TOLERANCE)
            .unwrap();
        assert!(evidence.bytes > 0);
        let imported = backend
            .import_file(ExchangeFormat::Step, &path, TOLERANCE)
            .unwrap();
        assert!(backend.validate(&imported.shape, TOLERANCE).unwrap().valid);
        assert_eq!(
            backend.topology_counts(&imported.shape, TOLERANCE).unwrap(),
            expected_counts
        );
    }

    std::fs::remove_file(path).unwrap();
}
