use core::prelude::*;
use core::fmt;
use core::fmt::FormatWriter;

use drivers::vga;

struct Console;

impl fmt::FormatWriter for Console {
    fn write(&mut self, bytes: &[u8]) -> fmt::Result {
        for &c in bytes.iter() {
            vga::putch(c as char);
        }
        Ok(())
    }
}

pub fn print_args(fmt: &fmt::Arguments) {
    do_print(|io| write!(io, "{}", fmt));
}

pub fn println_args(fmt: &fmt::Arguments) {
    do_print(|io| writeln!(io, "{}", fmt));
}

fn do_print(f: |&mut fmt::FormatWriter| -> fmt::Result) {
    let result = f(&mut Console);
    match result {
        Ok(()) => {}
        Err(_) => fail!("failed printing to stdout: {}")
    }
}
