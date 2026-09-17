use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::{EdgeId, FaceId, ShellOrientationGraph};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TopologyIncidenceEvidence {
    RepeatedEdgeFaceIncidence,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TopologyPathologyReport {
    pub face_count: usize,
    pub edge_count: usize,
    pub edge_use_count: usize,
    pub boundary_edges: BTreeSet<EdgeId>,
    pub non_manifold_edges: BTreeMap<EdgeId, usize>,
    pub repeated_edge_face_incidence: BTreeMap<EdgeId, BTreeSet<TopologyIncidenceEvidence>>,
    pub faces_without_edges: BTreeSet<FaceId>,
    pub unknown_face_uses: BTreeSet<FaceId>,
    pub connected_components: Vec<BTreeSet<FaceId>>,
}

impl TopologyPathologyReport {
    pub fn is_manifold(&self) -> bool {
        self.non_manifold_edges.is_empty()
    }

    pub fn is_closed(&self) -> bool {
        self.is_manifold()
            && self.boundary_edges.is_empty()
            && self.faces_without_edges.is_empty()
            && self.unknown_face_uses.is_empty()
    }

    pub fn is_connected(&self) -> bool {
        self.connected_components.len() == 1
    }

    pub fn is_closed_connected_manifold(&self) -> bool {
        self.is_closed() && self.is_connected()
    }
}

pub fn audit(graph: &ShellOrientationGraph) -> TopologyPathologyReport {
    let mut edge_uses = BTreeMap::<EdgeId, Vec<FaceId>>::new();
    let mut face_use_counts = BTreeMap::<FaceId, usize>::new();
    let mut unknown_face_uses = BTreeSet::new();

    for edge_use in &graph.edge_uses {
        edge_uses.entry(edge_use.edge).or_default().push(edge_use.face);
        if graph.faces.contains(&edge_use.face) {
            *face_use_counts.entry(edge_use.face).or_default() += 1;
        } else {
            unknown_face_uses.insert(edge_use.face);
        }
    }

    let boundary_edges = edge_uses
        .iter()
        .filter_map(|(edge, uses)| (uses.len() == 1).then_some(*edge))
        .collect::<BTreeSet<_>>();
    let non_manifold_edges = edge_uses
        .iter()
        .filter_map(|(edge, uses)| (uses.len() > 2).then_some((*edge, uses.len())))
        .collect::<BTreeMap<_, _>>();

    let mut repeated_edge_face_incidence = BTreeMap::<
        EdgeId,
        BTreeSet<TopologyIncidenceEvidence>,
    >::new();
    for (edge, uses) in &edge_uses {
        let mut seen_faces = BTreeSet::<FaceId>::new();
        for face in uses {
            if !seen_faces.insert(*face) {
                repeated_edge_face_incidence
                    .entry(*edge)
                    .or_default()
                    .insert(TopologyIncidenceEvidence::RepeatedEdgeFaceIncidence);
            }
        }
    }

    let faces_without_edges = graph
        .faces
        .iter()
        .filter_map(|face| (!face_use_counts.contains_key(face)).then_some(*face))
        .collect::<BTreeSet<_>>();

    let mut adjacency = BTreeMap::<FaceId, BTreeSet<FaceId>>::new();
    for face in &graph.faces {
        adjacency.entry(*face).or_default();
    }
    for uses in edge_uses.values() {
        let known_faces = uses
            .iter()
            .filter(|face| graph.faces.contains(face))
            .copied()
            .collect::<BTreeSet<_>>();
        for face in &known_faces {
            for neighbor in &known_faces {
                if face != neighbor {
                    adjacency.entry(*face).or_default().insert(*neighbor);
                }
            }
        }
    }

    let mut remaining = graph.faces.clone();
    let mut connected_components = Vec::new();
    while let Some(seed) = remaining.iter().next().copied() {
        let mut queue = VecDeque::new();
        let mut component = BTreeSet::new();
        remaining.remove(&seed);
        queue.push_back(seed);

        while let Some(face) = queue.pop_front() {
            component.insert(face);
            if let Some(neighbors) = adjacency.get(&face) {
                for neighbor in neighbors {
                    if remaining.remove(neighbor) {
                        queue.push_back(*neighbor);
                    }
                }
            }
        }
        connected_components.push(component);
    }

    TopologyPathologyReport {
        face_count: graph.faces.len(),
        edge_count: edge_uses.len(),
        edge_use_count: graph.edge_uses.len(),
        boundary_edges,
        non_manifold_edges,
        repeated_edge_face_incidence,
        faces_without_edges,
        unknown_face_uses,
        connected_components,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EdgeUse;

    fn cube_graph() -> ShellOrientationGraph {
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

    #[test]
    fn closed_cube_is_one_closed_connected_manifold() {
        let report = audit(&cube_graph());
        assert_eq!(report.face_count, 6);
        assert_eq!(report.edge_count, 12);
        assert_eq!(report.edge_use_count, 24);
        assert!(report.boundary_edges.is_empty());
        assert!(report.non_manifold_edges.is_empty());
        assert!(report.repeated_edge_face_incidence.is_empty());
        assert!(report.faces_without_edges.is_empty());
        assert!(report.unknown_face_uses.is_empty());
        assert_eq!(report.connected_components.len(), 1);
        assert!(report.is_closed_connected_manifold());
    }

    #[test]
    fn repeated_edge_face_incidence_is_reported_as_evidence() {
        let mut graph = cube_graph();
        graph.edge_uses.push(EdgeUse { edge: EdgeId(0), face: FaceId(0), forward: false });
        let report = audit(&graph);
        assert!(report.repeated_edge_face_incidence[&EdgeId(0)]
            .contains(&TopologyIncidenceEvidence::RepeatedEdgeFaceIncidence));
        assert!(!report.is_manifold());
    }

    #[test]
    fn same_face_two_use_is_not_classified_as_invalid_manifold_topology() {
        let mut graph = cube_graph();
        graph.edge_uses.retain(|use_| use_.edge != EdgeId(0));
        graph.edge_uses.extend([
            EdgeUse { edge: EdgeId(0), face: FaceId(0), forward: true },
            EdgeUse { edge: EdgeId(0), face: FaceId(0), forward: false },
        ]);
        let report = audit(&graph);
        assert!(report.repeated_edge_face_incidence[&EdgeId(0)]
            .contains(&TopologyIncidenceEvidence::RepeatedEdgeFaceIncidence));
        assert!(report.is_manifold());
    }

    #[test]
    fn open_manifold_reports_boundary_edges_and_multiple_components() {
        let mut graph = cube_graph();
        graph.faces.insert(FaceId(6));
        graph.faces.insert(FaceId(7));
        graph.edge_uses.extend([
            EdgeUse { edge: EdgeId(12), face: FaceId(6), forward: true },
            EdgeUse { edge: EdgeId(13), face: FaceId(7), forward: true },
        ]);
        let report = audit(&graph);
        assert_eq!(report.boundary_edges, BTreeSet::from([EdgeId(12), EdgeId(13)]));
        assert!(report.is_manifold());
        assert!(!report.is_closed());
        assert_eq!(report.connected_components.len(), 3);
    }

    #[test]
    fn non_manifold_edge_is_reported_without_repair() {
        let mut graph = cube_graph();
        graph.edge_uses.push(EdgeUse { edge: EdgeId(0), face: FaceId(2), forward: true });
        let report = audit(&graph);
        assert_eq!(report.non_manifold_edges.get(&EdgeId(0)), Some(&3));
        assert!(!report.is_manifold());
    }

    #[test]
    fn missing_and_unknown_face_references_are_distinct_pathologies() {
        let mut graph = cube_graph();
        graph.faces.insert(FaceId(6));
        graph.edge_uses.push(EdgeUse { edge: EdgeId(12), face: FaceId(99), forward: true });
        let report = audit(&graph);
        assert!(report.faces_without_edges.contains(&FaceId(6)));
        assert!(report.unknown_face_uses.contains(&FaceId(99)));
    }

    #[test]
    fn audit_is_deterministic_and_non_mutating() {
        let graph = cube_graph();
        let original = graph.clone();
        assert_eq!(audit(&graph), audit(&graph));
        assert_eq!(graph, original);
    }
}
