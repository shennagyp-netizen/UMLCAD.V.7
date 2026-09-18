//! Apple Metal acceleration boundary.
//!
//! M13 implements a real Metal compute path for conservative AABB candidate
//! generation. It deliberately does not implement authoritative f64 geometry:
//! Apple Metal does not expose hardware double precision, so CAD geometry stays
//! on the CPU f64 reference path. The Metal workload therefore consumes CPU
//! generated, outward-rounded f32 AABBs and returns only broad-phase candidates.
//!
//! The returned candidate set may contain false positives, but the CPU exact
//! AABB predicate is the acceptance/reference authority. The GPU never defines
//! topology or geometry identity.

use std::fmt;

use crate::functions::{
    gpu::{
        BackendCapability, BackendKind, ExecutionDeterminism, GpuOperation, Precision,
    },
    spatial_accel::Aabb3,
};

const MAX_ITEMS: usize = 4096;
const OPS: &[GpuOperation] = &[GpuOperation::CandidatePairBatch];

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MetalError {
    #[cfg_attr(not(target_os = "macos"), allow(dead_code))]
    UnsupportedPlatform,
    DeviceUnavailable,
    Pipeline(String),
    Allocation,
    InvalidInput,
    TooManyItems,
    OutputOverflow,
    CommandFailed(String),
    NonFinite,
}

impl fmt::Display for MetalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for MetalError {}

#[cfg(not(target_os = "macos"))]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MetalBackend;

#[cfg(not(target_os = "macos"))]
impl MetalBackend {
    pub fn new() -> Result<Self, MetalError> {
        Err(MetalError::UnsupportedPlatform)
    }

    pub fn capability() -> BackendCapability {
        BackendCapability {
            backend: BackendKind::Metal,
            available: false,
            f64: false,
            deterministic: true,
            max_batch: Some(MAX_ITEMS),
            operations: OPS,
            max_workgroup_size: Some(256),
            determinism: ExecutionDeterminism::Bitwise,
        }
    }

    pub fn candidate_pairs(&self, _boxes: &[Aabb3]) -> Result<Vec<(usize, usize)>, MetalError> {
        Err(MetalError::UnsupportedPlatform)
    }
}

#[cfg(target_os = "macos")]
mod imp {
    use super::*;
    use crate::functions::vec::Vec3;
    use objc2::rc::Retained;
    use objc2_foundation::{ns_string, NSString};
    use objc2_metal::{
        MTLBuffer, MTLCommandBuffer, MTLCommandQueue, MTLComputeCommandEncoder,
        MTLComputePipelineState, MTLCreateSystemDefaultDevice, MTLDevice, MTLFunction,
        MTLLibrary, MTLResourceOptions, MTLSize,
    };
    use std::ptr::NonNull;

    const SHADER: &str = r#"
#include <metal_stdlib>
using namespace metal;

kernel void candidate_pairs(
    const device float4 *mins [[buffer(0)]],
    const device float4 *maxs [[buffer(1)]],
    const device uint *count [[buffer(2)]],
    device uint *flags [[buffer(3)]],
    uint2 gid [[thread_position_in_grid]])
{
    uint n = count[0];
    uint i = gid.x;
    uint j = gid.y;

    if (i >= n || j >= n || i >= j) {
        return;
    }

    const float4 amin = mins[i];
    const float4 amax = maxs[i];
    const float4 bmin = mins[j];
    const float4 bmax = maxs[j];

    const bool overlap =
        amin.x <= bmax.x && bmin.x <= amax.x &&
        amin.y <= bmax.y && bmin.y <= amax.y &&
        amin.z <= bmax.z && bmin.z <= amax.z;

    uint prefix = i * n - (i * (i + 1u)) / 2u;
    uint index = prefix + (j - i - 1u);
    flags[index] = overlap ? 1u : 0u;
}
"#;

    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {}

