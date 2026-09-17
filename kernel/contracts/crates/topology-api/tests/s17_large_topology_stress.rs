use umlcad_v6_topology_api::{EdgeId, EdgeUse, FaceId, ShellOrientationGraph};

fn ring_graph(face_count: u32) -> ShellOrientationGraph {
    assert!(face_count >= 3);

    let faces = (0..face_count).map(FaceId).collect();
    let mut edge_uses = Vec::with_capacity(face_count as usize * 2);

    for face in 0..face_count {
        let next = (face + 1) % face_count;
        edge_uses.push(EdgeUse {
            edge: EdgeId(face),
            face: FaceId(face),
            forward: true,
        });
        edge_uses.push(EdgeUse {
            edge: EdgeId(face),
            face: FaceId(next),
            forward: false,
        });
    }

    ShellOrientationGraph { faces, edge_uses }
}

#[test]
fn large_closed_shell_repeated_validation_is_stable() {
    let graph = ring_graph(2_000);
    let before = graph.clone();

    for _ in 0..100 {
        assert_eq!(graph.validate(), Ok(()));
    }

    assert_eq!(graph, before);
}

#[test]
fn large_closed_shell_repeated_orientation_is_stable() {
    let graph = ring_graph(2_000);
    let expected = graph.propagate(FaceId(0)).unwrap();

    assert_eq!(expected.face_flip.len(), graph.faces.len());
    for _ in 0..100 {
        assert_eq!(graph.propagate(FaceId(0)).unwrap(), expected);
    }
}

#[test]
fn large_shell_identity_is_independent_of_edge_use_storage_order() {
    let original = ring_graph(2_000);
    let mut reordered = original.clone();
    reordered.edge_uses.reverse();

    assert_eq!(original.snapshot_id(), reordered.snapshot_id());
    assert_eq!(original, ring_graph(2_000));
}

#[test]
fn large_invalid_shell_fails_deterministically_without_mutation() {
    let mut graph = ring_graph(2_000);
    graph.edge_uses.pop();
    let before = graph.clone();
    let expected = graph.validate();

    for _ in 0..100 {
        assert_eq!(graph.validate(), expected);
    }

    assert_eq!(graph, before);
}
