extern "rust-intrinsic" {
    pub fn volatile_load<T>(src: *T) -> T;
    pub fn volatile_store<T>(dst: *mut T, val: T);
}