    fn next_down(value: f32) -> f32 {
        if value.is_nan() || value == f32::NEG_INFINITY {
            return value;
        }
        if value == 0.0 {
            return -f32::MIN_POSITIVE;
        }
        let bits = value.to_bits();
        let next = if value.is_sign_positive() {
            bits - 1
        } else {
            bits + 1
        };
        f32::from_bits(next)
    }

    fn next_up(value: f32) -> f32 {
        if value.is_nan() || value == f32::INFINITY {
            return value;
        }
        if value == 0.0 {
            return f32::MIN_POSITIVE;
        }
        let bits = value.to_bits();
        let next = if value.is_sign_positive() {
            bits + 1
        } else {
            bits - 1
        };
        f32::from_bits(next)
    }

    fn conservative_lower(value: f64) -> Result<f32, MetalError> {
        if !value.is_finite() {
            return Err(MetalError::NonFinite);
        }
        let converted = value as f32;
        if !converted.is_finite() {
            return Err(MetalError::InvalidInput);
        }
        if (converted as f64) > value {
            Ok(next_down(converted))
        } else {
            Ok(converted)
        }
    }

    fn conservative_upper(value: f64) -> Result<f32, MetalError> {
        if !value.is_finite() {
            return Err(MetalError::NonFinite);
        }
        let converted = value as f32;
        if !converted.is_finite() {
            return Err(MetalError::InvalidInput);
        }
        if (converted as f64) < value {
            Ok(next_up(converted))
        } else {
            Ok(converted)
        }
    }

    fn to_conservative_buffers(
        boxes: &[Aabb3],
    ) -> Result<(Vec<[f32; 4]>, Vec<[f32; 4]>), MetalError> {
        if boxes.is_empty() || boxes.len() > MAX_ITEMS {
            return Err(MetalError::TooManyItems);
        }

        let mut mins = Vec::with_capacity(boxes.len());
        let mut maxs = Vec::with_capacity(boxes.len());
        for bounds in boxes {
            if !bounds.min.is_finite() || !bounds.max.is_finite() {
                return Err(MetalError::NonFinite);
            }
            if bounds.min.x > bounds.max.x
                || bounds.min.y > bounds.max.y
                || bounds.min.z > bounds.max.z
            {
                return Err(MetalError::InvalidInput);
            }
            mins.push([
                conservative_lower(bounds.min.x)?,
                conservative_lower(bounds.min.y)?,
                conservative_lower(bounds.min.z)?,
                0.0,
            ]);
            maxs.push([
                conservative_upper(bounds.max.x)?,
                conservative_upper(bounds.max.y)?,
                conservative_upper(bounds.max.z)?,
                0.0,
            ]);
        }
        Ok((mins, maxs))
    }

