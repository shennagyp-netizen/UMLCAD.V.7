use crate::{EdgeUse, ShellOrientationGraph};
use sha2::{Digest, Sha256};
use std::fmt::Write as _;

/// Stable semantic identity of a topology snapshot.
///
/// The identity is derived only from the backend-neutral topology graph. It is
/// independent of native-kernel handles, allocation order, serialization,
/// rendering, or process state.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TopologySnapshotId(pub [u8; 32]);

impl TopologySnapshotId {
    pub fn bytes(self) -> [u8; 32] {
        self.0
    }

    pub fn hex(self) -> String {
        let mut result = String::with_capacity(64);
        for byte in self.0 {
            write!(&mut result, "{byte:02x}").expect("writing to String cannot fail");
        }
        result
    }
}

impl ShellOrientationGraph {
    /// Compute a canonical SHA-256 identity for this topology snapshot.
    ///
    /// Face identifiers are already canonically ordered by `BTreeSet`. Edge
    /// uses are copied and sorted by semantic tuple `(edge, face, forward)` so
    /// storage/insertion order does not affect the identity while duplicate
    /// use records remain represented exactly.
    pub fn snapshot_id(&self) -> TopologySnapshotId {
        let mut hasher = Sha256::new();
        hasher.update(b"UMLCAD.V6.SHELL_ORIENTATION_GRAPH\0");
        hasher.update((self.faces.len() as u64).to_le_bytes());
        for face in &self.faces {
            hasher.update(face.0.to_le_bytes());
        }

        let mut uses = self.edge_uses.clone();
        uses.sort_by_key(|use_record: &EdgeUse| {
            (use_record.edge.0, use_record.face.0, u8::from(use_record.forward))
        });
        hasher.update((uses.len() as u64).to_le_bytes());
        for use_record in uses {
            hasher.update(use_record.edge.0.to_le_bytes());
            hasher.update(use_record.face.0.to_le_bytes());
            hasher.update([u8::from(use_record.forward)]);
        }

        TopologySnapshotId(hasher.finalize().into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{EdgeId, FaceId};

    fn cube() -> ShellOrientationGraph {
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
    fn identical_snapshots_have_identical_ids() {
        assert_eq!(cube().snapshot_id(), cube().snapshot_id());
    }

    #[test]
    fn insertion_order_does_not_change_semantic_identity() {
        let original = cube();
        let mut reordered = original.clone();
        reordered.edge_uses.reverse();
        assert_eq!(original.snapshot_id(), reordered.snapshot_id());
    }

    #[test]
    fn semantic_change_changes_snapshot_identity() {
        let original = cube();
        let mut changed = original.clone();
        changed.edge_uses[0].forward = !changed.edge_uses[0].forward;
        assert_ne!(original.snapshot_id(), changed.snapshot_id());
    }

    #[test]
    fn duplicate_use_records_remain_identity_relevant() {
        let original = cube();
        let mut changed = original.clone();
        changed.edge_uses.push(changed.edge_uses[0]);
        assert_ne!(original.snapshot_id(), changed.snapshot_id());
    }

    #[test]
    fn hex_identity_is_fixed_width_and_byte_exact() {
        let id = cube().snapshot_id();
        assert_eq!(id.hex().len(), 64);
        let mut expected = String::with_capacity(64);
        for byte in id.bytes() {
            write!(&mut expected, "{byte:02x}").expect("writing to String cannot fail");
        }
        assert_eq!(id.hex(), expected);
    }
}
