//! Deterministic bounding-volume hierarchies for mathematical broad phase.
//!
//! BVH traversal is a computational aid only. Leaf identities come from caller
//! supplied semantic IDs; no traversal order becomes topology. Queries may
//! produce false positives but, for finite valid boxes, never omit overlapping
//! leaf boxes.

use super::vec::Vec3;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Aabb3 {
    pub min: Vec3,
    pub max: Vec3,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BvhItemId(pub usize);

#[derive(Clone, Debug, PartialEq)]
pub struct BvhLeaf {
    pub index: usize,
    pub bounds: Aabb3,
}

#[derive(Clone, Debug, PartialEq)]
pub enum BvhNode {
    Leaf { item: BvhLeaf },
    Branch { bounds: Aabb3, left: usize, right: usize },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Bvh3 {
    pub nodes: Vec<BvhNode>,
    pub root: Option<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BvhError {
    NonFinite,
    InvalidBounds,
    Empty,
    DuplicateItemId,
}

impl Aabb3 {
    pub fn new(min: Vec3, max: Vec3) -> Result<Self, BvhError> {
        if !min.is_finite() || !max.is_finite() {
            return Err(BvhError::NonFinite);
        }
        if min.x > max.x || min.y > max.y || min.z > max.z {
            return Err(BvhError::InvalidBounds);
        }
        Ok(Self { min, max })
    }

    pub fn from_points(points: &[Vec3]) -> Result<Self, BvhError> {
        if points.is_empty() {
            return Err(BvhError::Empty);
        }
        if points.iter().any(|p| !p.is_finite()) {
            return Err(BvhError::NonFinite);
        }
        let mut min = points[0];
        let mut max = points[0];
        for p in &points[1..] {
            min.x = min.x.min(p.x);
            min.y = min.y.min(p.y);
            min.z = min.z.min(p.z);
            max.x = max.x.max(p.x);
            max.y = max.y.max(p.y);
            max.z = max.z.max(p.z);
        }
        Ok(Self { min, max })
    }

    pub fn union(self, other: Self) -> Self {
        Self {
            min: Vec3::new(
                self.min.x.min(other.min.x),
                self.min.y.min(other.min.y),
                self.min.z.min(other.min.z),
            ),
            max: Vec3::new(
                self.max.x.max(other.max.x),
                self.max.y.max(other.max.y),
                self.max.z.max(other.max.z),
            ),
        }
    }

    /// Numerically stable coordinate-wise midpoint. Halving before addition
    /// prevents overflow for large finite bounds, including `MAX..MAX` boxes.
    pub fn center(self) -> Vec3 {
        Vec3::new(
            0.5 * self.min.x + 0.5 * self.max.x,
            0.5 * self.min.y + 0.5 * self.max.y,
            0.5 * self.min.z + 0.5 * self.max.z,
        )
    }

    pub fn diagonal(self) -> f64 {
        self.max.sub(self.min).length()
    }

    pub fn intersects(self, other: Self, tolerance: f64) -> bool {
        if !tolerance.is_finite() || tolerance < 0.0 {
            return false;
        }
        axis_overlap(self.min.x, self.max.x, other.min.x, other.max.x, tolerance)
            && axis_overlap(self.min.y, self.max.y, other.min.y, other.max.y, tolerance)
            && axis_overlap(self.min.z, self.max.z, other.min.z, other.max.z, tolerance)
    }
}

fn axis_overlap(a_min: f64, a_max: f64, b_min: f64, b_max: f64, tolerance: f64) -> bool {
    // Avoid `bound + tolerance`: that can overflow to `+∞` and manufacture a
    // false overlap for otherwise well-separated finite boxes. Comparing the
    // gap directly preserves the intended closed interval expansion.
    within_gap(a_min, b_max, tolerance) && within_gap(b_min, a_max, tolerance)
}

fn within_gap(lower: f64, upper: f64, tolerance: f64) -> bool {
    if lower <= upper {
        return true;
    }
    let gap = lower - upper;
    gap.is_finite() && gap <= tolerance
}

impl Bvh3 {
    pub fn build(items: &[(usize, Aabb3)]) -> Result<Self, BvhError> {
        if items.is_empty() {
            return Err(BvhError::Empty);
        }
        let mut ids = items.iter().map(|(id, _)| *id).collect::<Vec<_>>();
        ids.sort_unstable();
        if ids.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(BvhError::DuplicateItemId);
        }
        for (_, bounds) in items {
            if !bounds.min.is_finite()
                || !bounds.max.is_finite()
                || bounds.min.x > bounds.max.x
                || bounds.min.y > bounds.max.y
                || bounds.min.z > bounds.max.z
            {
                return Err(BvhError::InvalidBounds);
            }
            let center = bounds.center();
            if !center.is_finite() {
                return Err(BvhError::NonFinite);
            }
        }
        let mut leaves = items
            .iter()
            .map(|(index, bounds)| BvhLeaf {
                index: *index,
                bounds: *bounds,
            })
            .collect::<Vec<_>>();
        let mut nodes = Vec::with_capacity(items.len() * 2);
        let root = Self::build_recursive(&mut leaves, &mut nodes);
        Ok(Self {
            nodes,
            root: Some(root),
        })
    }

    fn build_recursive(items: &mut [BvhLeaf], nodes: &mut Vec<BvhNode>) -> usize {
        if items.len() == 1 {
            let index = nodes.len();
            nodes.push(BvhNode::Leaf {
                item: items[0].clone(),
            });
            return index;
        }
        let mut bounds = items[0].bounds;
        for item in &items[1..] {
            bounds = bounds.union(item.bounds);
        }
        let ext = bounds.max.sub(bounds.min);
        let axis = if ext.x >= ext.y && ext.x >= ext.z {
            0
        } else if ext.y >= ext.z {
            1
        } else {
            2
        };
        items.sort_by(|a, b| {
            let ac = a.bounds.center();
            let bc = b.bounds.center();
            let av = match axis {
                0 => ac.x,
                1 => ac.y,
                _ => ac.z,
            };
            let bv = match axis {
                0 => bc.x,
                1 => bc.y,
                _ => bc.z,
            };
            av.total_cmp(&bv).then_with(|| a.index.cmp(&b.index))
        });
        let mid = items.len() / 2;
        let left = Self::build_recursive(&mut items[..mid], nodes);
        let right = Self::build_recursive(&mut items[mid..], nodes);
        let index = nodes.len();
        nodes.push(BvhNode::Branch {
            bounds,
            left,
            right,
        });
        index
    }

    pub fn query_aabb(&self, query: Aabb3, tolerance: f64) -> Vec<usize> {
        let Some(root) = self.root else {
            return Vec::new();
        };
        if !query.min.is_finite()
            || !query.max.is_finite()
            || query.min.x > query.max.x
            || query.min.y > query.max.y
            || query.min.z > query.max.z
            || !tolerance.is_finite()
            || tolerance < 0.0
        {
            return Vec::new();
        }
        let mut out = Vec::new();
        let mut stack = vec![root];
        while let Some(index) = stack.pop() {
            match self.nodes[index] {
                BvhNode::Leaf { item } => {
                    if item.bounds.intersects(query, tolerance) {
                        out.push(item.index);
                    }
                }
                BvhNode::Branch { bounds, left, right } => {
                    if bounds.intersects(query, tolerance) {
                        stack.push(right);
                        stack.push(left);
                    }
                }
            }
        }
        out.sort_unstable();
        out
    }

    pub fn candidate_pairs(&self, tolerance: f64) -> Vec<(usize, usize)> {
        if !tolerance.is_finite() || tolerance < 0.0 {
            return Vec::new();
        }
        let mut out = Vec::new();
        for i in 0..self.nodes.len() {
            if let BvhNode::Leaf { item: a } = self.nodes[i] {
                for j in i + 1..self.nodes.len() {
                    if let BvhNode::Leaf { item: b } = self.nodes[j] {
                        if a.bounds.intersects(b.bounds, tolerance) {
                            let pair = if a.index < b.index {
                                (a.index, b.index)
                            } else {
                                (b.index, a.index)
                            };
                            if !out.contains(&pair) {
                                out.push(pair);
                            }
                        }
                    }
                }
            }
        }
        out.sort_unstable();
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn box3(x: f64, y: f64, z: f64) -> Aabb3 {
        Aabb3::new(
            Vec3::new(x, y, z),
            Vec3::new(x + 1.0, y + 1.0, z + 1.0),
        )
        .unwrap()
    }

    #[test]
    fn query_contains_all_bruteforce_overlaps() {
        let items = vec![
            (0, box3(0.0, 0.0, 0.0)),
            (1, box3(0.5, 0.0, 0.0)),
            (2, box3(3.0, 3.0, 3.0)),
            (3, box3(3.5, 3.0, 3.0)),
        ];
        let bvh = Bvh3::build(&items).unwrap();
        let hits = bvh.query_aabb(box3(0.25, 0.25, 0.25), 1.0e-12);
        assert_eq!(hits, vec![0, 1]);
    }

    #[test]
    fn deterministic_build_has_stable_candidate_pairs() {
        let items = vec![
            (3, box3(0.0, 0.0, 0.0)),
            (1, box3(0.5, 0.0, 0.0)),
            (2, box3(4.0, 0.0, 0.0)),
            (0, box3(4.5, 0.0, 0.0)),
        ];
        let a = Bvh3::build(&items).unwrap().candidate_pairs(1.0e-12);
        let b = Bvh3::build(&items).unwrap().candidate_pairs(1.0e-12);
        assert_eq!(a, b);
        assert_eq!(a, vec![(0, 2), (1, 3)]);
    }

    #[test]
    fn extreme_finite_box_has_finite_center() {
        let bounds = Aabb3::new(
            Vec3::new(f64::MAX, f64::MAX, f64::MAX),
            Vec3::new(f64::MAX, f64::MAX, f64::MAX),
        )
        .unwrap();
        assert!(bounds.center().is_finite());
    }

    #[test]
    fn tolerance_does_not_create_overflow_false_overlap() {
        let a = Aabb3::new(
            Vec3::new(f64::MAX * 0.75, 0.0, 0.0),
            Vec3::new(f64::MAX * 0.75, 1.0, 1.0),
        )
        .unwrap();
        let b = Aabb3::new(
            Vec3::new(-f64::MAX * 0.75, 0.0, 0.0),
            Vec3::new(-f64::MAX * 0.75, 1.0, 1.0),
        )
        .unwrap();
        assert!(!a.intersects(b, f64::MAX));
    }

    #[test]
    fn duplicate_item_ids_are_rejected() {
        let items = vec![(1, box3(0.0, 0.0, 0.0)), (1, box3(1.0, 0.0, 0.0))];
        assert_eq!(Bvh3::build(&items), Err(BvhError::DuplicateItemId));
    }

    #[test]
    fn invalid_box_is_rejected() {
        assert_eq!(
            Aabb3::new(
                Vec3::new(1.0, 0.0, 0.0),
                Vec3::new(0.0, 1.0, 1.0),
            ),
            Err(BvhError::InvalidBounds)
        );
    }
}
