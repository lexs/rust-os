#[inline(always)]
pub fn write_port(port: u16, val: u8) {
    unsafe {
        asm!("out $0, $1" :: "{al}"(val), "{dx}"(port) :: "volatile");
    }
}

#[inline(always)]
pub fn read_port(port: u16) -> u8 {
    unsafe {
        let mut val: u8;
        asm!("in $1, $0" : "={ax}"(val) : "N{dx}"(port) :: "volatile");
        val
    }
}
