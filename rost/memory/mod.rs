pub use self::virt::{
    kernel_directory,
    map,
    clone_directory,
    switch_page_directory,
    Flags,
    NONE,
    WRITE,
    USER,
    EXEC
};

mod physical;
mod virt;
pub mod malloc;

pub fn init() {
    physical::init();
    virt::init();
}
