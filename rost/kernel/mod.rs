#[macro_escape];

pub mod printf;
pub mod console;

macro_rules! panic (
    () => ({
        unsafe { asm!("cli"); }
        loop {}
    });
    ($format:expr) => ({
        kprintln!(concat!("PANIC: ", $format));
        panic!();
    });
    ($format:expr, $($arg:expr),*) => ({
        kprintln!(concat!("PANIC: ", $format), $($arg),*);
        panic!();
    })
)

pub fn panic(msg: &str) {
}
