use io;
use idt;
use irq;
use vga;
use util;

static mut tick: u32 = 0;

pub fn init(frequency: u32) {
    irq::register_handler(0, callback);

    let divisor = 1193180 / frequency;

    io::write_port(0x43, 0x36);

    let low = (divisor & 0xff) as u8;
    let high = (divisor >> 8 & 0xff) as u8;

    // Send the frequency divisor.
    io::write_port(0x40, low);
    io::write_port(0x40, high);
}

pub fn sleep(duration: u32) {
    unsafe {
        let target = tick + duration / 100;
        while (tick < target) {
            asm!("nop"); // Please don't optimize me away
        }
    }
}

fn callback(regs: &idt::Registers) {
    unsafe {
        tick += 1;
        if tick % 100 == 0 {
            vga::puts("\nOne second has passed\n");
        }
    }
}
