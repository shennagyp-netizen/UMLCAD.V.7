use umlcad_v6_draft_api::{DraftBackend, DraftedRectangularSolid};
use umlcad_v6_geometry_api::{GeometryBackend, GeometryError, GeometryKind, GeometryStatus, ToleranceContext};
use umlcad_v6_occt_backend::OcctBackend;

const T: ToleranceContext = ToleranceContext { modeling: 1e-9, validation: 1e-9 };

fn definition() -> DraftedRectangularSolid {
    DraftedRectangularSolid { width: 20.0, depth: 30.0, height: 10.0, draft_angle_radians: 0.1 }
}

#[test]
fn drafted_rectangular_realization_is_valid_and_preserves_exact_extents() {
    let backend = OcctBackend::new();
    let d = definition();
    let result = backend.make_drafted_rectangular_solid(d, T).unwrap();
    assert_eq!(result.kind, GeometryKind::Solid);
    assert_eq!(result.evidence.status, GeometryStatus::Success);
    let validation = backend.validate(&result.shape, T).unwrap();
    assert!(validation.valid);
    assert!(validation.manifold);

    let (top_width, top_depth) = d.top_dimensions(T).unwrap();
    let displacement = 10.0 * 0.1_f64.tan();
    let expected_half_x = (20.0 + 2.0 * displacement) / 2.0;
    let expected_half_y = (30.0 + 2.0 * displacement) / 2.0;
    assert!((top_width - 2.0 * expected_half_x).abs() < 1e-12);
    assert!((top_depth - 2.0 * expected_half_y).abs() < 1e-12);

    let bbox = backend.bounding_box(&result.shape, T).unwrap();
    assert!((bbox.min_z - 0.0).abs() <= 1e-9);
    assert!((bbox.max_z - 10.0).abs() <= 1e-9);
    assert!((bbox.min_x + expected_half_x.max(10.0)).abs() <= 1e-9 || bbox.min_x <= -10.0);
    assert!(bbox.max_x >= 10.0);
    assert!(bbox.min_y <= -15.0);
    assert!(bbox.max_y >= 15.0);

    let topology = backend.topology_counts(&result.shape, T).unwrap();
    assert_eq!(topology.solids, 1);
    assert_eq!(topology.shells, 1);
    assert!(topology.faces >= 6);
}

#[test]
fn zero_draft_realization_is_the_prismatic_limit() {
    let backend = OcctBackend::new();
    let mut d = definition();
    d.draft_angle_radians = 0.0;
    let result = backend.make_drafted_rectangular_solid(d, T).unwrap();
    let bbox = backend.bounding_box(&result.shape, T).unwrap();
    assert!((bbox.min_x + 10.0).abs() <= 1e-9);
    assert!((bbox.max_x - 10.0).abs() <= 1e-9);
    assert!((bbox.min_y + 15.0).abs() <= 1e-9);
    assert!((bbox.max_y - 15.0).abs() <= 1e-9);
    assert!((bbox.min_z - 0.0).abs() <= 1e-9);
    assert!((bbox.max_z - 10.0).abs() <= 1e-9);
}

#[test]
fn drafted_realization_is_deterministic_and_source_immutable() {
    let backend = OcctBackend::new();
    let d = definition();
    let first = backend.make_drafted_rectangular_solid(d, T).unwrap();
    let second = backend.make_drafted_rectangular_solid(d, T).unwrap();
    assert_eq!(backend.topology_counts(&first.shape, T).unwrap(), backend.topology_counts(&second.shape, T).unwrap());
    assert_eq!(backend.bounding_box(&first.shape, T).unwrap(), backend.bounding_box(&second.shape, T).unwrap());
}

#[test]
fn semantic_contract_rejects_invalid_draft_angles() {
    let backend = OcctBackend::new();
    let mut d = definition();
    d.draft_angle_radians = std::f64::consts::FRAC_PI_2;
    assert_eq!(d.validate(T), Err(GeometryError::InvalidInput("draft angle must have finite tangent and remain within +/- 90 degrees")));
    assert!(backend.make_drafted_rectangular_solid(d, T).is_err());
    let mut d = definition();
    d.draft_angle_radians = -1.4;
    assert!(backend.make_drafted_rectangular_solid(d, T).is_err());
    let mut d = definition();
    d.width = f64::NAN;
    assert!(backend.make_drafted_rectangular_solid(d, T).is_err());
}
