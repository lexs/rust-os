use drivers::serial;

pub fn write_char(c: char) {
    serial::write(c as u8);
}
