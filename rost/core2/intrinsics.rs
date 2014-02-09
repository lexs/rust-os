

extern "rust-intrinsic" {
    /// Create an uninitialized value.
    pub fn uninit<T>() -> T;

    /// Move a value out of scope without running drop glue.
    ///
    /// `forget` is unsafe because the caller is responsible for
    /// ensuring the argument is deallocated already.
    pub fn forget<T>(_: T) -> ();
    pub fn transmute<T,U>(e: T) -> U;

    pub fn offset<T>(dst: *T, offset: int) -> *T;

    pub fn volatile_load<T>(src: *T) -> T;
    pub fn volatile_store<T>(dst: *mut T, val: T);
}
