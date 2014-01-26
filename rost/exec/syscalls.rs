use core2::ptr::offset;

use arch::idt;
use kernel::console;

static NUM_SYSCALLS: uint = 128;

static mut syscalls: [fn(regs: &mut idt::Registers), ..NUM_SYSCALLS] = [
    unimplemented_syscall, ..NUM_SYSCALLS
];

macro_rules! syscall (
    // 1 arg
    (fn $name:ident($a0:ident: $t0:ty) $func:expr) => (
        fn $name(regs: &mut idt::Registers) {
            let $a0 = regs.ebx as $t0;
            $func
        }
    );
    // 3 args
    (fn $name:ident($a0:ident: $t0:ty, $a1:ident: $t1:ty, $a2:ident: $t2:ty) -> $ret:ty $func:expr) => (
        fn $name(regs: &mut idt::Registers) {
            let $a0 = regs.ebx as $t0;
            let $a1 = regs.ecx as $t1;
            let $a2 = regs.edx as $t2;
            regs.eax = { $func } as $ret;
        }
    );
    (fn $name:ident($a0:ident: $t0:ty, $a1:ident: $t1:ty, $a2:ident: $t2:ty) $func:expr) => (
        fn $name(regs: &mut idt::Registers) {
            let $a0 = regs.ebx as $t0;
            let $a1 = regs.ecx as $t1;
            let $a2 = regs.edx as $t2;
            $func
        }
    );
    
)

pub fn init() {
    unsafe {
        syscalls[1] = syscall_exit;
        syscalls[2] = syscall_write;
    }

    idt::register_isr_handler(0x80, syscall_handler);
}

fn syscall_handler(regs: &mut idt::Registers) {
    let index = regs.eax;
    if index as uint >= NUM_SYSCALLS {
        unimplemented_syscall(regs);
    } else {
        unsafe { syscalls[index](regs); }
    }
}

fn unimplemented_syscall(regs: &mut idt::Registers) {
    console::write_str("Unimplemented syscall, number=");
    console::write_num(regs.eax);
    console::write_newline();
}

syscall!(fn syscall_exit(code: u32) {
    console::write_str("Syscall exit, code=");
    console::write_num(code);
    console::write_newline();
    loop {}
})

syscall!(fn syscall_write(fd: u32, data: *u8, len: u32) -> u32 {
    kassert!(fd == 1);

    let mut i = 0;
    while i < len {
        let c = unsafe { *offset(data, i as int) as char };
        console::write_char(c);

        i += 1;
    }
    i
})
