mod physical;
pub mod virtual;

pub fn init() {
    physical::init();
    virtual::init();
}