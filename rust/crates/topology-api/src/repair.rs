use crate::{
    pathology::{self, TopologyPathologyReport},
    FaceId, OrientationError, OrientationResult, ShellOrientationGraph,
};
use std::collections::BTreeSet;

/// Result of a bounded orientation-only repair.
///
/// The repair is intentionally restricted to face-use traversal direction.
/// Face and edge identities and incidence are preserved exactly.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrientationRepairResult {
    pub graph: ShellOrientationGraph,
    pub orientation: OrientationResult,
    pub changed_faces: usize,
}

/// Apply deterministic face-orientation propagation as an immutable repair.
///
/// No geometry, topology incidence, edge identity, or face identity is created,
/// deleted, merged, or split. A contradictory orientation cycle remains a hard
/// failure rather than being coerced by arbitrary flipping.
pub fn repair_orientations(
    graph: &ShellOrientationGraph,
    seed: FaceId,
) -> Result<OrientationRepairResult, OrientationError> {
    let orientation = graph.propagate(seed)?;
    let mut repaired = graph.clone();
    let mut changed_faces = 0usize;

    for face in &graph.faces {
        if orientation.face_flip[face] {
            changed_faces += 1;
        }
    }

    for edge_use in &mut repaired.edge_uses {
        if orientation.face_flip[&edge_use.face] {
            edge_use.forward = !edge_use.forward;
        }
    }

    repaired.validate()?;
    Ok(OrientationRepairResult {
        graph: repaired,
        orientation,
        changed_faces,
    })
}

/// A controlled topology repair that removes one explicitly identified face
/// from a non-manifold graph, but only when the complete postcondition is a
/// closed, connected, consistently orientable manifold.
///
/// This function does not infer duplicate faces, use geometry, merge vertices,
/// weld gaps, widen tolerances, or ask a native backend to heal anything. The
/// caller must explicitly identify the face to remove. Any unrelated pathology
/// is a hard failure, and the source graph is never mutated.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FaceRemovalRepairResult {
    pub graph: ShellOrientationGraph,
    pub removed_face: FaceId,
    pub repaired_edges: BTreeSet<crate::EdgeId>,
    pub removed_edge_use_count: usize,
    pub before: TopologyPathologyReport,
    pub after: TopologyPathologyReport,
    pub orientation: OrientationResult,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FaceRemovalRepairError {
    UnknownFace(FaceId),
    FaceWithoutEdge(FaceId),
    UnsupportedPathology,
    NoMultipleIncidence(FaceId),
    IntroducesBoundary,
    RemainsNonManifold,
    DisconnectedAfterRepair,
    ContradictoryOrientation,
}

