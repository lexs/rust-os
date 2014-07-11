use arch::io;
use arch::idt;
use arch::irq;
use drivers::vga;

use core::intrinsics::{volatile_load, volatile_store};

static HZ: u32 = 100;

static mut tick: u32 = 0;

pub fn init() {
    irq::register_handler(0, timer_handler);

    let divisor = 1193180 / HZ;

    io::write_port(0x43, 0x36);

    let low = (divisor & 0xff) as u8;
    let high = (divisor >> 8 & 0xff) as u8;

    // Send the frequency divisor.
    io::write_port(0x40, low);
    io::write_port(0x40, high);
}

#[inline(always)]
pub fn read_ticks() -> u32 {
    unsafe { volatile_load(&tick) }
}

fn increment_ticks() -> u32 {
    unsafe {
        let current = volatile_load(&tick) + 1;
        volatile_store(&mut tick, current);
        current
    }
}

pub fn sleep(duration: u32) {
    let target = read_ticks() + duration / 100;
    while read_ticks() < target {}
}

fn timer_handler(_: &mut idt::Registers) {
    if increment_ticks() % HZ == 0 {
        vga::puts("\nOne second has passed\n");
    }
}