    fn buffer_from_bytes(
        device: &ProtocolObject<dyn MTLDevice>,
        bytes: &[u8],
    ) -> Result<Retained<ProtocolObject<dyn MTLBuffer>>, MetalError> {
        let buffer = device
            .newBufferWithLength_options(
                bytes.len(),
                MTLResourceOptions::StorageModeShared,
            )
            .ok_or(MetalError::Allocation)?;
        unsafe {
            let destination = buffer.contents().cast::<u8>();
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), destination, bytes.len());
        }
        Ok(buffer)
    }

    fn zeroed_buffer(
        device: &ProtocolObject<dyn MTLDevice>,
        byte_length: usize,
    ) -> Result<Retained<ProtocolObject<dyn MTLBuffer>>, MetalError> {
        device
            .newBufferWithLength_options(byte_length, MTLResourceOptions::StorageModeShared)
            .ok_or(MetalError::Allocation)
    }

    use objc2::runtime::ProtocolObject;

    pub struct MetalBackend {
        device: Retained<ProtocolObject<dyn MTLDevice>>,
        queue: Retained<ProtocolObject<dyn MTLCommandQueue>>,
        pipeline: Retained<ProtocolObject<dyn MTLComputePipelineState>>,
    }

    impl fmt::Debug for MetalBackend {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("MetalBackend").finish_non_exhaustive()
        }
    }

    impl MetalBackend {
        pub fn new() -> Result<Self, MetalError> {
            let device = MTLCreateSystemDefaultDevice().ok_or(MetalError::DeviceUnavailable)?;
            let queue = device
                .newCommandQueue()
                .ok_or(MetalError::DeviceUnavailable)?;

            let source = NSString::from_str(SHADER);
            let library = device
                .newLibraryWithSource_options_error(&source, None)
                .map_err(|error| MetalError::Pipeline(error.to_string()))?;
            let name = ns_string!("candidate_pairs");
            let function = library
                .newFunctionWithName(name)
                .ok_or_else(|| MetalError::Pipeline("missing candidate_pairs function".into()))?;
            let pipeline = device
                .newComputePipelineStateWithFunction_error(&function)
                .map_err(|error| MetalError::Pipeline(error.to_string()))?;

            Ok(Self {
                device,
                queue,
                pipeline,
            })
        }

        pub fn capability() -> BackendCapability {
            BackendCapability {
                backend: BackendKind::Metal,
                available: true,
                f64: false,
                deterministic: true,
                max_batch: Some(MAX_ITEMS),
                operations: OPS,
                max_workgroup_size: Some(256),
                determinism: ExecutionDeterminism::Bitwise,
            }
        }

        pub fn candidate_pairs(&self, boxes: &[Aabb3]) -> Result<Vec<(usize, usize)>, MetalError> {
            let (mins, maxs) = to_conservative_buffers(boxes)?;
            let n = boxes.len();
            let pair_count = n
                .checked_mul(n - 1)
                .and_then(|value| value.checked_div(2))
                .ok_or(MetalError::OutputOverflow)?;
            let mins_bytes = unsafe {
                std::slice::from_raw_parts(
                    mins.as_ptr().cast::<u8>(),
                    mins.len() * std::mem::size_of::<[f32; 4]>(),
                )
            };
            let maxs_bytes = unsafe {
                std::slice::from_raw_parts(
                    maxs.as_ptr().cast::<u8>(),
                    maxs.len() * std::mem::size_of::<[f32; 4]>(),
                )
            };

            let min_buffer = buffer_from_bytes(&self.device, mins_bytes)?;
            let max_buffer = buffer_from_bytes(&self.device, maxs_bytes)?;
            let count_bytes = (n as u32).to_ne_bytes();
            let count_buffer = buffer_from_bytes(&self.device, &count_bytes)?;
            let flags_buffer = zeroed_buffer(
                &self.device,
                pair_count
                    .checked_mul(std::mem::size_of::<u32>())
                    .ok_or(MetalError::OutputOverflow)?,
            )?;

            let command_buffer = self
                .queue
                .commandBuffer()
                .ok_or(MetalError::DeviceUnavailable)?;
            let encoder = command_buffer
                .computeCommandEncoder()
                .ok_or(MetalError::CommandFailed("no compute encoder".into()))?;

            unsafe {
                encoder.setComputePipelineState(&self.pipeline);
                encoder.setBuffer_offset_atIndex(Some(&min_buffer), 0, 0);
                encoder.setBuffer_offset_atIndex(Some(&max_buffer), 0, 1);
                encoder.setBuffer_offset_atIndex(Some(&count_buffer), 0, 2);
                encoder.setBuffer_offset_atIndex(Some(&flags_buffer), 0, 3);

                let threadgroups = MTLSize {
                    width: n,
                    height: n,
                    depth: 1,
                };
                let threads_per_group = MTLSize {
                    width: 8,
                    height: 8,
                    depth: 1,
                };
                encoder.dispatchThreads_threadsPerThreadgroup(
                    threadgroups,
                    threads_per_group,
                );
                encoder.endEncoding();
            }

            command_buffer.commit();
            command_buffer.waitUntilCompleted();

            if command_buffer.status() != objc2_metal::MTLCommandBufferStatus::Completed {
                let message = command_buffer
                    .error()
                    .map(|error| format!("{error:?}"))
                    .unwrap_or_else(|| "Metal command buffer failed".into());
                return Err(MetalError::CommandFailed(message));
            }

            let mut candidates = Vec::new();
            unsafe {
                let flags = std::slice::from_raw_parts(
                    flags_buffer.contents().cast::<u32>(),
                    pair_count,
                );
                for i in 0..n {
                    let prefix = i * n - (i * (i + 1)) / 2;
                    for j in (i + 1)..n {
                        let index = prefix + (j - i - 1);
                        if flags[index] != 0 {
                            candidates.push((i, j));
                        }
                    }
                }
            }

            // The Metal pass is a broad-phase accelerator only. Verify the
            // no-false-negative invariant against the authoritative f64 predicate
            // before exposing any candidate set to callers.
            let candidate_set = candidates.iter().copied().collect::<std::collections::BTreeSet<_>>();
            for i in 0..n {
                for j in (i + 1)..n {
                    if boxes[i].intersects(boxes[j], 0.0) && !candidate_set.contains(&(i, j)) {
                        return Err(MetalError::CommandFailed(
                            format!("Metal broad phase omitted exact CPU overlap pair ({i}, {j})"),
                        ));
                    }
                }
            }

            Ok(candidates)
        }
    }

    // implementation type remains private to this module
}

