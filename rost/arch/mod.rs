pub mod io;
pub mod gdt;
pub mod idt;
pub mod irq;

static RING3: u8 = 3;
