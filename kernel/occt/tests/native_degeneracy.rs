#[repr(C)]
struct NativeShape {
    _private: [u8; 0],
}

#[link(name = "umlcad_occt_bridge")]
unsafe extern "C" {
    fn umlcad_occt_box(
        width: f64,
        depth: f64,
        height: f64,
        out_shape: *mut *mut NativeShape,
    ) -> i32;
    fn umlcad_occt_sphere(radius: f64, out_shape: *mut *mut NativeShape) -> i32;
    fn umlcad_occt_shape_degenerated_edge_count(
        input: *const NativeShape,
        out_count: *mut u32,
    ) -> i32;
    fn umlcad_occt_shape_validate(
        input: *const NativeShape,
        valid: *mut i32,
        manifold: *mut i32,
    ) -> i32;
    fn umlcad_occt_shape_delete(shape: *mut NativeShape);
}

const OK: i32 = 0;
const INVALID_ARGUMENT: i32 = 1;

unsafe fn native_degenerated_count(shape: *const NativeShape) -> u32 {
    let mut count = 0_u32;
    assert_eq!(
        unsafe { umlcad_occt_shape_degenerated_edge_count(shape, &mut count) },
        OK
    );
    count
}

unsafe fn native_valid(shape: *const NativeShape) -> i32 {
    let mut valid = 0_i32;
    let mut manifold = 0_i32;
    assert_eq!(
        unsafe { umlcad_occt_shape_validate(shape, &mut valid, &mut manifold) },
        OK
    );
    valid
}

#[test]
fn regular_box_has_no_native_degenerated_edges() {
    let mut shape = std::ptr::null_mut();
    assert_eq!(unsafe { umlcad_occt_box(10.0, 20.0, 30.0, &mut shape) }, OK);
    assert!(!shape.is_null());

    assert_eq!(unsafe { native_degenerated_count(shape) }, 0);
    assert_eq!(unsafe { native_valid(shape) }, 1);

    unsafe { umlcad_occt_shape_delete(shape) };
}

#[test]
fn sphere_exposes_native_degenerated_edges_without_being_invalid() {
    let mut shape = std::ptr::null_mut();
    assert_eq!(unsafe { umlcad_occt_sphere(10.0, &mut shape) }, OK);
    assert!(!shape.is_null());

    assert!(unsafe { native_degenerated_count(shape) } > 0);
    assert_eq!(unsafe { native_valid(shape) }, 1);

    unsafe { umlcad_occt_shape_delete(shape) };
}

#[test]
fn native_degenerated_edge_count_rejects_null_shape() {
    let mut count = 123_u32;
    assert_eq!(
        unsafe { umlcad_occt_shape_degenerated_edge_count(std::ptr::null(), &mut count) },
        INVALID_ARGUMENT
    );
    assert_eq!(count, 0);
}
