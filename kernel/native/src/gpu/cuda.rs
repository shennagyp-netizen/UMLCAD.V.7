use std::fmt;

use crate::functions::{
    gpu::{BackendCapability, BackendKind, ExecutionDeterminism, GpuOperation},
    spatial_accel::Aabb3,
};

const MAX_ITEMS: usize = 4096;
const BLOCK_DIM: u32 = 16;
const OPS: &[GpuOperation] = &[GpuOperation::CandidatePairBatch];

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CudaError {
    UnsupportedPlatform,
    DeviceUnavailable(String),
    Compilation(String),
    Module(String),
    InvalidInput,
    TooManyItems,
    OutputOverflow,
    Launch(String),
    Synchronization(String),
    NonFinite,
    FalseNegative { first: (usize, usize) },
}

impl fmt::Display for CudaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{self:?}") }
}
impl std::error::Error for CudaError {}

#[cfg(not(target_os = "linux"))]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CudaBackend;

#[cfg(not(target_os = "linux"))]
impl CudaBackend {
    pub fn new() -> Result<Self, CudaError> { Err(CudaError::UnsupportedPlatform) }
    pub fn capability() -> BackendCapability {
        BackendCapability {
            backend: BackendKind::Cuda, available: false, f64: false, deterministic: true,
            max_batch: Some(MAX_ITEMS), operations: OPS,
            max_workgroup_size: Some((BLOCK_DIM * BLOCK_DIM) as usize),
            determinism: ExecutionDeterminism::Bitwise,
        }
    }
    pub fn candidate_pairs(&self, _boxes: &[Aabb3]) -> Result<Vec<(usize, usize)>, CudaError> {
        Err(CudaError::UnsupportedPlatform)
    }
}

#[cfg(target_os = "linux")]
mod imp {
    use super::*;
    use cudarc::{driver::{CudaContext, LaunchConfig, PushKernelArg}, nvrtc::compile_ptx};

    const KERNEL: &str = r#"
extern "C" __global__ void candidate_pairs(
    const double* mins, const double* maxs,
    unsigned int n, unsigned char* flags)
{
    const unsigned int i = blockIdx.y * blockDim.y + threadIdx.y;
    const unsigned int j = blockIdx.x * blockDim.x + threadIdx.x;
    if (i >= n || j >= n || i >= j) return;

    const unsigned int ai = 3u * i;
    const unsigned int aj = 3u * j;
    const bool overlap =
        mins[ai] <= maxs[aj] && mins[aj] <= maxs[ai] &&
        mins[ai + 1u] <= maxs[aj + 1u] && mins[aj + 1u] <= maxs[ai + 1u] &&
        mins[ai + 2u] <= maxs[aj + 2u] && mins[aj + 2u] <= maxs[ai + 2u];

    const unsigned int prefix = i * n - (i * (i + 1u)) / 2u;
    const unsigned int index = prefix + (j - i - 1u);
    flags[index] = overlap ? 1u : 0u;
}
"#;

    pub struct CudaBackend {
        context: std::sync::Arc<CudaContext>,
        module: std::sync::Arc<cudarc::driver::CudaModule>,
    }

    impl fmt::Debug for CudaBackend {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("CudaBackend").finish_non_exhaustive()
        }
    }

    impl CudaBackend {
        pub fn new() -> Result<Self, CudaError> {
            let context = CudaContext::new(0)
                .map_err(|e| CudaError::DeviceUnavailable(format!("{e:?}")))?;
            let ptx = compile_ptx(KERNEL)
                .map_err(|e| CudaError::Compilation(format!("{e:?}")))?;
            let module = context.load_module(ptx)
                .map_err(|e| CudaError::Module(format!("{e:?}")))?;
            Ok(Self { context, module })
        }

        pub fn capability() -> BackendCapability {
            BackendCapability {
                backend: BackendKind::Cuda,
                available: CudaContext::new(0).is_ok(),
                f64: true,
                deterministic: true,
                max_batch: Some(MAX_ITEMS),
                operations: OPS,
                max_workgroup_size: Some((BLOCK_DIM * BLOCK_DIM) as usize),
                determinism: ExecutionDeterminism::Bitwise,
            }
        }

        fn flatten_bounds(boxes: &[Aabb3]) -> Result<(Vec<f64>, Vec<f64>), CudaError> {
            if boxes.is_empty() { return Err(CudaError::InvalidInput); }
            if boxes.len() > MAX_ITEMS { return Err(CudaError::TooManyItems); }

            let capacity = boxes.len().checked_mul(3).ok_or(CudaError::OutputOverflow)?;
            let mut mins = Vec::with_capacity(capacity);
            let mut maxs = Vec::with_capacity(capacity);

            for bounds in boxes {
                if !bounds.min.is_finite() || !bounds.max.is_finite() {
                    return Err(CudaError::NonFinite);
                }
                if bounds.min.x > bounds.max.x ||
                   bounds.min.y > bounds.max.y ||
                   bounds.min.z > bounds.max.z {
                    return Err(CudaError::InvalidInput);
                }
                mins.extend([bounds.min.x, bounds.min.y, bounds.min.z]);
                maxs.extend([bounds.max.x, bounds.max.y, bounds.max.z]);
            }
            Ok((mins, maxs))
        }

        pub fn candidate_pairs(&self, boxes: &[Aabb3]) -> Result<Vec<(usize, usize)>, CudaError> {
            let (mins, maxs) = Self::flatten_bounds(boxes)?;
            let n = boxes.len();
            let pair_count = n.checked_mul(n - 1)
                .and_then(|v| v.checked_div(2))
                .ok_or(CudaError::OutputOverflow)?;
            let n_u32 = u32::try_from(n).map_err(|_| CudaError::TooManyItems)?;
            let grid = n_u32.div_ceil(BLOCK_DIM);

            let stream = self.context.default_stream();
            let function = self.module.load_function("candidate_pairs")
                .map_err(|e| CudaError::Module(format!("{e:?}")))?;

            let mins_device = stream.clone_htod(&mins)
                .map_err(|e| CudaError::Launch(format!("HtoD mins: {e:?}")))?;
            let maxs_device = stream.clone_htod(&maxs)
                .map_err(|e| CudaError::Launch(format!("HtoD maxs: {e:?}")))?;
            let mut flags_device = stream.alloc_zeros::<u8>(pair_count)
                .map_err(|e| CudaError::Launch(format!("allocate flags: {e:?}")))?;

            let mut launch = stream.launch_builder(&function);
            launch.arg(&mins_device);
            launch.arg(&maxs_device);
            launch.arg(&n_u32);
            launch.arg(&mut flags_device);

            let config = LaunchConfig {
                grid_dim: (grid, grid, 1),
                block_dim: (BLOCK_DIM, BLOCK_DIM, 1),
                shared_mem_bytes: 0,
            };
            unsafe {
                launch.launch(config)
                    .map_err(|e| CudaError::Launch(format!("{e:?}")))?;
            }
            stream.synchronize()
                .map_err(|e| CudaError::Synchronization(format!("{e:?}")))?;

            let flags: Vec<u8> = stream.clone_dtoh(&flags_device)
                .map_err(|e| CudaError::Launch(format!("DtoH flags: {e:?}")))?;

            let mut candidates = Vec::new();
            for i in 0..n {
                let prefix = i * n - (i * (i + 1)) / 2;
                for j in (i + 1)..n {
                    let index = prefix + (j - i - 1);
                    if flags[index] != 0 { candidates.push((i, j)); }
                }
            }

            let candidate_set = candidates.iter().copied().collect::<std::collections::BTreeSet<_>>();
            for i in 0..n {
                for j in (i + 1)..n {
                    if boxes[i].intersects(boxes[j], 0.0) && !candidate_set.contains(&(i, j)) {
                        return Err(CudaError::FalseNegative { first: (i, j) });
                    }
                }
            }
            Ok(candidates)
        }
    }
}

