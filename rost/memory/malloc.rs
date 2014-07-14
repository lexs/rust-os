use core::ptr::copy_nonoverlapping_memory;
use libc::{size_t, c_void};

use memory;

static PAGE_SIZE: u32 = 0x1000;

static mut heap: u32 = 0xd0000000;
static mut buffer: u32 = 0;

pub unsafe fn malloc(size: size_t) -> *mut c_void {
    klog!("Allocating {} bytes", size);

    while buffer < size {
        memory::map(heap + buffer, PAGE_SIZE, memory::WRITE);
        buffer += PAGE_SIZE;
    }

    let ptr = heap;
    heap += size;
    buffer -= size;

    ptr as *mut c_void
}

pub unsafe fn realloc(p: *mut c_void, size: size_t) -> *mut c_void {
    let ptr = malloc(size);
    copy_nonoverlapping_memory(ptr, p as *const c_void, size as uint);
    free(p);
    ptr
}

pub unsafe fn free(_: *mut c_void) {
    // Do nothing :(
}
