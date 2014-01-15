use core2::intrinsics;

/// Calculate the offset from a pointer.
/// The `count` argument is in units of T; e.g. a `count` of 3
/// represents a pointer offset of `3 * sizeof::<T>()` bytes.
#[inline]
pub unsafe fn offset<T>(ptr: *T, count: int) -> *T {
    intrinsics::offset(ptr, count)
}

/// Calculate the offset from a mut pointer. The count *must* be in bounds or
/// otherwise the loads of this address are undefined.
/// The `count` argument is in units of T; e.g. a `count` of 3
/// represents a pointer offset of `3 * sizeof::<T>()` bytes.
#[inline]
pub unsafe fn mut_offset<T>(ptr: *mut T, count: int) -> *mut T {
    intrinsics::offset(ptr as *T, count) as *mut T
}
