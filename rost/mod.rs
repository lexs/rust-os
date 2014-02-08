#[crate_id = "rost#0.1"];

#[no_std];
#[feature(asm, macro_rules)];

extern mod core;

use core::container::Container;

#[macro_escape]
mod macros;

mod arch;
mod kernel;
mod drivers;
mod memory;
mod exec;

mod core2;

mod util;

fn alloc_stack(size: uint) {

}

#[no_mangle]
pub extern fn kernel_main() {
    arch::gdt::init();
    arch::irq::init();
    arch::idt::init();
    drivers::init();

    memory::init();

    exec::syscalls::init();

    drivers::vga::clear_screen();
    drivers::vga::puts("Hello world!\n");

    //loop {}

    exec::tasking::user_mode(test);
    /*exec::tasking::exec(do_stuff);
    exec::tasking::schedule();
    loop {}*/

    /*
    loop {
        // As we are running on a small temporary stack we'll defer all work
        // to a real process.
        exec::tasking::schedule();
    }*/
}

fn test() {
    unsafe {
        asm!("cli");
    }
    loop {}
}

fn do_stuff() {
    drivers::vga::clear_screen();
    drivers::vga::puts("Hello world!\n");

    extern { static _binary_hello_world_elf_start: u8; }
    let do_nothing = &_binary_hello_world_elf_start as *u8;

    if unsafe { exec::elf::probe(do_nothing) } {
        drivers::vga::puts("Found program!\n");

        unsafe { exec::elf::exec(do_nothing); }
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
pub extern fn isr_handler(regs: &mut arch::idt::Registers) {
    // TODO: Why?
    arch::idt::isr_handler(regs);
}
