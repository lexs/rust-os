#[crate_id = "rost#0.1"];

#[no_std];
#[feature(asm, macro_rules)];

extern mod core;

use core::container::Container;

mod macros;

mod kernel;
mod arch;
mod drivers;
mod memory;
mod exec;

mod core2;

mod util;

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

    do_stuff();
}

fn do_stuff() {
    extern { static _binary_test_fork_elf_start: u8; }
    let do_nothing = &_binary_test_fork_elf_start as *u8;

    if exec::elf::probe(do_nothing) {
        drivers::vga::puts("Found program!\n");

        exec::elf::exec(do_nothing);
    }

    loop {}
}


#[no_mangle]
pub extern fn trap_handler(regs: &mut arch::idt::Registers) {
    // TODO: Why?
    arch::idt::trap_handler(regs);
}
