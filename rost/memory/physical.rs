static FRAME_SIZE: u32 = 0x1000;

static mut next_frame: u32 = 1024; // First 1MB is in use, TODO: Make this nicer


pub fn init() {
    unsafe {
        // Mark frames up to kernel_end as used
        extern { static kernel_end: u32; }
        next_frame = kernel_end % FRAME_SIZE + 1;
    }
}

pub fn allocate_frame() -> u32 {
    unsafe {
        let frame = next_frame;
        next_frame += 1;
        frame * FRAME_SIZE
    }
}
