use core::prelude::*;
use core::fmt;
use core::fmt::FormatWriter;

use drivers::serial;

struct Log;

impl fmt::FormatWriter for Log {
    fn write(&mut self, bytes: &[u8]) -> fmt::Result {
        for &c in bytes.iter() {
            serial::write(c);
        }
        Ok(())
    }
}

pub fn println_args(fmt: &fmt::Arguments) {
    do_print(|io| writeln!(io, "{}", fmt));
}

fn do_print(f: |&mut fmt::FormatWriter| -> fmt::Result) {
    let result = f(&mut Log);
    match result {
        Ok(()) => {}
        Err(_) => fail!("failed printing to log: {}")
    }
}
