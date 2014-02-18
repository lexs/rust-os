use core::option::{Option, Some, None};
use core::mem::{transmute, size_of};
use core::ptr::copy_nonoverlapping_memory;

use core2::list::{List, Node, Rawlink};
use core2::ptr::mut_offset;

use arch::{gdt, idt};
use memory;
use memory::malloc::malloc;

type KernelStack = [u8, ..STACK_SIZE];
pub struct Task {
    pid: uint,
    esp: u32,
    eip: u32,
    pd: u32,
    regs: *mut idt::Registers,
    kernel_stack: KernelStack
}

static STACK_SIZE: u32 = 16 * 1024;

static mut next_pid: uint = 1;
static mut tasks: List<~Task> = List { head: None, tail: Rawlink { p: 0 as *mut Node<~Task> }, length: 0 };

pub static mut current_task: Option<~Task> = None;

impl Task {
    pub fn stack_top(&self) -> u32 {
        let stack_bottom: u32 = unsafe { transmute(&self.kernel_stack) };
        stack_bottom + size_of::<KernelStack>() as u32
    }
}

pub fn init() {
    unsafe {
        let task = ~Task {
            pid: 0,
            esp: 0,
            eip: 0,
            pd: memory::kernel_directory,
            regs: 0 as *mut idt::Registers,
            kernel_stack: [0, ..STACK_SIZE]
        };

        gdt::set_kernel_stack(task.stack_top());

        current_task = Some(task);
    }
}

pub fn get_current_task() -> &mut ~Task {
    match unsafe { current_task.as_mut() } {
        None => panic!("Tasking not initialized!"),
        Some(task) => task
    }
}

pub fn exec(f: fn()) {
    let eip: u32 = unsafe { transmute(f) };

    let p = alloc_stack(STACK_SIZE);

    let mut new_task = ~Task {
        pid: aquire_pid(),
        esp: 0,
        eip: eip,
        pd: memory::clone_directory(),
        regs: 0 as *mut idt::Registers, // FIXME
        kernel_stack: [0, ..STACK_SIZE]
    };

    new_task.esp = new_task.stack_top();

    unsafe { tasks.append(new_task); }
}

pub fn fork() -> uint {
    unsafe {
        extern { static ret_from_trap: u32; }

        let mut new_task = ~Task {
            pid: aquire_pid(),
            esp: 0,
            eip: transmute(&ret_from_trap),
            pd: memory::clone_directory(),
            regs: 0 as *mut idt::Registers,
            kernel_stack: [0, ..STACK_SIZE]
        };

        let regs = mut_offset(new_task.stack_top() as *mut idt::Registers, -1);
        copy_nonoverlapping_memory(regs, get_current_task().regs as *idt::Registers, 1);

        new_task.esp = regs as u32;
        (*regs).eax = 0;

        let child_pid = new_task.pid;

        tasks.append(new_task);

        child_pid
    }
}

pub fn alloc_stack(size: u32) -> u32 {
    let stack_top: u32 = unsafe { transmute(malloc(size)) };
    stack_top + size
}

fn read_eflags() -> u32 {
    unsafe {
        let mut eflags: u32;
        asm!("pushf; pop $0;" : "=r"(eflags) ::: "volatile");
        eflags
    }
}

pub fn user_mode(entry: u32, stack: u32) {
    #[packed]
    struct FakeStack {
        eip: u32,
        cs: u32,
        eflags: u32,
        esp: u32,
        ss: u32,
    };

    let fake_stack = FakeStack {
        ss: 0x20 | 0x3,
        esp: stack,
        eflags: read_eflags() | 0x200, // Enable interrupts
        cs: 0x18 | 0x3,
        eip: entry
    };

    gdt::set_segments(0x20 | 0x3);
    extern { fn run_iret(stack: FakeStack); }
    unsafe { run_iret(fake_stack); }
}

pub fn schedule() {
    unsafe {
        let task = match tasks.pop_front() {
            None => return,
            Some(task) => task
        };

        let (last_task, next_task) = match current_task.take() {
            None => panic!("No current task, is tasking initialized?"),
            Some(current) => unsafe {
                tasks.append(current);
                current_task = Some(task);
                (tasks.back_mut().get(), current_task.as_ref().get())
            }
        };

        switch_to(last_task, next_task);
    }
}

#[inline(never)] // We can't inline because then the label "resume" would fail to be found
unsafe fn switch_to(prev: &mut ~Task, next: &~Task) {
    // These blocks are split in two because we need to guarantee that the store
    // into prev.esp and prev.eip happens BEFORE the jmp. Optimally we would like
    // to use "=m" as a constraint but rustc/llvm doesn't seem to like that.
    asm!(
        "cli;
        push %ebp;
        mov %esp, $0;
        lea resume, $1;"
        : "=r"(prev.esp), "=r"(prev.eip) ::: "volatile");

    gdt::set_kernel_stack(next.stack_top());
    memory::switch_page_directory(next.pd);

    asm!(
       "mov $0, %esp;
       sti;
       jmp *$1;
       resume:
       pop %ebp;"
       :: "m"(next.esp), "m"(next.eip) :: "volatile");
}

fn aquire_pid() -> uint {
    unsafe {
        let pid = next_pid;
        next_pid += 1;
        pid
    }
}
