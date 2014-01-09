use core::option::{ Option, Some, None };

trait Readable {
    fn read(&mut self, bytes: &mut [u8], len: uint) -> uint;
}

trait Writeable {
    fn write(&self, bytes: &[u8]);
}

pub struct Buffer {
    buf: [u8, ..256],
    cur: uint,
    lim: uint
}

impl Buffer {
    pub fn new() -> Buffer {
        Buffer {
            buf: [0, ..256],
            cur: 0,
            lim: 0
        }
    }

    pub fn read(&mut self) -> Option<u8> {
        if self.cur != self.lim {
            let pos = self.cur;
            self.cur += 1;
            Some(self.buf[pos])
        } else {
            None
        }
    }

    pub fn write(&mut self, byte: u8) -> bool {
        self.buf[self.lim] = byte;
        self.lim += 1;
        true
    }
}

/*
impl Readable for Buffer {
    fn read(&mut self, bytes: &mut [u8], len: uint) -> uint {
        let end: uint = if len < self.limit { len } else { self.limit };
        let mut i = self.cur;
        while i < end {
            bytes[i] = self.buf[i];
            i += 1;
        }
        self.cur = end - i;
        self.cur
    }
}

impl Writeable for Buffer {
    fn write(&self, bytes: &[u8]) {
        let len = bytes.len();
        let mut i = 0;
        while i < len {
            
            i += len;
        }
    }
}*/

#[inline(always)]
pub fn write_port<T>(port: u16, val: T) {
    unsafe {
        asm!("out $0, $1" :: "{al}"(val), "{dx}"(port));
    }
}

#[inline(always)]
pub fn read_port<T>(port: u16) -> T {
    unsafe {
        let mut val: T;
        asm!("in $1, $0" : "={ax}"(val) : "N{dx}"(port));
        val
    }
}
