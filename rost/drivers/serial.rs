use arch::io;

static PORT: u16 = 0x3f8;

pub fn init() {
    io::write_port(PORT + 1, 0x00); // Disable all interrupts
    io::write_port(PORT + 3, 0x80); // Enable DLAB (set baud rate divisor)
    io::write_port(PORT + 0, 0x03); // Set divisor to 3 (lo byte) 38400 baud
    io::write_port(PORT + 1, 0x00); //                  (hi byte)
    io::write_port(PORT + 3, 0x03); // 8 bits, no parity, one stop bit
    io::write_port(PORT + 2, 0xC7); // Enable FIFO, clear them, with 14-byte threshold
    io::write_port(PORT + 4, 0x0B); // IRQs enabled, RTS/DSR set
}

fn is_transmit_empty() -> bool {
    io::read_port(PORT + 5) & 0x20 != 0
}
 
pub fn write(c: u8) {
    loop {
        if is_transmit_empty() { break }
    }

    io::write_port(PORT, c);
}
