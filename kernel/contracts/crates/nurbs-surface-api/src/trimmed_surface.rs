use super::{NurbsSurface3DDefinition, ParameterDomain};
use umlcad_v6_geometry_api::{GeometryBackend, GeometryError, GeometryResult, ToleranceContext};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParameterPoint2 {
    pub u: f64,
    pub v: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TrimLoop2D {
    pub points: Vec<ParameterPoint2>,
}

impl TrimLoop2D {
    pub fn new(points: Vec<ParameterPoint2>) -> Self {
        Self { points }
    }

    fn signed_area2(&self) -> f64 {
        self.points
            .iter()
            .zip(self.points.iter().cycle().skip(1))
            .take(self.points.len())
            .map(|(a, b)| a.u * b.v - b.u * a.v)
            .sum()
    }

    fn contains_strict(&self, p: ParameterPoint2) -> bool {
        let mut inside = false;
        for (a, b) in self
            .points
            .iter()
            .zip(self.points.iter().cycle().skip(1))
            .take(self.points.len())
        {
            let crosses = (a.v > p.v) != (b.v > p.v);
            if crosses {
                let x = a.u + (p.v - a.v) * (b.u - a.u) / (b.v - a.v);
                if x > p.u {
                    inside = !inside;
                }
            }
        }
        inside
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TrimmedNurbsSurface3DDefinition {
    pub surface: NurbsSurface3DDefinition,
    pub loops: Vec<TrimLoop2D>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrimmedSurfaceDefinitionError {
    SurfaceInvalid,
    NoLoops,
    LoopTooSmall,
    LoopNonFinite,
    LoopOutOfDomain,
    DegenerateLoop,
    OuterLoopOrientation,
    InnerLoopOrientation,
    SelfIntersectingLoop,
    HoleOutsideOuter,
    IntersectingLoops,
}

impl TrimmedNurbsSurface3DDefinition {
    pub fn new(surface: NurbsSurface3DDefinition, loops: Vec<TrimLoop2D>) -> Self {
        Self { surface, loops }
    }

    pub fn validate(&self) -> Result<(), TrimmedSurfaceDefinitionError> {
        self.surface
            .validate()
            .map_err(|_| TrimmedSurfaceDefinitionError::SurfaceInvalid)?;
        if self.loops.is_empty() {
            return Err(TrimmedSurfaceDefinitionError::NoLoops);
        }
        let domain: ParameterDomain = self
            .surface
            .parameter_domain()
            .map_err(|_| TrimmedSurfaceDefinitionError::SurfaceInvalid)?;

        for (index, loop_) in self.loops.iter().enumerate() {
            if loop_.points.len() < 3 {
                return Err(TrimmedSurfaceDefinitionError::LoopTooSmall);
            }
            if loop_
                .points
                .iter()
                .any(|p| !p.u.is_finite() || !p.v.is_finite())
            {
                return Err(TrimmedSurfaceDefinitionError::LoopNonFinite);
            }
            let ((u0, u1), (v0, v1)) = domain;
            if loop_
                .points
                .iter()
                .any(|p| p.u < u0 || p.u > u1 || p.v < v0 || p.v > v1)
            {
                return Err(TrimmedSurfaceDefinitionError::LoopOutOfDomain);
            }
            if has_self_intersection(loop_) {
                return Err(TrimmedSurfaceDefinitionError::SelfIntersectingLoop);
            }
            if loop_.signed_area2() == 0.0 {
                return Err(TrimmedSurfaceDefinitionError::DegenerateLoop);
            }
            if index == 0 {
                if loop_.signed_area2() <= 0.0 {
                    return Err(TrimmedSurfaceDefinitionError::OuterLoopOrientation);
                }
            } else if loop_.signed_area2() >= 0.0 {
                return Err(TrimmedSurfaceDefinitionError::InnerLoopOrientation);
            }
        }

        for hole in self.loops.iter().skip(1) {
            if !self.loops[0].contains_strict(hole.points[0]) {
                return Err(TrimmedSurfaceDefinitionError::HoleOutsideOuter);
            }
            for edge_a in edges(hole) {
                for edge_b in edges(&self.loops[0]) {
                    if segments_intersect(edge_a.0, edge_a.1, edge_b.0, edge_b.1) {
                        return Err(TrimmedSurfaceDefinitionError::IntersectingLoops);
                    }
                }
            }
        }
        for i in 1..self.loops.len() {
            for j in (i + 1)..self.loops.len() {
                if self.loops[i].contains_strict(self.loops[j].points[0])
                    || self.loops[j].contains_strict(self.loops[i].points[0])
                {
                    return Err(TrimmedSurfaceDefinitionError::IntersectingLoops);
                }
                for edge_a in edges(&self.loops[i]) {
                    for edge_b in edges(&self.loops[j]) {
                        if segments_intersect(edge_a.0, edge_a.1, edge_b.0, edge_b.1) {
                            return Err(TrimmedSurfaceDefinitionError::IntersectingLoops);
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

pub trait TrimmedNurbsSurfaceBackend: GeometryBackend {
    fn trimmed_nurbs_surface3d(
        &self,
        definition: &TrimmedNurbsSurface3DDefinition,
        tolerance: ToleranceContext,
    ) -> Result<GeometryResult<Self::Shape>, GeometryError>;
}

fn edges(loop_: &TrimLoop2D) -> impl Iterator<Item = (ParameterPoint2, ParameterPoint2)> + '_ {
    loop_
        .points
        .iter()
        .copied()
        .zip(loop_.points.iter().copied().cycle().skip(1))
        .take(loop_.points.len())
}

fn has_self_intersection(loop_: &TrimLoop2D) -> bool {
    let e = loop_.points.len();
    for i in 0..e {
        for j in (i + 1)..e {
            if j == i + 1 || (i == 0 && j == e - 1) {
                continue;
            }
            let a = loop_.points[i];
            let b = loop_.points[(i + 1) % e];
            let c = loop_.points[j];
            let d = loop_.points[(j + 1) % e];
            if segments_intersect(a, b, c, d) {
                return true;
            }
        }
    }
    false
}

fn cross(a: ParameterPoint2, b: ParameterPoint2, c: ParameterPoint2) -> f64 {
    (b.u - a.u) * (c.v - a.v) - (b.v - a.v) * (c.u - a.u)
}

fn on_segment(a: ParameterPoint2, b: ParameterPoint2, p: ParameterPoint2) -> bool {
    p.u >= a.u.min(b.u) && p.u <= a.u.max(b.u) && p.v >= a.v.min(b.v) && p.v <= a.v.max(b.v)
}

fn segments_intersect(
    a: ParameterPoint2,
    b: ParameterPoint2,
    c: ParameterPoint2,
    d: ParameterPoint2,
) -> bool {
    let o1 = cross(a, b, c);
    let o2 = cross(a, b, d);
    let o3 = cross(c, d, a);
    let o4 = cross(c, d, b);
    if (o1 == 0.0 && on_segment(a, b, c)) || (o2 == 0.0 && on_segment(a, b, d)) {
        return true;
    }
    if (o3 == 0.0 && on_segment(c, d, a)) || (o4 == 0.0 && on_segment(c, d, b)) {
        return true;
    }
    ((o1 > 0.0) != (o2 > 0.0)) && ((o3 > 0.0) != (o4 > 0.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plane() -> NurbsSurface3DDefinition {
        NurbsSurface3DDefinition::new(
            (1, 1),
            vec![
                super::super::Point3 { x: 0.0, y: 0.0, z: 0.0 },
                super::super::Point3 { x: 0.0, y: 1.0, z: 0.0 },
                super::super::Point3 { x: 1.0, y: 0.0, z: 0.0 },
                super::super::Point3 { x: 1.0, y: 1.0, z: 0.0 },
            ],
            vec![1.0; 4],
            (2, 2),
            vec![0.0, 0.0, 1.0, 1.0],
            vec![0.0, 0.0, 1.0, 1.0],
        )
    }

    fn outer() -> TrimLoop2D {
        TrimLoop2D::new(vec![
            ParameterPoint2 { u: 0.0, v: 0.0 },
            ParameterPoint2 { u: 1.0, v: 0.0 },
            ParameterPoint2 { u: 1.0, v: 1.0 },
            ParameterPoint2 { u: 0.0, v: 1.0 },
        ])
    }

    #[test]
    fn rectangular_trim_is_valid() {
        assert!(TrimmedNurbsSurface3DDefinition::new(plane(), vec![outer()])
            .validate()
            .is_ok());
    }

    #[test]
    fn holes_are_required_to_be_clockwise_and_inside_outer_loop() {
        let hole = TrimLoop2D::new(vec![
            ParameterPoint2 { u: 0.25, v: 0.25 },
            ParameterPoint2 { u: 0.25, v: 0.75 },
            ParameterPoint2 { u: 0.75, v: 0.75 },
            ParameterPoint2 { u: 0.75, v: 0.25 },
        ]);
        assert!(TrimmedNurbsSurface3DDefinition::new(plane(), vec![outer(), hole])
            .validate()
            .is_ok());
    }

    #[test]
    fn self_intersection_is_rejected() {
        let bow_tie = TrimLoop2D::new(vec![
            ParameterPoint2 { u: 0.1, v: 0.1 },
            ParameterPoint2 { u: 0.9, v: 0.9 },
            ParameterPoint2 { u: 0.1, v: 0.8 },
            ParameterPoint2 { u: 0.9, v: 0.2 },
        ]);
        assert_eq!(
            TrimmedNurbsSurface3DDefinition::new(plane(), vec![bow_tie]).validate(),
            Err(TrimmedSurfaceDefinitionError::SelfIntersectingLoop)
        );
    }

    #[test]
    fn hole_outside_outer_loop_is_rejected() {
        let small_outer = TrimLoop2D::new(vec![
            ParameterPoint2 { u: 0.1, v: 0.1 },
            ParameterPoint2 { u: 0.8, v: 0.1 },
            ParameterPoint2 { u: 0.8, v: 0.8 },
            ParameterPoint2 { u: 0.1, v: 0.8 },
        ]);
        let hole = TrimLoop2D::new(vec![
            ParameterPoint2 { u: 0.85, v: 0.2 },
            ParameterPoint2 { u: 0.85, v: 0.3 },
            ParameterPoint2 { u: 0.95, v: 0.3 },
            ParameterPoint2 { u: 0.95, v: 0.2 },
        ]);
        assert_eq!(
            TrimmedNurbsSurface3DDefinition::new(plane(), vec![small_outer, hole]).validate(),
            Err(TrimmedSurfaceDefinitionError::HoleOutsideOuter)
        );
    }
}
