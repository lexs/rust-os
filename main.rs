#[no_std];

#[feature(asm, macro_rules)];

extern mod core;
use core::container::Container;

mod vga;
mod gdt;
mod irq;
mod idt;
mod io;
mod timer;
mod util;

#[no_mangle]
pub extern fn kernel_main() {
    gdt::init();
    irq::init();
    idt::init();
    timer::init(100);

    vga::clear_screen();
    vga::puts("Hello world! ");

    let chars = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ ";
    let mut current: uint = 0;
    loop {
        timer::sleep(1000);
        vga::putch(chars[current] as char);
        current = (current + 1) % chars.len();
    }
}


#[no_mangle]
pub extern fn isr_handler(regs: idt::Registers) {
    // TODO: Why?
    idt::isr_handler(regs);
}
