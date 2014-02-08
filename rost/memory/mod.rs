pub use self::virtual::{
    map,
    Flags,
    NONE,
    PRESENT,
    WRITE,
    USER
};

mod physical;
mod virtual;
pub mod malloc;

pub fn init() {
    physical::init();
    virtual::init();
}