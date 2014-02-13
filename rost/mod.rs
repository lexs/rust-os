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

    exec::tasking::exec(thread2);
    thread1();
}

fn thread1() {
    loop {
        kprintln!("Hello from thread 1");
        exec::tasking::schedule();
    }
}

fn thread2() {
    loop {
        kprintln!("Hello from thread 2");
        exec::tasking::schedule();
    }
}

fn do_stuff() {
    //drivers::vga::clear_screen();
    drivers::vga::puts("Hello world!\n");

    extern { static _binary_hello_world_elf_start: u8; }
    let do_nothing = &_binary_hello_world_elf_start as *u8;

    if exec::elf::probe(do_nothing) {
        drivers::vga::puts("Found program!\n");

        exec::elf::exec(do_nothing);
    }


    loop {}
    /*
    let chars = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ ";
    let mut current: uint = 0;
    loop {
        drivers::timer::sleep(1000);
        drivers::vga::putch(chars[current] as char);
        current = (current + 1) % chars.len();
    }

    unsafe {
        let ptr = 0xa0000000 as *u32;
        let value = *ptr;
        kernel::console::write_num(value);
    }
    */
}


#[no_mangle]
pub extern fn trap_handler(regs: &mut arch::idt::Registers) {
    // TODO: Why?
    arch::idt::trap_handler(regs);
}
