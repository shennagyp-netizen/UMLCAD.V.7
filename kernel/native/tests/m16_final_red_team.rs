use std::panic::{catch_unwind, AssertUnwindSafe};

use umlcad_kernel_rust::functions::{
    gpu::{compare_candidate_pairs, BackendKind},
    geometry::{Arc, Circle, Geometry, Line, Point},
    intersections::{
        circle_circle_2d, line_circle_2d, line_line_2d, IntersectionKind, Line2,
    },
    nurbs3d::{NurbsCurve3D, Point3},
    solver::scaled_damped_qr,
    spatial_accel::Aabb3,
    vec::{Vec2, Vec3},
};

fn assert_finite_vec3(value: Vec3) {
    assert!(value.x.is_finite() && value.y.is_finite() && value.z.is_finite());
}

#[test]
fn scale_family_aabb_predicate_remains_finite_and_translation_safe() {
    for scale in [1.0e-12, 1.0e-9, 1.0e-6, 1.0e-3, 1.0, 1.0e3, 1.0e6, 1.0e9, 1.0e12] {
        let a = Aabb3::new(
            Vec3::new(scale, scale, scale),
            Vec3::new(2.0 * scale, 2.0 * scale, 2.0 * scale),
        )
        .unwrap();
        let b = Aabb3::new(
            Vec3::new(1.5 * scale, 1.5 * scale, 1.5 * scale),
            Vec3::new(3.0 * scale, 3.0 * scale, 3.0 * scale),
        )
        .unwrap();
        assert!(a.intersects(b, 0.0));

        let far = Aabb3::new(
            Vec3::new(5.0 * scale, 5.0 * scale, 5.0 * scale),
            Vec3::new(6.0 * scale, 6.0 * scale, 6.0 * scale),
        )
        .unwrap();
        assert!(!a.intersects(far, 0.0));
    }
}

#[test]
fn extreme_finite_vectors_and_spheres_fail_closed_without_nan() {
    let values = [
        Vec3::new(f64::MAX * 0.25, 0.0, 0.0),
        Vec3::new(-f64::MAX * 0.25, 0.0, 0.0),
        Vec3::new(1.0e-300, -1.0e-300, 1.0e-300),
    ];

    for value in values {
        let bounds = Aabb3::new(value, value).unwrap();
        assert_finite_vec3(bounds.center());
        let sphere = bounds.bounding_sphere();
        assert!(sphere.center.is_finite());
        assert!(sphere.radius.is_finite());
    }

    let overflow = catch_unwind(AssertUnwindSafe(|| {
        let a = Vec3::new(f64::MAX, 0.0, 0.0);
        let b = Vec3::new(-f64::MAX, 0.0, 0.0);
        let _ = a.sub(b);
    }));
    assert!(overflow.is_ok());
}

#[test]
fn tangent_and_near_parallel_intersections_classify_without_panics() {
    let tangent = line_circle_2d(
        Line2 {
            origin: Vec2::new(-2.0, 1.0),
            direction: Vec2::new(4.0, 0.0),
        },
        Circle {
            center: Point { x: 0.0, y: 0.0 },
            radius: 1.0,
        },
        1.0e-12,
    );
    assert_eq!(tangent.kind, IntersectionKind::Tangent);
    assert!(tangent.points.iter().all(|p| p.is_finite()));

    let near_parallel = line_line_2d(
        Line2 {
            origin: Vec2::new(0.0, 0.0),
            direction: Vec2::new(1.0, 1.0),
        },
        Line2 {
            origin: Vec2::new(0.0, 1.0),
            direction: Vec2::new(1.0, 1.0 + 1.0e-14),
        },
        1.0e-12,
    );
    assert!(matches!(
        near_parallel.kind,
        IntersectionKind::Parallel
            | IntersectionKind::Coincident
            | IntersectionKind::Indeterminate
            | IntersectionKind::Point
    ));
    assert!(near_parallel.points.iter().all(|p| p.is_finite()));

    let coincident = circle_circle_2d(
        Circle {
            center: Point { x: 0.0, y: 0.0 },
            radius: 2.0,
        },
        Circle {
            center: Point { x: 0.0, y: 0.0 },
            radius: 2.0,
        },
        1.0e-12,
    );
    assert_eq!(coincident.kind, IntersectionKind::Coincident);
}

