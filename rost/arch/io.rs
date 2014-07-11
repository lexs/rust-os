#[inline(always)]
pub fn write_port(port: u16, val: u8) {
    unsafe {
        asm!("out $0, $1" :: "{al}"(val), "{dx}"(port) :: "volatile");
    }
}

#[inline(always)]
pub fn read_port<T>(port: u16) -> T {
    unsafe {
        let mut val: T;
        asm!("in $1, $0" : "={ax}"(val) : "N{dx}"(port) :: "volatile");
        val
    }
}
