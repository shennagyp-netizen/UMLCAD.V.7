//! Explicit B-Rep and solid mathematics for the CPU authority.
//!
//! This module operates on authoritative mathematical geometry and explicit
//! topological incidence. It never derives exact topology from tessellation or
//! display meshes. The certified surface domain is convex planar regions; the
//! certified Boolean domain is axis-aligned boxes.

use std::collections::{BTreeMap, BTreeSet};

use thiserror::Error;

use super::{
    tolerance::Tolerance,
    vec::{Vec2, Vec3},
};

#[derive(Error, Clone, Copy, Debug, PartialEq, Eq)]
pub enum BRepError {
    #[error("non-finite B-Rep value")]
    NonFinite,
    #[error("invalid B-Rep tolerance")]
    InvalidTolerance,
    #[error("duplicate B-Rep identifier")]
    DuplicateId,
    #[error("missing B-Rep reference")]
    MissingReference,
    #[error("degenerate B-Rep geometry")]
    Degenerate,
    #[error("wire is not closed or has inconsistent incidence")]
    OpenWire,
    #[error("edge incidence is not manifold")]
    NonManifoldEdge,
    #[error("vertex incidence is not manifold")]
    NonManifoldVertex,
    #[error("inconsistent B-Rep orientation")]
    InconsistentOrientation,
    #[error("invalid planar surface region")]
    InvalidRegion,
    #[error("empty or zero-volume solid")]
    ZeroVolume,
    #[error("solid orientation is not outward")]
    InvalidSolidOrientation,
    #[error("B-Rep numerical result overflowed")]
    Overflow,
    #[error("Boolean operation has no certified result")]
    UnsupportedBoolean,
}

fn validate_tolerance(t: Tolerance) -> Result<(), BRepError> {
    if t.absolute.is_finite() && t.relative.is_finite() && t.absolute >= 0.0 && t.relative >= 0.0 {
        Ok(())
    } else {
        Err(BRepError::InvalidTolerance)
    }
}

fn near(a: Vec3, b: Vec3, t: f64) -> bool {
    a.sub(b).length() <= t
}

fn signed_area2(loop_points: &[Vec2]) -> f64 {
    if loop_points.is_empty() {
        return 0.0;
    }
    // Translate the local calculation to the first vertex to avoid
    // catastrophic cancellation for large absolute UV coordinates.
    let reference = loop_points[0];
    0.5 * loop_points.iter().enumerate().map(|(i, p)| {
        let q = loop_points[(i + 1) % loop_points.len()];
        let a = p.sub(reference);
        let b = q.sub(reference);
        a.cross(b)
    }).sum::<f64>()
}

fn polygon_centroid2(loop_points: &[Vec2], area: f64) -> Result<Vec2, BRepError> {
    if area == 0.0 {
        return Err(BRepError::Degenerate);
    }
    let reference = loop_points[0];
    let mut x = 0.0;
    let mut y = 0.0;
    for i in 0..loop_points.len() {
        let a = loop_points[i].sub(reference);
        let b = loop_points[(i + 1) % loop_points.len()].sub(reference);
        let c = a.cross(b);
        x += (a.x + b.x) * c;
        y += (a.y + b.y) * c;
    }
    let local = Vec2::new(x / (6.0 * area), y / (6.0 * area));
    let result = reference.add(local);
    if result.is_finite() { Ok(result) } else { Err(BRepError::Overflow) }
}

fn loop_boundary_contains(point: Vec2, loop_points: &[Vec2], tolerance: f64) -> bool {
    loop_points.iter().enumerate().any(|(i, a)| {
        let b = loop_points[(i + 1) % loop_points.len()];
        let edge = b.sub(*a);
        let rel = point.sub(*a);
        let cross = edge.cross(rel);
        cross.is_finite() && cross.abs() <= tolerance * (edge.length() + rel.length() + 1.0)
            && point.x >= a.x.min(b.x) - tolerance
            && point.x <= a.x.max(b.x) + tolerance
            && point.y >= a.y.min(b.y) - tolerance
            && point.y <= a.y.max(b.y) + tolerance
    })
}

fn is_convex_loop(loop_points: &[Vec2], tolerance: f64) -> bool {
    if loop_points.len() < 3 {
        return false;
    }
    let area = signed_area2(loop_points);
    if !area.is_finite() || area == 0.0 {
        return false;
    }
    let sign = area.signum();
    for i in 0..loop_points.len() {
        let a = loop_points[i];
        let b = loop_points[(i + 1) % loop_points.len()];
        let d = loop_points[(i + 2) % loop_points.len()];
        let value = b.sub(a).cross(d.sub(b));
        let edge_scale = b.sub(a).length().max(d.sub(b).length()).max(1.0);
        let eps = tolerance * edge_scale;
        if !value.is_finite() || !eps.is_finite() || value * sign <= eps {
            return false;
        }
    }
    true
}

fn point_in_convex_loop(point: Vec2, loop_points: &[Vec2], tolerance: f64) -> bool {
    let area = signed_area2(loop_points);
    let sign = area.signum();
    for i in 0..loop_points.len() {
        let a = loop_points[i];
        let b = loop_points[(i + 1) % loop_points.len()];
        let cross = b.sub(a).cross(point.sub(a));
        let edge_scale = b.sub(a).length().max(point.sub(a).length()).max(1.0);
        let eps = tolerance * edge_scale;
        if sign > 0.0 {
            if cross < -eps { return false; }
        } else if sign < 0.0 {
            if cross > eps { return false; }
        } else {
            return false;
        }
    }
    true
}

fn segment_intersects_2d(a: Vec2, b: Vec2, c: Vec2, d: Vec2, tolerance: f64) -> bool {
    fn orient(a: Vec2, b: Vec2, c: Vec2) -> f64 { b.sub(a).cross(c.sub(a)) }
    fn on_segment(a: Vec2, b: Vec2, p: Vec2, t: f64) -> bool {
        p.x >= a.x.min(b.x) - t && p.x <= a.x.max(b.x) + t
            && p.y >= a.y.min(b.y) - t && p.y <= a.y.max(b.y) + t
    }
    let ab_c = orient(a, b, c);
    let ab_d = orient(a, b, d);
    let cd_a = orient(c, d, a);
    let cd_b = orient(c, d, b);
    let geometric_scale = b.sub(a).length()
        .max(d.sub(c).length())
        .max(a.sub(c).length())
        .max(a.sub(d).length())
        .max(1.0);
    let orient_eps = tolerance * geometric_scale;
    if !orient_eps.is_finite() || [ab_c, ab_d, cd_a, cd_b].iter().any(|v| !v.is_finite()) {
        return false;
    }
    if ab_c.abs() <= orient_eps && on_segment(a, b, c, tolerance) { return true; }
    if ab_d.abs() <= orient_eps && on_segment(a, b, d, tolerance) { return true; }
    if cd_a.abs() <= orient_eps && on_segment(c, d, a, tolerance) { return true; }
    if cd_b.abs() <= orient_eps && on_segment(c, d, b, tolerance) { return true; }
    ((ab_c > orient_eps && ab_d < -orient_eps) || (ab_c < -orient_eps && ab_d > orient_eps))
        && ((cd_a > orient_eps && cd_b < -orient_eps) || (cd_a < -orient_eps && cd_b > orient_eps))
}

#[derive(Clone, Debug, PartialEq)]
pub struct PlanarRegion3 {
    pub origin: Vec3,
    pub u_dir: Vec3,
    pub v_dir: Vec3,
    pub outer: Vec<Vec2>,
    pub holes: Vec<Vec<Vec2>>,
}

