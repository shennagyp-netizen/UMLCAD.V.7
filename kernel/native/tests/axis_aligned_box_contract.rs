use umlcad_kernel_rust::api::{dispatch, AxisAlignedBoxEvaluationResult, KernelRequest, KernelResponse};
use umlcad_kernel_rust::functions::brep::AxisAlignedBox;
use umlcad_kernel_rust::functions::tolerance::Tolerance;
use umlcad_kernel_rust::functions::vec::Vec3;

fn unit_box() -> AxisAlignedBox {
    AxisAlignedBox {
        min: Vec3::new(0.0, 0.0, 0.0),
        max: Vec3::new(1.0, 2.0, 3.0),
    }
}

#[test]
fn axis_aligned_box_dispatch_returns_authoritative_numeric_evidence() {
    let response = dispatch(KernelRequest::EvaluateAxisAlignedBox {
        evaluation_identity: "eval-box-1".into(),
        result_identity: "result-box-1".into(),
        box_geometry: unit_box(),
        tolerance: Tolerance::new(0.0, 1.0e-12).unwrap(),
    })
    .unwrap();

    let KernelResponse::AxisAlignedBox(result) = response else {
        panic!("unexpected response")
    };

    assert_eq!(
        result,
        AxisAlignedBoxEvaluationResult {
            evaluation_identity: "eval-box-1".into(),
            result_identity: "result-box-1".into(),
            min: [0.0, 0.0, 0.0],
            max: [1.0, 2.0, 3.0],
            volume: 6.0,
            surface_area: 22.0,
            centroid: [0.5, 1.0, 1.5],
        }
    );
}

#[test]
fn axis_aligned_box_contract_rejects_degenerate_boxes() {
    let mut geometry = unit_box();
    geometry.max.x = geometry.min.x;

    let result = dispatch(KernelRequest::EvaluateAxisAlignedBox {
        evaluation_identity: "eval-box-degenerate".into(),
        result_identity: "result-box-degenerate".into(),
        box_geometry: geometry,
        tolerance: Tolerance::new(0.0, 1.0e-12).unwrap(),
    });

    assert!(result.is_err());
}

#[test]
fn axis_aligned_box_contract_rejects_non_finite_values() {
    let mut geometry = unit_box();
    geometry.max.z = f64::INFINITY;

    let result = dispatch(KernelRequest::EvaluateAxisAlignedBox {
        evaluation_identity: "eval-box-nonfinite".into(),
        result_identity: "result-box-nonfinite".into(),
        box_geometry: geometry,
        tolerance: Tolerance::new(0.0, 1.0e-12).unwrap(),
    });

    assert!(result.is_err());
}

#[test]
fn axis_aligned_box_contract_is_repeat_deterministic() {
    let request = KernelRequest::EvaluateAxisAlignedBox {
        evaluation_identity: "eval-box-deterministic".into(),
        result_identity: "result-box-deterministic".into(),
        box_geometry: unit_box(),
        tolerance: Tolerance::new(0.0, 1.0e-12).unwrap(),
    };

    let first = dispatch(request.clone()).unwrap();
    let second = dispatch(request).unwrap();

    assert_eq!(first, second);
}
