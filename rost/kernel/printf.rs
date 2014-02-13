#[macro_escape];

use core::container::Container;

use kernel::panic;
use kernel::console::write_char;
use util::{range, convert_radix};

pub enum Format {
    Default,
    Hex,
}

pub trait Printable {
    fn print(&self, format: Format, out: |char|);
}

pub fn print_formatted<T : Printable>(format: &str, start: uint, value: T) -> uint {
    let mut flags = Default;

    let mut i = start;

    kassert!(format[i] as char == '{');
    i += 1;

    while i < format.len() {
        let c = format[i] as char;
        i += 1;

        match c {
            'x' => flags = Hex,
            '}' => {
                value.print(flags, |c| {
                    write_char(c);
                });
                break;
            },
            _ => {
                panic("Invalid format");
            }
        }
    }

    i - start
}

macro_rules! kprintln (
    ($format:expr) => ({
        use kernel::console::{write_char, write_str};

        write_str($format);
        write_char('\n');
    });
    ($format:expr, $($arg:expr),*) => ({
        use core::container::Container;

        use kernel::console::write_char;
        use kernel::printf::print_formatted;

        let format: &str = $format;
        let mut i = 0;

        $(
        while i < format.len() {
            match format[i] as char {
                '{' => {
                    i += print_formatted(format, i, $arg);
                    break;
                },
                c => {
                    write_char(c);
                    i += 1;
                }
            }
        }
        )*

        // Print remaining
        while i < format.len() {
            write_char(format[i] as char);
            i += 1;
        }

        write_char('\n');
    })
)

impl<'a> Printable for &'a str {
    fn print(&self, _: Format, out: |char|) {
        range(0, self.len(), |i| {
            out(self[i] as char);
        });
    }
}

impl<'a> Printable for bool {
    fn print(&self, format: Format, out: |char|) {
        if *self {
            "true".print(Default, out);
        } else {
            "false".print(Default, out);
        }
    }
}

impl<'a, T> Printable for *T {
    fn print(&self, format: Format, out: |char|) {
        convert_radix(*self as u32, 16, out);
    }
}

impl<'a, T> Printable for *mut T {
    fn print(&self, format: Format, out: |char|) {
        convert_radix(*self as u32, 16, out);
    }
}

macro_rules! printable_integer (
    ($t:ty) => (
        impl<'a> Printable for $t {
            fn print(&self, format: Format, out: |char|) {
                let radix = match format {
                    Hex => 16,
                    _ => 10
                };
                convert_radix(*self as u32, radix, out);
            }
        }
    )
)

printable_integer!(int)
printable_integer!(i8)
printable_integer!(i16)
printable_integer!(i32)
printable_integer!(uint)
printable_integer!(u8)
printable_integer!(u16)
printable_integer!(u32)
