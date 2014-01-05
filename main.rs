#[no_std];

#[feature(asm, macro_rules)];

extern mod core;
use core::container::Container;

mod vga;
mod gdt;
mod idt;
mod io;
mod timer;
mod util;

#[no_mangle]
pub extern fn kernel_main() {
    gdt::init();
    idt::init();
    timer::init(50);

    vga::clear_screen();
    vga::puts("Hello world! ");

    let chars = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ ";
    let mut current: uint = 0;
    loop {
        sleep();
        vga::putch(chars[current] as char);
        current = (current + 1) % chars.len();
    }
}


#[no_mangle]
pub extern fn isr_handler(regs: idt::Registers) {
    // TODO: Why?
    idt::isr_handler(regs);
}

fn sleep() {
    let mut i: uint = 0;
    while i < 10000000 {
        i += 1;
        unsafe { asm!("nop"); }
    }
}
