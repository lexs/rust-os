pub use self::virtual::{
    kernel_directory,
    map,
    clone_directory,
    switch_page_directory,
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