#[cfg(target_os = "macos")]
pub use imp::MetalBackend;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capability_is_explicitly_non_f64_for_metal() {
        let capability = MetalBackend::capability();
        assert_eq!(capability.backend, BackendKind::Metal);
        assert!(!capability.f64);
        assert!(capability.operations.contains(&GpuOperation::CandidatePairBatch));
    }

    #[test]
    fn non_macos_reports_unsupported_instead_of_faking_execution() {
        #[cfg(not(target_os = "macos"))]
        assert_eq!(
            MetalBackend::new(),
            Err(MetalError::UnsupportedPlatform)
        );
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn metal_candidate_pairs_match_cpu_overlap_subset() {
        let boxes = vec![
            Aabb3::new(
                Vec3::new(0.0, 0.0, 0.0),
                super::Vec3::new(1.0, 1.0, 1.0),
            )
            .unwrap(),
            Aabb3::new(
                super::Vec3::new(0.5, 0.5, 0.5),
                super::Vec3::new(2.0, 2.0, 2.0),
            )
            .unwrap(),
            Aabb3::new(
                super::Vec3::new(3.0, 3.0, 3.0),
                super::Vec3::new(4.0, 4.0, 4.0),
            )
            .unwrap(),
            Aabb3::new(
                super::Vec3::new(1.0e12, 1.0e12, 1.0e12),
                super::Vec3::new(1.0e12 + 1.0e5, 1.0e12 + 1.0e5, 1.0e12 + 1.0e5),
            )
            .unwrap(),
            Aabb3::new(
                super::Vec3::new(1.0e12 + 5.0e4, 1.0e12 + 5.0e4, 1.0e12 + 5.0e4),
                super::Vec3::new(1.0e12 + 2.0e5, 1.0e12 + 2.0e5, 1.0e12 + 2.0e5),
            )
            .unwrap(),
        ];

        let backend = MetalBackend::new().expect("Metal device and pipeline");
        let actual = backend.candidate_pairs(&boxes).expect("Metal candidate pass");

        let actual_set = actual.iter().copied().collect::<std::collections::BTreeSet<_>>();
        for i in 0..boxes.len() {
            for j in (i + 1)..boxes.len() {
                if boxes[i].intersects(boxes[j], 0.0) {
                    assert!(
                        actual_set.contains(&(i, j)),
                        "Metal omitted exact CPU overlap pair ({i}, {j})"
                    );
                }
            }
        }

        let repeat = backend.candidate_pairs(&boxes).expect("repeat Metal candidate pass");
        assert_eq!(actual, repeat);
    }
}