#[cfg(target_os = "linux")]
pub use imp::CudaBackend;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::functions::vec::Vec3;

    #[test]
    fn capability_declares_cuda_f64_without_claiming_other_platforms() {
        let capability = CudaBackend::capability();
        assert_eq!(capability.backend, BackendKind::Cuda);
        #[cfg(target_os = "linux")] assert!(capability.f64);
        #[cfg(not(target_os = "linux"))] assert!(!capability.f64);
    }

    #[test]
    fn cpu_oracle_fixture_is_well_formed() {
        let a = Aabb3::new(Vec3::new(0.0,0.0,0.0), Vec3::new(1.0,1.0,1.0)).unwrap();
        let b = Aabb3::new(Vec3::new(0.5,0.5,0.5), Vec3::new(2.0,2.0,2.0)).unwrap();
        let c = Aabb3::new(Vec3::new(3.0,3.0,3.0), Vec3::new(4.0,4.0,4.0)).unwrap();
        assert!(a.intersects(b,0.0));
        assert!(!a.intersects(c,0.0));
        assert!(!b.intersects(c,0.0));
    }

    #[test]
    #[cfg(target_os = "linux")]
    #[ignore = "requires an NVIDIA CUDA device and NVRTC runtime"]
    fn cuda_candidate_pairs_match_cpu_subset_and_repeat_bitwise() {
        let boxes = vec![
            Aabb3::new(Vec3::new(0.0,0.0,0.0), Vec3::new(1.0,1.0,1.0)).unwrap(),
            Aabb3::new(Vec3::new(0.5,0.5,0.5), Vec3::new(2.0,2.0,2.0)).unwrap(),
            Aabb3::new(Vec3::new(3.0,3.0,3.0), Vec3::new(4.0,4.0,4.0)).unwrap(),
            Aabb3::new(Vec3::new(1.0e12,1.0e12,1.0e12),
                       Vec3::new(1.0e12+1.0e5,1.0e12+1.0e5,1.0e12+1.0e5)).unwrap(),
            Aabb3::new(Vec3::new(1.0e12+5.0e4,1.0e12+5.0e4,1.0e12+5.0e4),
                       Vec3::new(1.0e12+2.0e5,1.0e12+2.0e5,1.0e12+2.0e5)).unwrap(),
        ];
        let backend = CudaBackend::new().expect("CUDA device and NVRTC");
        let first = backend.candidate_pairs(&boxes).expect("CUDA candidate pass");
        let repeat = backend.candidate_pairs(&boxes).expect("repeat CUDA candidate pass");
        assert_eq!(first, repeat);
        let actual = first.iter().copied().collect::<std::collections::BTreeSet<_>>();
        for i in 0..boxes.len() {
            for j in (i+1)..boxes.len() {
                if boxes[i].intersects(boxes[j],0.0) { assert!(actual.contains(&(i,j))); }
            }
        }
    }

    #[test]
    #[cfg(target_os = "linux")]
    #[ignore = "requires an NVIDIA CUDA device and NVRTC runtime"]
    fn cuda_extreme_fixture_preserves_f64_overlaps() {
        let a = Aabb3::new(
            Vec3::new(f64::MAX*0.25,0.0,0.0),
            Vec3::new(f64::MAX*0.25,1.0,1.0)
        ).unwrap();
        let b = a;
        let backend = CudaBackend::new().expect("CUDA device and NVRTC");
        assert_eq!(backend.candidate_pairs(&[a,b]).unwrap(), vec![(0,1)]);
    }
}
