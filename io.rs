#[inline(always)]
pub fn out<T>(port: u16, val: T) {
    unsafe {
        asm!("out $1, $0" :: "{al}"(val), "{dx}"(port) :: "intel");
    }
}
