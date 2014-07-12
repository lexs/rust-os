#![macro_escape]

pub mod printf;
pub mod console;
pub mod log;

macro_rules! panic (
    () => ({
        unsafe { asm!("cli"); }
        loop {}
    });
    ($format:expr) => ({
        use kernel::console::write_str;
        write_str("PANIC: ");
        kprintln!($format);
        panic!();
    });
    ($format:expr, $($arg:expr),*) => ({
        use kernel::console::write_str;
        write_str("PANIC: ");
        kprintln!($format, $($arg),*);
        panic!();
    })
)

pub fn panic(msg: &str) {
    panic!(msg);
}
