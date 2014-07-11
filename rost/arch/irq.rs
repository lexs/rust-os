use arch::idt;
use arch::io;

static IRQ_BASE: uint = 32;

pub fn init() {
    // Remap the irq table.
    io::write_port(0x20, 0x11);
    io::write_port(0xA0, 0x11);
    io::write_port(0x21, 0x20);
    io::write_port(0xA1, 0x28);
    io::write_port(0x21, 0x04);
    io::write_port(0xA1, 0x02);
    io::write_port(0x21, 0x01);
    io::write_port(0xA1, 0x01);
    io::write_port(0x21, 0x0);
    io::write_port(0xA1, 0x0);

    // Disable all lines
    io::write_port(0x21, 0xFF);
    io::write_port(0xA1, 0xFF);
}

pub fn register_handler(irq: uint, f: fn(regs: &mut idt::Registers)) {
    idt::register_interrupt(irq + IRQ_BASE, f);
    enable(irq);
}

pub fn enable(irq: uint) {
    if irq > 7 {
        let actual = irq - 8;
        let curr: u8 = io::read_port(0xA1);
        io::write_port(0xA1, curr & !((1u << actual) as u8))
    } else {
        let curr: u8 = io::read_port(0x21);
        io::write_port(0x21, curr & !((1u << irq) as u8))
    }
}
