use drivers::vga;
use util;

pub fn write_str(s: &str) {
    vga::puts(s);
}

pub fn write_char(c: char) {
    vga::putch(c);
}

pub fn write_num(value: u32) {
    util::convert(value, |c| vga::putch(c));
}

pub fn write_hex(value: u32) {
    util::convert_radix(value, 16, |c| vga::putch(c));
}

pub fn write_newline() {
    vga::putch('\n');
}