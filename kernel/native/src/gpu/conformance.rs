use crate::functions::{
    gpu::{compare_candidate_pairs, BackendKind},
    spatial_accel::Aabb3,
    vec::Vec3,
};

fn fixture() -> Vec<Aabb3> {
    vec![
        Aabb3::new(
            Vec3::new(-2.0, -2.0, -2.0),
            Vec3::new(-1.0, -1.0, -1.0),
        ).unwrap(),
        Aabb3::new(
            Vec3::new(-1.5, -1.5, -1.5),
            Vec3::new(0.25, 0.25, 0.25),
        ).unwrap(),
        Aabb3::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 1.0, 1.0),
        ).unwrap(),
        Aabb3::new(
            Vec3::new(1.0, 1.0, 1.0),
            Vec3::new(2.0, 2.0, 2.0),
        ).unwrap(),
        Aabb3::new(
            Vec3::new(1.0e12, 1.0e12, 1.0e12),
            Vec3::new(1.0e12 + 1.0e5, 1.0e12 + 1.0e5, 1.0e12 + 1.0e5),
        ).unwrap(),
        Aabb3::new(
            Vec3::new(1.0e12 + 5.0e4, 1.0e12 + 5.0e4, 1.0e12 + 5.0e4),
            Vec3::new(1.0e12 + 2.0e5, 1.0e12 + 2.0e5, 1.0e12 + 2.0e5),
        ).unwrap(),
        Aabb3::new(
            Vec3::new(-1.0e12, 4.0, 8.0),
            Vec3::new(-1.0e12 + 1.0e5, 5.0, 9.0),
        ).unwrap(),
    ]
}

fn cpu_reference(boxes: &[Aabb3]) -> Vec<(usize, usize)> {
    let mut pairs = Vec::new();
    for i in 0..boxes.len() {
        for j in (i + 1)..boxes.len() {
            if boxes[i].intersects(boxes[j], 0.0) {
                pairs.push((i, j));
            }
        }
    }
    pairs
}

#[test]
fn common_fixture_cpu_reference_is_deterministic() {
    let boxes = fixture();
    let first = cpu_reference(&boxes);
    let repeat = cpu_reference(&boxes);
    assert_eq!(first, repeat);
    assert!(first.contains(&(0, 1)));
    assert!(first.contains(&(1, 2)));
    assert!(first.contains(&(2, 3)));
    assert!(first.contains(&(4, 5)));
    assert!(!first.contains(&(0, 6)));

    let report = compare_candidate_pairs(
        BackendKind::Cpu,
        &first,
        &first,
        &repeat,
    ).expect("CPU reference candidate set");
    assert!(report.conforms());
    assert!(report.exact_match());
}

#[cfg(target_os = "macos")]
#[test]
fn metal_conforms_to_common_cpu_reference() {
    let boxes = fixture();
    let reference = cpu_reference(&boxes);
    let backend = super::metal::MetalBackend::new().expect("Metal device and pipeline");
    let candidate = backend.candidate_pairs(&boxes).expect("Metal candidate pass");
    let repeat = backend.candidate_pairs(&boxes).expect("repeat Metal candidate pass");

    let report = compare_candidate_pairs(
        BackendKind::Metal,
        &reference,
        &candidate,
        &repeat,
    ).expect("Metal conformance report");

    assert!(report.conforms(), "Metal false negative: {:?}", report.first_false_negative);
    assert!(report.repeat_exact);
}

#[cfg(all(target_os = "linux", feature = "cuda-hardware"))]
#[test]
fn cuda_conforms_to_common_cpu_reference() {
    let boxes = fixture();
    let reference = cpu_reference(&boxes);
    let backend = super::cuda::CudaBackend::new().expect("CUDA device and NVRTC");
    let candidate = backend.candidate_pairs(&boxes).expect("CUDA candidate pass");
    let repeat = backend.candidate_pairs(&boxes).expect("repeat CUDA candidate pass");

    let report = compare_candidate_pairs(
        BackendKind::Cuda,
        &reference,
        &candidate,
        &repeat,
    ).expect("CUDA conformance report");

    assert!(report.conforms(), "CUDA false negative: {:?}", report.first_false_negative);
    assert!(report.repeat_exact);
    assert!(report.exact_match(), "CUDA broad phase returned unexpected false positives: {:?}", report.first_false_positive);
}
