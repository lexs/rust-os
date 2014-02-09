use drivers::vga;

pub fn write_str(s: &str) {
    vga::puts(s);
}

pub fn write_char(c: char) {
    vga::putch(c);
}