impl PlanarRegion3 {
    pub fn validate(&self, tolerance: Tolerance) -> Result<(), BRepError> {
        validate_tolerance(tolerance)?;
        if [self.origin, self.u_dir, self.v_dir].iter().any(|v| !v.is_finite()) {
            return Err(BRepError::NonFinite);
        }
        let un = self.u_dir.length();
        let vn = self.v_dir.length();
        if !un.is_finite() || !vn.is_finite() || un == 0.0 || vn == 0.0 {
            return Err(BRepError::Degenerate);
        }
        let eps = tolerance.threshold(un.max(vn).max(1.0))
            .map_err(|_| BRepError::InvalidTolerance)?;
        if (un - 1.0).abs() > eps || (vn - 1.0).abs() > eps
            || self.u_dir.dot(self.v_dir).abs() > eps {
            return Err(BRepError::InvalidRegion);
        }
        if self.outer.len() < 3 || self.outer.iter().any(|p| !p.is_finite()) {
            return Err(BRepError::InvalidRegion);
        }
        let outer_area = signed_area2(&self.outer);
        if !outer_area.is_finite() || outer_area.abs() <= eps * eps {
            return Err(BRepError::Degenerate);
        }
        if !is_convex_loop(&self.outer, eps) {
            return Err(BRepError::InvalidRegion);
        }
        for i in 0..self.outer.len() {
            let a = self.outer[i];
            let b = self.outer[(i + 1) % self.outer.len()];
            if a.sub(b).length() <= eps {
                return Err(BRepError::Degenerate);
            }
            for j in (i + 1)..self.outer.len() {
                if i == j || (i + 1) % self.outer.len() == j
                    || (j + 1) % self.outer.len() == i {
                    continue;
                }
                if segment_intersects_2d(a, b, self.outer[j], self.outer[(j + 1) % self.outer.len()], eps) {
                    return Err(BRepError::InvalidRegion);
                }
            }
        }

        for hole in &self.holes {
            if hole.len() < 3 || hole.iter().any(|p| !p.is_finite()) {
                return Err(BRepError::InvalidRegion);
            }
            let hole_area = signed_area2(hole);
            if !hole_area.is_finite() || hole_area.abs() <= eps * eps {
                return Err(BRepError::Degenerate);
            }
            if !is_convex_loop(hole, eps) {
                return Err(BRepError::InvalidRegion);
            }
            if !point_in_convex_loop(hole[0], &self.outer, eps) {
                return Err(BRepError::InvalidRegion);
            }
            for i in 0..hole.len() {
                for j in 0..self.outer.len() {
                    if segment_intersects_2d(
                        hole[i], hole[(i + 1) % hole.len()],
                        self.outer[j], self.outer[(j + 1) % self.outer.len()],
                        eps,
                    ) {
                        return Err(BRepError::InvalidRegion);
                    }
                }
            }
        }

        // Hole loops must be pairwise disjoint and non-nested.
        for i in 0..self.holes.len() {
            for j in (i + 1)..self.holes.len() {
                if point_in_convex_loop(self.holes[i][0], &self.holes[j], eps)
                    || point_in_convex_loop(self.holes[j][0], &self.holes[i], eps)
                {
                    return Err(BRepError::InvalidRegion);
                }
                for a in 0..self.holes[i].len() {
                    for b in 0..self.holes[j].len() {
                        if segment_intersects_2d(
                            self.holes[i][a],
                            self.holes[i][(a + 1) % self.holes[i].len()],
                            self.holes[j][b],
                            self.holes[j][(b + 1) % self.holes[j].len()],
                            eps,
                        ) {
                            return Err(BRepError::InvalidRegion);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub fn area(&self, tolerance: Tolerance) -> Result<f64, BRepError> {
        self.validate(tolerance)?;
        let value = signed_area2(&self.outer).abs()
            - self.holes.iter().map(|h| signed_area2(h).abs()).sum::<f64>();
        if value.is_finite() && value > 0.0 { Ok(value) } else { Err(BRepError::Degenerate) }
    }

    pub fn centroid(&self, tolerance: Tolerance) -> Result<Vec3, BRepError> {
        self.validate(tolerance)?;
        let outer_signed = signed_area2(&self.outer);
        let outer = polygon_centroid2(&self.outer, outer_signed)?;
        let outer_area = outer_signed.abs();
        let mut weighted = Vec2::new(outer.x * outer_area, outer.y * outer_area);
        let mut total = outer_area;
        for hole in &self.holes {
            let signed = signed_area2(hole);
            let centroid = polygon_centroid2(hole, signed)?;
            let area = signed.abs();
            weighted = weighted.sub(centroid.scale(area));
            total -= area;
        }
        if !total.is_finite() || total <= 0.0 {
            return Err(BRepError::Degenerate);
        }
        let uv = weighted.scale(1.0 / total);
        let point = self.origin.add(self.u_dir.scale(uv.x)).add(self.v_dir.scale(uv.y));
        if point.is_finite() { Ok(point) } else { Err(BRepError::Overflow) }
    }

    pub fn classify_point(&self, p: Vec3, tolerance: Tolerance) -> Result<RegionClass, BRepError> {
        self.validate(tolerance)?;
        if !p.is_finite() { return Err(BRepError::NonFinite); }
        let d = p.sub(self.origin);
        let normal = self.u_dir.cross(self.v_dir);
        let distance = d.dot(normal);
        if !distance.is_finite() {
            return Err(BRepError::Overflow);
        }
        let scale = d.length().max(1.0);
        let band = tolerance.threshold(scale).map_err(|_| BRepError::InvalidTolerance)?;
        if distance.abs() > band {
            return Ok(RegionClass::Outside);
        }
        let u = d.dot(self.u_dir);
        let v = d.dot(self.v_dir);
        let uv = Vec2::new(u, v);
        if loop_boundary_contains(uv, &self.outer, band) {
            return Ok(RegionClass::OnBoundary);
        }
        if !point_in_convex_loop(uv, &self.outer, band) {
            return Ok(RegionClass::Outside);
        }
        for hole in &self.holes {
            if loop_boundary_contains(uv, hole, band) {
                return Ok(RegionClass::OnBoundary);
            }
        }
        for hole in &self.holes {
            if point_in_convex_loop(uv, hole, band) {
                return Ok(RegionClass::InsideHole);
            }
        }
        Ok(RegionClass::Inside)
    }

    pub fn point_from_uv(&self, uv: Vec2, tolerance: Tolerance) -> Result<Vec3, BRepError> {
        self.validate(tolerance)?;
        if !uv.is_finite() { return Err(BRepError::NonFinite); }
        let point = self.origin.add(self.u_dir.scale(uv.x)).add(self.v_dir.scale(uv.y));
        if point.is_finite() { Ok(point) } else { Err(BRepError::Overflow) }
    }

    fn outer_points_3d(&self, tolerance: Tolerance) -> Result<Vec<Vec3>, BRepError> {
        self.validate(tolerance)?;
        self.outer.iter().map(|uv| self.point_from_uv(*uv, tolerance)).collect()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegionClass {
    Inside,
    InsideHole,
    Outside,
    OnBoundary,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BRepVertex {
    pub id: String,
    pub point: Vec3,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BRepEdge {
    pub id: String,
    pub start_vertex: String,
    pub end_vertex: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BRepCoedge {
    pub id: String,
    pub edge: String,
    pub wire: String,
    pub face: String,
    pub forward: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BRepWire {
    pub id: String,
    pub coedges: Vec<String>,
    pub closed: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BRepFace {
    pub id: String,
    pub outer_wire: String,
    pub inner_wires: Vec<String>,
    pub region: PlanarRegion3,
    pub orientation: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BRepShell {
    pub id: String,
    pub faces: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BRepSolid {
    pub vertices: Vec<BRepVertex>,
    pub edges: Vec<BRepEdge>,
    pub coedges: Vec<BRepCoedge>,
    pub wires: Vec<BRepWire>,
    pub faces: Vec<BRepFace>,
    pub shells: Vec<BRepShell>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SolidMoments {
    pub signed_volume: f64,
    pub volume: f64,
    pub centroid: Vec3,
    pub inertia_origin: [[f64; 3]; 3],
    pub inertia_centroid: [[f64; 3]; 3],
    pub surface_area: f64,
}

fn matrix_zero() -> [[f64; 3]; 3] {
    [[0.0; 3]; 3]
}

fn add3(a: [[f64; 3]; 3], b: [[f64; 3]; 3]) -> [[f64; 3]; 3] {
    let mut out = matrix_zero();
    for i in 0..3 {
        for j in 0..3 {
            out[i][j] = a[i][j] + b[i][j];
        }
    }
    out
}

fn scale3(a: [[f64; 3]; 3], factor: f64) -> [[f64; 3]; 3] {
    let mut out = matrix_zero();
    for i in 0..3 {
        for j in 0..3 {
            out[i][j] = a[i][j] * factor;
        }
    }
    out
}

fn outer_product(a: Vec3, b: Vec3) -> [[f64; 3]; 3] {
    [
        [a.x * b.x, a.x * b.y, a.x * b.z],
        [a.y * b.x, a.y * b.y, a.y * b.z],
        [a.z * b.x, a.z * b.y, a.z * b.z],
    ]
}

fn tetra_raw_second(a: Vec3, b: Vec3, c: Vec3, signed_volume: f64) -> [[f64; 3]; 3] {
    let points = [a, b, c];
    let mut second = matrix_zero();
    for i in 0..3 {
        second = add3(second, scale3(outer_product(points[i], points[i]), signed_volume / 10.0));
        for j in (i + 1)..3 {
            let cross = add3(outer_product(points[i], points[j]), outer_product(points[j], points[i]));
            second = add3(second, scale3(cross, signed_volume / 20.0));
        }
    }
    second
}

fn inertia_from_second(second: [[f64; 3]; 3]) -> [[f64; 3]; 3] {
    [
        [second[1][1] + second[2][2], -second[0][1], -second[0][2]],
        [-second[1][0], second[0][0] + second[2][2], -second[1][2]],
        [-second[2][0], -second[2][1], second[0][0] + second[1][1]],
    ]
}

impl BRepSolid {
    fn validate_structure(&self, tolerance: Tolerance) -> Result<(), BRepError> {
        validate_tolerance(tolerance)?;
        if self.shells.len() != 1 {
            return Err(BRepError::UnsupportedBoolean);
        }
        let vertex_ids = unique_ids(self.vertices.iter().map(|v| v.id.as_str()))?;
        let edge_ids = unique_ids(self.edges.iter().map(|e| e.id.as_str()))?;
        let wire_ids = unique_ids(self.wires.iter().map(|w| w.id.as_str()))?;
        let face_ids = unique_ids(self.faces.iter().map(|f| f.id.as_str()))?;
        let coedge_ids = unique_ids(self.coedges.iter().map(|c| c.id.as_str()))?;

        for vertex in &self.vertices {
            if !vertex_ids.contains(&vertex.id) || !vertex.point.is_finite() {
                return Err(BRepError::NonFinite);
            }
        }
        for edge in &self.edges {
            let a = self.vertices.iter().find(|v| v.id == edge.start_vertex).ok_or(BRepError::MissingReference)?;
            let b = self.vertices.iter().find(|v| v.id == edge.end_vertex).ok_or(BRepError::MissingReference)?;
            if edge.start_vertex == edge.end_vertex || !a.point.is_finite() || !b.point.is_finite() {
                return Err(BRepError::Degenerate);
            }
            let eps = tolerance.threshold(a.point.sub(b.point).length().max(1.0))
                .map_err(|_| BRepError::InvalidTolerance)?;
            if near(a.point, b.point, eps) {
                return Err(BRepError::Degenerate);
            }
        }

        let shell = &self.shells[0];
        if shell.faces.is_empty() || shell.faces.iter().any(|id| !face_ids.contains(id)) {
            return Err(BRepError::MissingReference);
        }

        let mut edge_incidence: BTreeMap<&str, Vec<&BRepCoedge>> = BTreeMap::new();
        for coedge in &self.coedges {
            if !coedge_ids.contains(&coedge.id)
                || !edge_ids.contains(&coedge.edge)
                || !wire_ids.contains(&coedge.wire)
                || !face_ids.contains(&coedge.face)
            {
                return Err(BRepError::MissingReference);
            }
            edge_incidence.entry(coedge.edge.as_str()).or_default().push(coedge);
        }

        for edge in &self.edges {
            let incidence = edge_incidence.get(edge.id.as_str()).cloned().unwrap_or_default();
            if incidence.len() != 2 {
                return Err(BRepError::NonManifoldEdge);
            }
            if incidence[0].face == incidence[1].face {
                return Err(BRepError::NonManifoldEdge);
            }
            if incidence[0].forward == incidence[1].forward {
                return Err(BRepError::InconsistentOrientation);
            }
        }

        // A single shell must be face-connected through shared edges.
        let mut adjacency: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
        for face_id in &shell.faces {
            adjacency.entry(face_id.as_str()).or_default();
        }
        for incidence in edge_incidence.values() {
            if incidence.len() == 2 {
                adjacency.entry(incidence[0].face.as_str()).or_default()
                    .insert(incidence[1].face.as_str());
                adjacency.entry(incidence[1].face.as_str()).or_default()
                    .insert(incidence[0].face.as_str());
            }
        }
        let mut reachable = BTreeSet::new();
        let mut stack = vec![shell.faces[0].as_str()];
        while let Some(face_id) = stack.pop() {
            if reachable.insert(face_id) {
                if let Some(neighbors) = adjacency.get(face_id) {
                    stack.extend(neighbors.iter().copied());
                }
            }
        }
        if reachable.len() != shell.faces.len() {
            return Err(BRepError::NonManifoldEdge);
        }

        for wire in &self.wires {
            if !wire.closed || wire.coedges.len() < 3 {
                return Err(BRepError::OpenWire);
            }
            let wire_coedges = wire.coedges.iter().map(|id| {
                self.coedges.iter().find(|c| c.id == *id).ok_or(BRepError::MissingReference)
            }).collect::<Result<Vec<_>, _>>()?;
            let start_end = |coedge: &BRepCoedge| -> Result<(Vec3, Vec3), BRepError> {
                let edge = self.edges.iter().find(|e| e.id == coedge.edge).ok_or(BRepError::MissingReference)?;
                let a = self.vertices.iter().find(|v| v.id == edge.start_vertex).ok_or(BRepError::MissingReference)?;
                let b = self.vertices.iter().find(|v| v.id == edge.end_vertex).ok_or(BRepError::MissingReference)?;
                Ok(if coedge.forward { (a.point, b.point) } else { (b.point, a.point) })
            };
            for i in 0..wire_coedges.len() {
                if wire_coedges[i].wire != wire.id {
                    return Err(BRepError::InconsistentOrientation);
                }
                if i > 0 {
                    let (_, previous_end) = start_end(wire_coedges[i - 1])?;
                    let (current_start, _) = start_end(wire_coedges[i])?;
                    let eps = tolerance.threshold(previous_end.sub(current_start).length().max(1.0))
                        .map_err(|_| BRepError::InvalidTolerance)?;
                    if !near(previous_end, current_start, eps) {
                        return Err(BRepError::OpenWire);
                    }
                }
            }
            let (_, last_end) = start_end(wire_coedges[wire_coedges.len() - 1])?;
            let (first_start, _) = start_end(wire_coedges[0])?;
            let eps = tolerance.threshold(last_end.sub(first_start).length().max(1.0))
                .map_err(|_| BRepError::InvalidTolerance)?;
            if !near(last_end, first_start, eps) {
                return Err(BRepError::OpenWire);
            }
        }

        for face in &self.faces {
            if !shell.faces.contains(&face.id)
                || !wire_ids.contains(&face.outer_wire)
                || face.inner_wires.iter().any(|id| !wire_ids.contains(id))
            {
                return Err(BRepError::MissingReference);
            }
            face.region.validate(tolerance)?;
            validate_face_region_edge_correspondence(self, face, tolerance)?;
        }

        for vertex in &self.vertices {
            let incident_faces = self.coedges.iter()
                .filter(|c| {
                    let edge = self.edges.iter().find(|e| e.id == c.edge);
                    let Some(edge) = edge else { return false; };
                    edge.start_vertex == vertex.id || edge.end_vertex == vertex.id
                })
                .map(|c| c.face.as_str())
                .collect::<BTreeSet<_>>();
            if incident_faces.len() < 3 {
                return Err(BRepError::NonManifoldVertex);
            }

            // The link of a manifold vertex is one connected cycle. Each
            // incident face contributes exactly two link edges at the vertex.
            let mut link_degree: BTreeMap<&str, usize> =
                incident_faces.iter().copied().map(|face| (face, 0usize)).collect();
            let mut link_adjacency: BTreeMap<&str, BTreeSet<&str>> =
                incident_faces.iter().copied().map(|face| (face, BTreeSet::new())).collect();

            for edge in &self.edges {
                if edge.start_vertex != vertex.id && edge.end_vertex != vertex.id {
                    continue;
                }
                let incidence = edge_incidence.get(edge.id.as_str()).ok_or(BRepError::NonManifoldEdge)?;
                if incidence.len() != 2 || incidence[0].face == incidence[1].face {
                    return Err(BRepError::NonManifoldVertex);
                }
                let f0 = incidence[0].face.as_str();
                let f1 = incidence[1].face.as_str();
                if !incident_faces.contains(f0) || !incident_faces.contains(f1) {
                    return Err(BRepError::NonManifoldVertex);
                }
                *link_degree.get_mut(f0).ok_or(BRepError::NonManifoldVertex)? += 1;
                *link_degree.get_mut(f1).ok_or(BRepError::NonManifoldVertex)? += 1;
                link_adjacency.get_mut(f0).ok_or(BRepError::NonManifoldVertex)?.insert(f1);
                link_adjacency.get_mut(f1).ok_or(BRepError::NonManifoldVertex)?.insert(f0);
            }

            if link_degree.values().any(|degree| *degree != 2) {
                return Err(BRepError::NonManifoldVertex);
            }

            let mut link_reachable = BTreeSet::new();
            let mut link_stack = vec![*incident_faces.iter().next().ok_or(BRepError::NonManifoldVertex)?];
            while let Some(face_id) = link_stack.pop() {
                if link_reachable.insert(face_id) {
                    if let Some(neighbors) = link_adjacency.get(face_id) {
                        link_stack.extend(neighbors.iter().copied());
                    }
                }
            }
            if link_reachable.len() != incident_faces.len() {
                return Err(BRepError::NonManifoldVertex);
            }
        }


        Ok(())
    }

    pub fn validate(&self, tolerance: Tolerance) -> Result<(), BRepError> {
        self.validate_structure(tolerance)?;
        // Exact solid moments below triangulate convex outer face loops. A
        // planar face with an inner loop requires a certified polygon-with-hole
        // decomposition that is not yet part of this authority. Reject it here
        // rather than silently counting the hole as material.
        if self.faces.iter().any(|face| {
            !face.inner_wires.is_empty() || !face.region.holes.is_empty()
        }) {
            return Err(BRepError::UnsupportedBoolean);
        }
        let volume = self.moments(tolerance)?.signed_volume;
        if volume <= 0.0 {
            return Err(BRepError::InvalidSolidOrientation);
        }
        Ok(())
    }

    pub fn surface_area(&self, tolerance: Tolerance) -> Result<f64, BRepError> {
        self.validate_structure(tolerance)?;
        let mut area = 0.0;
        for face in &self.faces {
            area += face.region.area(tolerance)?;
        }
        if area.is_finite() && area > 0.0 { Ok(area) } else { Err(BRepError::Overflow) }
    }

    pub fn classify_point(&self, p: Vec3, direction: Vec3, tolerance: Tolerance)
        -> Result<SolidPointClass, BRepError>
    {
        self.validate(tolerance)?;
        if !p.is_finite() || !direction.is_finite() {
            return Err(BRepError::NonFinite);
        }
        let d = direction.normalized().map_err(|_| BRepError::Degenerate)?;
        let triangles = self.triangles(tolerance)?;
        let boundary_band = tolerance.threshold(self.bbox_extent().max(1.0))
            .map_err(|_| BRepError::InvalidTolerance)?;
        let mut hits = 0usize;
        for triangle in triangles {
            match ray_triangle(p, d, triangle, boundary_band) {
                RayHit::Hit => hits += 1,
                RayHit::Boundary => return Ok(SolidPointClass::OnBoundary),
                RayHit::Miss => {}
                RayHit::Indeterminate => return Ok(SolidPointClass::Indeterminate),
            }
        }
        Ok(if hits % 2 == 1 { SolidPointClass::Inside } else { SolidPointClass::Outside })
    }

    pub fn moments(&self, tolerance: Tolerance) -> Result<SolidMoments, BRepError> {
        self.validate_structure(tolerance)?;
        if self.faces.iter().any(|face| {
            !face.inner_wires.is_empty() || !face.region.holes.is_empty()
        }) {
            return Err(BRepError::UnsupportedBoolean);
        }

        // Evaluate the exact tetrahedral polynomial integrals in a local
        // reference frame. This avoids catastrophic cancellation when the
        // solid is translated far from the world origin.
        let reference = self.vertices.first().ok_or(BRepError::Degenerate)?.point;
        if !reference.is_finite() {
            return Err(BRepError::NonFinite);
        }

        let triangles = self.triangles(tolerance)?;
        let mut signed_volume = 0.0;
        let mut first_local = Vec3::new(0.0, 0.0, 0.0);
        let mut second_local = matrix_zero();
        let mut surface_area = 0.0;

        for triangle in triangles {
            let a = triangle.a.sub(reference);
            let b = triangle.b.sub(reference);
            let c = triangle.c.sub(reference);
            let tetra = a.dot(b.cross(c)) / 6.0;
            if !tetra.is_finite() {
                return Err(BRepError::Overflow);
            }
            signed_volume += tetra;
            first_local = first_local.add(a.add(b).add(c).scale(tetra / 4.0));
            second_local = add3(second_local, tetra_raw_second(a, b, c, tetra));

            let area = 0.5 * b.sub(a).cross(c.sub(a)).length();
            if !area.is_finite() {
                return Err(BRepError::Overflow);
            }
            surface_area += area;
        }

        let extent = self.bbox_extent().max(1.0);
        let volume_scale = extent * extent * extent;
        let volume_tolerance = tolerance.threshold(volume_scale)
            .map_err(|_| BRepError::InvalidTolerance)?;
        if !signed_volume.is_finite() || signed_volume.abs() <= volume_tolerance {
            return Err(BRepError::ZeroVolume);
        }

        let centroid_local = first_local.scale(1.0 / signed_volume);
        let centroid = reference.add(centroid_local);
        if !centroid.is_finite() {
            return Err(BRepError::Overflow);
        }

        let inertia_local_origin = inertia_from_second(second_local);
        let m = signed_volume;
        let c = centroid_local;
        let inertia_centroid = [
            [
                inertia_local_origin[0][0] - m * (c.y * c.y + c.z * c.z),
                inertia_local_origin[0][1] + m * c.x * c.y,
                inertia_local_origin[0][2] + m * c.x * c.z,
            ],
            [
                inertia_local_origin[1][0] + m * c.y * c.x,
                inertia_local_origin[1][1] - m * (c.x * c.x + c.z * c.z),
                inertia_local_origin[1][2] + m * c.y * c.z,
            ],
            [
                inertia_local_origin[2][0] + m * c.z * c.x,
                inertia_local_origin[2][1] + m * c.z * c.y,
                inertia_local_origin[2][2] - m * (c.x * c.x + c.y * c.y),
            ],
        ];

        // Move the centroidal tensor from the local reference frame to the
        // world-origin frame with the parallel-axis theorem.
        let inertia_origin = [
            [
                inertia_centroid[0][0] + m * (centroid.y * centroid.y + centroid.z * centroid.z),
                inertia_centroid[0][1] - m * centroid.x * centroid.y,
                inertia_centroid[0][2] - m * centroid.x * centroid.z,
            ],
            [
                inertia_centroid[1][0] - m * centroid.y * centroid.x,
                inertia_centroid[1][1] + m * (centroid.x * centroid.x + centroid.z * centroid.z),
                inertia_centroid[1][2] - m * centroid.y * centroid.z,
            ],
            [
                inertia_centroid[2][0] - m * centroid.z * centroid.x,
                inertia_centroid[2][1] - m * centroid.z * centroid.y,
                inertia_centroid[2][2] + m * (centroid.x * centroid.x + centroid.y * centroid.y),
            ],
        ];

        if inertia_centroid.iter().flatten().any(|v| !v.is_finite())
            || inertia_origin.iter().flatten().any(|v| !v.is_finite())
            || !surface_area.is_finite()
        {
            return Err(BRepError::Overflow);
        }

        Ok(SolidMoments {
            signed_volume,
            volume: signed_volume.abs(),
            centroid,
            inertia_origin,
            inertia_centroid,
            surface_area,
        })
    }

    pub fn bbox_extent(&self) -> f64 {
        let mut min = Vec3::new(f64::INFINITY, f64::INFINITY, f64::INFINITY);
        let mut max = Vec3::new(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY);
        for v in &self.vertices {
            min.x = min.x.min(v.point.x);
            min.y = min.y.min(v.point.y);
            min.z = min.z.min(v.point.z);
            max.x = max.x.max(v.point.x);
            max.y = max.y.max(v.point.y);
            max.z = max.z.max(v.point.z);
        }
        max.sub(min).length()
    }

    fn triangles(&self, tolerance: Tolerance) -> Result<Vec<Triangle3Ref>, BRepError> {
        let mut triangles = Vec::new();
        for face in &self.faces {
            let points = face.region.outer_points_3d(tolerance)?;
            if points.len() < 3 {
                return Err(BRepError::Degenerate);
            }
            let base = points[0];
            for i in 1..points.len() - 1 {
                let (a, b, c) = if face.orientation {
                    (base, points[i], points[i + 1])
                } else {
                    (base, points[i + 1], points[i])
                };
                triangles.push(Triangle3Ref { a, b, c });
            }
        }
        Ok(triangles)
    }
}

#[derive(Clone, Copy)]
struct Triangle3Ref { a: Vec3, b: Vec3, c: Vec3 }

fn sub_outer(_a: Vec3, _b: Vec3) -> [[f64;3];3] {
    matrix_zero()
}

fn validate_face_region_edge_correspondence(
    solid: &BRepSolid,
    face: &BRepFace,
    tolerance: Tolerance,
) -> Result<(), BRepError> {
    let mut wire_ids: Vec<&str> = vec![face.outer_wire.as_str()];
    wire_ids.extend(face.inner_wires.iter().map(String::as_str));
    for wire_id in wire_ids {
        let wire = solid.wires.iter().find(|w| w.id == wire_id).ok_or(BRepError::MissingReference)?;
        let coedges = wire.coedges.iter()
            .map(|id| solid.coedges.iter().find(|c| c.id == *id).ok_or(BRepError::MissingReference))
            .collect::<Result<Vec<_>, _>>()?;
        if coedges.iter().any(|coedge| coedge.face != face.id) {
            return Err(BRepError::InconsistentOrientation);
        }
        // The topological loop is authoritative; the planar region supplies
        // the corresponding surface mathematics. Explicit correspondence is
        // required at construction time through matching endpoint positions.
        let region_points = if wire_id == face.outer_wire.as_str() {
            face.region.outer_points_3d(tolerance)?
        } else {
            let index = face.inner_wires.iter().position(|id| id.as_str() == wire_id).ok_or(BRepError::MissingReference)?;
            face.region.holes[index].iter()
                .map(|uv| face.region.point_from_uv(*uv, tolerance))
                .collect::<Result<Vec<_>, _>>()?
        };
        if region_points.len() != coedges.len() {
            return Err(BRepError::InconsistentOrientation);
        }
        for (i, coedge) in coedges.iter().enumerate() {
            let edge = solid.edges.iter().find(|e| e.id == coedge.edge).ok_or(BRepError::MissingReference)?;
            let a = solid.vertices.iter().find(|v| v.id == edge.start_vertex).ok_or(BRepError::MissingReference)?.point;
            let b = solid.vertices.iter().find(|v| v.id == edge.end_vertex).ok_or(BRepError::MissingReference)?.point;
            let start = if coedge.forward { a } else { b };
            let expected = region_points[i];
            let eps = tolerance.threshold(start.sub(expected).length().max(1.0))
                .map_err(|_| BRepError::InvalidTolerance)?;
            if !near(start, expected, eps) {
                return Err(BRepError::InconsistentOrientation);
            }
        }
    }
    Ok(())
}

fn unique_ids<'a, I>(values: I) -> Result<BTreeSet<String>, BRepError>
where
    I: Iterator<Item = &'a str>,
{
    let mut set = BTreeSet::new();
    for value in values {
        if value.is_empty() { return Err(BRepError::DuplicateId); }
        if !set.insert(value.to_string()) { return Err(BRepError::DuplicateId); }
    }
    Ok(set)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SolidPointClass { Inside, Outside, OnBoundary, Indeterminate }

fn ray_triangle(origin: Vec3, direction: Vec3, t: Triangle3Ref, tolerance: f64) -> RayHit {
    let e1 = t.b.sub(t.a);
    let e2 = t.c.sub(t.a);
    let h = direction.cross(e2);
    let det = e1.dot(h);
    let scale = e1.length() * e2.length();
    if !det.is_finite() || !scale.is_finite() { return RayHit::Indeterminate; }
    if det.abs() <= tolerance * scale.max(f64::MIN_POSITIVE) {
        return RayHit::Indeterminate;
    }
    let inv = 1.0 / det;
    let s = origin.sub(t.a);
    let u = inv * s.dot(h);
    let q = s.cross(e1);
    let v = inv * direction.dot(q);
    let tau = inv * e2.dot(q);
    if !u.is_finite() || !v.is_finite() || !tau.is_finite() { return RayHit::Indeterminate; }
    let band = tolerance.max(1.0e-14);
    if tau < -band || u < -band || v < -band || u + v > 1.0 + band { return RayHit::Miss; }
    if u <= band || v <= band || 1.0 - u - v <= band { return RayHit::Boundary; }
    if tau >= -band { RayHit::Hit } else { RayHit::Miss }
}

enum RayHit { Hit, Miss, Boundary, Indeterminate }

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AxisAlignedBox {
    pub min: Vec3,
    pub max: Vec3,
}

impl AxisAlignedBox {
    pub fn validate(&self, tolerance: Tolerance) -> Result<(), BRepError> {
        validate_tolerance(tolerance)?;
        if !self.min.is_finite() || !self.max.is_finite() {
            return Err(BRepError::NonFinite);
        }
        let eps = tolerance.threshold(
            self.max.sub(self.min).length().max(1.0)
        ).map_err(|_| BRepError::InvalidTolerance)?;
        if self.max.x - self.min.x <= eps
            || self.max.y - self.min.y <= eps
            || self.max.z - self.min.z <= eps {
            return Err(BRepError::Degenerate);
        }
        Ok(())
    }

    pub fn volume(&self, tolerance: Tolerance) -> Result<f64, BRepError> {
        self.validate(tolerance)?;
        let d = self.max.sub(self.min);
        let value = d.x * d.y * d.z;
        if value.is_finite() && value > 0.0 { Ok(value) } else { Err(BRepError::Overflow) }
    }

    pub fn surface_area(&self, tolerance: Tolerance) -> Result<f64, BRepError> {
        self.validate(tolerance)?;
        let d = self.max.sub(self.min);
        let value = 2.0 * (d.x * d.y + d.x * d.z + d.y * d.z);
        if value.is_finite() && value > 0.0 { Ok(value) } else { Err(BRepError::Overflow) }
    }

    pub fn centroid(&self, tolerance: Tolerance) -> Result<Vec3, BRepError> {
        self.validate(tolerance)?;
        let result = self.min.add(self.max).scale(0.5);
        if result.is_finite() { Ok(result) } else { Err(BRepError::Overflow) }
    }

    pub fn inertia_centroid(&self, tolerance: Tolerance) -> Result<[[f64;3];3], BRepError> {
        self.validate(tolerance)?;
        let d = self.max.sub(self.min);
        let v = self.volume(tolerance)?;
        Ok([
            [v * (d.y*d.y + d.z*d.z) / 12.0, 0.0, 0.0],
            [0.0, v * (d.x*d.x + d.z*d.z) / 12.0, 0.0],
            [0.0, 0.0, v * (d.x*d.x + d.y*d.y) / 12.0],
        ])
    }

    pub fn classify_point(&self, p: Vec3, tolerance: Tolerance) -> Result<SolidPointClass, BRepError> {
        self.validate(tolerance)?;
        if !p.is_finite() { return Err(BRepError::NonFinite); }
        let eps = tolerance.threshold(self.max.sub(self.min).length().max(1.0))
            .map_err(|_| BRepError::InvalidTolerance)?;
        let inside = p.x > self.min.x + eps && p.x < self.max.x - eps
            && p.y > self.min.y + eps && p.y < self.max.y - eps
            && p.z > self.min.z + eps && p.z < self.max.z - eps;
        if inside { return Ok(SolidPointClass::Inside); }
        let boundary = p.x >= self.min.x - eps && p.x <= self.max.x + eps
            && p.y >= self.min.y - eps && p.y <= self.max.y + eps
            && p.z >= self.min.z - eps && p.z <= self.max.z + eps;
        if boundary { Ok(SolidPointClass::OnBoundary) } else { Ok(SolidPointClass::Outside) }
    }
}

fn overlaps(a: &AxisAlignedBox, b: &AxisAlignedBox, tolerance: f64) -> bool {
    a.min.x < b.max.x - tolerance && b.min.x < a.max.x - tolerance
        && a.min.y < b.max.y - tolerance && b.min.y < a.max.y - tolerance
        && a.min.z < b.max.z - tolerance && b.min.z < a.max.z - tolerance
}

pub fn box_intersection(
    a: AxisAlignedBox,
    b: AxisAlignedBox,
    tolerance: Tolerance,
) -> Result<Option<AxisAlignedBox>, BRepError> {
    a.validate(tolerance)?;
    b.validate(tolerance)?;
    let min = Vec3::new(a.min.x.max(b.min.x), a.min.y.max(b.min.y), a.min.z.max(b.min.z));
    let max = Vec3::new(a.max.x.min(b.max.x), a.max.y.min(b.max.y), a.max.z.min(b.max.z));
    let result = AxisAlignedBox { min, max };
    if overlaps(&a, &b, tolerance.threshold(1.0).map_err(|_| BRepError::InvalidTolerance)? ) {
        result.validate(tolerance)?;
        Ok(Some(result))
    } else {
        Ok(None)
    }
}

fn split_box_raw(
    b: AxisAlignedBox,
    axis: usize,
    coordinate: f64,
    tolerance: f64,
) -> Vec<AxisAlignedBox> {
    let mut parts = Vec::new();
    let mut lo = b.min;
    let mut hi = b.max;
    let low = match axis { 0 => b.min.x, 1 => b.min.y, _ => b.min.z };
    let high = match axis { 0 => b.max.x, 1 => b.max.y, _ => b.max.z };
    if coordinate <= low + tolerance || coordinate >= high - tolerance {
        return vec![b];
    }
    match axis {
        0 => { let mut m = b.max; m.x = coordinate; hi.x = coordinate; parts.push(AxisAlignedBox{min:lo,max:m}); lo.x=coordinate; parts.push(AxisAlignedBox{min:lo,max:b.max}); }
        1 => { let mut m = b.max; m.y = coordinate; hi.y = coordinate; parts.push(AxisAlignedBox{min:lo,max:m}); lo.y=coordinate; parts.push(AxisAlignedBox{min:lo,max:b.max}); }
        _ => { let mut m = b.max; m.z = coordinate; hi.z = coordinate; parts.push(AxisAlignedBox{min:lo,max:m}); lo.z=coordinate; parts.push(AxisAlignedBox{min:lo,max:b.max}); }
    }
    let _ = hi;
    parts
}

fn box_grid(
    a: AxisAlignedBox,
    b: AxisAlignedBox,
    tolerance: Tolerance,
    predicate: impl Fn(Vec3) -> bool,
) -> Result<Vec<AxisAlignedBox>, BRepError> {
    a.validate(tolerance)?;
    b.validate(tolerance)?;
    let xs = sorted_unique([a.min.x, a.max.x, b.min.x, b.max.x], tolerance.threshold(1.0).map_err(|_| BRepError::InvalidTolerance)?);
    let ys = sorted_unique([a.min.y, a.max.y, b.min.y, b.max.y], tolerance.threshold(1.0).map_err(|_| BRepError::InvalidTolerance)?);
    let zs = sorted_unique([a.min.z, a.max.z, b.min.z, b.max.z], tolerance.threshold(1.0).map_err(|_| BRepError::InvalidTolerance)?);
    let mut parts = Vec::new();
    for xw in xs.windows(2) {
        for yw in ys.windows(2) {
            for zw in zs.windows(2) {
                let candidate = AxisAlignedBox {
                    min: Vec3::new(xw[0], yw[0], zw[0]),
                    max: Vec3::new(xw[1], yw[1], zw[1]),
                };
                if candidate.max.x - candidate.min.x <= 0.0
                    || candidate.max.y - candidate.min.y <= 0.0
                    || candidate.max.z - candidate.min.z <= 0.0 {
                    continue;
                }
                let center = candidate.min.add(candidate.max).scale(0.5);
                if predicate(center) {
                    candidate.validate(tolerance)?;
                    parts.push(candidate);
                }
            }
        }
    }
    Ok(parts)
}

fn sorted_unique<const N: usize>(mut values: [f64; N], tolerance: f64) -> Vec<f64> {
    values.sort_by(|a,b| a.total_cmp(b));
    let mut out=Vec::new();
    for v in values {
        if out.last().map_or(true, |p: &f64| (v-*p).abs()>tolerance) {
            out.push(v);
        }
    }
    out
}

pub fn box_union(
    a: AxisAlignedBox,
    b: AxisAlignedBox,
    tolerance: Tolerance,
) -> Result<Vec<AxisAlignedBox>, BRepError> {
    let _ = overlaps(&a, &b, tolerance.threshold(1.0).map_err(|_| BRepError::InvalidTolerance)?);
    box_grid(a, b, tolerance, |p| contains_closed(&a,p) || contains_closed(&b,p))
}

pub fn box_difference(
    a: AxisAlignedBox,
    b: AxisAlignedBox,
    tolerance: Tolerance,
) -> Result<Vec<AxisAlignedBox>, BRepError> {
    box_grid(a, b, tolerance, |p| contains_closed(&a,p) && !contains_open(&b,p))
}

pub fn box_split_by_plane(
    b: AxisAlignedBox,
    axis: usize,
    coordinate: f64,
    tolerance: Tolerance,
) -> Result<Vec<AxisAlignedBox>, BRepError> {
    b.validate(tolerance)?;
    if !coordinate.is_finite() { return Err(BRepError::NonFinite); }
    let eps = tolerance.threshold(b.max.sub(b.min).length().max(1.0))
        .map_err(|_| BRepError::InvalidTolerance)?;
    if coordinate <= axis_value(b.min,axis)+eps || coordinate >= axis_value(b.max,axis)-eps {
        return Ok(vec![b]);
    }
    Ok(split_box_raw(b,axis,coordinate,eps))
}

/// Exact boundary-grid imprint for two AABBs. The returned planes are
/// mathematical partition coordinates; topology generation remains a later
/// explicit operation.
pub fn box_imprint_planes(
    a: AxisAlignedBox,
    b: AxisAlignedBox,
    tolerance: Tolerance,
) -> Result<(Vec<f64>,Vec<f64>,Vec<f64>), BRepError> {
    a.validate(tolerance)?;
    b.validate(tolerance)?;
    let eps=tolerance.threshold(a.max.sub(a.min).length().max(b.max.sub(b.min).length()).max(1.0))
        .map_err(|_| BRepError::InvalidTolerance)?;
    Ok((
        sorted_unique([a.min.x,a.max.x,b.min.x,b.max.x],eps),
        sorted_unique([a.min.y,a.max.y,b.min.y,b.max.y],eps),
        sorted_unique([a.min.z,a.max.z,b.min.z,b.max.z],eps),
    ))
}

fn axis_value(v: Vec3, axis: usize) -> f64 { match axis {0=>v.x,1=>v.y,_=>v.z} }
fn contains_open(b:&AxisAlignedBox,p:Vec3)->bool{p.x>b.min.x&&p.x<b.max.x&&p.y>b.min.y&&p.y<b.max.y&&p.z>b.min.z&&p.z<b.max.z}
fn contains_closed(b:&AxisAlignedBox,p:Vec3)->bool{p.x>=b.min.x&&p.x<=b.max.x&&p.y>=b.min.y&&p.y<=b.max.y&&p.z>=b.min.z&&p.z<=b.max.z}

#[cfg(test)]
mod tests {
    use super::*;

    fn tol() -> Tolerance { Tolerance::new(1.0e-9, 1.0e-12).unwrap() }

    fn box_a() -> AxisAlignedBox {
        AxisAlignedBox { min: Vec3::new(0.0,0.0,0.0), max: Vec3::new(10.0,20.0,30.0) }
    }

    fn tetra_brep() -> BRepSolid {
        let vertices = vec![
            BRepVertex{id:"v0".into(),point:Vec3::new(0.0,0.0,0.0)},
            BRepVertex{id:"v1".into(),point:Vec3::new(1.0,0.0,0.0)},
            BRepVertex{id:"v2".into(),point:Vec3::new(0.0,1.0,0.0)},
            BRepVertex{id:"v3".into(),point:Vec3::new(0.0,0.0,1.0)},
        ];
        let edges = vec![
            BRepEdge{id:"e01".into(),start_vertex:"v0".into(),end_vertex:"v1".into()},
            BRepEdge{id:"e12".into(),start_vertex:"v1".into(),end_vertex:"v2".into()},
            BRepEdge{id:"e20".into(),start_vertex:"v2".into(),end_vertex:"v0".into()},
            BRepEdge{id:"e03".into(),start_vertex:"v0".into(),end_vertex:"v3".into()},
            BRepEdge{id:"e13".into(),start_vertex:"v1".into(),end_vertex:"v3".into()},
            BRepEdge{id:"e23".into(),start_vertex:"v2".into(),end_vertex:"v3".into()},
        ];

        // Each face is explicitly parameterized in an orthonormal planar basis.
        // The loop order is outward-facing for the tetrahedron.
        let rt2 = 2.0_f64.sqrt();
        let rt6 = 6.0_f64.sqrt();
        let u3 = Vec3::new(-1.0,1.0,0.0).scale(1.0/rt2);
        let v3 = Vec3::new(-1.0,-1.0,2.0).scale(1.0/rt6);
        let faces = vec![
            BRepFace{
                id:"f0".into(), outer_wire:"w0".into(), inner_wires:vec![],
                region:PlanarRegion3{origin:vertices[0].point,u_dir:Vec3::new(1.0,0.0,0.0),v_dir:Vec3::new(0.0,0.0,1.0),
                    outer:vec![Vec2::new(0.0,0.0),Vec2::new(1.0,0.0),Vec2::new(0.0,1.0)],holes:vec![]},
                orientation:true,
            },
            BRepFace{
                id:"f1".into(), outer_wire:"w1".into(), inner_wires:vec![],
                region:PlanarRegion3{origin:vertices[0].point,u_dir:Vec3::new(1.0,0.0,0.0),v_dir:Vec3::new(0.0,1.0,0.0),
                    outer:vec![Vec2::new(0.0,0.0),Vec2::new(0.0,1.0),Vec2::new(1.0,0.0)],holes:vec![]},
                orientation:true,
            },
            BRepFace{
                id:"f2".into(), outer_wire:"w2".into(), inner_wires:vec![],
                region:PlanarRegion3{origin:vertices[0].point,u_dir:Vec3::new(0.0,1.0,0.0),v_dir:Vec3::new(0.0,0.0,1.0),
                    outer:vec![Vec2::new(0.0,0.0),Vec2::new(0.0,1.0),Vec2::new(1.0,0.0)],holes:vec![]},
                orientation:true,
            },
            BRepFace{
                id:"f3".into(), outer_wire:"w3".into(), inner_wires:vec![],
                region:PlanarRegion3{origin:vertices[1].point,u_dir:u3,v_dir:v3,
                    outer:vec![
                        Vec2::new(0.0,0.0),
                        Vec2::new(rt2,0.0),
                        Vec2::new(1.0/rt2,3.0/rt6),
                    ],holes:vec![]},
                orientation:true,
            },
        ];

        let wire_specs: [(&str,[&str;3],[bool;3]);4] = [
            ("w0",["e01","e13","e03"],[true,true,false]),
            ("w1",["e20","e12","e01"],[false,false,false]),
            ("w2",["e03","e23","e20"],[true,false,true]),
            ("w3",["e12","e23","e13"],[true,true,false]),
        ];
        let mut coedges=Vec::new();
        let mut wires=Vec::new();
        for (fi,(wid,eids,directions)) in wire_specs.iter().enumerate() {
            let fid=format!("f{}",fi);
            let mut ids=Vec::new();
            for i in 0..3 {
                let cid=format!("c{}_{}",fi,i);
                coedges.push(BRepCoedge{id:cid.clone(),edge:eids[i].into(),wire:(*wid).into(),face:fid.clone(),forward:directions[i]});
                ids.push(cid);
            }
            wires.push(BRepWire{id:(*wid).into(),coedges:ids,closed:true});
        }
        let shell_faces=faces.iter().map(|f|f.id.clone()).collect();
        BRepSolid{vertices,edges,coedges,wires,faces,shells:vec![BRepShell{id:"s0".into(),faces:shell_faces}]}
    }

    #[test]
    fn explicit_brep_incidence_is_closed_and_manifold() {
        let solid = tetra_brep();
        assert!(solid.validate(tol()).is_ok());
    }

    #[test]
    fn planar_region_has_exact_area_centroid_and_point_classification() {
        let region = PlanarRegion3 {
            origin: Vec3::new(10.0,20.0,30.0),
            u_dir: Vec3::new(1.0,0.0,0.0),
            v_dir: Vec3::new(0.0,1.0,0.0),
            outer: vec![
                Vec2::new(0.0,0.0),Vec2::new(4.0,0.0),
                Vec2::new(4.0,3.0),Vec2::new(0.0,3.0),
            ],
            holes: vec![],
        };
        assert!((region.area(tol()).unwrap()-12.0).abs()<=1.0e-12);
        let c=region.centroid(tol()).unwrap();
        assert!((c.x-12.0).abs()<=1.0e-12 && (c.y-21.5).abs()<=1.0e-12 && (c.z-30.0).abs()<=1.0e-12);
        assert_eq!(region.classify_point(Vec3::new(12.0,21.0,30.0),tol()).unwrap(),RegionClass::Inside);
        assert_eq!(region.classify_point(Vec3::new(12.0,25.0,30.0),tol()).unwrap(),RegionClass::Outside);
    }

    #[test]
    fn planar_region_area_and_centroid_are_translation_invariant() {
        let local = PlanarRegion3 {
            origin: Vec3::new(0.0,0.0,0.0),
            u_dir: Vec3::new(1.0,0.0,0.0),
            v_dir: Vec3::new(0.0,1.0,0.0),
            outer: vec![
                Vec2::new(0.0,0.0), Vec2::new(10.0,0.0),
                Vec2::new(10.0,20.0), Vec2::new(0.0,20.0),
            ],
            holes: vec![],
        };
        let shifted = PlanarRegion3 {
            origin: Vec3::new(1.0e12,1.0e12,0.0),
            u_dir: Vec3::new(1.0,0.0,0.0),
            v_dir: Vec3::new(0.0,1.0,0.0),
            outer: vec![
                Vec2::new(0.0,0.0), Vec2::new(10.0,0.0),
                Vec2::new(10.0,20.0), Vec2::new(0.0,20.0),
            ],
            holes: vec![],
        };
        assert!((local.area(tol()).unwrap() - shifted.area(tol()).unwrap()).abs() <= 1.0e-8);
        assert!((local.centroid(tol()).unwrap().x - 5.0).abs() <= 1.0e-9);
        assert!((shifted.centroid(tol()).unwrap().x - (1.0e12 + 5.0)).abs() <= 1.0e-3);
        assert!((shifted.centroid(tol()).unwrap().y - (1.0e12 + 10.0)).abs() <= 1.0e-3);
    }

    #[test]
    fn planar_region_rejects_nested_holes() {
        let region = PlanarRegion3 {
            origin: Vec3::new(0.0,0.0,0.0),
            u_dir: Vec3::new(1.0,0.0,0.0),
            v_dir: Vec3::new(0.0,1.0,0.0),
            outer: vec![
                Vec2::new(0.0,0.0),Vec2::new(10.0,0.0),
                Vec2::new(10.0,10.0),Vec2::new(0.0,10.0),
            ],
            holes: vec![
                vec![Vec2::new(2.0,2.0),Vec2::new(8.0,2.0),Vec2::new(8.0,8.0),Vec2::new(2.0,8.0)],
                vec![Vec2::new(4.0,4.0),Vec2::new(6.0,4.0),Vec2::new(6.0,6.0),Vec2::new(4.0,6.0)],
            ],
        };
        assert_eq!(region.validate(tol()), Err(BRepError::InvalidRegion));
    }

    #[test]
    fn planar_region_predicates_are_scale_consistent() {
        let small = PlanarRegion3 {
            origin: Vec3::new(0.0,0.0,0.0),
            u_dir: Vec3::new(1.0,0.0,0.0),
            v_dir: Vec3::new(0.0,1.0,0.0),
            outer: vec![
                Vec2::new(0.0,0.0), Vec2::new(1.0e-6,0.0),
                Vec2::new(1.0e-6,1.0e-6), Vec2::new(0.0,1.0e-6),
            ],
            holes: vec![],
        };
        assert!(small.validate(tol()).is_ok());

        let large = PlanarRegion3 {
            origin: Vec3::new(0.0,0.0,0.0),
            u_dir: Vec3::new(1.0,0.0,0.0),
            v_dir: Vec3::new(0.0,1.0,0.0),
            outer: vec![
                Vec2::new(0.0,0.0), Vec2::new(1.0e6,0.0),
                Vec2::new(1.0e6,1.0e6), Vec2::new(0.0,1.0e6),
            ],
            holes: vec![],
        };
        assert!(large.validate(tol()).is_ok());
    }

    #[test]
    fn planar_region_rejects_nonconvex_certification_and_marks_hole_boundary() {
        let nonconvex = PlanarRegion3 {
            origin: Vec3::new(0.0,0.0,0.0),
            u_dir: Vec3::new(1.0,0.0,0.0),
            v_dir: Vec3::new(0.0,1.0,0.0),
            outer: vec![
                Vec2::new(0.0,0.0), Vec2::new(4.0,0.0),
                Vec2::new(2.0,1.0), Vec2::new(4.0,4.0),
                Vec2::new(0.0,4.0),
            ],
            holes: vec![],
        };
        assert_eq!(nonconvex.validate(tol()), Err(BRepError::InvalidRegion));

        let region = PlanarRegion3 {
            origin: Vec3::new(0.0,0.0,0.0),
            u_dir: Vec3::new(1.0,0.0,0.0),
            v_dir: Vec3::new(0.0,1.0,0.0),
            outer: vec![
                Vec2::new(0.0,0.0),Vec2::new(10.0,0.0),
                Vec2::new(10.0,10.0),Vec2::new(0.0,10.0),
            ],
            holes: vec![vec![
                Vec2::new(3.0,3.0),Vec2::new(7.0,3.0),
                Vec2::new(7.0,7.0),Vec2::new(3.0,7.0),
            ]],
        };
        assert_eq!(
            region.classify_point(Vec3::new(3.0,5.0,0.0), tol()).unwrap(),
            RegionClass::OnBoundary
        );
    }

    #[test]
    fn planar_region_with_hole_preserves_area_and_classifies_hole_region() {
        let region = PlanarRegion3 {
            origin: Vec3::new(0.0, 0.0, 0.0),
            u_dir: Vec3::new(1.0,0.0,0.0),
            v_dir: Vec3::new(0.0,1.0,0.0),
            outer: vec![
                Vec2::new(0.0,0.0),Vec2::new(10.0,0.0),
                Vec2::new(10.0,10.0),Vec2::new(0.0,10.0),
            ],
            holes: vec![vec![
                Vec2::new(3.0,3.0),Vec2::new(7.0,3.0),
                Vec2::new(7.0,7.0),Vec2::new(3.0,7.0),
            ]],
        };
        assert!((region.area(tol()).unwrap() - 84.0).abs() <= 1.0e-12);
        assert_eq!(
            region.classify_point(Vec3::new(1.0,1.0,0.0),tol()).unwrap(),
            RegionClass::Inside
        );
        assert_eq!(
            region.classify_point(Vec3::new(5.0,5.0,0.0),tol()).unwrap(),
            RegionClass::InsideHole
        );
        assert_eq!(
            region.classify_point(Vec3::new(12.0,5.0,0.0),tol()).unwrap(),
            RegionClass::Outside
        );
    }

    #[test]
    fn solid_rejects_wire_coedge_attached_to_different_face() {
        let mut solid = tetra_brep();
        solid.coedges.iter_mut()
            .find(|c| c.id == "c0_0")
            .expect("tetrahedron test coedge exists")
            .face = "f1".into();
        assert_eq!(solid.validate(tol()), Err(BRepError::InconsistentOrientation));
    }

    #[test]
    fn solid_rejects_edge_with_same_face_on_both_sides() {
        let mut solid = tetra_brep();
        solid.coedges.iter_mut()
            .find(|c| c.id == "c1_2")
            .expect("tetrahedron test coedge exists")
            .face = "f0".into();
        assert_eq!(solid.validate(tol()), Err(BRepError::NonManifoldEdge));
    }

    #[test]
    fn solid_rejects_reversed_global_orientation() {
        let mut solid = tetra_brep();
        for face in &mut solid.faces { face.orientation = !face.orientation; }
        assert_eq!(solid.validate(tol()), Err(BRepError::InvalidSolidOrientation));
    }

    #[test]
    fn boolean_support_is_deterministic_for_disjoint_and_touching_boxes() {
        let a = box_a();
        let disjoint = AxisAlignedBox {
            min: Vec3::new(20.0,0.0,0.0),
            max: Vec3::new(30.0,20.0,30.0),
        };
        let pieces = box_union(a, disjoint, tol()).unwrap();
        assert_eq!(pieces.len(), 2);
        let total: f64 = pieces.iter().map(|p| p.volume(tol()).unwrap()).sum();
        assert!((total - 12000.0).abs() <= 1.0e-12);

        let touching = AxisAlignedBox {
            min: Vec3::new(10.0,0.0,0.0),
            max: Vec3::new(20.0,20.0,30.0),
        };
        let pieces = box_union(a, touching, tol()).unwrap();
        let total: f64 = pieces.iter().map(|p| p.volume(tol()).unwrap()).sum();
        assert!((total - 12000.0).abs() <= 1.0e-12);
    }

    #[test]
    fn solid_moment_domain_rejects_holed_faces_until_exact_decomposition_exists() {
        let mut solid = tetra_brep();
        solid.faces[0].region.holes.push(vec![
            Vec2::new(0.2,0.1), Vec2::new(0.3,0.1), Vec2::new(0.3,0.2), Vec2::new(0.2,0.2),
        ]);
        assert_eq!(solid.validate(tol()), Err(BRepError::UnsupportedBoolean));
        assert_eq!(solid.moments(tol()), Err(BRepError::UnsupportedBoolean));
    }

    #[test]
    fn translated_tetra_moments_are_stable() {
        let mut solid = tetra_brep();
        let shift = Vec3::new(1.0e12, -1.0e12, 5.0e11);
        for vertex in &mut solid.vertices {
            vertex.point = vertex.point.add(shift);
        }
        for face in &mut solid.faces {
            face.region.origin = face.region.origin.add(shift);
        }

        let moments = solid.moments(tol()).unwrap();
        assert!((moments.volume - 1.0 / 6.0).abs() <= 1.0e-10);
        assert!((moments.centroid.x - (1.0e12 + 0.25)).abs() <= 1.0e-3);
        assert!((moments.centroid.y - (-1.0e12 + 0.25)).abs() <= 1.0e-3);
        assert!((moments.centroid.z - (5.0e11 + 0.25)).abs() <= 1.0e-3);
        assert!(moments.inertia_centroid.iter().flatten().all(|v| v.is_finite()));
    }

    #[test]
    fn tetra_moments_are_exact() {
        let solid=tetra_brep();
        let m=solid.moments(tol()).unwrap();
        assert!((m.volume-1.0/6.0).abs()<=1.0e-12);
        assert!((m.centroid.x-0.25).abs()<=1.0e-12);
        assert!((m.centroid.y-0.25).abs()<=1.0e-12);
        assert!((m.centroid.z-0.25).abs()<=1.0e-12);
        assert!(m.inertia_centroid[0][0].is_finite());
        assert!(m.inertia_centroid[1][1].is_finite());
        assert!(m.inertia_centroid[2][2].is_finite());
    }

    #[test]
    fn closed_tetra_classification_is_explicit() {
        let solid=tetra_brep();
        assert_eq!(solid.classify_point(Vec3::new(0.1,0.1,0.1),Vec3::new(1.0,0.2,0.3),tol()).unwrap(),SolidPointClass::Inside);
        assert_eq!(solid.classify_point(Vec3::new(2.0,2.0,2.0),Vec3::new(1.0,0.2,0.3),tol()).unwrap(),SolidPointClass::Outside);
    }

    #[test]
    fn malformed_incidence_fails_closed() {
        let mut solid=tetra_brep();
        solid.coedges.pop();
        assert!(matches!(solid.validate(tol()),Err(BRepError::NonManifoldEdge)|Err(BRepError::OpenWire)|Err(BRepError::InconsistentOrientation)));
        let mut bad=tetra_brep();
        bad.shells[0].faces[0]="missing".into();
        assert_eq!(bad.validate(tol()),Err(BRepError::MissingReference));
    }

    #[test]
    fn box_boolean_partition_is_exact_and_deterministic() {
        let a=box_a();
        let b=AxisAlignedBox{min:Vec3::new(5.0,10.0,15.0),max:Vec3::new(15.0,25.0,35.0)};
        let intersection=box_intersection(a,b,tol()).unwrap().unwrap();
        assert_eq!(intersection.min,Vec3::new(5.0,10.0,15.0));
        assert_eq!(intersection.max,Vec3::new(10.0,20.0,30.0));
        assert!((intersection.volume(tol()).unwrap()-750.0).abs()<=1.0e-12);

        let difference=box_difference(a,b,tol()).unwrap();
        let diff_volume:f64 = difference.iter().map(|x|x.volume(tol()).unwrap()).sum();
        assert!((diff_volume + intersection.volume(tol()).unwrap()-a.volume(tol()).unwrap()).abs()<=1.0e-9);

        let union=box_union(a,b,tol()).unwrap();
        let union_volume:f64=union.iter().map(|x|x.volume(tol()).unwrap()).sum();
        assert!((union_volume-(a.volume(tol()).unwrap()+b.volume(tol()).unwrap()-intersection.volume(tol()).unwrap())).abs()<=1.0e-9);
        assert_eq!(box_split_by_plane(a,0,4.0,tol()).unwrap().len(),2);
        let imprint=box_imprint_planes(a,b,tol()).unwrap();
        assert!(imprint.0.len()>=3 && imprint.1.len()>=3 && imprint.2.len()>=3);
    }

    #[test]
    fn box_solid_moments_and_boundary_are_exact() {
        let b=box_a();
        assert!((b.volume(tol()).unwrap()-6000.0).abs()<=1.0e-12);
        assert!((b.surface_area(tol()).unwrap()-2200.0).abs()<=1.0e-12);
        assert_eq!(b.centroid(tol()).unwrap(),Vec3::new(5.0,10.0,15.0));
        assert_eq!(b.classify_point(Vec3::new(5.0,10.0,15.0),tol()).unwrap(),SolidPointClass::Inside);
        assert_eq!(b.classify_point(Vec3::new(0.0,10.0,15.0),tol()).unwrap(),SolidPointClass::OnBoundary);
        let inertia=b.inertia_centroid(tol()).unwrap();
        assert!((inertia[0][0]-6000.0*(400.0+900.0)/12.0).abs()<=1.0e-9);
        assert!((inertia[1][1]-6000.0*(100.0+900.0)/12.0).abs()<=1.0e-9);
        assert!((inertia[2][2]-6000.0*(100.0+400.0)/12.0).abs()<=1.0e-9);
    }

    #[test]
    fn nonfinite_and_zero_thickness_boolean_inputs_fail_closed() {
        let mut b=box_a();
        b.max.x=f64::NAN;
        assert_eq!(b.validate(tol()),Err(BRepError::NonFinite));
        let mut z=box_a();
        z.max.x=z.min.x;
        assert_eq!(z.validate(tol()),Err(BRepError::Degenerate));
    }
}