#[test]
fn nonfinite_intersection_inputs_do_not_panic() {
    let result = catch_unwind(AssertUnwindSafe(|| {
        line_circle_2d(
            Line2 {
                origin: Vec2::new(f64::NAN, 0.0),
                direction: Vec2::new(1.0, 0.0),
            },
            Circle {
                center: Point { x: 0.0, y: 0.0 },
                radius: 1.0,
            },
            1.0e-12,
        )
    }));
    assert!(result.is_ok());
    assert_eq!(result.unwrap().kind, IntersectionKind::Degenerate);
}

#[test]
fn extreme_positive_nurbs_weights_remain_finite_and_in_domain() {
    let curve = NurbsCurve3D::new(
        2,
        vec![
            Point3 { x: 0.0, y: 0.0, z: 0.0 },
            Point3 { x: 1.0, y: 0.5, z: -1.0 },
            Point3 { x: 2.0, y: 0.0, z: 0.0 },
        ],
        vec![1.0e300, 1.0e-300, 1.0e300],
        vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
    );
    assert!(curve.validate().is_ok());
    for u in [0.0, 0.1, 0.25, 0.5, 0.75, 0.9, 1.0] {
        let point = curve.point_at(u).unwrap();
        assert!(point.x.is_finite() && point.y.is_finite() && point.z.is_finite());
    }
}

#[test]
fn invalid_nurbs_domain_and_weights_are_fail_closed() {
    let curve = NurbsCurve3D::new(
        2,
        vec![
            Point3 { x: 0.0, y: 0.0, z: 0.0 },
            Point3 { x: 1.0, y: 0.0, z: 0.0 },
            Point3 { x: 2.0, y: 0.0, z: 0.0 },
        ],
        vec![1.0, 0.0, 1.0],
        vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
    );
    assert!(curve.validate().is_err());
    assert!(curve.point_at(f64::NAN).is_err());
    assert!(curve.point_at(-1.0e-9).is_err());
    assert!(curve.point_at(1.0 + 1.0e-9).is_err());
}

#[test]
fn rank_deficient_solver_and_gpu_ordering_fail_closed() {
    let report = scaled_damped_qr(
        &[vec![1.0, 1.0], vec![2.0, 2.0]],
        &[1.0, 2.0],
        0.0,
        1.0e-10,
    )
    .unwrap();
    assert_eq!(report.rank, 1);
    assert_eq!(report.degrees_of_freedom, 1);
    assert!(report.delta.iter().all(|value| value.is_finite()));

    let conformance = compare_candidate_pairs(
        BackendKind::Cuda,
        &[(0, 1), (1, 2)],
        &[(1, 2), (0, 1)],
        &[(0, 1), (1, 2)],
    )
    .unwrap();
    assert!(conformance.conforms());
    assert!(conformance.exact_match());
}

#[test]
fn malformed_gpu_candidate_sets_are_rejected() {
    assert_eq!(
        compare_candidate_pairs(
            BackendKind::Metal,
            &[(0, 1)],
            &[(0, 1), (0, 1)],
            &[(0, 1)],
        ),
        Err(umlcad_kernel_rust::functions::gpu::GpuError::ConformanceDuplicateCandidate)
    );
    assert_eq!(
        compare_candidate_pairs(
            BackendKind::Cuda,
            &[(0, 1)],
            &[(1, 0)],
            &[(1, 0)],
        ),
        Err(umlcad_kernel_rust::functions::gpu::GpuError::ConformanceInvalidCandidateSet)
    );
}
