//#![feature(unsafe_destructor)]

extern crate libc;
use libc::{c_void, size_t};
use core::prelude::*;

use core::mem;
use core::ptr;

use memory::malloc::{malloc, free};

// Define a wrapper around the handle returned by the foreign code.
// Unique<T> has the same semantics as Box<T>
pub struct Unique<T> {
    // It contains a single raw, mutable pointer to the object in question.
    ptr: *mut T
}

// Implement methods for creating and using the values in the box.

// NB: For simplicity and correctness, we require that T has kind Send
// (owned boxes relax this restriction, and can contain managed (GC) boxes).
// This is because, as implemented, the garbage collector would not know
// about any shared boxes stored in the malloc'd region of memory.
#[allow(dead_code)]
impl<T: Send> Unique<T> {
    pub fn empty() -> Unique<T> {
        unsafe {
            let ptr = malloc(mem::size_of::<T>() as size_t) as *mut T;
            kassert!(!ptr.is_null());

            ptr::zero_memory(ptr, 1);
            Unique{ptr: ptr}
        }
    }

    #[inline]
    pub fn new(value: T) -> Unique<T> {
        unsafe {
            let ptr = malloc(mem::size_of::<T>() as size_t) as *mut T;
            kassert!(!ptr.is_null());

            ptr::write(&mut *ptr, value);
            Unique{ptr: ptr}
        }
    }

    pub fn take<'r>(&'r mut self) -> T {
        unsafe {
            let value = ptr::read(self.ptr as *const T);
            free(self.ptr as *mut c_void);
            self.ptr = 0 as *mut T;
            value
        }
    }

    pub fn drop(&mut self) {
        unsafe {
            // Copy the object out from the pointer onto the stack,
            // where it is covered by normal Rust destructor semantics
            // and cleans itself up, if necessary
            ptr::read(self.ptr as *const T);

            // clean-up our allocation
            free(self.ptr as *mut c_void)
        }
    }
}

/*
// A key ingredient for safety, we associate a destructor with
// Unique<T>, making the struct manage the raw pointer: when the
// struct goes out of scope, it will automatically free the raw pointer.
//
// NB: This is an unsafe destructor, because rustc will not normally
// allow destructors to be associated with parameterized types, due to
// bad interaction with managed boxes. (With the Send restriction,
// we don't have this problem.) Note that the `#[unsafe_destructor]`
// feature gate is required to use unsafe destructors.
#[unsafe_destructor]
impl<T: Send> Drop for Unique<T> {
    fn drop(&mut self) {
        unsafe {
            // Copy the object out from the pointer onto the stack,
            // where it is covered by normal Rust destructor semantics
            // and cleans itself up, if necessary
            ptr::read(self.ptr as *const T);

            // clean-up our allocation
            free(self.ptr as *mut c_void)
        }
    }
}
*/

impl<T> Deref<T> for Unique<T> {
    #[inline]
    fn deref<'a>(&'a self) -> &'a T {
        unsafe { &*self.ptr }
    }
}

impl<T> DerefMut<T> for Unique<T> {
    #[inline]
    fn deref_mut<'a>(&'a mut self) -> &'a mut T {
        unsafe { &mut *self.ptr }
    }
}
