#[crate_id = "rost#0.1"];

#[no_std];
#[feature(asm, macro_rules)];

extern mod core;

use core::container::Container;

mod arch;
mod kernel;
mod drivers;
mod memory;

mod core2;

mod util;

#[no_mangle]
pub extern fn kernel_main() {
    arch::gdt::init();
    arch::irq::init();
    arch::idt::init();
    drivers::init();

    drivers::vga::clear_screen();
    drivers::vga::puts("Hello world! ");

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

    loop {}
}


#[no_mangle]
pub extern fn isr_handler(regs: &arch::idt::Registers) {
    // TODO: Why?
    arch::idt::isr_handler(regs);
}
