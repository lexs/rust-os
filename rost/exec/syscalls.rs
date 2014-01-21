use core2::ptr::offset;

use arch::idt;
use kernel::console;

static NUM_SYSCALLS: uint = 128;

static mut syscalls: [fn(regs: &idt::Registers), ..NUM_SYSCALLS] = [
    unimplemented_syscall, ..NUM_SYSCALLS
];

pub fn init() {
    unsafe {
        syscalls[1] = syscall_exit;
        syscalls[2] = syscall_write;
    }

    idt::register_isr_handler(0x80, syscall_handler);
}

fn syscall_handler(regs: &idt::Registers) {
    let index = regs.eax;
    if index as uint >= NUM_SYSCALLS {
        unimplemented_syscall(regs);
    } else {
        unsafe { syscalls[index](regs); }
    }
}

fn unimplemented_syscall(regs: &idt::Registers) {
    console::write_str("Unimplemented syscall, number=");
    console::write_num(regs.eax);
    console::write_newline();
}

fn syscall_exit(regs: &idt::Registers) {
    console::write_str("Syscall exit, code=");
    console::write_num(regs.ebx);
    console::write_newline();
    loop {}
}

fn syscall_write(regs: &idt::Registers) {
    let fd = regs.ebx;
    let data = regs.ecx as *u8;
    let len = regs.edx;

    kassert!(fd == 1);

    let mut i = 0;
    while i < len {
        let c = unsafe { *offset(data, i as int) as char };
        console::write_char(c);

        i += 1;
    }
}
