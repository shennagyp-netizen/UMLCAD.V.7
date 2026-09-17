use umlcad_v6_topology_api::{EdgeId, EdgeUse, FaceId, ShellOrientationGraph};

fn closed_three_face_graph() -> ShellOrientationGraph {
    ShellOrientationGraph {
        faces: [FaceId(1), FaceId(2), FaceId(3)].into_iter().collect(),
        edge_uses: vec![
            EdgeUse { edge: EdgeId(1), face: FaceId(1), forward: true },
            EdgeUse { edge: EdgeId(1), face: FaceId(2), forward: false },
            EdgeUse { edge: EdgeId(2), face: FaceId(2), forward: true },
            EdgeUse { edge: EdgeId(2), face: FaceId(3), forward: false },
            EdgeUse { edge: EdgeId(3), face: FaceId(3), forward: true },
            EdgeUse { edge: EdgeId(3), face: FaceId(1), forward: false },
        ],
    }
}

#[test]
fn repeated_validation_is_stable() {
    let graph = closed_three_face_graph();
    for _ in 0..10_000 {
        assert_eq!(graph.validate(), Ok(()));
    }
}

#[test]
fn repeated_orientation_propagation_is_stable() {
    let graph = closed_three_face_graph();
    let expected = graph.propagate(FaceId(1)).unwrap();
    for _ in 0..5_000 {
        assert_eq!(graph.propagate(FaceId(1)).unwrap(), expected);
    }
}

#[test]
fn repeated_invalid_graph_validation_fails_identically() {
    let graph = ShellOrientationGraph {
        faces: [FaceId(1), FaceId(2)].into_iter().collect(),
        edge_uses: vec![EdgeUse {
            edge: EdgeId(7),
            face: FaceId(1),
            forward: true,
        }],
    };
    let expected = graph.validate();
    for _ in 0..5_000 {
        assert_eq!(graph.validate(), expected);
    }
}

#[test]
fn repeated_operations_do_not_mutate_source_graph() {
    let graph = closed_three_face_graph();
    let before = graph.clone();
    for _ in 0..2_000 {
        let _ = graph.validate();
        let _ = graph.propagate(FaceId(2));
        assert_eq!(graph, before);
    }
}
