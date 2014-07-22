#![macro_escape]

use core::prelude::*;
use core::fmt;
use core::fmt::FormatWriter;

pub mod console;
pub mod log;

pub fn print_args(fmt: &fmt::Arguments) {
    do_print(|io| write!(io, "{}", fmt));
}

pub fn println_args(fmt: &fmt::Arguments) {
    do_print(|io| writeln!(io, "{}", fmt));
}

fn do_print(f: |&mut fmt::FormatWriter| -> fmt::Result) {
    use kernel::console::AnsiConsole;

    let result = f(&mut AnsiConsole);
    match result {
        Ok(()) => {}
        Err(_) => fail!("failed printing to stdout: {}")
    }
}

macro_rules! kprint(
    ($text:tt) => (kprint!("{}", $text));
    ($($arg:tt)*) => (format_args!(::kernel::print_args, $($arg)*));
)

macro_rules! kprintln(
    ($text:tt) => (kprintln!("{}", $text));
    ($($arg:tt)*) => (format_args!(::kernel::println_args, $($arg)*));
)

macro_rules! klog(
    ($text:tt) => (klog!("{}", $text));
    ($($arg:tt)*) => (format_args!(::kernel::log::println_args, $($arg)*));
)

macro_rules! panic(
    () => ({
        // Avoid warning about unneeded unsafe block
        fn freeze() -> ! {
            unsafe { asm!("cli"); }
            loop {}
        }
        freeze();
    });
    ($format:expr) => ({
        kprint!("PANIC: ");
        kprintln!($format);
        panic!();
    });
    ($format:expr, $($arg:expr),*) => ({
        kprint!("PANIC: ");
        kprintln!($format, $($arg),*);
        panic!();
    })
)
