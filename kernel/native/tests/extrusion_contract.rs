use umlcad_kernel_rust::math::brep::{
    extrude_convex_planar_region, PlanarRegion3, SolidPointClass,
};
use umlcad_kernel_rust::math::tolerance::Tolerance;
use umlcad_kernel_rust::math::vec::{Vec2, Vec3};

fn rectangle_region() -> PlanarRegion3 {
    PlanarRegion3 {
        origin: Vec3::new(10.0, 20.0, 30.0),
        u_dir: Vec3::new(1.0, 0.0, 0.0),
        v_dir: Vec3::new(0.0, 1.0, 0.0),
        outer: vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(2.0, 0.0),
            Vec2::new(2.0, 3.0),
            Vec2::new(0.0, 3.0),
        ],
        holes: Vec::new(),
    }
}

#[test]
fn convex_planar_rectangle_extrudes_to_closed_solid() {
    let tolerance = Tolerance::new(1.0e-9, 1.0e-9).unwrap();
    let solid = extrude_convex_planar_region(rectangle_region(), 4.0, tolerance).unwrap();

    assert_eq!(solid.vertices.len(), 8);
    assert_eq!(solid.edges.len(), 12);
    assert_eq!(solid.coedges.len(), 24);
    assert_eq!(solid.wires.len(), 6);
    assert_eq!(solid.faces.len(), 6);

    solid.validate(tolerance).unwrap();

    assert!((solid.volume(tolerance).unwrap() - 24.0).abs() < 1.0e-9);
    assert!((solid.surface_area(tolerance).unwrap() - 52.0).abs() < 1.0e-9);

    let centroid = solid.centroid(tolerance).unwrap();
    assert!((centroid.x - 11.0).abs() < 1.0e-9);
    assert!((centroid.y - 21.5).abs() < 1.0e-9);
    assert!((centroid.z - 32.0).abs() < 1.0e-9);

    assert_eq!(
        solid.classify_point(
            Vec3::new(11.0, 21.5, 32.0),
            tolerance,
        ),
        Ok(SolidPointClass::Inside)
    );
}

#[test]
fn convex_planar_profile_rejects_holes_in_current_certified_domain() {
    let tolerance = Tolerance::new(1.0e-9, 1.0e-9).unwrap();
    let mut region = rectangle_region();
    region.holes.push(vec![
        Vec2::new(0.5, 0.5),
        Vec2::new(1.0, 0.5),
        Vec2::new(1.0, 1.0),
        Vec2::new(0.5, 1.0),
    ]);

    assert!(extrude_convex_planar_region(region, 4.0, tolerance).is_err());
}
