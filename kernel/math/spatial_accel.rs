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


#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoundingSphere3 {
    pub center: Vec3,
    pub radius: f64,
}

impl BoundingSphere3 {
    pub fn new(center: Vec3, radius: f64) -> Result<Self, BvhError> {
        if !center.is_finite() || !radius.is_finite() {
            return Err(BvhError::NonFinite);
        }
        if radius < 0.0 {
            return Err(BvhError::InvalidBounds);
        }
        Ok(Self { center, radius })
    }

    pub fn from_aabb(bounds: Aabb3) -> Self {
        let center = bounds.center();
        let radius = 0.5 * bounds.diagonal();
        Self { center, radius }
    }

    pub fn from_points(points: &[Vec3]) -> Result<Self, BvhError> {
        let bounds = Aabb3::from_points(points)?;
        Ok(Self::from_aabb(bounds))
    }

    pub fn contains_point(self, point: Vec3, tolerance: f64) -> bool {
        if !point.is_finite() || !tolerance.is_finite() || tolerance < 0.0 {
            return false;
        }
        let distance = point.sub(self.center).length();
        distance.is_finite() && distance <= self.radius + tolerance
    }

    pub fn intersects(self, other: Self, tolerance: f64) -> bool {
        if !tolerance.is_finite() || tolerance < 0.0 {
            return false;
        }
        let distance = self.center.sub(other.center).length();
        distance.is_finite() && distance <= self.radius + other.radius + tolerance
    }

