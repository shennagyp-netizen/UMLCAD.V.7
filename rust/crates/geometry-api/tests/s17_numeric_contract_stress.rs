use umlcad_v6_geometry_api::{
    BoundingBox, EdgeDescriptor, FaceDescriptor, ToleranceContext, VertexDescriptor,
};

#[test]
fn repeated_tolerance_validation_is_stable_at_numeric_edges() {
    let cases = [
        ToleranceContext { modeling: 0.0, validation: 0.0 },
        ToleranceContext { modeling: 1e-12, validation: 1e-12 },
        ToleranceContext { modeling: 1e-6, validation: 1e-6 },
        ToleranceContext { modeling: f64::MAX, validation: f64::MAX },
        ToleranceContext { modeling: -1e-12, validation: 0.0 },
        ToleranceContext { modeling: f64::NAN, validation: 0.0 },
        ToleranceContext { modeling: 0.0, validation: f64::INFINITY },
    ];

    for case in cases {
        let expected = case.validate();
        for _ in 0..10_000 {
            assert_eq!(case.validate(), expected);
        }
    }
}

#[test]
fn repeated_descriptor_validation_is_immutable_and_deterministic() {
    let bbox = BoundingBox {
        min_x: -1000.0,
        min_y: -1e-9,
        min_z: 0.0,
        max_x: 1000.0,
        max_y: 1e-9,
        max_z: 1e-6,
    };
    let face = FaceDescriptor { area: 2000.0, bounds: bbox, boundary_edge_count: 4 };
    let edge = EdgeDescriptor {
        length: 2000.0,
        bounds: bbox,
        face_use_count: 2,
        vertex_use_count: 2,
    };
    let vertex = VertexDescriptor { x: 1000.0, y: 1e-9, z: 1e-6, edge_use_count: 2, face_use_count: 3 };

    for _ in 0..10_000 {
        assert!(bbox.validate().is_ok());
        assert!(face.validate().is_ok());
        assert!(edge.validate().is_ok());
        assert!(vertex.validate().is_ok());
    }
}

#[test]
fn repeated_tolerance_matching_is_stable_for_large_and_small_coordinates() {
    let left = VertexDescriptor { x: 1e9, y: 1e-9, z: -1e6, edge_use_count: 2, face_use_count: 3 };
    let inside = VertexDescriptor { x: 1e9 + 1e3, y: 1.0000005e-9, z: -1e6 + 0.5, edge_use_count: 2, face_use_count: 3 };
    let outside = VertexDescriptor { x: 1e9 + 1e4, y: 1.1e-9, z: -1e6 + 5.0, edge_use_count: 2, face_use_count: 3 };

    for _ in 0..10_000 {
        assert!(left.matches_within(inside, 1e-6));
        assert!(!left.matches_within(outside, 1e-6));
        assert!(!left.matches_within(inside, f64::NAN));
        assert!(!left.matches_within(inside, -1.0));
    }
}
