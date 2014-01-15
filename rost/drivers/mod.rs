pub mod vga;
pub mod keyboard;
pub mod timer;

pub fn init() {
    timer::init();
    keyboard::init();
}