//! Foreign Function Interface (FFI): talking to C.
//!
//! The Rust ABI is unstable by design; `extern "C"` pins the calling
//! convention to the platform C ABI so Rust and C can call each other.
//! Three pieces are demonstrated:
//!
//! - **`extern "C"` blocks**: declaring C symbols (`strlen` from the C
//!   runtime) so Rust can call them.
//! - **`unsafe`**: every FFI call is unsafe — the compiler trusts the caller
//!   to pass valid pointers and honor the C ABI.
//! - **`repr(C)`**: laying out a Rust struct exactly like C would, so
//!   structs can cross the boundary byte-for-byte.
//! - **Function pointers**: passing a Rust callback to a C function
//!   (`qsort`), with the C side calling back into Rust.
//!
//! # Safety
//!
//! Every call into C is `unsafe`: the compiler trusts us to pass valid
//! pointers, honor the ABI, and respect the C side's invariants.

use std::ffi::c_int;
#[cfg(any(unix, windows))]
use std::ffi::{c_char, c_void};

/// A `repr(C)` struct: field order and layout match C's `struct Point`.
///
/// ```c
/// struct Point { double x; double y; int id; };
/// ```
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    /// X coordinate.
    pub x: f64,
    /// Y coordinate.
    pub y: f64,
    /// Identifier.
    pub id: c_int,
}

impl Point {
    /// Creates a new point. O(1).
    #[must_use]
    pub const fn new(x: f64, y: f64, id: c_int) -> Self {
        Self { x, y, id }
    }
}

/// The size of a `repr(C)` struct on this platform (informational): fields
/// are packed per C rules, so this is 24 bytes on 64-bit (8 + 8 + 4 + padding).
#[must_use]
pub fn point_layout_size() -> usize {
    std::mem::size_of::<Point>()
}

// Declares C functions from the C runtime library.
// On Windows-MSVC the C runtime is linked by default; on other platforms the
// symbols come from libc. `strlen` returns the length of a NUL-terminated
// string; `qsort` sorts an array in place using a C comparator.
#[cfg(unix)]
unsafe extern "C" {
    fn strlen(s: *const c_char) -> usize;
    fn qsort(
        base: *mut c_void,
        num: usize,
        size: usize,
        compar: Option<unsafe extern "C" fn(*const c_void, *const c_void) -> c_int>,
    );
}

#[cfg(windows)]
unsafe extern "C" {
    // MSVC: the C runtime exports these with these exact names.
    #[link_name = "strlen"]
    fn msvc_strlen(s: *const c_char) -> usize;
    #[link_name = "qsort"]
    fn msvc_qsort(
        base: *mut c_void,
        num: usize,
        size: usize,
        compar: Option<unsafe extern "C" fn(*const c_void, *const c_void) -> c_int>,
    );
}

/// Calls the C `strlen` on a Rust string slice. O(n).
///
/// This shows the essential FFI pattern: make a NUL-terminated C string, pass
/// a raw pointer, and translate the result back.
///
/// # Panics
///
/// Panics if `s` contains an interior NUL byte (`CString` rejects those).
#[must_use]
pub fn c_strlen(s: &str) -> usize {
    #[cfg(any(unix, windows))]
    {
        // CString guarantees the NUL terminator C expects.
        let c_string = std::ffi::CString::new(s).expect("no interior NUL bytes");
        // SAFETY: c_string is a valid NUL-terminated buffer while this call runs.
        let len = unsafe {
            #[cfg(unix)]
            {
                strlen(c_string.as_ptr())
            }
            #[cfg(windows)]
            {
                msvc_strlen(c_string.as_ptr())
            }
        };
        // `s.len()` is the byte length; C sees the same bytes.
        debug_assert_eq!(len, s.len());
        len
    }
    #[cfg(target_arch = "wasm32")]
    {
        // The browser has no C runtime; the byte length is the answer.
        s.len()
    }
}

