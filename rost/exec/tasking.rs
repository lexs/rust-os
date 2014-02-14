use core::option::{Option, Some, None};
use core::mem::{transmute, size_of};

use core2::list::List;

use arch::{gdt, idt};
use memory;
use memory::malloc::malloc;

type KernelStack = [u8, ..1024];
pub struct Task {
    pid: uint,
    esp: u32,
    eip: u32,
    pd: u32,
    kernel_stack: KernelStack
}

static STACK_SIZE: u32 = 8 * 1024;

static mut next_pid: uint = 1;
static mut tasks: List<~Task> = List { head: None, length: 0 };

static mut current_task: Option<~Task> = None;

impl Task {
    pub fn stack_top(&self) -> u32 {
        let stack_bottom: u32 = unsafe { transmute(&self.kernel_stack) };
        stack_bottom - size_of::<KernelStack>() as u32
    }
}

pub fn init() {
    unsafe {
        current_task = Some(~Task {
            pid: 0,
            esp: 0,
            eip: 0,
            pd: memory::kernel_directory,
            kernel_stack: [0, ..1024]
        });
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
        kernel_stack: [0, ..1024]
    };

    new_task.esp = new_task.stack_top();
    kprintln!("Stack is at {x}", new_task.esp);

    unsafe { tasks.add(new_task); }
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

pub fn user_mode(f: fn()) {
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
        esp: alloc_stack(STACK_SIZE),
        eflags: read_eflags() | 0x200, // Enable interrupts
        cs: 0x18 | 0x3,
        eip: f as u32
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
                tasks.add(current);
                current_task = Some(task);
                (tasks.front_mut().get(), current_task.as_ref().get())
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
