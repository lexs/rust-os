use core::ptr::copy_nonoverlapping_memory;
use core2::global_heap::{c_void, c_char, size_t, uintptr_t};

use memory;

static PAGE_SIZE: u32 = 0x1000;

static mut heap: u32 = 0xd0000000;
static mut buffer: u32 = 0;

pub unsafe fn malloc(size: size_t) -> *mut c_void {
    while buffer < size {
        memory::map(heap + buffer, PAGE_SIZE, memory::PRESENT | memory::WRITE | memory::USER);
        buffer += PAGE_SIZE;
    }

    let ptr = heap;
    heap += size;
    buffer -= size;

    ptr as *mut c_void
}

pub unsafe fn realloc(p: *mut c_void, size: size_t) -> *mut c_void {
    let ptr = malloc(size);
    copy_nonoverlapping_memory(ptr, p as *c_void, size as uint);
    free(p);
    ptr
}

pub unsafe fn free(p: *mut c_void) {
    // Do nothing :(
}
