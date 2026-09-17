use thiserror::Error;

use super::geometry::{Line, Point, EPSILON};

#[derive(Error, Clone, Copy, Debug, PartialEq)]
pub enum OffsetError {
    #[error("offset input contains non-finite values")]
    NonFinite,
    #[error("cannot offset a degenerate line")]
    Degenerate,
    #[error("offset result is not representable as finite geometry")]
    Overflow,
}

fn finite_point(p: Point) -> bool {
    p.x.is_finite() && p.y.is_finite()
}

fn length(line: Line) -> f64 {
    line.start.distance(line.end)
}

pub fn offset_line(line: Line, distance: f64) -> Result<Line, OffsetError> {
    if !finite_point(line.start) || !finite_point(line.end) || !distance.is_finite() {
        return Err(OffsetError::NonFinite);
    }
    let l = length(line);
    if !l.is_finite() {
        return Err(OffsetError::Overflow);
    }
    if l <= EPSILON {
        return Err(OffsetError::Degenerate);
    }
    if distance == 0.0 {
        return Ok(line);
    }
    let dx = line.end.x - line.start.x;
    let dy = line.end.y - line.start.y;
    let nx = -dy / l;
    let ny = dx / l;
    let ox = nx * distance;
    let oy = ny * distance;
    if !ox.is_finite() || !oy.is_finite() {
        return Err(OffsetError::Overflow);
    }
    let start = Point {
        x: line.start.x + ox,
        y: line.start.y + oy,
    };
    let end = Point {
        x: line.end.x + ox,
        y: line.end.y + oy,
    };
    if !finite_point(start) || !finite_point(end) {
        return Err(OffsetError::Overflow);
    }
    Ok(Line { start, end })
}