/// A C-compatible comparator for `qsort`: returns negative/zero/positive
/// according to the relative order of two `Point`s (by `id`).
///
/// The signature must match C's `int (*)(const void*, const void*)`.
#[cfg(any(unix, windows))]
extern "C" fn compare_points_by_id(a: *const c_void, b: *const c_void) -> c_int {
    // SAFETY: qsort passes pointers into the array we handed it.
    let a = unsafe { a.cast::<Point>().as_ref() };
    let b = unsafe { b.cast::<Point>().as_ref() };
    // SAFETY: qsort guarantees both pointers point at live elements.
    unsafe { a.unwrap_unchecked().id.cmp(&b.unwrap_unchecked().id) as c_int }
}

/// Sorts a slice of [`Point`]s by `id` using the C runtime's `qsort`, with a
/// Rust callback as the comparator. O(n log n).
///
/// Demonstrates: Rust -> C -> Rust callbacks, and how a `repr(C)` slice maps
/// to a C array (data pointer + element count).
pub fn qsort_points(points: &mut [Point]) {
    if points.is_empty() {
        return;
    }
    #[cfg(any(unix, windows))]
    // SAFETY: `points` is a valid array of contiguous Points for the duration
    // of the call; the comparator never escapes or aliases.
    unsafe {
        #[cfg(unix)]
        qsort(
            points.as_mut_ptr().cast::<c_void>(),
            points.len(),
            std::mem::size_of::<Point>(),
            Some(compare_points_by_id),
        );
        #[cfg(windows)]
        msvc_qsort(
            points.as_mut_ptr().cast::<c_void>(),
            points.len(),
            std::mem::size_of::<Point>(),
            Some(compare_points_by_id),
        );
    }
    #[cfg(target_arch = "wasm32")]
    {
        // No C runtime in the browser: fall back to Rust's own sort.
        // (Same semantics: ascending by id.)
        points.sort_unstable_by_key(|p| p.id);
    }
}

/// A Rust function declared with the C ABI: callable from C, and also usable
/// as a plain function pointer. O(1).
#[must_use]
pub extern "C" fn add_i32(a: i32, b: i32) -> i32 {
    a + b
}

/// Calls the extern "C" function through a typed function pointer — the exact
/// mechanism a C caller would use. O(1).
#[must_use]
pub fn call_via_c_fn_ptr(a: i32, b: i32) -> i32 {
    let fptr: extern "C" fn(i32, i32) -> i32 = add_i32;
    fptr(a, b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn c_strlen_matches_rust_len() {
        assert_eq!(c_strlen(""), 0);
        assert_eq!(c_strlen("hello"), 5);
        assert_eq!(c_strlen("héllo"), "héllo".len());
        assert_eq!(c_strlen("🦀"), "🦀".len());
    }

    #[test]
    #[should_panic(expected = "no interior NUL bytes")]
    fn c_strlen_rejects_interior_nul() {
        let _ = c_strlen("a\0b");
    }

    #[test]
    fn repr_c_layout_is_stable() {
        assert_eq!(point_layout_size(), 24); // 8 + 8 + 4 + 4 padding on 64-bit
        let p = Point::new(1.0, 2.0, 3);
        // The struct is Copy and trivially serializable as bytes.
        let bytes = unsafe {
            std::slice::from_raw_parts(
                std::ptr::addr_of!(p).cast::<u8>(),
                std::mem::size_of::<Point>(),
            )
        };
        assert_eq!(bytes.len(), 24);
    }

    #[test]
    fn qsort_sorts_points_by_id() {
        let mut points = vec![
            Point::new(3.0, 3.0, 3),
            Point::new(1.0, 1.0, 1),
            Point::new(2.0, 2.0, 2),
        ];
        qsort_points(&mut points);
        assert_eq!(
            points.iter().map(|p| p.id).collect::<Vec<_>>(),
            vec![1, 2, 3]
        );
        // x/y fields ride along untouched.
        assert!((points[0].x - 1.0).abs() < 1e-9);
    }

    #[test]
    fn qsort_empty_slice_is_noop() {
        let mut points: Vec<Point> = Vec::new();
        qsort_points(&mut points);
        assert!(points.is_empty());
    }

    #[test]
    fn extern_c_fn_and_pointer() {
        assert_eq!(add_i32(2, 3), 5);
        assert_eq!(call_via_c_fn_ptr(-1, 1), 0);
        let fptr: extern "C" fn(i32, i32) -> i32 = add_i32;
        assert_eq!(fptr(10, 32), 42);
    }
}
