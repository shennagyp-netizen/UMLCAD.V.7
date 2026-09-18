//! M16 release-mode performance evidence for the certified AABB workload.
//!
//! This is a measurement test, not a correctness authority. It records median
//! CPU latency, Metal latency, pair throughput, host preparation, Metal buffer
//! setup/upload, device execution, readback, and CPU post-processing. No fixed
//! performance threshold is asserted because hosted hardware and CI load vary.
//! Correctness remains guarded by the M15 conformance contract.

use std::time::{Duration, Instant};

use umlcad_kernel_rust::functions::{
    gpu::{compare_candidate_pairs, BackendKind},
    spatial_accel::Aabb3,
    vec::Vec3,
};

fn boxes(n: usize) -> Vec<Aabb3> {
    (0..n)
        .map(|i| {
            let x = (i % 32) as f64 * 2.0;
            let y = ((i / 32) % 32) as f64 * 2.0;
            let z = (i / 1024) as f64 * 2.0;
            Aabb3::new(
                Vec3::new(x, y, z),
                Vec3::new(x + 1.25, y + 1.25, z + 1.25),
            )
            .unwrap()
        })
        .collect()
}

fn cpu_candidate_pairs(input: &[Aabb3]) -> Vec<(usize, usize)> {
    let mut result = Vec::new();
    for i in 0..input.len() {
        for j in (i + 1)..input.len() {
            if input[i].intersects(input[j], 0.0) {
                result.push((i, j));
            }
        }
    }
    result
}

fn median(values: &mut [Duration]) -> Duration {
    values.sort_unstable();
    values[values.len() / 2]
}

fn micros(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1.0e6
}

fn million_pairs_per_second(pair_count: usize, duration: Duration) -> f64 {
    pair_count as f64 / duration.as_secs_f64() / 1.0e6
}

#[test]
#[cfg(target_os = "macos")]
fn m16_metal_performance_evidence_is_finite_and_reports_crossover() {
    let backend = umlcad_kernel_rust::gpu::metal::MetalBackend::new()
        .expect("Metal device and pipeline");
    let sizes = [32usize, 64, 128, 256, 512];
    let repeats = 5usize;

    println!("M16_PERFORMANCE,candidate_pair_workload");
    println!(
        "batch,cpu_us,metal_total_us,host_prepare_us,buffer_setup_us,device_us,readback_us,postprocess_us,cpu_mpairs_s,metal_mpairs_s,roundtrip_overhead_pct,batch_efficiency_pct,cpu_pairs,crossover"
    );

    let mut metal_throughputs = Vec::new();
    let mut crossover = None;

    for &n in &sizes {
        let input = boxes(n);
        let reference = cpu_candidate_pairs(&input);
        let pair_count = n * (n - 1) / 2;

        // One warm-up run avoids charging the first command-buffer/pipeline path
        // as representative steady-state latency.
        let (warm_pairs, _) = backend
            .candidate_pairs_timed(&input)
            .expect("Metal warm-up");
        let warm_report = compare_candidate_pairs(
            BackendKind::Metal,
            &reference,
            &warm_pairs,
            &warm_pairs,
        ).unwrap();
        assert!(warm_report.conforms());

        let mut cpu_times = Vec::with_capacity(repeats);
        let mut metal_times = Vec::with_capacity(repeats);
        let mut prep_times = Vec::with_capacity(repeats);
        let mut setup_times = Vec::with_capacity(repeats);
        let mut device_times = Vec::with_capacity(repeats);
        let mut readback_times = Vec::with_capacity(repeats);
        let mut postprocess_times = Vec::with_capacity(repeats);

        for _ in 0..repeats {
            let cpu_start = Instant::now();
            let cpu_pairs = cpu_candidate_pairs(&input);
            let cpu_elapsed = cpu_start.elapsed();
            assert_eq!(cpu_pairs, reference);
            cpu_times.push(cpu_elapsed);

            let metal_start = Instant::now();
            let (metal_pairs, timing) = backend
                .candidate_pairs_timed(&input)
                .expect("Metal candidate pass");
            let metal_elapsed = metal_start.elapsed();

            let report = compare_candidate_pairs(
                BackendKind::Metal,
                &reference,
                &metal_pairs,
                &metal_pairs,
            ).unwrap();
            assert!(report.conforms());
            assert_eq!(report.false_negative_count, 0);

            metal_times.push(metal_elapsed);
            prep_times.push(timing.host_prepare);
            setup_times.push(timing.buffer_upload_setup);
            device_times.push(timing.device_execution);
            readback_times.push(timing.buffer_readback);
            postprocess_times.push(timing.cpu_postprocess);
        }

        let cpu_median = median(&mut cpu_times);
        let metal_median = median(&mut metal_times);
        let prep_median = median(&mut prep_times);
        let setup_median = median(&mut setup_times);
        let device_median = median(&mut device_times);
        let readback_median = median(&mut readback_times);
        let postprocess_median = median(&mut postprocess_times);

        let transfer_overhead = setup_median + readback_median;
        let roundtrip_overhead_pct =
            100.0 * transfer_overhead.as_secs_f64() / metal_median.as_secs_f64();

        let cpu_throughput = million_pairs_per_second(pair_count, cpu_median);
        let metal_throughput = million_pairs_per_second(pair_count, metal_median);
        metal_throughputs.push(metal_throughput);
        let max_seen = metal_throughputs
            .iter()
            .copied()
            .fold(0.0_f64, f64::max);
        let batch_efficiency_pct = if max_seen > 0.0 {
            100.0 * metal_throughput / max_seen
        } else {
            0.0
        };

        if crossover.is_none() && metal_median < cpu_median {
            crossover = Some(n);
        }

        assert!(cpu_median > Duration::ZERO);
        assert!(metal_median > Duration::ZERO);
        for timing in [
            prep_median,
            setup_median,
            device_median,
            readback_median,
            postprocess_median,
        ] {
            assert!(timing.is_finite());
        }
        assert!(roundtrip_overhead_pct.is_finite());
        assert!(cpu_throughput.is_finite() && metal_throughput.is_finite());
        assert!(batch_efficiency_pct.is_finite());

        println!(
            "{n},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.4},{:.4},{:.2},{:.2},{},{}",
            micros(cpu_median),
            micros(metal_median),
            micros(prep_median),
            micros(setup_median),
            micros(device_median),
            micros(readback_median),
            micros(postprocess_median),
            cpu_throughput,
            metal_throughput,
            roundtrip_overhead_pct,
            batch_efficiency_pct,
            reference.len(),
            crossover.map_or_else(|| "-".into(), |value| value.to_string()),
        );
    }

    println!(
        "M16_CROSSOVER,batch={}",
        crossover.map_or_else(|| "none".to_string(), |value| value.to_string())
    );
}

#[test]
#[cfg(not(target_os = "macos"))]
fn m16_performance_is_explicitly_unavailable_off_macos() {
    // This protects against accidentally claiming Metal benchmark evidence on
    // platforms where the certified Metal workload cannot execute.
}
