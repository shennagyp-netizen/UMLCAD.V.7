#[repr(C)]
struct NativeShape {
    _private: [u8; 0],
}

#[link(name = "umlcad_occt_bridge")]
unsafe extern "C" {
    fn umlcad_occt_sew_box_faces(
        width: f64,
        depth: f64,
        height: f64,
        tolerance: f64,
        out_shape: *mut *mut NativeShape,
        out_free_edges: *mut u32,
        out_multiple_edges: *mut u32,
        out_degenerated: *mut u32,
    ) -> i32;
    fn umlcad_occt_shape_topology_counts(input: *const NativeShape, out_counts: *mut u32) -> i32;
    fn umlcad_occt_shape_delete(shape: *mut NativeShape);
}

const OK: i32 = 0;
const INVALID_ARGUMENT: i32 = 1;

#[test]
fn native_box_faces_sew_into_valid_closed_shell() {
    let mut shape = std::ptr::null_mut();
    let mut free_edges = 0;
    let mut multiple_edges = 0;
    let mut degenerated = 0;

    let status = unsafe {
        umlcad_occt_sew_box_faces(
            10.0,
            20.0,
            30.0,
            1e-6,
            &mut shape,
            &mut free_edges,
            &mut multiple_edges,
            &mut degenerated,
        )
    };

    assert_eq!(status, OK);
    assert!(!shape.is_null());
    assert_eq!(free_edges, 0);
    assert_eq!(multiple_edges, 0);
    assert_eq!(degenerated, 0);

    let mut counts = [0_u32; 5];
    assert_eq!(unsafe { umlcad_occt_shape_topology_counts(shape, counts.as_mut_ptr()) }, OK);
    assert_eq!(counts, [0, 1, 6, 12, 8]);

    unsafe { umlcad_occt_shape_delete(shape) };
}

#[test]
fn native_sewing_rejects_invalid_tolerance_before_occt() {
    let mut shape = std::ptr::null_mut();
    let mut free_edges = 0;
    let mut multiple_edges = 0;
    let mut degenerated = 0;

    let status = unsafe {
        umlcad_occt_sew_box_faces(
            10.0,
            20.0,
            30.0,
            f64::NAN,
            &mut shape,
            &mut free_edges,
            &mut multiple_edges,
            &mut degenerated,
        )
    };

    assert_eq!(status, INVALID_ARGUMENT);
    assert!(shape.is_null());
}
