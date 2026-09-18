//! Backend-neutral GPU execution contracts.
//!
//! M12 defines planning, capability discovery, batch-memory/dispatch metadata,
//! and CPU/accelerator conformance without implementing a device backend.
//! There are intentionally no Metal/CUDA handles here and no CPU-as-GPU shim.
//!
//! CPU mathematics remains normative. An accelerator is eligible only when its
//! reported precision, operation, batch-size, and determinism capabilities satisfy
//! the requested contract. Automatic selection records whether an accelerator or
//! CPU fallback was selected.
//!
//! Geometry payloads are represented as `f64`; this module never authorizes an
//! implicit `f32` downgrade or a fast-math semantic mode.

use super::vec::Vec3;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum BackendKind {
    Auto,
    Cpu,
    Metal,
    Cuda,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Precision {
    F64,
    /// Explicitly non-authoritative f32 representation used only for conservative
    /// broad-phase candidate generation. Exact geometry remains CPU f64.
    F32ConservativeBroadPhase,
}

const LEGACY_VECTOR_OPS: &[GpuOperation] = &[GpuOperation::VectorBatch];

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum GpuOperation {
    VectorBatch,
    MatrixBatch,
    TransformBatch,
    PredicateBatch,
    CurveEvaluationBatch,
    SurfaceEvaluationBatch,
    NurbsEvaluationBatch,
    JacobianResidualBatch,
    CandidatePairBatch,
    TessellationBatch,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExecutionDeterminism {
    /// Every result byte must match the CPU reference exactly.
    Bitwise,
    /// Results may differ only within an explicit mathematical tolerance.
    Numerical,
    /// Ordering/reduction order is unspecified and unsuitable for deterministic
    /// authority operations.
    NonDeterministic,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BackendCapability {
    pub backend: BackendKind,
    pub available: bool,
    pub f64: bool,
    pub deterministic: bool,
    pub max_batch: Option<usize>,
    /// Operations explicitly supported by this backend capability record.
    pub operations: &'static [GpuOperation],
    pub max_workgroup_size: Option<usize>,
    pub determinism: ExecutionDeterminism,
}

impl BackendCapability {
    pub const fn legacy(
        backend: BackendKind,
        available: bool,
        f64: bool,
        deterministic: bool,
        max_batch: Option<usize>,
    ) -> Self {
        Self {
            backend,
            available,
            f64,
            deterministic,
            max_batch,
            operations: LEGACY_VECTOR_OPS,
            max_workgroup_size: None,
            determinism: if deterministic {
                ExecutionDeterminism::Bitwise
            } else {
                ExecutionDeterminism::NonDeterministic
            },
        }
    }

    pub fn validate(self) -> Result<(), GpuError> {
        if self.max_batch == Some(0) {
            return Err(GpuError::InvalidCapability);
        }
        if self.max_workgroup_size == Some(0) {
            return Err(GpuError::InvalidCapability);
        }
        if self.operations.windows(2).any(|pair| pair[0] >= pair[1]) {
            return Err(GpuError::InvalidCapability);
        }
        Ok(())
    }

    pub fn supports_operation(self, operation: GpuOperation) -> bool {
        self.operations.binary_search(&operation).is_ok()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DispatchPolicy {
    pub preferred: BackendKind,
    pub minimum_parallel_batch: usize,
    pub require_determinism: bool,
    pub required_precision: Precision,
}

impl Default for DispatchPolicy {
    fn default() -> Self {
        Self {
            preferred: BackendKind::Auto,
            minimum_parallel_batch: 1024,
            require_determinism: true,
            required_precision: Precision::F64,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AutoSelectionPolicy {
    pub preferred: Option<BackendKind>,
    pub minimum_batch_elements: usize,
    pub required_determinism: ExecutionDeterminism,
    pub allow_accelerator: bool,
}

impl Default for AutoSelectionPolicy {
    fn default() -> Self {
        Self {
            preferred: None,
            minimum_batch_elements: 1,
            required_determinism: ExecutionDeterminism::Numerical,
            allow_accelerator: true,
        }
    }
}

impl AutoSelectionPolicy {
    pub fn validate(self) -> Result<(), GpuError> {
        if self.minimum_batch_elements == 0 {
            return Err(GpuError::InvalidBatch);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectionReason {
    ExplicitCpu,
    ExplicitAccelerator,
    PreferredAccelerator,
    AutomaticAccelerator,
    AutomaticCpuFallback,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BackendSelection {
    pub backend: BackendKind,
    pub reason: SelectionReason,
}

#[derive(Clone, Debug, PartialEq)]
pub struct VectorBatch3 {
    pub values: Vec<Vec3>,
}

impl VectorBatch3 {
    pub fn new(values: Vec<Vec3>) -> Result<Self, GpuError> {
        if values.iter().any(|v| !v.is_finite()) {
            Err(GpuError::NonFinite)
        } else {
            Ok(Self { values })
        }
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MemoryDirection {
    UploadOnly,
    DownloadOnly,
    RoundTrip,
}

impl MemoryDirection {
    fn needs_upload(self) -> bool {
        matches!(self, Self::UploadOnly | Self::RoundTrip)
    }

    fn needs_download(self) -> bool {
        matches!(self, Self::DownloadOnly | Self::RoundTrip)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GpuBatchMemory {
    pub precision: Precision,
    pub elements: usize,
    pub components_per_element: usize,
    pub alignment_bytes: usize,
    pub direction: MemoryDirection,
}

impl GpuBatchMemory {
    pub fn validate(self) -> Result<(), GpuError> {
        if self.elements == 0 || self.components_per_element == 0 {
            return Err(GpuError::InvalidBatch);
        }
        if self.alignment_bytes == 0 || !self.alignment_bytes.is_power_of_two() {
            return Err(GpuError::InvalidBatch);
        }
        let _ = self.payload_bytes()?;
        Ok(())
    }

    pub fn payload_bytes(self) -> Result<usize, GpuError> {
        let element_bytes = match self.precision {
            Precision::F64 => 8usize,
            Precision::F32ConservativeBroadPhase => 4usize,
        };
        self.elements
            .checked_mul(self.components_per_element)
            .and_then(|count| count.checked_mul(element_bytes))
            .ok_or(GpuError::SizeOverflow)
    }

    pub fn transfer_bytes(self) -> Result<(usize, usize), GpuError> {
        self.validate()?;
        let bytes = self.payload_bytes()?;
        Ok((
            if self.direction.needs_upload() { bytes } else { 0 },
            if self.direction.needs_download() { bytes } else { 0 },
        ))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GpuDispatchPlan {
    pub operation: GpuOperation,
    pub elements: usize,
    pub workgroup_size: usize,
    pub memory: GpuBatchMemory,
    pub determinism: ExecutionDeterminism,
}

impl GpuDispatchPlan {
    pub fn validate_for(self, capability: BackendCapability) -> Result<(), GpuError> {
        capability.validate()?;
        self.memory.validate()?;
        if self.elements == 0 || self.elements != self.memory.elements {
            return Err(GpuError::InvalidBatch);
        }
        if self.workgroup_size == 0 {
            return Err(GpuError::InvalidDispatch);
        }
        if capability.max_workgroup_size.is_some_and(|limit| self.workgroup_size > limit) {
            return Err(GpuError::InvalidDispatch);
        }
        if capability.max_batch.is_some_and(|limit| self.elements > limit) {
            return Err(GpuError::InvalidDispatch);
        }
        if capability.operations.is_empty() {
            return Err(GpuError::UnsupportedOperation);
        }
        if !capability.supports_operation(self.operation) {
            return Err(GpuError::UnsupportedOperation);
        }
        if self.memory.precision == Precision::F64 && !capability.f64 {
            return Err(GpuError::UnsupportedPrecision);
        }
        if !determinism_satisfies(self.determinism, capability.determinism) {
            return Err(GpuError::UnsupportedDeterminism);
        }
        Ok(())
    }

    pub fn workgroups(self) -> Result<usize, GpuError> {
        if self.elements == 0 || self.workgroup_size == 0 {
            return Err(GpuError::InvalidDispatch);
        }
        self.elements
            .checked_add(self.workgroup_size - 1)
            .map(|value| value / self.workgroup_size)
            .ok_or(GpuError::SizeOverflow)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GpuExecutionReceipt {
    /// Diagnostic backend-local state. Never part of semantic identity.
    pub submission_id: u64,
    pub backend: BackendKind,
    pub workgroups: usize,
}

pub trait BatchExecutor {
    fn backend(&self) -> BackendKind;
    fn capability(&self) -> BackendCapability;
    fn evaluate_points(&self, input: &VectorBatch3) -> Result<VectorBatch3, GpuError>;
}

pub trait GpuBatchExecutor {
    fn backend(&self) -> BackendKind;
    fn capabilities(&self) -> BackendCapability;
    fn submit(&self, dispatch: GpuDispatchPlan) -> Result<GpuExecutionReceipt, GpuExecutionError>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GpuExecutionError {
    Unsupported,
    BackendFailure,
    Cancelled,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GpuError {
    Unavailable,
    UnsupportedPrecision,
    UnsupportedOperation,
    UnsupportedDeterminism,
    InvalidBatch,
    InvalidDispatch,
    InvalidCapability,
    NonFinite,
    BackendFailure,
    Cancelled,
    SizeOverflow,
    InvalidComparisonTolerance,
    ConformanceLengthMismatch,
    ConformanceNonFiniteOutput,
    ConformanceInvalidCandidateSet,
    ConformanceDuplicateCandidate,
    NumericalMismatch,
    BitwiseMismatch,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ConformanceReport {
    pub backend: BackendKind,
    pub determinism: ExecutionDeterminism,
    pub matched: bool,
    pub exact: bool,
    pub compared_values: usize,
    pub max_absolute_error: f64,
    pub max_relative_error: f64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CandidatePairConformanceReport {
    pub backend: BackendKind,
    pub reference_pair_count: usize,
    pub candidate_pair_count: usize,
    pub false_negative_count: usize,
    pub false_positive_count: usize,
    pub first_false_negative: Option<(usize, usize)>,
    pub first_false_positive: Option<(usize, usize)>,
    pub repeat_exact: bool,
}

impl CandidatePairConformanceReport {
    pub fn conforms(&self) -> bool {
        self.false_negative_count == 0 && self.repeat_exact
    }

    pub fn exact_match(&self) -> bool {
        self.conforms() && self.false_positive_count == 0
    }
}

pub fn compare_candidate_pairs(
    backend: BackendKind,
    reference: &[(usize, usize)],
    candidate: &[(usize, usize)],
    repeat: &[(usize, usize)],
) -> Result<CandidatePairConformanceReport, GpuError> {
    fn canonicalize(pairs: &[(usize, usize)]) -> Result<Vec<(usize, usize)>, GpuError> {
        let mut canonical = pairs.to_vec();
        if canonical
            .iter()
            .any(|&(left, right)| left >= right)
        {
            return Err(GpuError::ConformanceInvalidCandidateSet);
        }
        canonical.sort_unstable();
        if canonical.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(GpuError::ConformanceDuplicateCandidate);
        }
        Ok(canonical)
    }

    let reference = canonicalize(reference)?;
    let candidate = canonicalize(candidate)?;
    let repeat = canonicalize(repeat)?;

    let mut false_negatives = reference
        .iter()
        .copied()
        .filter(|pair| candidate.binary_search(pair).is_err());
    let first_false_negative = false_negatives.next();
    let false_negative_count = usize::from(first_false_negative.is_some())
        + false_negatives.count();

    let mut false_positives = candidate
        .iter()
        .copied()
        .filter(|pair| reference.binary_search(pair).is_err());
    let first_false_positive = false_positives.next();
    let false_positive_count = usize::from(first_false_positive.is_some())
        + false_positives.count();

    Ok(CandidatePairConformanceReport {
        backend,
        reference_pair_count: reference.len(),
        candidate_pair_count: candidate.len(),
        false_negative_count,
        false_positive_count,
        first_false_negative,
        first_false_positive,
        repeat_exact: candidate == repeat,
    })
}

pub fn compare_f64_batches(
    backend: BackendKind,
    determinism: ExecutionDeterminism,
    reference: &[f64],
    candidate: &[f64],
    absolute_tolerance: f64,
    relative_tolerance: f64,
) -> Result<ConformanceReport, GpuError> {
    if !absolute_tolerance.is_finite()
        || !relative_tolerance.is_finite()
        || absolute_tolerance < 0.0
        || relative_tolerance < 0.0
    {
        return Err(GpuError::InvalidComparisonTolerance);
    }
    if reference.len() != candidate.len() {
        return Err(GpuError::ConformanceLengthMismatch);
    }

    let mut exact = true;
    let mut max_absolute_error: f64 = 0.0;
    let mut max_relative_error: f64 = 0.0;

    for (&expected, &actual) in reference.iter().zip(candidate) {
        if !expected.is_finite() || !actual.is_finite() {
            return Err(GpuError::ConformanceNonFiniteOutput);
        }

        if expected.to_bits() != actual.to_bits() {
            exact = false;
        }

        let absolute = (expected - actual).abs();
        let scale = expected.abs().max(actual.abs()).max(f64::MIN_POSITIVE);
        let relative = absolute / scale;
        if !absolute.is_finite() || !relative.is_finite() {
            return Err(GpuError::ConformanceNonFiniteOutput);
        }

        max_absolute_error = max_absolute_error.max(absolute);
        max_relative_error = max_relative_error.max(relative);

        if determinism == ExecutionDeterminism::Bitwise
            && expected.to_bits() != actual.to_bits()
        {
            return Err(GpuError::BitwiseMismatch);
        }
        if !matches!(determinism, ExecutionDeterminism::NonDeterministic)
            && absolute > absolute_tolerance + relative_tolerance * scale
        {
            return Err(GpuError::NumericalMismatch);
        }
    }

    Ok(ConformanceReport {
        backend,
        determinism,
        matched: true,
        exact,
        compared_values: reference.len(),
        max_absolute_error,
        max_relative_error,
    })
}

fn determinism_satisfies(
    required: ExecutionDeterminism,
    provided: ExecutionDeterminism,
) -> bool {
    match required {
        ExecutionDeterminism::Bitwise => provided == ExecutionDeterminism::Bitwise,
        ExecutionDeterminism::Numerical => {
            matches!(
                provided,
                ExecutionDeterminism::Bitwise | ExecutionDeterminism::Numerical
            )
        }
        ExecutionDeterminism::NonDeterministic => true,
    }
}

fn capability_can_run(
    capability: BackendCapability,
    operation: GpuOperation,
    precision: Precision,
    batch_size: usize,
    policy: AutoSelectionPolicy,
) -> bool {
    capability.available
        && capability.max_batch.is_none_or(|max| batch_size <= max)
        && batch_size >= policy.minimum_batch_elements
        && capability.operations.binary_search(&operation).is_ok()
        && (precision != Precision::F64 || capability.f64)
        && determinism_satisfies(policy.required_determinism, capability.determinism)
}

pub fn select_backend_report(
    request: BackendKind,
    operation: GpuOperation,
    precision: Precision,
    batch_size: usize,
    capabilities: &[BackendCapability],
    policy: AutoSelectionPolicy,
) -> Result<BackendSelection, GpuError> {
    policy.validate()?;
    if batch_size == 0 {
        return Err(GpuError::InvalidBatch);
    }
    for capability in capabilities {
        capability.validate()?;
    }

    let find = |kind| capabilities.iter().find(|capability| capability.backend == kind);
    let eligible = |kind| {
        find(kind).is_some_and(|capability| {
            capability_can_run(*capability, operation, precision, batch_size, policy)
        })
    };

    match request {
        BackendKind::Cpu => {
            if eligible(BackendKind::Cpu) {
                Ok(BackendSelection {
                    backend: BackendKind::Cpu,
                    reason: SelectionReason::ExplicitCpu,
                })
            } else {
                Err(GpuError::Unavailable)
            }
        }
        BackendKind::Metal | BackendKind::Cuda => {
            let kind = request;
            if eligible(kind) {
                Ok(BackendSelection {
                    backend: kind,
                    reason: SelectionReason::ExplicitAccelerator,
                })
            } else {
                Err(GpuError::UnsupportedOperation)
            }
        }
        BackendKind::Auto => {
            if policy.allow_accelerator {
                if let Some(preferred) = policy.preferred.filter(|kind| eligible(*kind)) {
                    return Ok(BackendSelection {
                        backend: preferred,
                        reason: SelectionReason::PreferredAccelerator,
                    });
                }
                for kind in [BackendKind::Metal, BackendKind::Cuda] {
                    if eligible(kind) {
                        return Ok(BackendSelection {
                            backend: kind,
                            reason: SelectionReason::AutomaticAccelerator,
                        });
                    }
                }
            }

            if find(BackendKind::Cpu).is_some_and(|capability| {
                capability.available
                    && capability.operations.binary_search(&operation).is_ok()
                    && (precision != Precision::F64 || capability.f64)
                    && determinism_satisfies(policy.required_determinism, capability.determinism)
            }) {
                Ok(BackendSelection {
                    backend: BackendKind::Cpu,
                    reason: SelectionReason::AutomaticCpuFallback,
                })
            } else {
                Err(GpuError::Unavailable)
            }
        }
    }
}

/// Compatibility wrapper for the early M12 API. New authority code should use
/// `select_backend_report`, because a bare backend value cannot explain fallback.
#[deprecated(note = "use select_backend_report for authority decisions")]
pub fn select_backend(
    policy: DispatchPolicy,
    cpu: BackendCapability,
    metal: BackendCapability,
    cuda: BackendCapability,
    batch_size: usize,
) -> BackendKind {
    let min_batch = policy.minimum_parallel_batch;
    let determinism = if policy.require_determinism {
        ExecutionDeterminism::Bitwise
    } else {
        ExecutionDeterminism::Numerical
    };
    let auto_policy = AutoSelectionPolicy {
        preferred: match policy.preferred {
            BackendKind::Auto => None,
            backend => Some(backend),
        },
        minimum_batch_elements: min_batch.max(1),
        required_determinism: determinism,
        allow_accelerator: true,
    };
    select_backend_report(
        policy.preferred,
        GpuOperation::VectorBatch,
        policy.required_precision,
        batch_size,
        &[cpu, metal, cuda],
        auto_policy,
    )
    .map(|selection| selection.backend)
    .unwrap_or(BackendKind::Cpu)
}

#[derive(Clone, Copy, Debug, Default)]
pub struct CpuReferenceExecutor;

impl BatchExecutor for CpuReferenceExecutor {
    fn backend(&self) -> BackendKind {
        BackendKind::Cpu
    }

    fn capability(&self) -> BackendCapability {
        BackendCapability::legacy(
            BackendKind::Cpu,
            true,
            true,
            true,
            None,
        )
    }

    fn evaluate_points(&self, input: &VectorBatch3) -> Result<VectorBatch3, GpuError> {
        VectorBatch3::new(input.values.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VECTOR_OPS: &[GpuOperation] = &[GpuOperation::VectorBatch];
    const CURVE_OPS: &[GpuOperation] = &[
        GpuOperation::VectorBatch,
        GpuOperation::CurveEvaluationBatch,
    ];

    fn capability(
        backend: BackendKind,
        available: bool,
        f64: bool,
        determinism: ExecutionDeterminism,
        max_batch: Option<usize>,
        operations: &'static [GpuOperation],
    ) -> BackendCapability {
        BackendCapability {
            backend,
            available,
            f64,
            deterministic: matches!(
                determinism,
                ExecutionDeterminism::Bitwise | ExecutionDeterminism::Numerical
            ),
            max_batch,
            operations,
            max_workgroup_size: Some(256),
            determinism,
        }
    }

    fn cpu_capability() -> BackendCapability {
        capability(
            BackendKind::Cpu,
            true,
            true,
            ExecutionDeterminism::Bitwise,
            None,
            CURVE_OPS,
        )
    }

    fn metal_capability() -> BackendCapability {
        capability(
            BackendKind::Metal,
            true,
            true,
            ExecutionDeterminism::Numerical,
            Some(100_000),
            CURVE_OPS,
        )
    }

    #[test]
    fn auto_selection_records_cpu_fallback_instead_of_faking_acceleration() {
        let selected = select_backend_report(
            BackendKind::Auto,
            GpuOperation::VectorBatch,
            Precision::F64,
            16,
            &[cpu_capability()],
            AutoSelectionPolicy {
                minimum_batch_elements: 64,
                ..Default::default()
            },
        )
        .unwrap();

        assert_eq!(
            selected,
            BackendSelection {
                backend: BackendKind::Cpu,
                reason: SelectionReason::AutomaticCpuFallback
            }
        );
    }

    #[test]
    fn explicit_unavailable_accelerator_is_rejected() {
        let result = select_backend_report(
            BackendKind::Cuda,
            GpuOperation::VectorBatch,
            Precision::F64,
            64,
            &[cpu_capability()],
            AutoSelectionPolicy::default(),
        );
        assert_eq!(result, Err(GpuError::UnsupportedOperation));
    }

    #[test]
    fn auto_selection_honors_preferred_accelerator_and_batch_threshold() {
        let result = select_backend_report(
            BackendKind::Auto,
            GpuOperation::CurveEvaluationBatch,
            Precision::F64,
            256,
            &[cpu_capability(), metal_capability()],
            AutoSelectionPolicy {
                preferred: Some(BackendKind::Metal),
                minimum_batch_elements: 128,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(result.backend, BackendKind::Metal);

        let fallback = select_backend_report(
            BackendKind::Auto,
            GpuOperation::CurveEvaluationBatch,
            Precision::F64,
            64,
            &[cpu_capability(), metal_capability()],
            AutoSelectionPolicy {
                preferred: Some(BackendKind::Metal),
                minimum_batch_elements: 128,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(fallback.backend, BackendKind::Cpu);
    }

    #[test]
    fn f64_and_determinism_are_explicit_capabilities() {
        let mut metal = metal_capability();
        metal.f64 = false;
        assert_eq!(
            select_backend_report(
                BackendKind::Auto,
                GpuOperation::VectorBatch,
                Precision::F64,
                256,
                &[cpu_capability(), metal],
                AutoSelectionPolicy::default(),
            )
            .unwrap()
            .backend,
            BackendKind::Cpu
        );

        let nondeterministic = capability(
            BackendKind::Metal,
            true,
            true,
            ExecutionDeterminism::NonDeterministic,
            None,
            VECTOR_OPS,
        );
        assert_eq!(
            select_backend_report(
                BackendKind::Auto,
                GpuOperation::VectorBatch,
                Precision::F64,
                256,
                &[cpu_capability(), nondeterministic],
                AutoSelectionPolicy {
                    required_determinism: ExecutionDeterminism::Bitwise,
                    ..Default::default()
                },
            )
            .unwrap()
            .backend,
            BackendKind::Cpu
        );
    }

    #[test]
    fn batch_memory_and_dispatch_are_checked_for_overflow_and_shape() {
        let memory = GpuBatchMemory {
            precision: Precision::F64,
            elements: usize::MAX,
            components_per_element: 2,
            alignment_bytes: 8,
            direction: MemoryDirection::RoundTrip,
        };
        assert_eq!(memory.payload_bytes(), Err(GpuError::SizeOverflow));

        let memory = GpuBatchMemory {
            precision: Precision::F64,
            elements: 1000,
            components_per_element: 3,
            alignment_bytes: 8,
            direction: MemoryDirection::RoundTrip,
        };
        let dispatch = GpuDispatchPlan {
            operation: GpuOperation::VectorBatch,
            elements: 1000,
            workgroup_size: 128,
            memory,
            determinism: ExecutionDeterminism::Numerical,
        };
        dispatch
            .validate_for(metal_capability())
            .expect("valid dispatch");
        assert_eq!(dispatch.workgroups().unwrap(), 8);
        assert_eq!(memory.transfer_bytes().unwrap(), (24_000, 24_000));
    }

    #[test]
    fn conformance_is_fail_closed_on_length_nonfinite_and_bitwise_mismatch() {
        assert_eq!(
            compare_f64_batches(
                BackendKind::Cuda,
                ExecutionDeterminism::Numerical,
                &[1.0],
                &[],
                1.0e-12,
                1.0e-12,
            ),
            Err(GpuError::ConformanceLengthMismatch)
        );
        assert_eq!(
            compare_f64_batches(
                BackendKind::Cuda,
                ExecutionDeterminism::Numerical,
                &[f64::NAN],
                &[f64::NAN],
                1.0e-12,
                1.0e-12,
            ),
            Err(GpuError::ConformanceNonFiniteOutput)
        );
        assert_eq!(
            compare_f64_batches(
                BackendKind::Metal,
                ExecutionDeterminism::Bitwise,
                &[1.0],
                &[1.0 + f64::EPSILON],
                1.0,
                1.0,
            ),
            Err(GpuError::BitwiseMismatch)
        );
    }

    #[test]
    fn candidate_pair_conformance_reports_false_negatives_and_false_positives() {
        let report = compare_candidate_pairs(
            BackendKind::Metal,
            &[(0, 1), (2, 3)],
            &[(0, 1), (0, 2)],
            &[(0, 1), (0, 2)],
        ).unwrap();

        assert_eq!(report.reference_pair_count, 2);
        assert_eq!(report.candidate_pair_count, 2);
        assert_eq!(report.false_negative_count, 1);
        assert_eq!(report.first_false_negative, Some((2, 3)));
        assert_eq!(report.false_positive_count, 1);
        assert_eq!(report.first_false_positive, Some((0, 2)));
        assert!(report.repeat_exact);
        assert!(!report.conforms());
        assert!(!report.exact_match());
    }

    #[test]
    fn candidate_pair_conformance_accepts_conservative_superset() {
        let report = compare_candidate_pairs(
            BackendKind::Cuda,
            &[(0, 1), (2, 3)],
            &[(0, 1), (0, 2), (2, 3)],
            &[(0, 1), (0, 2), (2, 3)],
        ).unwrap();

        assert_eq!(report.false_negative_count, 0);
        assert_eq!(report.false_positive_count, 1);
        assert!(report.conforms());
        assert!(!report.exact_match());
    }

    #[test]
    fn candidate_pair_conformance_rejects_malformed_sets() {
        assert_eq!(
            compare_candidate_pairs(
                BackendKind::Metal,
                &[(0, 1)],
                &[(1, 0)],
                &[(1, 0)],
            ),
            Err(GpuError::ConformanceInvalidCandidateSet)
        );
        assert_eq!(
            compare_candidate_pairs(
                BackendKind::Metal,
                &[(0, 1)],
                &[(0, 1), (0, 1)],
                &[(0, 1)],
            ),
            Err(GpuError::ConformanceDuplicateCandidate)
        );
    }

    #[test]
    fn conformance_accepts_numerically_deterministic_f64_batch() {
        let report = compare_f64_batches(
            BackendKind::Metal,
            ExecutionDeterminism::Numerical,
            &[1.0, 2.0, 10.0],
            &[1.0, 2.0 + 1.0e-12, 10.0 - 1.0e-11],
            1.0e-10,
            1.0e-12,
        )
        .unwrap();
        assert!(report.matched);
        assert!(!report.exact);
        assert_eq!(report.compared_values, 3);
        assert!(report.max_absolute_error <= 1.0e-10);
    }

    #[test]
    fn cpu_reference_executor_keeps_reference_path_explicit() {
        let input = VectorBatch3::new(vec![Vec3::new(1.0, 2.0, 3.0)]).unwrap();
        let output = CpuReferenceExecutor.evaluate_points(&input).unwrap();
        assert_eq!(input, output);
        assert_eq!(CpuReferenceExecutor.backend(), BackendKind::Cpu);
    }

    #[test]
    #[allow(deprecated)]
    fn legacy_selector_remains_safe_for_existing_callers() {
        let cpu = BackendCapability::legacy(BackendKind::Cpu, true, true, true, None);
        let metal = BackendCapability::legacy(BackendKind::Metal, true, true, true, None);
        let selected = select_backend(
            DispatchPolicy {
                minimum_parallel_batch: 8,
                ..Default::default()
            },
            cpu,
            metal,
            metal,
            8,
        );
        assert_eq!(selected, BackendKind::Metal);
    }

    #[test]
    fn execution_receipt_contains_only_diagnostic_backend_state() {
        struct DescriptorOnlyExecutor {
            capability: BackendCapability,
        }

        impl GpuBatchExecutor for DescriptorOnlyExecutor {
            fn backend(&self) -> BackendKind {
                self.capability.backend
            }

            fn capabilities(&self) -> BackendCapability {
                self.capability
            }

            fn submit(
                &self,
                dispatch: GpuDispatchPlan,
            ) -> Result<GpuExecutionReceipt, GpuExecutionError> {
                dispatch
                    .validate_for(self.capability)
                    .map_err(|_| GpuExecutionError::Unsupported)?;
                Ok(GpuExecutionReceipt {
                    submission_id: 7,
                    backend: self.capability.backend,
                    workgroups: dispatch
                        .workgroups()
                        .map_err(|_| GpuExecutionError::BackendFailure)?,
                })
            }
        }

        let executor = DescriptorOnlyExecutor {
            capability: cpu_capability(),
        };
        let dispatch = GpuDispatchPlan {
            operation: GpuOperation::VectorBatch,
            elements: 9,
            workgroup_size: 4,
            memory: GpuBatchMemory {
                precision: Precision::F64,
                elements: 9,
                components_per_element: 3,
                alignment_bytes: 8,
                direction: MemoryDirection::RoundTrip,
            },
            determinism: ExecutionDeterminism::Bitwise,
        };
        let receipt = executor.submit(dispatch).unwrap();
        assert_eq!(receipt.backend, BackendKind::Cpu);
        assert_eq!(receipt.workgroups, 3);
        assert_eq!(receipt.submission_id, 7);
    }
}