    pub fn intersects_aabb(self, bounds: Aabb3, tolerance: f64) -> bool {
        if !tolerance.is_finite() || tolerance < 0.0 {
            return false;
        }
        let closest = Vec3::new(
            self.center.x.clamp(bounds.min.x, bounds.max.x),
            self.center.y.clamp(bounds.min.y, bounds.max.y),
            self.center.z.clamp(bounds.min.z, bounds.max.z),
        );
        let distance = self.center.sub(closest).length();
        distance.is_finite() && distance <= self.radius + tolerance
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParameterBounds2 {
    pub min_u: f64,
    pub max_u: f64,
    pub min_v: f64,
    pub max_v: f64,
}

impl ParameterBounds2 {
    pub fn new(min_u: f64, max_u: f64, min_v: f64, max_v: f64) -> Result<Self, BvhError> {
        let values = [min_u, max_u, min_v, max_v];
        if values.iter().any(|value| !value.is_finite()) {
            return Err(BvhError::NonFinite);
        }
        if min_u > max_u || min_v > max_v {
            return Err(BvhError::InvalidBounds);
        }
        Ok(Self { min_u, max_u, min_v, max_v })
    }

    pub fn center(self) -> (f64, f64) {
        (
            0.5 * self.min_u + 0.5 * self.max_u,
            0.5 * self.min_v + 0.5 * self.max_v,
        )
    }

    pub fn subdivide(self) -> [Self; 4] {
        let (mid_u, mid_v) = self.center();
        [
            Self { min_u: self.min_u, max_u: mid_u, min_v: self.min_v, max_v: mid_v },
            Self { min_u: mid_u, max_u: self.max_u, min_v: self.min_v, max_v: mid_v },
            Self { min_u: self.min_u, max_u: mid_u, min_v: mid_v, max_v: self.max_v },
            Self { min_u: mid_u, max_u: self.max_u, min_v: mid_v, max_v: self.max_v },
        ]
    }
}

pub fn subdivide_aabb8(bounds: Aabb3) -> [Aabb3; 8] {
    let center = bounds.center();
    let min = bounds.min;
    let max = bounds.max;
    [
        Aabb3 { min, max: Vec3::new(center.x, center.y, center.z) },
        Aabb3 { min: Vec3::new(center.x, min.y, min.z), max: Vec3::new(max.x, center.y, center.z) },
        Aabb3 { min: Vec3::new(min.x, center.y, min.z), max: Vec3::new(center.x, max.y, center.z) },
        Aabb3 { min: Vec3::new(center.x, center.y, min.z), max: Vec3::new(max.x, max.y, center.z) },
        Aabb3 { min: Vec3::new(min.x, min.y, center.z), max: Vec3::new(center.x, center.y, max.z) },
        Aabb3 { min: Vec3::new(center.x, min.y, center.z), max: Vec3::new(max.x, center.y, max.z) },
        Aabb3 { min: Vec3::new(min.x, center.y, center.z), max: Vec3::new(center.x, max.y, max.z) },
        Aabb3 { min: center, max },
    ]
}


#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpatialSubdivisionItem {
    pub id: usize,
    pub bounds: Aabb3,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SpatialSubdivisionNode {
    pub bounds: Aabb3,
    pub items: Vec<SpatialSubdivisionItem>,
    pub children: Vec<usize>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SpatialSubdivision3 {
    pub nodes: Vec<SpatialSubdivisionNode>,
    pub root: usize,
    pub max_depth: u32,
    pub max_items_per_leaf: usize,
}

fn contains_aabb(container: Aabb3, item: Aabb3) -> bool {
    item.min.x >= container.min.x
        && item.max.x <= container.max.x
        && item.min.y >= container.min.y
        && item.max.y <= container.max.y
        && item.min.z >= container.min.z
        && item.max.z <= container.max.z
}

fn can_subdivide(bounds: Aabb3) -> bool {
    let center = bounds.center();
    center.is_finite()
        && center.x > bounds.min.x && center.x < bounds.max.x
        && center.y > bounds.min.y && center.y < bounds.max.y
        && center.z > bounds.min.z && center.z < bounds.max.z
}

impl SpatialSubdivision3 {
    pub fn build(
        items: &[(usize, Aabb3)],
        max_depth: u32,
        max_items_per_leaf: usize,
    ) -> Result<Self, BvhError> {
        if items.is_empty() {
            return Err(BvhError::Empty);
        }
        if max_items_per_leaf == 0 {
            return Err(BvhError::InvalidBounds);
        }

        let mut ids = items.iter().map(|(id, _)| *id).collect::<Vec<_>>();
        ids.sort_unstable();
        if ids.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(BvhError::DuplicateItemId);
        }

        let mut root_bounds = items[0].1;
        for (id, bounds) in items {
            if !bounds.min.is_finite()
                || !bounds.max.is_finite()
                || bounds.min.x > bounds.max.x
                || bounds.min.y > bounds.max.y
                || bounds.min.z > bounds.max.z
            {
                return Err(BvhError::InvalidBounds);
            }
            let _ = id;
            root_bounds = root_bounds.union(*bounds);
        }

        let mut nodes = Vec::new();
        let root = Self::build_node(
            root_bounds,
            items.to_vec(),
            0,
            max_depth,
            max_items_per_leaf,
            &mut nodes,
        );
        Ok(Self {
            nodes,
            root,
            max_depth,
            max_items_per_leaf,
        })
    }

    fn build_node(
        bounds: Aabb3,
        items: Vec<(usize, Aabb3)>,
        depth: u32,
        max_depth: u32,
        max_items_per_leaf: usize,
        nodes: &mut Vec<SpatialSubdivisionNode>,
    ) -> usize {
        if depth >= max_depth || items.len() <= max_items_per_leaf || !can_subdivide(bounds) {
            let index = nodes.len();
            let mut resident_items = items
                .into_iter()
                .map(|(id, bounds)| SpatialSubdivisionItem { id, bounds })
                .collect::<Vec<_>>();
            resident_items.sort_by_key(|item| item.id);
            nodes.push(SpatialSubdivisionNode {
                bounds,
                items: resident_items,
                children: Vec::new(),
            });
            return index;
        }

        let child_bounds = subdivide_aabb8(bounds);
        let mut child_items: [Vec<(usize, Aabb3)>; 8] = std::array::from_fn(|_| Vec::new());
        let mut resident = Vec::new();

        for (id, item_bounds) in items {
            let mut containing_child = None;
            for (child_index, child_bounds) in child_bounds.iter().enumerate() {
                if contains_aabb(*child_bounds, item_bounds) {
                    if containing_child.is_some() {
                        containing_child = None;
                        break;
                    }
                    containing_child = Some(child_index);
                }
            }
            if let Some(child_index) = containing_child {
                child_items[child_index].push((id, item_bounds));
            } else {
                resident.push((id, item_bounds));
            }
        }

        if child_items.iter().all(|items| items.is_empty()) {
            let index = nodes.len();
            let mut resident_items = resident
                .into_iter()
                .map(|(id, bounds)| SpatialSubdivisionItem { id, bounds })
                .collect::<Vec<_>>();
            resident_items.sort_by_key(|item| item.id);
            nodes.push(SpatialSubdivisionNode {
                bounds,
                items: resident_items,
                children: Vec::new(),
            });
            return index;
        }

        let index = nodes.len();
        let mut resident_items = resident
            .into_iter()
            .map(|(id, bounds)| SpatialSubdivisionItem { id, bounds })
            .collect::<Vec<_>>();
        resident_items.sort_by_key(|item| item.id);
        nodes.push(SpatialSubdivisionNode {
            bounds,
            items: resident_items,
            children: Vec::new(),
        });

        let mut children = Vec::new();
        for child_index in 0..8 {
            if child_items[child_index].is_empty() {
                continue;
            }
            let child = Self::build_node(
                child_bounds[child_index],
                std::mem::take(&mut child_items[child_index]),
                depth + 1,
                max_depth,
                max_items_per_leaf,
                nodes,
            );
            children.push(child);
        }
        nodes[index].children = children;
        index
    }

    pub fn query_aabb(&self, query: Aabb3, tolerance: f64) -> Vec<usize> {
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
        let mut stack = vec![self.root];
        while let Some(index) = stack.pop() {
            let node = &self.nodes[index];
            if !node.bounds.intersects(query, tolerance) {
                continue;
            }
            for item in &node.items {
                if item.bounds.intersects(query, tolerance) {
                    out.push(item.id);
                }
            }
            stack.extend(node.children.iter().copied().rev());
        }
        out.sort_unstable();
        out.dedup();
        out
    }
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

    pub fn bounding_sphere(self) -> BoundingSphere3 {
        BoundingSphere3::from_aabb(self)
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
            match &self.nodes[index] {
                BvhNode::Leaf { item } => {
                    if item.bounds.intersects(query, tolerance) {
                        out.push(item.index);
                    }
                }
                BvhNode::Branch { bounds, left, right } => {
                    if bounds.intersects(query, tolerance) {
                        stack.push(*right);
                        stack.push(*left);
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
            if let BvhNode::Leaf { item: a } = &self.nodes[i] {
                for j in i + 1..self.nodes.len() {
                    if let BvhNode::Leaf { item: b } = &self.nodes[j] {
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
    fn bounding_sphere_is_conservative_for_aabb() {
        let bounds = box3(-2.0, 1.0, 4.0);
        let sphere = bounds.bounding_sphere();
        assert!(sphere.contains_point(bounds.min, 1.0e-12));
        assert!(sphere.contains_point(bounds.max, 1.0e-12));
        assert!(sphere.intersects_aabb(bounds, 0.0));
    }

    #[test]
    fn parameter_bounds_and_spatial_subdivision_are_deterministic() {
        let bounds = ParameterBounds2::new(0.0, 1.0, -2.0, 2.0).unwrap();
        let children = bounds.subdivide();
        assert_eq!(children[0].center(), (0.25, -1.0));
        assert_eq!(children[3].center(), (0.75, 1.0));

        let root = Aabb3::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(2.0, 4.0, 6.0),
        ).unwrap();
        let children = subdivide_aabb8(root);
        assert_eq!(children.len(), 8);
        assert_eq!(children[0].max, Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(children[7].min, Vec3::new(1.0, 2.0, 3.0));
        for child in children {
            assert!(child.min.x <= child.max.x);
            assert!(child.min.y <= child.max.y);
            assert!(child.min.z <= child.max.z);
        }
    }

    #[test]
    fn bounding_sphere_rejects_invalid_values() {
        assert_eq!(
            BoundingSphere3::new(Vec3::new(0.0, 0.0, 0.0), -1.0),
            Err(BvhError::InvalidBounds)
        );
        assert_eq!(
            BoundingSphere3::new(Vec3::new(f64::NAN, 0.0, 0.0), 1.0),
            Err(BvhError::NonFinite)
        );
    }

    #[test]
    fn spatial_subdivision_filters_nonintersecting_resident_items() {
        let items = vec![
            (0, Aabb3::new(
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(0.25, 0.25, 0.25),
            ).unwrap()),
            (1, Aabb3::new(
                Vec3::new(0.75, 0.75, 0.75),
                Vec3::new(1.0, 1.0, 1.0),
            ).unwrap()),
            (2, Aabb3::new(
                Vec3::new(0.25, 0.25, 0.25),
                Vec3::new(0.75, 0.75, 0.75),
            ).unwrap()),
        ];
        let tree = SpatialSubdivision3::build(&items, 6, 1).unwrap();
        assert_eq!(
            tree.query_aabb(
                Aabb3::new(
                    Vec3::new(0.0, 0.0, 0.0),
                    Vec3::new(0.1, 0.1, 0.1),
                ).unwrap(),
                0.0,
            ),
            vec![0]
        );
        assert_eq!(
            tree.query_aabb(
                Aabb3::new(
                    Vec3::new(0.9, 0.9, 0.9),
                    Vec3::new(0.95, 0.95, 0.95),
                ).unwrap(),
                0.0,
            ),
            vec![1]
        );
    }

    #[test]
    fn spatial_subdivision_preserves_cross_boundary_items() {
        let items = vec![
            (0, Aabb3::new(
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(0.25, 0.25, 0.25),
            ).unwrap()),
            (1, Aabb3::new(
                Vec3::new(0.75, 0.75, 0.75),
                Vec3::new(1.0, 1.0, 1.0),
            ).unwrap()),
            (2, Aabb3::new(
                Vec3::new(0.25, 0.25, 0.25),
                Vec3::new(0.75, 0.75, 0.75),
            ).unwrap()),
        ];
        let tree = SpatialSubdivision3::build(&items, 6, 1).unwrap();
        assert!(tree.nodes.len() > 1);
        assert_eq!(tree.query_aabb(
            Aabb3::new(
                Vec3::new(0.2, 0.2, 0.2),
                Vec3::new(0.8, 0.8, 0.8),
            ).unwrap(),
            0.0,
        ), vec![0, 1, 2]);
        let boundary_node = tree.nodes.iter().find(|node| node.items.iter().any(|item| item.id == 2))
            .expect("cross-boundary item remains resident at a parent");
        assert!(boundary_node.children.len() > 0 || boundary_node.items.iter().any(|item| item.id == 2));
    }

    #[test]
    fn spatial_subdivision_is_deterministic() {
        let items = vec![
            (2, box3(2.0, 2.0, 2.0)),
            (0, box3(0.0, 0.0, 0.0)),
            (1, box3(1.0, 1.0, 1.0)),
        ];
        let a = SpatialSubdivision3::build(&items, 4, 1).unwrap();
        let b = SpatialSubdivision3::build(&items, 4, 1).unwrap();
        assert_eq!(a, b);
        assert_eq!(a.query_aabb(
            Aabb3::new(
                Vec3::new(1.1, 1.1, 1.1),
                Vec3::new(1.9, 1.9, 1.9),
            ).unwrap(),
            0.0,
        ), vec![1]);
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