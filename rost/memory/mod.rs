pub use self::virtual::{
    map,
    FLAG_PRESENT,
    FLAG_WRITE,
    FLAG_USER
};

mod physical;
mod virtual;
pub mod malloc;

pub fn init() {
    physical::init();
    virtual::init();
}