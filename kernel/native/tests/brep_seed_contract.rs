use umlcad_native::math::brep::{
    build_axis_aligned_box_solid, AxisAlignedBox, SolidPointClass,
};
use umlcad_native::math::tolerance::Tolerance;
use umlcad_native::math::vec::Vec3;

#[test]
fn axis_aligned_box_seed_is_closed_oriented_and_measurable() {
    let tolerance = Tolerance::new(1.0e-9, 1.0e-9).unwrap();
    let bounds = AxisAlignedBox {
        min: Vec3::new(10.0, 20.0, 30.0),
        max: Vec3::new(12.0, 23.0, 34.0),
    };

    let solid = build_axis_aligned_box_solid(bounds, tolerance).unwrap();

    assert_eq!(solid.vertices.len(), 8);
    assert_eq!(solid.edges.len(), 12);
    assert_eq!(solid.coedges.len(), 24);
    assert_eq!(solid.wires.len(), 6);
    assert_eq!(solid.faces.len(), 6);
    assert_eq!(solid.shells.len(), 1);

    solid.validate(tolerance).unwrap();

    let volume = solid.volume(tolerance).unwrap();
    let area = solid.surface_area(tolerance).unwrap();
    let centroid = solid.centroid(tolerance).unwrap();

    assert!((volume - 24.0).abs() <= 1.0e-9);
    assert!((area - 52.0).abs() <= 1.0e-9);
    assert!((centroid.x - 11.0).abs() <= 1.0e-9);
    assert!((centroid.y - 21.5).abs() <= 1.0e-9);
    assert!((centroid.z - 32.0).abs() <= 1.0e-9);
}

#[test]
fn axis_aligned_box_seed_preserves_exact_face_topology_keys() {
    let tolerance = Tolerance::new(1.0e-9, 1.0e-9).unwrap();
    let bounds = AxisAlignedBox {
        min: Vec3::new(0.0, 0.0, 0.0),
        max: Vec3::new(1.0, 2.0, 3.0),
    };

    let solid = build_axis_aligned_box_solid(bounds, tolerance).unwrap();

    let keys = solid
        .faces
        .iter()
        .map(|face| face.id.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        keys,
        vec![
            "f_bottom",
            "f_top",
            "f_back",
            "f_front",
            "f_left",
            "f_right"
        ]
    );
}

#[test]
fn axis_aligned_box_seed_classifies_points_without_tessellation() {
    let tolerance = Tolerance::new(1.0e-9, 1.0e-9).unwrap();
    let bounds = AxisAlignedBox {
        min: Vec3::new(0.0, 0.0, 0.0),
        max: Vec3::new(2.0, 3.0, 4.0),
    };

    let solid = build_axis_aligned_box_solid(bounds, tolerance).unwrap();

    assert_eq!(
        solid.classify_point(
            Vec3::new(1.0, 1.5, 2.0),
            Vec3::new(1.0, 0.37, 0.11),
            tolerance
        ),
        Ok(SolidPointClass::Inside)
    );

    assert_eq!(
        solid.classify_point(
            Vec3::new(2.0, 1.5, 2.0),
            Vec3::new(1.0, 0.37, 0.11),
            tolerance
        ),
        Ok(SolidPointClass::OnBoundary)
    );

    assert_eq!(
        solid.classify_point(
            Vec3::new(3.0, 1.5, 2.0),
            Vec3::new(1.0, 0.37, 0.11),
            tolerance
        ),
        Ok(SolidPointClass::Outside)
    );
}
