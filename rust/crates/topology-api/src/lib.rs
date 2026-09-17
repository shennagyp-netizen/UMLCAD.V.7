use std::collections::{BTreeMap, BTreeSet, VecDeque};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FaceId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EdgeId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EdgeUse {
    pub edge: EdgeId,
    pub face: FaceId,
    pub forward: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShellOrientationGraph {
    pub faces: BTreeSet<FaceId>,
    pub edge_uses: Vec<EdgeUse>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BoundarySewingGraph {
    pub faces: BTreeSet<FaceId>,
    pub edge_uses: Vec<EdgeUse>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ShellId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContainmentRelation {
    Disjoint,
    StrictlyContained,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShellContainmentEvidence {
    pub container: ShellId,
    pub contained: ShellId,
    pub relation: ContainmentRelation,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShellContainmentGraph {
    pub shells: BTreeSet<ShellId>,
    pub evidence: Vec<ShellContainmentEvidence>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShellContainmentResult {
    pub parent: BTreeMap<ShellId, ShellId>,
    pub depth: BTreeMap<ShellId, usize>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ContainmentError {
    EmptyGraph,
    UnknownShell(ShellId),
    SelfRelation(ShellId),
    DuplicateRelation { container: ShellId, contained: ShellId },
    ConflictingRelation { left: ShellId, right: ShellId },
    MultipleContainers(ShellId),
    ContainmentCycle(ShellId),
    InvalidStrictContainmentOrder,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrientationResult {
    pub face_flip: BTreeMap<FaceId, bool>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OrientationError {
    EmptyShell,
    FaceWithoutEdge(FaceId),
    UnknownFace(FaceId),
    NonManifoldEdge { edge: EdgeId, uses: usize },
    DisconnectedShell { assigned: usize, total: usize },
    ContradictoryOrientation { edge: EdgeId, face: FaceId },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SewingError {
    EmptyGraph,
    FaceWithoutEdge(FaceId),
    UnknownFace(FaceId),
    NonManifoldEdge { edge: EdgeId, uses: usize },
    BoundaryEdgeNotFound(EdgeId),
    EdgeIsNotBoundary { edge: EdgeId, uses: usize },
    SameEdge(EdgeId),
    DuplicateBoundaryPair(EdgeId),
    ResultNotClosed,
    Orientation(OrientationError),
}

impl ShellOrientationGraph {
    pub fn validate(&self) -> Result<(), OrientationError> {
        if self.faces.is_empty() { return Err(OrientationError::EmptyShell); }
        let mut counts = BTreeMap::<FaceId, usize>::new();
        for edge_use in &self.edge_uses {
            if !self.faces.contains(&edge_use.face) { return Err(OrientationError::UnknownFace(edge_use.face)); }
            *counts.entry(edge_use.face).or_default() += 1;
        }
        for face in &self.faces {
            if counts.get(face).copied().unwrap_or(0) == 0 { return Err(OrientationError::FaceWithoutEdge(*face)); }
        }
        let mut edges = BTreeMap::<EdgeId, usize>::new();
        for edge_use in &self.edge_uses { *edges.entry(edge_use.edge).or_default() += 1; }
        for (edge, uses) in edges {
            if uses != 2 { return Err(OrientationError::NonManifoldEdge { edge, uses }); }
        }
        Ok(())
    }

    pub fn propagate(&self, seed: FaceId) -> Result<OrientationResult, OrientationError> {
        self.validate()?;
        if !self.faces.contains(&seed) { return Err(OrientationError::UnknownFace(seed)); }
        let mut by_edge = BTreeMap::<EdgeId, Vec<EdgeUse>>::new();
        let mut by_face = BTreeMap::<FaceId, Vec<EdgeUse>>::new();
        for edge_use in &self.edge_uses {
            by_edge.entry(edge_use.edge).or_default().push(*edge_use);
            by_face.entry(edge_use.face).or_default().push(*edge_use);
        }
        let mut face_flip = BTreeMap::new();
        let mut queue = VecDeque::new();
        face_flip.insert(seed, false);
        queue.push_back(seed);
        while let Some(face) = queue.pop_front() {
            let current = face_flip[&face];
            for face_use in by_face.get(&face).into_iter().flatten() {
                let uses = &by_edge[&face_use.edge];
                let neighbor = if uses[0].face == face { uses[1] } else { uses[0] };
                let required = current ^ face_use.forward ^ neighbor.forward ^ true;
                match face_flip.get(&neighbor.face).copied() {
                    None => { face_flip.insert(neighbor.face, required); queue.push_back(neighbor.face); }
                    Some(existing) if existing == required => {}
                    Some(_) => return Err(OrientationError::ContradictoryOrientation { edge: face_use.edge, face: neighbor.face }),
                }
            }
        }
        if face_flip.len() != self.faces.len() { return Err(OrientationError::DisconnectedShell { assigned: face_flip.len(), total: self.faces.len() }); }
        Ok(OrientationResult { face_flip })
    }
}

impl BoundarySewingGraph {
    pub fn validate(&self) -> Result<(), SewingError> {
        if self.faces.is_empty() { return Err(SewingError::EmptyGraph); }
        let mut face_counts = BTreeMap::<FaceId, usize>::new();
        let mut edge_counts = BTreeMap::<EdgeId, usize>::new();
        for edge_use in &self.edge_uses {
            if !self.faces.contains(&edge_use.face) { return Err(SewingError::UnknownFace(edge_use.face)); }
            *face_counts.entry(edge_use.face).or_default() += 1;
            let uses = edge_counts.entry(edge_use.edge).or_default();
            *uses += 1;
            if *uses > 2 { return Err(SewingError::NonManifoldEdge { edge: edge_use.edge, uses: *uses }); }
        }
        for face in &self.faces {
            if face_counts.get(face).copied().unwrap_or(0) == 0 { return Err(SewingError::FaceWithoutEdge(*face)); }
        }
        Ok(())
    }

    pub fn sew_boundary_edges(&self, pairs: &[(EdgeId, EdgeId)]) -> Result<ShellOrientationGraph, SewingError> {
        self.validate()?;
        let mut edge_counts = BTreeMap::<EdgeId, usize>::new();
        for edge_use in &self.edge_uses { *edge_counts.entry(edge_use.edge).or_default() += 1; }
        let mut paired = BTreeSet::<EdgeId>::new();
        let mut replacements = BTreeMap::<EdgeId, EdgeId>::new();
        for &(left, right) in pairs {
            if left == right { return Err(SewingError::SameEdge(left)); }
            for edge in [left, right] {
                match edge_counts.get(&edge).copied() {
                    None => return Err(SewingError::BoundaryEdgeNotFound(edge)),
                    Some(1) => {}
                    Some(uses) => return Err(SewingError::EdgeIsNotBoundary { edge, uses }),
                }
                if !paired.insert(edge) { return Err(SewingError::DuplicateBoundaryPair(edge)); }
            }
            replacements.insert(right, left);
        }
        let mut sewn_uses = Vec::with_capacity(self.edge_uses.len());
        for edge_use in &self.edge_uses {
            let edge = replacements.get(&edge_use.edge).copied().unwrap_or(edge_use.edge);
            sewn_uses.push(EdgeUse { edge, ..*edge_use });
        }
        let result = ShellOrientationGraph { faces: self.faces.clone(), edge_uses: sewn_uses };
        result.validate().map_err(|error| match error {
            OrientationError::NonManifoldEdge { edge, uses } => SewingError::NonManifoldEdge { edge, uses },
            _ => SewingError::ResultNotClosed,
        })?;
        Ok(result)
    }

    pub fn sew_and_orient(&self, pairs: &[(EdgeId, EdgeId)], seed: FaceId) -> Result<OrientationResult, SewingError> {
        let sewn = self.sew_boundary_edges(pairs)?;
        sewn.propagate(seed).map_err(SewingError::Orientation)
    }
}

#[allow(clippy::collapsible_if)]
impl ShellContainmentGraph {
    pub fn validate(&self) -> Result<(), ContainmentError> {
        if self.shells.is_empty() { return Err(ContainmentError::EmptyGraph); }
        let mut relations = BTreeSet::<(ShellId, ShellId)>::new();
        for item in &self.evidence {
            if !self.shells.contains(&item.container) { return Err(ContainmentError::UnknownShell(item.container)); }
            if !self.shells.contains(&item.contained) { return Err(ContainmentError::UnknownShell(item.contained)); }
            if item.container == item.contained { return Err(ContainmentError::SelfRelation(item.container)); }
            let key = (item.container, item.contained);
            if !relations.insert(key) { return Err(ContainmentError::DuplicateRelation { container: item.container, contained: item.contained }); }
        }
        for a in &self.shells {
            for b in &self.shells {
                if a == b { continue; }
                let ab = self.evidence.iter().any(|e| e.container == *a && e.contained == *b && e.relation == ContainmentRelation::StrictlyContained);
                let ba = self.evidence.iter().any(|e| e.container == *b && e.contained == *a && e.relation == ContainmentRelation::StrictlyContained);
                if ab && ba { return Err(ContainmentError::ConflictingRelation { left: *a, right: *b }); }
            }
        }
        Ok(())
    }

    pub fn classify(&self) -> Result<ShellContainmentResult, ContainmentError> {
        self.validate()?;
        let mut parent = BTreeMap::new();
        for evidence in &self.evidence {
            if evidence.relation == ContainmentRelation::StrictlyContained {
                if parent.insert(evidence.contained, evidence.container).is_some() {
                    return Err(ContainmentError::MultipleContainers(evidence.contained));
                }
            }
        }
        let mut depth = BTreeMap::new();
        for shell in &self.shells {
            let mut current = *shell;
            let mut d = 0usize;
            let mut seen = BTreeSet::new();
            while let Some(next) = parent.get(&current).copied() {
                if !seen.insert(current) { return Err(ContainmentError::ContainmentCycle(current)); }
                d += 1;
                current = next;
                if d > self.shells.len() { return Err(ContainmentError::ContainmentCycle(current)); }
            }
            depth.insert(*shell, d);
        }
        Ok(ShellContainmentResult { parent, depth })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn cube_graph() -> ShellOrientationGraph {
        let faces = (0..6).map(FaceId).collect();
        let edge_uses = vec![
            EdgeUse { edge: EdgeId(0), face: FaceId(0), forward: true }, EdgeUse { edge: EdgeId(0), face: FaceId(1), forward: false },
            EdgeUse { edge: EdgeId(1), face: FaceId(0), forward: true }, EdgeUse { edge: EdgeId(1), face: FaceId(2), forward: false },
            EdgeUse { edge: EdgeId(2), face: FaceId(0), forward: true }, EdgeUse { edge: EdgeId(2), face: FaceId(3), forward: false },
            EdgeUse { edge: EdgeId(3), face: FaceId(0), forward: true }, EdgeUse { edge: EdgeId(3), face: FaceId(4), forward: false },
            EdgeUse { edge: EdgeId(4), face: FaceId(1), forward: true }, EdgeUse { edge: EdgeId(4), face: FaceId(2), forward: false },
            EdgeUse { edge: EdgeId(5), face: FaceId(1), forward: true }, EdgeUse { edge: EdgeId(5), face: FaceId(4), forward: false },
            EdgeUse { edge: EdgeId(6), face: FaceId(2), forward: true }, EdgeUse { edge: EdgeId(6), face: FaceId(3), forward: false },
            EdgeUse { edge: EdgeId(7), face: FaceId(2), forward: true }, EdgeUse { edge: EdgeId(7), face: FaceId(4), forward: false },
            EdgeUse { edge: EdgeId(8), face: FaceId(3), forward: true }, EdgeUse { edge: EdgeId(8), face: FaceId(4), forward: false },
            EdgeUse { edge: EdgeId(9), face: FaceId(3), forward: true }, EdgeUse { edge: EdgeId(9), face: FaceId(5), forward: false },
            EdgeUse { edge: EdgeId(10), face: FaceId(4), forward: true }, EdgeUse { edge: EdgeId(10), face: FaceId(5), forward: false },
            EdgeUse { edge: EdgeId(11), face: FaceId(5), forward: true }, EdgeUse { edge: EdgeId(11), face: FaceId(1), forward: false },
        ];
        ShellOrientationGraph { faces, edge_uses }
    }
    fn two_open_tetrahedra() -> BoundarySewingGraph {
        let faces = (0..6).map(FaceId).collect();
        let edge_uses = vec![
            EdgeUse { edge: EdgeId(0), face: FaceId(0), forward: true }, EdgeUse { edge: EdgeId(0), face: FaceId(1), forward: false },
            EdgeUse { edge: EdgeId(1), face: FaceId(0), forward: true }, EdgeUse { edge: EdgeId(1), face: FaceId(2), forward: false },
            EdgeUse { edge: EdgeId(2), face: FaceId(1), forward: true }, EdgeUse { edge: EdgeId(2), face: FaceId(2), forward: false },
            EdgeUse { edge: EdgeId(3), face: FaceId(0), forward: true }, EdgeUse { edge: EdgeId(4), face: FaceId(1), forward: true }, EdgeUse { edge: EdgeId(5), face: FaceId(2), forward: true },
            EdgeUse { edge: EdgeId(6), face: FaceId(3), forward: true }, EdgeUse { edge: EdgeId(6), face: FaceId(4), forward: false },
            EdgeUse { edge: EdgeId(7), face: FaceId(3), forward: true }, EdgeUse { edge: EdgeId(7), face: FaceId(5), forward: false },
            EdgeUse { edge: EdgeId(8), face: FaceId(4), forward: true }, EdgeUse { edge: EdgeId(8), face: FaceId(5), forward: false },
            EdgeUse { edge: EdgeId(9), face: FaceId(3), forward: false }, EdgeUse { edge: EdgeId(10), face: FaceId(4), forward: false }, EdgeUse { edge: EdgeId(11), face: FaceId(5), forward: false },
        ];
        BoundarySewingGraph { faces, edge_uses }
    }
    #[test] fn validates_and_orients_connected_closed_shell() { let graph = cube_graph(); assert!(graph.validate().is_ok()); let result = graph.propagate(FaceId(0)).unwrap(); assert_eq!(result.face_flip.len(), 6); assert!(!result.face_flip[&FaceId(0)]); }
    #[test] fn propagation_is_deterministic() { let graph = cube_graph(); assert_eq!(graph.propagate(FaceId(0)).unwrap(), graph.propagate(FaceId(0)).unwrap()); }
    #[test] fn rejects_non_manifold_edges() { let mut graph = cube_graph(); graph.edge_uses.push(EdgeUse { edge: EdgeId(0), face: FaceId(2), forward: true }); assert_eq!(graph.validate(), Err(OrientationError::NonManifoldEdge { edge: EdgeId(0), uses: 3 })); }
    #[test] fn rejects_disconnected_shells() { let mut graph = cube_graph(); graph.faces.insert(FaceId(6)); graph.faces.insert(FaceId(7)); graph.edge_uses.extend([EdgeUse { edge: EdgeId(12), face: FaceId(6), forward: true }, EdgeUse { edge: EdgeId(12), face: FaceId(7), forward: false }, EdgeUse { edge: EdgeId(13), face: FaceId(6), forward: false }, EdgeUse { edge: EdgeId(13), face: FaceId(7), forward: true }]); assert_eq!(graph.propagate(FaceId(0)), Err(OrientationError::DisconnectedShell { assigned: 6, total: 8 })); }
    #[test] fn detects_contradictory_orientation_cycle() { let faces = [FaceId(0), FaceId(1), FaceId(2)].into_iter().collect(); let edge_uses = vec![EdgeUse { edge: EdgeId(0), face: FaceId(0), forward: false }, EdgeUse { edge: EdgeId(0), face: FaceId(1), forward: false }, EdgeUse { edge: EdgeId(1), face: FaceId(1), forward: false }, EdgeUse { edge: EdgeId(1), face: FaceId(2), forward: false }, EdgeUse { edge: EdgeId(2), face: FaceId(2), forward: false }, EdgeUse { edge: EdgeId(2), face: FaceId(0), forward: false }]; let graph = ShellOrientationGraph { faces, edge_uses }; assert!(matches!(graph.propagate(FaceId(0)), Err(OrientationError::ContradictoryOrientation { .. }))); }
    #[test] fn boundary_sewing_closes_and_orients_two_open_shells() { let graph = two_open_tetrahedra(); let sewn = graph.sew_boundary_edges(&[(EdgeId(3), EdgeId(9)), (EdgeId(4), EdgeId(10)), (EdgeId(5), EdgeId(11))]).unwrap(); assert_eq!(sewn.faces.len(), 6); assert!(sewn.validate().is_ok()); let result = sewn.propagate(FaceId(0)).unwrap(); assert_eq!(result.face_flip.len(), 6); }
    #[test] fn sewing_is_deterministic() { let graph = two_open_tetrahedra(); let pairs = [(EdgeId(3), EdgeId(9)), (EdgeId(4), EdgeId(10)), (EdgeId(5), EdgeId(11))]; assert_eq!(graph.sew_and_orient(&pairs, FaceId(0)).unwrap(), graph.sew_and_orient(&pairs, FaceId(0)).unwrap()); }
    #[test] fn sewing_rejects_non_boundary_edge() { let graph = two_open_tetrahedra(); assert_eq!(graph.sew_boundary_edges(&[(EdgeId(0), EdgeId(9))]), Err(SewingError::EdgeIsNotBoundary { edge: EdgeId(0), uses: 2 })); }
    #[test] fn sewing_rejects_reusing_boundary_edge() { let graph = two_open_tetrahedra(); assert_eq!(graph.sew_boundary_edges(&[(EdgeId(3), EdgeId(9)), (EdgeId(3), EdgeId(10))]), Err(SewingError::DuplicateBoundaryPair(EdgeId(3)))); }
    fn containment_graph(evidence: Vec<ShellContainmentEvidence>) -> ShellContainmentGraph { ShellContainmentGraph { shells: (0..3).map(ShellId).collect(), evidence } }
    #[test] fn distinguishes_disjoint_shells_from_nested_shells() { let graph = containment_graph(vec![ShellContainmentEvidence { container: ShellId(0), contained: ShellId(1), relation: ContainmentRelation::Disjoint }]); let result = graph.classify().unwrap(); assert!(result.parent.is_empty()); assert_eq!(result.depth[&ShellId(0)], 0); assert_eq!(result.depth[&ShellId(1)], 0); }
    #[test] fn records_strict_nesting_and_deterministic_depth() { let graph = containment_graph(vec![ShellContainmentEvidence { container: ShellId(0), contained: ShellId(1), relation: ContainmentRelation::StrictlyContained }, ShellContainmentEvidence { container: ShellId(1), contained: ShellId(2), relation: ContainmentRelation::StrictlyContained }]); let result = graph.classify().unwrap(); assert_eq!(result.parent[&ShellId(1)], ShellId(0)); assert_eq!(result.parent[&ShellId(2)], ShellId(1)); assert_eq!(result.depth[&ShellId(0)], 0); assert_eq!(result.depth[&ShellId(1)], 1); assert_eq!(result.depth[&ShellId(2)], 2); }
    #[test] fn rejects_multiple_containers_and_cycles() { let multiple = containment_graph(vec![ShellContainmentEvidence { container: ShellId(0), contained: ShellId(1), relation: ContainmentRelation::StrictlyContained }, ShellContainmentEvidence { container: ShellId(2), contained: ShellId(1), relation: ContainmentRelation::StrictlyContained }]); assert_eq!(multiple.classify(), Err(ContainmentError::MultipleContainers(ShellId(1)))); let cycle = containment_graph(vec![ShellContainmentEvidence { container: ShellId(0), contained: ShellId(1), relation: ContainmentRelation::StrictlyContained }, ShellContainmentEvidence { container: ShellId(1), contained: ShellId(2), relation: ContainmentRelation::StrictlyContained }, ShellContainmentEvidence { container: ShellId(2), contained: ShellId(0), relation: ContainmentRelation::StrictlyContained }]); assert!(matches!(cycle.classify(), Err(ContainmentError::ContainmentCycle(_)))); }
    #[test] fn rejects_unknown_and_self_containment() { let unknown = ShellContainmentGraph { shells: [ShellId(0)].into_iter().collect(), evidence: vec![ShellContainmentEvidence { container: ShellId(0), contained: ShellId(1), relation: ContainmentRelation::StrictlyContained }] }; assert_eq!(unknown.validate(), Err(ContainmentError::UnknownShell(ShellId(1)))); let self_relation = ShellContainmentGraph { shells: [ShellId(0)].into_iter().collect(), evidence: vec![ShellContainmentEvidence { container: ShellId(0), contained: ShellId(0), relation: ContainmentRelation::StrictlyContained }] }; assert_eq!(self_relation.validate(), Err(ContainmentError::SelfRelation(ShellId(0)))); }
}
