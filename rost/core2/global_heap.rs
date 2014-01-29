use core::fail::abort;

use memory::malloc::{malloc, realloc, free};

pub type uintptr_t = uint;
pub type c_char = i8;
pub type size_t = u32;
pub enum c_void {}

/*
extern {
    pub fn malloc(size: size_t) -> *mut c_void;
    pub fn realloc(p: *mut c_void, size: size_t) -> *mut c_void;
    pub fn free(p: *c_void);
}
*/

/// A wrapper around libc::malloc, aborting on out-of-memory
#[inline]
pub unsafe fn malloc_raw(size: uint) -> *mut c_void {
    // `malloc(0)` may allocate, but it may also return a null pointer
    // http://pubs.opengroup.org/onlinepubs/9699919799/functions/malloc.html
    if size == 0 {
        0 as *mut c_void
    } else {
        let p = malloc(size as size_t);
        if p == 0 as *mut c_void {
            // we need a non-allocating way to print an error here
            abort();
        }
        p
    }
}

/// A wrapper around libc::realloc, aborting on out-of-memory
#[inline]
pub unsafe fn realloc_raw(ptr: *mut c_void, size: uint) -> *mut c_void {
    // `realloc(ptr, 0)` may allocate, but it may also return a null pointer
    // http://pubs.opengroup.org/onlinepubs/9699919799/functions/realloc.html
    if size == 0 {
        free(ptr);
        0 as *mut c_void
    } else {
        let p = realloc(ptr, size as size_t);
        if p == 0 as *mut c_void {
            // we need a non-allocating way to print an error here
            abort();
        }
        p
    }
}

/// The allocator for unique pointers without contained managed pointers.
#[cfg(not(test))]
#[lang="exchange_malloc"]
#[inline]
pub unsafe fn exchange_malloc(size: uintptr_t) -> *c_char {
    malloc_raw(size as uint) as *c_char
}

// NB: Calls to free CANNOT be allowed to fail, as throwing an exception from
// inside a landing pad may corrupt the state of the exception handler.
#[cfg(not(test))]
#[lang="exchange_free"]
#[inline]
pub unsafe fn exchange_free_(ptr: *c_char) {
    free(ptr as *mut c_void);
}
