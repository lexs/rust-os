#![crate_id = "rost#0.1"]
#![crate_type = "staticlib"]
#![no_std]
#![feature(asm, macro_rules, lang_items, phase, globs)]

#[phase(plugin, link)]

extern crate core;
extern crate libc;
extern crate rlibc;
extern crate alloc;

use libc::{size_t, c_void, c_int};

mod macros;

mod kernel;
mod arch;
mod drivers;
mod memory;
mod exec;
mod util;

mod std {
    // Macros refer to absolute paths
    pub use core::fmt;
    pub use core::option;
}

#[no_mangle]
pub extern fn kernel_main() {
    arch::gdt::init();
    arch::irq::init();
    arch::idt::init();
    drivers::init();

    memory::init();
    exec::tasking::init();

    exec::syscalls::init();

    drivers::vga::clear_screen();
    drivers::vga::puts("Hello world!\n");

    exec::tasking::exec(do_stuff);

    idle();
}

fn idle() -> ! {
    loop {
        exec::tasking::schedule();
    }   
}

fn do_stuff() -> ! {
    extern { static _binary_test_fork_elf_start: u8; }
    let do_nothing = &_binary_test_fork_elf_start as *const u8;

    if exec::elf::probe(do_nothing) {
        kprintln!("Found program");

        exec::elf::exec(do_nothing);
    }

    unreachable!();
}

#[allow(visible_private_types)]
#[no_mangle]
pub extern fn trap_handler(regs: &mut arch::idt::Registers) {
    // TODO: Why?
    arch::idt::trap_handler(regs);
}

// These are for liballoc
#[no_mangle]
pub unsafe extern fn malloc(size: size_t) -> *mut c_void {
    use memory::malloc::malloc;
    malloc(size)
}

#[no_mangle]
pub unsafe extern fn realloc(p: *mut c_void, size: size_t) -> *mut c_void {
    use memory::malloc::realloc;
    realloc(p, size)
}

#[no_mangle]
pub unsafe extern fn free(p: *mut c_void) {
    use memory::malloc::free;
    free(p)
}

#[no_mangle]
pub unsafe extern fn posix_memalign(memptr: *mut *mut c_void, align: size_t, size: size_t) -> c_int {
    kprintln!("Allocating {} bytes with align {}", size, align);
    *memptr = malloc(size);
    0
}

#[lang = "begin_unwind"]
extern fn begin_unwind(_: &core::fmt::Arguments,
                       file: &str,
                       line: uint) -> ! {
    kprintln!("begin_unwind in {}:{}", file, line);
    loop {}
}

#[lang = "stack_exhausted"] extern fn stack_exhausted() {}
#[lang = "eh_personality"] extern fn eh_personality() {}
