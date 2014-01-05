use io;
use idt;
use vga;
use util;

static mut tick: u32 = 0;

fn callback(regs: &idt::Registers) {
    unsafe {
        tick += 1;
        if tick % 50 == 0 {
            vga::puts("tick: ");
            util::convert(tick, |c| vga::putch(c));
            vga::putch('\n');
        }
    }
}

pub fn init(frequency: u32) {
    idt::register_irq_handler(0, callback);

    let divisor = 1193180 / frequency;

    io::out(0x43, 0x36);

    let low = (divisor & 0xff) as u8;
    let high = (divisor >> 8 & 0xff) as u8;

    // Send the frequency divisor.
    io::out(0x40, low);
    io::out(0x40, high);
}
