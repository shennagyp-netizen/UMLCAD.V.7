use umlcad_kernel_rust::functions::{
    geometry::{Line, Point},
    offsets::{offset_line, OffsetError},
};

fn line(a: Point, b: Point) -> Line {
    Line { start: a, end: b }
}

#[test]
fn horizontal_line_offsets_up_for_positive_distance() {
    let source = line(
        Point { x: 0.0, y: 0.0 },
        Point { x: 10.0, y: 0.0 },
    );
    let result = offset_line(source, 2.0).unwrap();
    assert_eq!(result.start, Point { x: 0.0, y: 2.0 });
    assert_eq!(result.end, Point { x: 10.0, y: 2.0 });
}

#[test]
fn negative_distance_offsets_to_the_right_of_the_directed_line() {
    let source = line(
        Point { x: 0.0, y: 0.0 },
        Point { x: 0.0, y: 10.0 },
    );
    let result = offset_line(source, -3.0).unwrap();
    assert_eq!(result.start, Point { x: 3.0, y: 0.0 });
    assert_eq!(result.end, Point { x: 3.0, y: 10.0 });
}

#[test]
fn offset_preserves_length_and_direction() {
    let source = line(
        Point { x: -2.0, y: 4.0 },
        Point { x: 5.0, y: 13.0 },
    );
    let result = offset_line(source, 7.5).unwrap();
    let source_direction = source.end.sub(source.start);
    let result_direction = result.end.sub(result.start);
    assert!((source.start.distance(source.end) - result.start.distance(result.end)).abs() < 1e-12);
    let cross = source_direction.x * result_direction.y
        - source_direction.y * result_direction.x;
    assert!(cross.abs() < 1e-12);
    assert!(result_direction.dot(source_direction) > 0.0);
}

#[test]
fn every_point_on_source_has_the_declared_signed_left_normal_offset() {
    let source = line(
        Point { x: 2.0, y: -1.0 },
        Point { x: 8.0, y: 2.0 },
    );
    let distance = 4.0;
    let result = offset_line(source, distance).unwrap();
    let direction = source.end.sub(source.start);
    let length = direction.norm();
    let left = Point {
        x: -direction.y / length,
        y: direction.x / length,
    };
    let delta = result.start.sub(source.start);
    assert!((delta.dot(left) - distance).abs() < 1e-12);
    assert!(delta.dot(direction).abs() < 1e-12);
}

#[test]
fn zero_distance_is_exact_identity_and_source_is_unchanged() {
    let source = line(
        Point { x: -7.0, y: 2.0 },
        Point { x: 3.0, y: 9.0 },
    );
    assert_eq!(offset_line(source, 0.0).unwrap(), source);
    assert_eq!(source.start, Point { x: -7.0, y: 2.0 });
    assert_eq!(source.end, Point { x: 3.0, y: 9.0 });
}

#[test]
fn line_offset_rejects_exact_and_near_degenerate_geometry() {
    assert_eq!(
        offset_line(
            line(Point { x: 0.0, y: 0.0 }, Point { x: 0.0, y: 0.0 }),
            1.0,
        ),
        Err(OffsetError::Degenerate)
    );
    assert_eq!(
        offset_line(
            line(Point { x: 0.0, y: 0.0 }, Point { x: 5e-10, y: 0.0 }),
            1.0,
        ),
        Err(OffsetError::Degenerate)
    );
}

#[test]
fn line_offset_rejects_nonfinite_and_overflow_inputs() {
    assert_eq!(
        offset_line(
            line(Point { x: f64::NAN, y: 0.0 }, Point { x: 1.0, y: 0.0 }),
            1.0,
        ),
        Err(OffsetError::NonFinite)
    );
    assert_eq!(
        offset_line(
            line(Point { x: 0.0, y: 0.0 }, Point { x: 1.0, y: 0.0 }),
            f64::INFINITY,
        ),
        Err(OffsetError::NonFinite)
    );
    assert_eq!(
        offset_line(
            line(
                Point { x: -f64::MAX * 0.75, y: 0.0 },
                Point { x: f64::MAX * 0.75, y: 0.0 },
            ),
            f64::MAX * 0.5,
        ),
        Err(OffsetError::Overflow)
    );
}

#[test]
fn translation_commutes_with_line_offset() {
    let source = line(
        Point { x: 1.0, y: 2.0 },
        Point { x: 6.0, y: 5.0 },
    );
    let moved = line(
        Point { x: 101.0, y: -48.0 },
        Point { x: 106.0, y: -45.0 },
    );
    let offset_source = offset_line(source, 3.25).unwrap();
    let offset_moved = offset_line(moved, 3.25).unwrap();
    assert!((offset_moved.start.x - offset_source.start.x - 100.0).abs() < 1e-12);
    assert!((offset_moved.start.y - offset_source.start.y + 50.0).abs() < 1e-12);
    assert!((offset_moved.end.x - offset_source.end.x - 100.0).abs() < 1e-12);
    assert!((offset_moved.end.y - offset_source.end.y + 50.0).abs() < 1e-12);
}
