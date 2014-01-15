use core::container::Container;

use drivers::vga;

pub mod console;

static PANIC_MSG: &'static str = "PANIC: ";

pub fn panic(msg: &str) {
    vga::clear_screen();

    let len = PANIC_MSG.len() + msg.len();
    let x = vga::COLS / 2 - len / 2;
    vga::move_cursor(x, vga::ROWS / 2);

    vga::puts(PANIC_MSG);
    vga::puts(msg);

    unsafe { asm!("cli"); }
    loop {}
}
