use core::prelude::*;

use core::mem::{transmute, size_of};
use core::ptr::copy_nonoverlapping_memory;

use util::Unique;
use util::list::{List, Node, Rawlink};

use arch::{gdt, idt};
use memory;

pub type KernelStack = [u8, ..STACK_SIZE];
pub struct Task {
    pub pid: uint,
    pub esp: u32,
    pub eip: u32,
    pub pd: u32,
    pub regs: *mut idt::Registers,
    pub kernel_stack: KernelStack
}

static STACK_SIZE: uint = 8 * 1024;

static mut next_pid: uint = 1;
static mut tasks: List<Unique<Task>> = List { head: None, tail: Rawlink { p: 0 as *mut Node<Unique<Task>> }, length: 0 };

pub static mut current_task: Option<Unique<Task>> = None;

impl Task {
    pub fn stack_top(&self) -> u32 {
        let stack_bottom: u32 = unsafe { transmute(&self.kernel_stack) };
        stack_bottom + size_of::<KernelStack>() as u32
    }
}

// We can't use the Unique::new() constructor becase it
// creates the Task on the stack and then copies it into
// the memory. This fails because this task is actually
// larger than the stack we're operating on.
macro_rules! new_task (
    (Task {
        pid: $pid:expr,
        esp: $esp:expr,
        eip: $eip:expr,
        pd: $pd:expr,
        regs: $regs:expr

    }) => ({
        let mut task: Unique<Task> = Unique::empty();
        task.pid = $pid;
        task.eip = $eip;
        task.pd = $pd;
        task.regs = $regs;
        task
    })
)

pub fn init() {
    unsafe {
        let mut task = new_task!(Task {
            pid: 0,
            esp: 0,
            eip: 0,
            pd: memory::kernel_directory,
            regs: 0 as *mut idt::Registers
        });

        gdt::set_kernel_stack(task.stack_top());

        current_task = Some(task);
    }
}

pub fn get_current_task() -> &mut Unique<Task> {
    match unsafe { current_task.as_mut() } {
        None => panic!("Tasking not initialized!"),
        Some(task) => task
    }
}

pub fn kill() {
    unsafe {
        if get_current_task().pid == 0 {
            panic!("Can not kill idle task");
        }

        current_task = tasks.pop_front();
        replace_current(get_current_task());
    }
}

pub fn exec(f: fn()) {
    let mut new_task = new_task!(Task {
        pid: aquire_pid(),
        esp: 0,
        eip: unsafe { transmute(f) },
        pd: memory::clone_directory(),
        regs: 0 as *mut idt::Registers // FIXME  
    });

    new_task.esp = new_task.stack_top();

    unsafe { tasks.append(new_task); }
}

pub fn fork() -> uint {
    unsafe {
        extern { static ret_from_trap: u32; }

        let mut new_task = new_task!(Task {
            pid: aquire_pid(),
            esp: 0,
            eip: transmute(&ret_from_trap),
            pd: memory::clone_directory(),
            regs: 0 as *mut idt::Registers      
        });

        let regs_end = new_task.stack_top() as *mut idt::Registers;
        let regs = regs_end.offset(-1);
        copy_nonoverlapping_memory(regs, get_current_task().regs as *const idt::Registers, 1);

        new_task.esp = regs as u32;
        (*regs).eax = 0;

        let child_pid = new_task.pid;

        tasks.append(new_task);

        child_pid
    }
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
    #[allow(dead_code)]
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
            Some(current) => {
                tasks.append(current);
                current_task = Some(task);
                (tasks.back_mut().unwrap(), current_task.get_ref())
            }
        };

        switch_to(last_task, next_task);
    }
}

#[inline(never)] // We can't inline because then the label "resume" would fail to be found
unsafe fn switch_to(prev: &mut Unique<Task>, next: &Unique<Task>) {
    // These blocks are split in two because we need to guarantee that the store
    // into prev.esp and prev.eip happens BEFORE the jmp. Optimally we would like
    // to use "=m" as a constraint but rustc/llvm doesn't seem to like that.
    // Without the explicit deref_mut() the values are borrowed as immutable.
    asm!(
        "cli;
        push %ebp;
        mov %esp, $0;
        lea resume, $1;"
        : "=r"(prev.deref_mut().esp), "=r"(prev.deref_mut().eip) ::: "volatile");

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

unsafe fn replace_current(next: &Unique<Task>) {
    asm!(
       "mov $0, %esp;
       jmp *$1;"
       :: "m"(next.esp), "m"(next.eip) :: "volatile");
}

fn aquire_pid() -> uint {
    unsafe {
        let pid = next_pid;
        next_pid += 1;
        pid
    }
}
