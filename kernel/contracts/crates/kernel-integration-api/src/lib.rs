use sha2::{Digest, Sha256};
use topology_api::TopologySnapshotId;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OperationId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OperationKind {
    Create,
    Transform,
    Modify,
    Boolean,
    Repair,
    Import,
    Export,
}

pub trait OperationCancellation {
    fn is_cancelled(&self) -> bool;
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NeverCancel;

impl OperationCancellation for NeverCancel {
    fn is_cancelled(&self) -> bool {
        false
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OperationPhase {
    Preconditions,
    Execute,
    Validate,
    Commit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OperationErrorKind {
    Cancelled,
    InvalidInput,
    Unsupported,
    SemanticInvariantViolation,
    BackendFailure,
    Internal,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OperationError {
    pub operation_id: OperationId,
    pub phase: OperationPhase,
    pub kind: OperationErrorKind,
}

impl OperationError {
    pub const fn new(
        operation_id: OperationId,
        phase: OperationPhase,
        kind: OperationErrorKind,
    ) -> Self {
        Self { operation_id, phase, kind }
    }

    pub const fn cancelled(operation_id: OperationId, phase: OperationPhase) -> Self {
        Self::new(operation_id, phase, OperationErrorKind::Cancelled)
    }

    pub fn check_cancellation<C: OperationCancellation + ?Sized>(
        cancellation: &C,
        operation_id: OperationId,
        phase: OperationPhase,
    ) -> Result<(), Self> {
        if cancellation.is_cancelled() {
            Err(Self::cancelled(operation_id, phase))
        } else {
            Ok(())
        }
    }
}

pub type OperationOutcome<T> = Result<T, OperationError>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OperationDescriptor {
    pub id: OperationId,
    pub kind: OperationKind,
    pub inputs: Vec<TopologySnapshotId>,
    pub output: TopologySnapshotId,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProvenanceError {
    ZeroOperationId,
    NoInputs,
    ZeroInput(usize),
    ZeroOutput,
    NonIncreasingOperationId { previous: OperationId, current: OperationId },
    DuplicateOperationId(OperationId),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OperationProvenance {
    operations: Vec<OperationDescriptor>,
}

impl OperationProvenance {
    pub fn new() -> Self {
        Self { operations: Vec::new() }
    }

    pub fn operations(&self) -> &[OperationDescriptor] {
        &self.operations
    }

    pub fn append(&self, operation: OperationDescriptor) -> Result<Self, ProvenanceError> {
        if operation.id.0 == 0 {
            return Err(ProvenanceError::ZeroOperationId);
        }
        if operation.inputs.is_empty() {
            return Err(ProvenanceError::NoInputs);
        }
        if let Some((index, _)) = operation
            .inputs
            .iter()
            .enumerate()
            .find(|(_, input)| input.bytes() == [0; 32])
        {
            return Err(ProvenanceError::ZeroInput(index));
        }
        if operation.output.bytes() == [0; 32] {
            return Err(ProvenanceError::ZeroOutput);
        }
        if let Some(previous) = self.operations.last() {
            if previous.id == operation.id {
                return Err(ProvenanceError::DuplicateOperationId(operation.id));
            }
            if previous.id >= operation.id {
                return Err(ProvenanceError::NonIncreasingOperationId {
                    previous: previous.id,
                    current: operation.id,
                });
            }
        }
        let mut next = self.clone();
        next.operations.push(operation);
        Ok(next)
    }

    pub fn identity(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(b"UMLCAD.V6.OPERATION_PROVENANCE\0");
        hasher.update((self.operations.len() as u64).to_le_bytes());
        for operation in &self.operations {
            hasher.update(operation.id.0.to_le_bytes());
            hasher.update([operation.kind as u8]);
            hasher.update((operation.inputs.len() as u64).to_le_bytes());
            for input in &operation.inputs {
                hasher.update(input.bytes());
            }
            hasher.update(operation.output.bytes());
        }
        hasher.finalize().into()
    }
}

impl Default for OperationProvenance {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IntegrationSnapshot {
    provenance: OperationProvenance,
    roots: Vec<TopologySnapshotId>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SnapshotError {
    NonEmptyRootsContainZeroIdentity(usize),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SnapshotOperationError {
    InvalidOperation(ProvenanceError),
    InputNotActive { index: usize, input: TopologySnapshotId },
    DuplicateInput { first_index: usize, second_index: usize, input: TopologySnapshotId },
    OutputAlreadyActive { output: TopologySnapshotId },
}

impl IntegrationSnapshot {
    pub fn new(
        provenance: OperationProvenance,
        roots: Vec<TopologySnapshotId>,
    ) -> Result<Self, SnapshotError> {
        for (index, root) in roots.iter().enumerate() {
            if root.bytes() == [0; 32] {
                return Err(SnapshotError::NonEmptyRootsContainZeroIdentity(index));
            }
        }
        Ok(Self { provenance, roots })
    }

    pub fn provenance(&self) -> &OperationProvenance {
        &self.provenance
    }

    pub fn roots(&self) -> &[TopologySnapshotId] {
        &self.roots
    }

    pub fn with_roots(&self, roots: Vec<TopologySnapshotId>) -> Result<Self, SnapshotError> {
        Self::new(self.provenance.clone(), roots)
    }

    pub fn apply_operation(
        &self,
        operation: OperationDescriptor,
    ) -> Result<Self, SnapshotOperationError> {
        self.provenance
            .append(operation.clone())
            .map_err(SnapshotOperationError::InvalidOperation)?;

        for (index, input) in operation.inputs.iter().enumerate() {
            if let Some(first_index) = operation.inputs[..index]
                .iter()
                .position(|candidate| candidate == input)
            {
                return Err(SnapshotOperationError::DuplicateInput {
                    first_index,
                    second_index: index,
                    input: *input,
                });
            }
            if !self.roots.contains(input) {
                return Err(SnapshotOperationError::InputNotActive { index, input: *input });
            }
        }

        if self.roots.contains(&operation.output)
            && !operation.inputs.contains(&operation.output)
        {
            return Err(SnapshotOperationError::OutputAlreadyActive { output: operation.output });
        }

        let provenance = self
            .provenance
            .append(operation.clone())
            .map_err(SnapshotOperationError::InvalidOperation)?;
        let mut roots = self
            .roots
            .iter()
            .copied()
            .filter(|root| !operation.inputs.contains(root))
            .collect::<Vec<_>>();
        roots.push(operation.output);
        match Self::new(provenance, roots) {
            Ok(snapshot) => Ok(snapshot),
            Err(SnapshotError::NonEmptyRootsContainZeroIdentity(_)) => {
                Err(SnapshotOperationError::InvalidOperation(ProvenanceError::ZeroOutput))
            }
        }
    }

    /// Applies an ordered operation sequence using immutable intermediate values.
    /// No intermediate state is externally observable, so a failure returns without
    /// changing the source snapshot. This is composition, not transaction semantics.
    pub fn apply_operations(
        &self,
        operations: &[OperationDescriptor],
    ) -> Result<Self, SnapshotOperationError> {
        let mut current = self.clone();
        for operation in operations {
            current = current.apply_operation(operation.clone())?;
        }
        Ok(current)
    }

    pub fn identity(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(b"UMLCAD.V6.INTEGRATION_SNAPSHOT\0");
        hasher.update(self.provenance.identity());
        hasher.update((self.roots.len() as u64).to_le_bytes());
        for root in &self.roots {
            hasher.update(root.bytes());
        }
        hasher.finalize().into()
    }
}

impl Default for IntegrationSnapshot {
    fn default() -> Self {
        Self { provenance: OperationProvenance::new(), roots: Vec::new() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use topology_api::{EdgeId, EdgeUse, FaceId, ShellOrientationGraph};

    fn snapshot(seed: u32) -> TopologySnapshotId {
        let graph = ShellOrientationGraph {
            faces: [FaceId(seed)].into_iter().collect(),
            edge_uses: vec![EdgeUse { edge: EdgeId(seed), face: FaceId(seed), forward: true }],
        };
        graph.snapshot_id()
    }

    fn op(id: u64) -> OperationDescriptor {
        OperationDescriptor {
            id: OperationId(id),
            kind: OperationKind::Modify,
            inputs: vec![snapshot(id as u32)],
            output: snapshot(id as u32 + 1),
        }
    }

    struct FixedCancellation;

    impl OperationCancellation for FixedCancellation {
        fn is_cancelled(&self) -> bool {
            true
        }
    }

    #[test]
    fn append_is_immutable_and_ordered() {
        let empty = OperationProvenance::new();
        let first = empty.append(op(1)).unwrap();
        let second = first.append(op(2)).unwrap();
        assert!(empty.operations().is_empty());
        assert_eq!(first.operations().len(), 1);
        assert_eq!(second.operations().len(), 2);
        assert_eq!(second.operations()[0].id, OperationId(1));
        assert_eq!(second.operations()[1].id, OperationId(2));
    }

    #[test]
    fn rejects_zero_and_missing_values() {
        let provenance = OperationProvenance::new();
        assert_eq!(provenance.append(OperationDescriptor {
            id: OperationId(0), kind: OperationKind::Create,
            inputs: vec![snapshot(1)], output: snapshot(2),
        }), Err(ProvenanceError::ZeroOperationId));
        assert_eq!(provenance.append(OperationDescriptor {
            id: OperationId(1), kind: OperationKind::Create,
            inputs: Vec::new(), output: snapshot(2),
        }), Err(ProvenanceError::NoInputs));
        assert_eq!(provenance.append(OperationDescriptor {
            id: OperationId(1), kind: OperationKind::Create,
            inputs: vec![TopologySnapshotId([0; 32])], output: snapshot(2),
        }), Err(ProvenanceError::ZeroInput(0)));
    }

    #[test]
    fn rejects_non_monotonic_and_duplicate_operation_ids() {
        let first = OperationProvenance::new().append(op(2)).unwrap();
        assert_eq!(first.append(op(1)), Err(ProvenanceError::NonIncreasingOperationId {
            previous: OperationId(2), current: OperationId(1),
        }));
        assert_eq!(first.append(op(2)), Err(ProvenanceError::DuplicateOperationId(OperationId(2))));
    }

    #[test]
    fn provenance_identity_is_deterministic_and_sensitive_to_semantics() {
        let a = OperationProvenance::new().append(op(1)).unwrap().append(op(2)).unwrap();
        let b = OperationProvenance::new().append(op(1)).unwrap().append(op(2)).unwrap();
        assert_eq!(a.identity(), b.identity());
        let changed = OperationProvenance::new().append(OperationDescriptor {
            id: OperationId(1), kind: OperationKind::Repair,
            inputs: vec![snapshot(1)], output: snapshot(2),
        }).unwrap().append(op(2)).unwrap();
        assert_ne!(a.identity(), changed.identity());
    }

    #[test]
    fn provenance_does_not_mutate_input_snapshots() {
        let graph = ShellOrientationGraph {
            faces: [FaceId(1)].into_iter().collect(),
            edge_uses: vec![EdgeUse { edge: EdgeId(1), face: FaceId(1), forward: true }],
        };
        let before = graph.clone();
        let _ = OperationProvenance::new().append(OperationDescriptor {
            id: OperationId(1), kind: OperationKind::Import,
            inputs: vec![graph.snapshot_id()], output: snapshot(2),
        }).unwrap();
        assert_eq!(graph, before);
    }

    #[test]
    fn cancellation_is_read_only_and_distinguishable() {
        let never = NeverCancel;
        assert_eq!(OperationError::check_cancellation(
            &never, OperationId(1), OperationPhase::Preconditions,
        ), Ok(()));
        let cancelled = FixedCancellation;
        let error = OperationError::check_cancellation(
            &cancelled, OperationId(7), OperationPhase::Execute,
        ).unwrap_err();
        assert_eq!(error, OperationError::cancelled(OperationId(7), OperationPhase::Execute));
    }

    #[test]
    fn error_categories_and_propagation_remain_structured() {
        let categories = [
            OperationErrorKind::Cancelled,
            OperationErrorKind::InvalidInput,
            OperationErrorKind::Unsupported,
            OperationErrorKind::SemanticInvariantViolation,
            OperationErrorKind::BackendFailure,
            OperationErrorKind::Internal,
        ];
        for kind in categories {
            let source = OperationError::new(OperationId(9), OperationPhase::Validate, kind);
            let propagated: OperationOutcome<()> = Err(source);
            assert_eq!(propagated, Err(source));
        }
    }

    #[test]
    fn cancellation_does_not_mutate_provenance() {
        let provenance = OperationProvenance::new().append(op(1)).unwrap();
        let before = provenance.clone();
        let cancelled = FixedCancellation;
        let result = OperationError::check_cancellation(
            &cancelled, OperationId(2), OperationPhase::Commit,
        );
        assert_eq!(result, Err(OperationError::cancelled(OperationId(2), OperationPhase::Commit)));
        assert_eq!(provenance, before);
    }

    #[test]
    fn integration_snapshot_is_immutable_and_order_sensitive() {
        let provenance = OperationProvenance::new().append(op(1)).unwrap();
        let first = IntegrationSnapshot::new(
            provenance.clone(), vec![snapshot(2), snapshot(3)],
        ).unwrap();
        let second = IntegrationSnapshot::new(provenance, vec![snapshot(2), snapshot(3)]).unwrap();
        assert_eq!(first, second);
        assert_eq!(first.identity(), second.identity());
        assert_ne!(
            first.identity(),
            IntegrationSnapshot::new(
                second.provenance().clone(), vec![snapshot(3), snapshot(2)],
            ).unwrap().identity(),
        );
        let changed = first.with_roots(vec![snapshot(2)]).unwrap();
        assert_ne!(first, changed);
        assert_eq!(first.roots().len(), 2);
    }

    #[test]
    fn integration_snapshot_rejects_zero_roots_and_preserves_provenance() {
        let provenance = OperationProvenance::new().append(op(1)).unwrap();
        assert_eq!(
            IntegrationSnapshot::new(provenance.clone(), vec![TopologySnapshotId([0; 32])]),
            Err(SnapshotError::NonEmptyRootsContainZeroIdentity(0)),
        );
        let snapshot = IntegrationSnapshot::new(provenance.clone(), vec![snapshot(2)]).unwrap();
        let updated = snapshot.with_roots(Vec::new()).unwrap();
        assert_eq!(updated.provenance(), &provenance);
        assert!(updated.roots().is_empty());
    }

    #[test]
    fn apply_operation_atomically_replaces_inputs_and_appends_output() {
        let state = IntegrationSnapshot::new(
            OperationProvenance::new(),
            vec![snapshot(1), snapshot(2), snapshot(3)],
        ).unwrap();
        let operation = OperationDescriptor {
            id: OperationId(1),
            kind: OperationKind::Boolean,
            inputs: vec![snapshot(1), snapshot(2)],
            output: snapshot(9),
        };
        let next = state.apply_operation(operation.clone()).unwrap();
        assert_eq!(state.roots(), &[snapshot(1), snapshot(2), snapshot(3)]);
        assert_eq!(next.roots(), &[snapshot(3), snapshot(9)]);
        assert_eq!(next.provenance().operations(), &[operation]);
    }

    #[test]
    fn apply_operation_rejects_inactive_duplicate_and_conflicting_outputs() {
        let state = IntegrationSnapshot::new(OperationProvenance::new(), vec![snapshot(1), snapshot(2)]).unwrap();
        assert_eq!(
            state.apply_operation(OperationDescriptor {
                id: OperationId(1), kind: OperationKind::Modify,
                inputs: vec![snapshot(7)], output: snapshot(8),
            }),
            Err(SnapshotOperationError::InputNotActive { index: 0, input: snapshot(7) }),
        );
        assert_eq!(
            state.apply_operation(OperationDescriptor {
                id: OperationId(1), kind: OperationKind::Boolean,
                inputs: vec![snapshot(1), snapshot(1)], output: snapshot(8),
            }),
            Err(SnapshotOperationError::DuplicateInput {
                first_index: 0, second_index: 1, input: snapshot(1),
            }),
        );
        assert_eq!(
            state.apply_operation(OperationDescriptor {
                id: OperationId(1), kind: OperationKind::Modify,
                inputs: vec![snapshot(1)], output: snapshot(2),
            }),
            Err(SnapshotOperationError::OutputAlreadyActive { output: snapshot(2) }),
        );
        assert!(state.provenance().operations().is_empty());
    }

    #[test]
    fn apply_operations_is_empty_identity_and_sequentially_equivalent() {
        let state = IntegrationSnapshot::new(
            OperationProvenance::new(),
            vec![snapshot(1), snapshot(4)],
        ).unwrap();
        let operations = vec![
            OperationDescriptor {
                id: OperationId(1),
                kind: OperationKind::Modify,
                inputs: vec![snapshot(1)],
                output: snapshot(2),
            },
            OperationDescriptor {
                id: OperationId(2),
                kind: OperationKind::Modify,
                inputs: vec![snapshot(2)],
                output: snapshot(3),
            },
        ];
        let empty = state.apply_operations(&[]).unwrap();
        assert_eq!(empty, state);
        let batch = state.apply_operations(&operations).unwrap();
        let sequential = state
            .apply_operation(operations[0].clone())
            .unwrap()
            .apply_operation(operations[1].clone())
            .unwrap();
        assert_eq!(batch, sequential);
    }

    #[test]
    fn apply_operations_failure_preserves_source() {
        let state = IntegrationSnapshot::new(
            OperationProvenance::new(),
            vec![snapshot(1), snapshot(4)],
        ).unwrap();
        let operations = vec![
            OperationDescriptor {
                id: OperationId(1),
                kind: OperationKind::Modify,
                inputs: vec![snapshot(1)],
                output: snapshot(2),
            },
            OperationDescriptor {
                id: OperationId(2),
                kind: OperationKind::Modify,
                inputs: vec![snapshot(99)],
                output: snapshot(3),
            },
        ];
        let before = state.clone();
        assert!(state.apply_operations(&operations).is_err());
        assert_eq!(state, before);
    }
}
