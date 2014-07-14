#![macro_escape]

#[cfg(debug)]
macro_rules! kassert (
    ($condition:expr) => {
        if !($condition) {
            let msg = concat!("assert failed: ", stringify!($condition), " at ", file!(), ":", line!());
            panic!(msg);
        }
    }
)

#[cfg(not(debug))]
macro_rules! kassert (
    ($condition:expr) => (())
)