/// Remove `face` only when the requested repair is fully validated.
///
/// Preconditions are intentionally narrow:
/// - the face must exist and have edge uses;
/// - the input may contain non-manifold edge incidence, but no boundary,
///   missing-face, or unknown-face pathology;
/// - every pre-existing non-manifold edge must be affected by the named face;
/// - at least one affected edge must have more than two uses.
///
/// Postconditions are stronger than the preconditions: the returned graph is
/// audited as a closed connected manifold and orientation propagation succeeds.
/// Otherwise the original graph is left untouched and a deterministic error is
/// returned.
pub fn repair_remove_face_if_closed_manifold(
    graph: &ShellOrientationGraph,
    face: FaceId,
) -> Result<FaceRemovalRepairResult, FaceRemovalRepairError> {
    let before = pathology::audit(graph);

    if !graph.faces.contains(&face) {
        return Err(FaceRemovalRepairError::UnknownFace(face));
    }

    let target_edges = graph
        .edge_uses
        .iter()
        .filter_map(|edge_use| (edge_use.face == face).then_some(edge_use.edge))
        .collect::<BTreeSet<_>>();
    if target_edges.is_empty() {
        return Err(FaceRemovalRepairError::FaceWithoutEdge(face));
    }

    if !before.boundary_edges.is_empty()
        || !before.faces_without_edges.is_empty()
        || !before.unknown_face_uses.is_empty()
    {
        return Err(FaceRemovalRepairError::UnsupportedPathology);
    }

    let affected_non_manifold = before
        .non_manifold_edges
        .keys()
        .filter(|edge| target_edges.contains(edge))
        .copied()
        .collect::<BTreeSet<_>>();

    if affected_non_manifold.is_empty() {
        return Err(FaceRemovalRepairError::NoMultipleIncidence(face));
    }

    if before
        .non_manifold_edges
        .keys()
        .any(|edge| !target_edges.contains(edge))
    {
        return Err(FaceRemovalRepairError::UnsupportedPathology);
    }

    let mut repaired = graph.clone();
    repaired.faces.remove(&face);
    let old_edge_use_count = repaired.edge_uses.len();
    repaired.edge_uses.retain(|edge_use| edge_use.face != face);
    let removed_edge_use_count = old_edge_use_count - repaired.edge_uses.len();

    let after = pathology::audit(&repaired);
    if !after.boundary_edges.is_empty() {
        return Err(FaceRemovalRepairError::IntroducesBoundary);
    }
    if !after.non_manifold_edges.is_empty() {
        return Err(FaceRemovalRepairError::RemainsNonManifold);
    }
    if after.connected_components.len() != 1 {
        return Err(FaceRemovalRepairError::DisconnectedAfterRepair);
    }
    if !after.is_closed_connected_manifold() {
        return Err(FaceRemovalRepairError::UnsupportedPathology);
    }

    let seed = repaired
        .faces
        .iter()
        .next()
        .copied()
        .ok_or(FaceRemovalRepairError::DisconnectedAfterRepair)?;
    let orientation = repaired
        .propagate(seed)
        .map_err(|error| match error {
            OrientationError::ContradictoryOrientation { .. } => {
                FaceRemovalRepairError::ContradictoryOrientation
            }
            OrientationError::DisconnectedShell { .. } => FaceRemovalRepairError::DisconnectedAfterRepair,
            OrientationError::EmptyShell | OrientationError::FaceWithoutEdge(_) => {
                FaceRemovalRepairError::UnsupportedPathology
            }
            _ => FaceRemovalRepairError::UnsupportedPathology,
        })?;

    repaired
        .validate()
        .map_err(|error| match error {
            OrientationError::ContradictoryOrientation { .. } => {
                FaceRemovalRepairError::ContradictoryOrientation
            }
            OrientationError::DisconnectedShell { .. } => FaceRemovalRepairError::DisconnectedAfterRepair,
            _ => FaceRemovalRepairError::RemainsNonManifold,
        })?;

    Ok(FaceRemovalRepairResult {
        graph: repaired,
        removed_face: face,
        repaired_edges: affected_non_manifold,
        removed_edge_use_count,
        before,
        after,
        orientation,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{EdgeId, EdgeUse};
    use std::collections::BTreeSet;

    fn consistent_cube() -> ShellOrientationGraph {
        let faces = (0..6).map(FaceId).collect();
        let edge_uses = vec![
            EdgeUse { edge: EdgeId(0), face: FaceId(0), forward: true },
            EdgeUse { edge: EdgeId(0), face: FaceId(1), forward: false },
            EdgeUse { edge: EdgeId(1), face: FaceId(0), forward: true },
            EdgeUse { edge: EdgeId(1), face: FaceId(2), forward: false },
            EdgeUse { edge: EdgeId(2), face: FaceId(0), forward: true },
            EdgeUse { edge: EdgeId(2), face: FaceId(3), forward: false },
            EdgeUse { edge: EdgeId(3), face: FaceId(0), forward: true },
            EdgeUse { edge: EdgeId(3), face: FaceId(4), forward: false },
            EdgeUse { edge: EdgeId(4), face: FaceId(1), forward: true },
            EdgeUse { edge: EdgeId(4), face: FaceId(2), forward: false },
            EdgeUse { edge: EdgeId(5), face: FaceId(1), forward: true },
            EdgeUse { edge: EdgeId(5), face: FaceId(4), forward: false },
            EdgeUse { edge: EdgeId(6), face: FaceId(2), forward: true },
            EdgeUse { edge: EdgeId(6), face: FaceId(3), forward: false },
            EdgeUse { edge: EdgeId(7), face: FaceId(2), forward: true },
            EdgeUse { edge: EdgeId(7), face: FaceId(4), forward: false },
            EdgeUse { edge: EdgeId(8), face: FaceId(3), forward: true },
            EdgeUse { edge: EdgeId(8), face: FaceId(4), forward: false },
            EdgeUse { edge: EdgeId(9), face: FaceId(3), forward: true },
            EdgeUse { edge: EdgeId(9), face: FaceId(5), forward: false },
            EdgeUse { edge: EdgeId(10), face: FaceId(4), forward: true },
            EdgeUse { edge: EdgeId(10), face: FaceId(5), forward: false },
            EdgeUse { edge: EdgeId(11), face: FaceId(5), forward: true },
            EdgeUse { edge: EdgeId(11), face: FaceId(1), forward: false },
        ];
        ShellOrientationGraph { faces, edge_uses }
    }

    fn duplicate_face_cube() -> ShellOrientationGraph {
        let mut graph = consistent_cube();
        graph.faces.insert(FaceId(6));
        graph.edge_uses.extend([
            EdgeUse { edge: EdgeId(0), face: FaceId(6), forward: true },
            EdgeUse { edge: EdgeId(1), face: FaceId(6), forward: true },
            EdgeUse { edge: EdgeId(2), face: FaceId(6), forward: true },
            EdgeUse { edge: EdgeId(3), face: FaceId(6), forward: true },
        ]);
        graph
    }

    #[test]
    fn repairs_inverted_face_without_changing_incidence() {
        let original = consistent_cube();
        let mut damaged = original.clone();
        for edge_use in &mut damaged.edge_uses {
            if edge_use.face == FaceId(1) {
                edge_use.forward = !edge_use.forward;
            }
        }

        let result = repair_orientations(&damaged, FaceId(0)).unwrap();
        assert_eq!(result.changed_faces, 1);
        assert_eq!(result.graph.faces, damaged.faces);
        assert_eq!(
            result
                .graph
                .edge_uses
                .iter()
                .map(|u| (u.edge, u.face))
                .collect::<BTreeSet<_>>(),
            damaged
                .edge_uses
                .iter()
                .map(|u| (u.edge, u.face))
                .collect::<BTreeSet<_>>()
        );
        assert!(result.graph.propagate(FaceId(0)).is_ok());
    }

    #[test]
    fn already_consistent_graph_requires_no_change() {
        let graph = consistent_cube();
        let result = repair_orientations(&graph, FaceId(0)).unwrap();
        assert_eq!(result.changed_faces, 0);
        assert_eq!(result.graph, graph);
    }

    #[test]
    fn source_graph_remains_unchanged() {
        let graph = consistent_cube();
        let original = graph.clone();
        let _ = repair_orientations(&graph, FaceId(0)).unwrap();
        assert_eq!(graph, original);
    }

    #[test]
    fn contradictory_cycle_is_not_repaired() {
        let faces = [FaceId(0), FaceId(1), FaceId(2)].into_iter().collect();
        let edge_uses = vec![
            EdgeUse { edge: EdgeId(0), face: FaceId(0), forward: false },
            EdgeUse { edge: EdgeId(0), face: FaceId(1), forward: false },
            EdgeUse { edge: EdgeId(1), face: FaceId(1), forward: false },
            EdgeUse { edge: EdgeId(1), face: FaceId(2), forward: false },
            EdgeUse { edge: EdgeId(2), face: FaceId(2), forward: false },
            EdgeUse { edge: EdgeId(2), face: FaceId(0), forward: false },
        ];
        let graph = ShellOrientationGraph { faces, edge_uses };
        assert!(matches!(
            repair_orientations(&graph, FaceId(0)),
            Err(OrientationError::ContradictoryOrientation { .. })
        ));
    }

    #[test]
    fn explicitly_removes_non_manifold_face_and_restores_closed_cube() {
        let graph = duplicate_face_cube();
        let original = graph.clone();
        let result = repair_remove_face_if_closed_manifold(&graph, FaceId(6)).unwrap();

        assert_eq!(result.removed_face, FaceId(6));
        assert_eq!(result.removed_edge_use_count, 4);
        assert_eq!(
            result.repaired_edges,
            BTreeSet::from([EdgeId(0), EdgeId(1), EdgeId(2), EdgeId(3)])
        );
        assert_eq!(result.before.non_manifold_edges.len(), 4);
        assert!(result.before.boundary_edges.is_empty());
        assert!(result.after.is_closed_connected_manifold());
        assert_eq!(result.after.face_count, 6);
        assert_eq!(result.after.edge_count, 12);
        assert_eq!(result.after.edge_use_count, 24);
        assert_eq!(result.graph, consistent_cube());
        assert_eq!(graph, original);
    }

    #[test]
    fn ordinary_face_removal_is_rejected() {
        let graph = consistent_cube();
        assert_eq!(
            repair_remove_face_if_closed_manifold(&graph, FaceId(0)),
            Err(FaceRemovalRepairError::NoMultipleIncidence(FaceId(0)))
        );
    }

    #[test]
    fn unknown_target_face_is_rejected() {
        let graph = duplicate_face_cube();
        assert_eq!(
            repair_remove_face_if_closed_manifold(&graph, FaceId(99)),
            Err(FaceRemovalRepairError::UnknownFace(FaceId(99)))
        );
    }

    #[test]
    fn unrelated_boundary_pathology_is_rejected_without_mutation() {
        let mut graph = duplicate_face_cube();
        graph.edge_uses.push(EdgeUse {
            edge: EdgeId(99),
            face: FaceId(5),
            forward: true,
        });
        let original = graph.clone();
        assert_eq!(
            repair_remove_face_if_closed_manifold(&graph, FaceId(6)),
            Err(FaceRemovalRepairError::UnsupportedPathology)
        );
        assert_eq!(graph, original);
    }

    #[test]
    fn unrelated_non_manifold_pathology_is_rejected() {
        let mut graph = duplicate_face_cube();
        graph.faces.insert(FaceId(7));
        graph.edge_uses.extend([
            EdgeUse { edge: EdgeId(12), face: FaceId(0), forward: true },
            EdgeUse { edge: EdgeId(12), face: FaceId(1), forward: false },
            EdgeUse { edge: EdgeId(12), face: FaceId(7), forward: true },
        ]);
        assert_eq!(
            repair_remove_face_if_closed_manifold(&graph, FaceId(6)),
            Err(FaceRemovalRepairError::UnsupportedPathology)
        );
    }

    #[test]
    fn contradictory_orientation_after_removal_is_rejected() {
        let mut graph = duplicate_face_cube();
        for edge_use in &mut graph.edge_uses {
            if edge_use.face == FaceId(2) && edge_use.edge == EdgeId(4) {
                edge_use.forward = true;
            }
        }
        assert_eq!(
            repair_remove_face_if_closed_manifold(&graph, FaceId(6)),
            Err(FaceRemovalRepairError::ContradictoryOrientation)
        );
    }

    #[test]
    fn controlled_repair_is_deterministic() {
        let graph = duplicate_face_cube();
        let first = repair_remove_face_if_closed_manifold(&graph, FaceId(6)).unwrap();
        let second = repair_remove_face_if_closed_manifold(&graph, FaceId(6)).unwrap();
        assert_eq!(first, second);
    }
}
