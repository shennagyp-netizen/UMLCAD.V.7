use umlcad_kernel_rust::api::{dispatch, KernelRequest, KernelResponse};
use umlcad_kernel_rust::math::brep::PlanarRegion3;
use umlcad_kernel_rust::math::tolerance::Tolerance;
use umlcad_kernel_rust::math::vec::{Vec2, Vec3};

fn region() -> PlanarRegion3 {
    PlanarRegion3 {
        origin: Vec3::new(1.0, 2.0, 3.0),
        u_dir: Vec3::new(1.0, 0.0, 0.0),
        v_dir: Vec3::new(0.0, 1.0, 0.0),
        outer: vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(4.0, 0.0),
            Vec2::new(4.0, 5.0),
            Vec2::new(0.0, 5.0),
        ],
        holes: Vec::new(),
    }
}

#[test]
fn extrusion_dispatch_is_deterministic_and_returns_prism_metrics() {
    let tolerance = Tolerance::new(1.0e-9, 1.0e-9).unwrap();
    let request = KernelRequest::ExtrudeConvexPlanarProfile {
        operation_identity: "extrude-001".to_string(),
        region: region(),
        depth: 6.0,
        tolerance,
    };

    let first = dispatch(request.clone()).unwrap();
    let second = dispatch(request).unwrap();

    assert_eq!(first, second);

    let KernelResponse::Extrusion(report) = first else {
        panic!("expected extrusion response");
    };

    assert_eq!(report.topology.len(), 6);
    assert_eq!(report.volume, 120.0);
    assert_eq!(report.surface_area, 148.0);
    assert_eq!(report.centroid, Vec3::new(3.0, 4.5, 6.0));
    assert_eq!(report.solid.vertices.len(), 8);
    assert!(report.solid.validate(tolerance).is_ok());
}

#[test]
fn extrusion_dispatch_rejects_non_orthonormal_profile_frame() {
    let tolerance = Tolerance::new(1.0e-9, 1.0e-9).unwrap();
    let mut profile = region();
    profile.v_dir = Vec3::new(1.0, 0.0, 0.0);

    let result = dispatch(KernelRequest::ExtrudeConvexPlanarProfile {
        operation_identity: "extrude-invalid-frame".to_string(),
        region: profile,
        depth: 1.0,
        tolerance,
    });

    assert!(result.is_err());
}

#[test]
fn extrusion_dispatch_rejects_profile_holes_in_current_certified_domain() {
    let tolerance = Tolerance::new(1.0e-9, 1.0e-9).unwrap();
    let mut profile = region();
    profile.holes.push(vec![
        Vec2::new(1.0, 1.0),
        Vec2::new(2.0, 1.0),
        Vec2::new(2.0, 2.0),
        Vec2::new(1.0, 2.0),
    ]);

    let result = dispatch(KernelRequest::ExtrudeConvexPlanarProfile {
        operation_identity: "extrude-hole-profile".to_string(),
        region: profile,
        depth: 1.0,
        tolerance,
    });

    assert!(result.is_err());
}
