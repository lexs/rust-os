use core::container::Container;
use core::mem::size_of;

use arch::io;
use util::range;

use arch::RING3;

static PRESENT: u8 = 1 << 7;
static USER: u8 = RING3 << 5;

static INTERRUPT_GATE: u8 = 0xE;

static IDT_SIZE: uint = 256;
type IdtTable = [IdtEntry, ..IDT_SIZE];

#[packed]
struct IdtEntry {
    handler_low: u16,
    selector: u16,
    always0: u8,
    flags: u8,
    handler_high: u16
}

#[packed]
struct IdtPtr {
    limit: u16,
    base: *IdtTable
}

#[packed]
pub struct Registers {
    edi: u32, esi: u32, ebp: u32, esp: u32, ebx: u32, edx: u32, ecx: u32, eax: u32,
    gs: u32, fs: u32, es: u32, ds: u32,
    int_no: u32, err_code: u32,
    eip: u32, cs: u32, eflags: u32, useresp: u32, ss: u32
}

impl IdtEntry {
    fn new(handler: u32, selector: u16, flags: u8) -> IdtEntry {
        IdtEntry {
            handler_low: (handler & 0xFFFF) as u16,
            handler_high: ((handler >> 16) & 0xFFFF) as u16,
            selector: selector,
            always0: 0,
            // We must uncomment the OR below when we get to using user-mode.
            // It sets the interrupt gate's privilege level to 3.
            flags: flags //| USER
        }
    }
}

impl IdtPtr {
    fn new(table: &IdtTable) -> IdtPtr {
        IdtPtr {
            limit: (size_of::<IdtEntry>() * table.len() - 1) as u16,
            base: table as *IdtTable
        }
    }
}

static mut entries: IdtTable = [
    IdtEntry {
        handler_low: 0,
        selector: 0,
        always0: 0,
        flags: 0,
        handler_high: 0
    }, ..IDT_SIZE
];

static mut table: IdtPtr = IdtPtr {
    limit: 0,
    base: 0 as *IdtTable
};

static EXCEPTIONS: &'static [&'static str] = &[
    "Divide-by-zero Error",
    "Debug",
    "Non-maskable Interrupt",
    "Breakpoint",
    "Overflow",
    "Bound Range Exceeded",
    "Invalid Opcode",
    "Device Not Available",
    "Double Fault",
    "Coprocessor Segment Overrun",
    "Invalid TSS",
    "Segment Not Present",
    "Stack-Segment Fault",
    "General Protection Fault",
    "Page Fault",
    "Reserved",
    "x87 Floating-Point Exception",
    "Alignment Check",
    "Machine Check",
    "SIMD Floating-Point Exception",
    "Virtualization Exception",
];

fn dummy_handler(regs: &mut Registers) {
    panic!("Unhandled interrupt: {}, error: {}", regs.int_no, regs.err_code);
}

fn exception_handler(regs: &mut Registers) {
    panic!("{}, error: {x}", EXCEPTIONS[regs.int_no], regs.err_code);
}

static mut interrupt_handlers: [fn(regs: &mut Registers), ..IDT_SIZE] = [
    dummy_handler, ..IDT_SIZE
];

pub fn init() {
    unsafe {
        table = IdtPtr::new(&entries);
        idt_flush(&table);
        idt_enable();

        // Register default exception handlers
        range(0, EXCEPTIONS.len(), |i| {
            register_interrupt(i, exception_handler);
        });
    }
}

pub fn register_user_interrupt(which: uint, f: fn(regs: &mut Registers)) {
    register_handler(which, INTERRUPT_GATE | USER, f);
}

pub fn register_interrupt(which: uint, f: fn(regs: &mut Registers)) {
    register_handler(which, INTERRUPT_GATE, f);
}

fn register_handler(which: uint, flags: u8, f: fn(regs: &mut Registers)) {
    // Defined in handlers.asm
    extern { static trap_handler_array: [u32, ..IDT_SIZE]; }

    unsafe {
        entries[which] = IdtEntry::new(trap_handler_array[which], 0x08, PRESENT | flags);
        interrupt_handlers[which] = f;
    }
}

#[no_mangle]
pub extern fn trap_handler(regs: &mut Registers) {
    let which = regs.int_no;

    // If this is a irq we need to eoi it
    if which >= 32 && which <= 47 {
        let irq = which - 32;
        if irq <= 7 {
            io::write_port(0x20, 0x20); // Master
        }
        io::write_port(0xA0, 0x20); // Slave
    }

    unsafe { interrupt_handlers[which](regs); }
}

unsafe fn idt_enable() {
    asm!("sti");
}

unsafe fn idt_flush(ptr: *IdtPtr) {
    asm!("lidt ($0)" :: "r"(ptr));
}
