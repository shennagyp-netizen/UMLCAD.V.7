use topology_api::{EdgeId, EdgeUse, FaceId, ShellOrientationGraph};
use topology_api::snapshot::TopologySnapshotId;
use umlcad_v6_kernel_integration_api::{
    IntegrationSnapshot, OperationDescriptor, OperationId, OperationKind, OperationProvenance,
};

fn snapshot(seed: u32) -> TopologySnapshotId {
    let graph = ShellOrientationGraph {
        faces: [FaceId(seed)].into_iter().collect(),
        edge_uses: vec![EdgeUse {
            edge: EdgeId(seed),
            face: FaceId(seed),
            forward: true,
        }],
    };
    graph.snapshot_id()
}

fn operation(id: u64, input: TopologySnapshotId, output: TopologySnapshotId) -> OperationDescriptor {
    OperationDescriptor {
        id: OperationId(id),
        kind: OperationKind::Modify,
        inputs: vec![input],
        output,
    }
}

#[test]
fn repeated_topology_identity_is_deterministic() {
    let graph = ShellOrientationGraph {
        faces: [FaceId(1), FaceId(2), FaceId(3)].into_iter().collect(),
        edge_uses: vec![
            EdgeUse { edge: EdgeId(1), face: FaceId(1), forward: true },
            EdgeUse { edge: EdgeId(1), face: FaceId(2), forward: false },
            EdgeUse { edge: EdgeId(2), face: FaceId(2), forward: true },
            EdgeUse { edge: EdgeId(2), face: FaceId(3), forward: false },
        ],
    };
    let expected = graph.snapshot_id();

    for _ in 0..10_000 {
        assert_eq!(graph.snapshot_id(), expected);
    }
}

#[test]
fn repeated_immutable_batch_composition_preserves_source() {
    let a = snapshot(1);
    let b = snapshot(2);
    let c = snapshot(3);
    let d = snapshot(4);
    let initial = IntegrationSnapshot::new(OperationProvenance::new(), vec![a, d]).unwrap();
    let operations = [
        operation(1, a, b),
        operation(2, b, c),
    ];
    let before = initial.clone();
    let expected = initial.apply_operations(&operations).unwrap();

    for _ in 0..2_000 {
        let current = initial.apply_operations(&operations).unwrap();
        assert_eq!(current, expected);
        assert_eq!(initial, before);
    }
}

#[test]
fn repeated_provenance_identity_remains_stable() {
    let provenance = OperationProvenance::new()
        .append(operation(1, snapshot(1), snapshot(2)))
        .unwrap()
        .append(operation(2, snapshot(2), snapshot(3)))
        .unwrap();
    let expected = provenance.identity();

    for _ in 0..10_000 {
        assert_eq!(provenance.identity(), expected);
    }
}

#[test]
fn clone_and_identity_cycles_preserve_semantics() {
    let state = IntegrationSnapshot::new(
        OperationProvenance::new(),
        vec![snapshot(10), snapshot(11), snapshot(12)],
    )
    .unwrap();
    let expected_identity = state.identity();
    let expected_roots = state.roots().to_vec();

    let mut current = state.clone();
    for _ in 0..5_000 {
        let cloned = current.clone();
        assert_eq!(cloned.identity(), expected_identity);
        assert_eq!(cloned.roots(), expected_roots.as_slice());
        current = cloned;
    }
    assert_eq!(state.identity(), expected_identity);
}
