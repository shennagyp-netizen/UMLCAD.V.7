use crate::{IntersectionStatus, LineSegment3D, LineSurfaceIntersectionError};
use umlcad_v6_nurbs_surface_api::Point3;

#[derive(Clone, Debug, PartialEq)]
pub struct SplitLineSegmentResult {
    pub segments: Vec<LineSegment3D>,
    pub parameters: Vec<f64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SplitLineSegmentError {
    Intersection(LineSurfaceIntersectionError),
    AmbiguousIntersection,
    NoInteriorIntersection,
    NonFinite,
}

fn lerp(a: Point3, b: Point3, t: f64) -> Point3 {
    Point3 {
        x: a.x + (b.x - a.x) * t,
        y: a.y + (b.y - a.y) * t,
        z: a.z + (b.z - a.z) * t,
    }
}

pub fn split_line_segment_at_intersections(
    line: LineSegment3D,
    status: IntersectionStatus,
    parameters: &[f64],
    tolerance: f64,
) -> Result<SplitLineSegmentResult, SplitLineSegmentError> {
    line.validate().map_err(SplitLineSegmentError::Intersection)?;
    if !tolerance.is_finite() || tolerance < 0.0 || parameters.iter().any(|t| !t.is_finite()) {
        return Err(SplitLineSegmentError::NonFinite);
    }
    if status == IntersectionStatus::Ambiguous {
        return Err(SplitLineSegmentError::AmbiguousIntersection);
    }
    let mut cuts: Vec<f64> = parameters
        .iter()
        .copied()
        .filter(|t| *t >= -tolerance && *t <= 1.0 + tolerance)
        .map(|t| t.clamp(0.0, 1.0))
        .collect();
    cuts.sort_by(f64::total_cmp);
    cuts.dedup_by(|a, b| (*a - *b).abs() <= tolerance.max(1e-12));
    cuts.retain(|t| *t > tolerance && *t < 1.0 - tolerance);
    if cuts.is_empty() {
        return Err(SplitLineSegmentError::NoInteriorIntersection);
    }
    let mut boundaries = Vec::with_capacity(cuts.len() + 2);
    boundaries.push(0.0);
    boundaries.extend(cuts.iter().copied());
    boundaries.push(1.0);
    let segments = boundaries
        .windows(2)
        .map(|w| LineSegment3D {
            start: lerp(line.start, line.end, w[0]),
            end: lerp(line.start, line.end, w[1]),
        })
        .collect();
    Ok(SplitLineSegmentResult {
        segments,
        parameters: boundaries,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::intersect_line_segment_nurbs_surface;

    fn plane() -> umlcad_v6_nurbs_surface_api::NurbsSurface3DDefinition {
        umlcad_v6_nurbs_surface_api::NurbsSurface3DDefinition::new(
            (1, 1),
            vec![
                Point3 { x: 0.0, y: 0.0, z: 0.0 },
                Point3 { x: 0.0, y: 1.0, z: 0.0 },
                Point3 { x: 1.0, y: 0.0, z: 0.0 },
                Point3 { x: 1.0, y: 1.0, z: 0.0 },
            ],
            vec![1.0; 4],
            (2, 2),
            vec![0.0, 0.0, 1.0, 1.0],
            vec![0.0, 0.0, 1.0, 1.0],
        )
    }

    #[test]
    fn unique_interior_cut_is_deterministic() {
        let line = LineSegment3D {
            start: Point3 { x: 0.5, y: 0.5, z: -1.0 },
            end: Point3 { x: 0.5, y: 0.5, z: 1.0 },
        };
        let intersection = intersect_line_segment_nurbs_surface(line, &plane(), 1e-10).unwrap();
        assert_eq!(intersection.status, IntersectionStatus::Unique);
        let parameter = intersection.points[0].line_parameter;
        let split = split_line_segment_at_intersections(
            line,
            intersection.status,
            &[parameter],
            1e-10,
        )
        .unwrap();
        assert_eq!(split.segments.len(), 2);
        assert_eq!(split.parameters, vec![0.0, 0.5, 1.0]);
        assert!((split.segments[0].end.z).abs() < 1e-10);
    }

    #[test]
    fn ambiguous_is_fail_closed() {
        let line = LineSegment3D {
            start: Point3 { x: 0.0, y: 0.0, z: 0.0 },
            end: Point3 { x: 1.0, y: 0.0, z: 0.0 },
        };
        assert_eq!(
            split_line_segment_at_intersections(
                line,
                IntersectionStatus::Ambiguous,
                &[0.5],
                1e-10,
            ),
            Err(SplitLineSegmentError::AmbiguousIntersection)
        );
    }
}
