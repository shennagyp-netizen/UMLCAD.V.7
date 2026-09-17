#[repr(C)]
struct NativeShape {
    _private: [u8; 0],
}

#[link(name = "umlcad_occt_bridge")]
unsafe extern "C" {
    fn umlcad_occt_box(width: f64, depth: f64, height: f64, out_shape: *mut *mut NativeShape) -> i32;
    fn umlcad_occt_sphere(radius: f64, out_shape: *mut *mut NativeShape) -> i32;
    fn umlcad_occt_make_open_box_shell(width: f64, depth: f64, height: f64, out_shape: *mut *mut NativeShape) -> i32;
    fn umlcad_occt_make_non_manifold_box_shell(width: f64, depth: f64, height: f64, out_shape: *mut *mut NativeShape) -> i32;
    fn umlcad_occt_shape_pathology_evidence(input: *const NativeShape, out_valid: *mut i32, out_manifold: *mut i32, out_free_edges: *mut u32, out_multiple_edges: *mut u32, out_degenerated_edges: *mut u32, out_counts: *mut u32) -> i32;
    fn umlcad_occt_shape_delete(shape: *mut NativeShape);
}

const OK: i32 = 0;
const INVALID_ARGUMENT: i32 = 1;

unsafe fn evidence(shape: *const NativeShape) -> (i32, i32, u32, u32, u32, [u32; 5]) {
    let mut valid = 0_i32;
    let mut manifold = 0_i32;
    let mut free_edges = 0_u32;
    let mut multiple_edges = 0_u32;
    let mut degenerated_edges = 0_u32;
    let mut counts = [0_u32; 5];
    assert_eq!(unsafe { umlcad_occt_shape_pathology_evidence(shape, &mut valid, &mut manifold, &mut free_edges, &mut multiple_edges, &mut degenerated_edges, counts.as_mut_ptr()) }, OK);
    (valid, manifold, free_edges, multiple_edges, degenerated_edges, counts)
}

#[test]
fn native_pathology_snapshot_reports_regular_box_without_pathology() {
    let mut shape = std::ptr::null_mut();
    assert_eq!(unsafe { umlcad_occt_box(10.0, 20.0, 30.0, &mut shape) }, OK);
    assert!(!shape.is_null());
    assert_eq!(unsafe { evidence(shape) }, (1, 1, 0, 0, 0, [1, 1, 6, 12, 8]));
    unsafe { umlcad_occt_shape_delete(shape) };
}

#[test]
fn native_pathology_snapshot_distinguishes_valid_sphere_degeneracy_from_invalidity() {
    let mut shape = std::ptr::null_mut();
    assert_eq!(unsafe { umlcad_occt_sphere(10.0, &mut shape) }, OK);
    assert!(!shape.is_null());
    let (valid, manifold, free_edges, multiple_edges, degenerated_edges, counts) = unsafe { evidence(shape) };
    assert_eq!((valid, manifold, free_edges, multiple_edges), (1, 1, 0, 0));
    assert!(degenerated_edges > 0);
    assert_eq!(counts, [1, 1, 1, 3, 2]);
    unsafe { umlcad_occt_shape_delete(shape) };
}

#[test]
fn native_pathology_snapshot_exposes_open_shell_boundary_without_repair() {
    let mut shape = std::ptr::null_mut();
    assert_eq!(unsafe { umlcad_occt_make_open_box_shell(10.0, 20.0, 30.0, &mut shape) }, OK);
    assert!(!shape.is_null());
    let (valid, manifold, free_edges, multiple_edges, degenerated_edges, counts) = unsafe { evidence(shape) };
    assert_eq!(valid, 1);
    assert_eq!(manifold, 0);
    assert_eq!(free_edges, 4);
    assert_eq!(multiple_edges, 0);
    assert_eq!(degenerated_edges, 0);
    assert_eq!(counts, [0, 1, 5, 12, 8]);
    unsafe { umlcad_occt_shape_delete(shape) };
}

#[test]
fn native_pathology_snapshot_exposes_non_manifold_imported_shell_without_repair() {
    let mut shape = std::ptr::null_mut();
    assert_eq!(unsafe { umlcad_occt_make_non_manifold_box_shell(10.0, 20.0, 30.0, &mut shape) }, OK);
    assert!(!shape.is_null());
    let (valid, manifold, free_edges, multiple_edges, degenerated_edges, counts) = unsafe { evidence(shape) };
    assert_eq!(valid, 0);
    assert_eq!(manifold, 0);
    assert_eq!(free_edges, 0);
    assert_eq!(multiple_edges, 4);
    assert_eq!(degenerated_edges, 0);
    assert_eq!(counts, [0, 1, 6, 12, 8]);
    unsafe { umlcad_occt_shape_delete(shape) };
}

#[test]
fn native_pathology_snapshot_is_deterministic_and_initializes_outputs() {
    let mut shape = std::ptr::null_mut();
    assert_eq!(unsafe { umlcad_occt_box(2.0, 3.0, 4.0, &mut shape) }, OK);
    let first = unsafe { evidence(shape) };
    let second = unsafe { evidence(shape) };
    assert_eq!(first, second);
    let mut valid = 99_i32;
    let mut manifold = 99_i32;
    let mut free_edges = 99_u32;
    let mut multiple_edges = 99_u32;
    let mut degenerated_edges = 99_u32;
    let mut counts = [99_u32; 5];
    assert_eq!(unsafe { umlcad_occt_shape_pathology_evidence(std::ptr::null(), &mut valid, &mut manifold, &mut free_edges, &mut multiple_edges, &mut degenerated_edges, counts.as_mut_ptr()) }, INVALID_ARGUMENT);
    assert_eq!(valid, 0);
    assert_eq!(manifold, 0);
    assert_eq!(free_edges, 0);
    assert_eq!(multiple_edges, 0);
    assert_eq!(degenerated_edges, 0);
    assert_eq!(counts, [0; 5]);
    unsafe { umlcad_occt_shape_delete(shape) };
}
