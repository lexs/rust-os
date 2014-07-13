#![macro_escape]

pub mod console;
pub mod log;

macro_rules! kprint(
    ($text:tt) => (kprint!("{}", $text));
    ($($arg:tt)*) => (format_args!(::kernel::console::print_args, $($arg)*));
)

macro_rules! kprintln(
    ($text:tt) => (kprintln!("{}", $text));
    ($($arg:tt)*) => (format_args!(::kernel::console::println_args, $($arg)*));
)

macro_rules! klog(
    ($text:tt) => (kprint!("{}", $text));
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
