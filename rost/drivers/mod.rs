pub mod vga;
pub mod keyboard;
pub mod timer;
pub mod serial;

pub fn init() {
    timer::init();
    keyboard::init();
    serial::init();
}
