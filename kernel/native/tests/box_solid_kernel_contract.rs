use umlcad_kernel_rust::api::{dispatch, KernelRequest, KernelResponse};
use umlcad_kernel_rust::math::brep::AxisAlignedBox;
use umlcad_kernel_rust::math::tolerance::Tolerance;
use umlcad_kernel_rust::math::vec::Vec3;

#[test]
fn box_solid_kernel_contract_returns_deterministic_authoritative_result() {
    let request = KernelRequest::BuildAxisAlignedBoxSolid {
        operation_identity: "seed-cube-001".to_string(),
        bounds: AxisAlignedBox {
            min: Vec3::new(0.0, 0.0, 0.0),
            max: Vec3::new(10.0, 20.0, 30.0),
        },
        tolerance: Tolerance::new(1.0e-9, 1.0e-9).unwrap(),
    };

    let first = dispatch(request.clone()).unwrap();
    let second = dispatch(request).unwrap();

    assert_eq!(first, second);

    let KernelResponse::BoxSolid(report) = first else {
        panic!("expected BoxSolid response");
    };

    assert_eq!(report.topology.len(), 6);
    assert_eq!(report.volume, 6000.0);
    assert_eq!(report.surface_area, 2200.0);
    assert_eq!(report.centroid, Vec3::new(5.0, 10.0, 15.0));
    assert!(report.result_id.starts_with("solid:"));
    assert_eq!(report.evidence_hash.len(), 64);
    assert!(report.solid.validate(Tolerance::new(1.0e-9, 1.0e-9).unwrap()).is_ok());
}

#[test]
fn box_solid_kernel_contract_fails_closed_for_degenerate_bounds() {
    let result = dispatch(KernelRequest::BuildAxisAlignedBoxSolid {
        operation_identity: "invalid-box".to_string(),
        bounds: AxisAlignedBox {
            min: Vec3::new(0.0, 0.0, 0.0),
            max: Vec3::new(0.0, 1.0, 1.0),
        },
        tolerance: Tolerance::new(1.0e-9, 1.0e-9).unwrap(),
    });

    assert!(result.is_err());
}
