pub mod vga;
pub mod keyboard;
pub mod timer;
pub mod serial;
pub mod ansi;

pub fn init() {
    timer::init();
    keyboard::init();
    serial::init();
}
