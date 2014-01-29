use core::option::{Option, Some, None};
use core::mem::transmute;

use core2::list::List;

use kernel::console;
use memory::malloc::malloc;

pub struct Task {
    pid: uint,
    esp: u32,
    eip: u32
}

static STACK_SIZE: u32 = 8 * 1024;

static mut next_pid: uint = 1;
static mut tasks: List<Task> = List { head: None, length: 0 };

static mut current_task: Task = Task {
        pid: 0,
        esp: 0,
        eip: 0
};

pub fn exec(f: fn()) {
    let eip: u32 = unsafe { transmute(f) };
    let stack: u32 = unsafe { transmute(malloc(STACK_SIZE)) };

    let new_task = Task {
        pid: aquire_pid(),
        esp: stack + STACK_SIZE,
        eip: eip
    };

    unsafe { tasks.add(new_task); }
}

pub fn schedule() {
    let next_task = match unsafe { tasks.pop_front() } {
        None => return,
        Some(task) => task
    };

    let last_task = unsafe {
        tasks.add(current_task);
        current_task = next_task;
        tasks.front_mut().get()
    };

    unsafe { switch_to(last_task, &current_task); }
}

#[inline(never)] // We can't inline because then the label "resume" would fail to be found
unsafe fn switch_to(prev: &mut Task, next: &Task) {
    // These blocks are split in two because we need to guarantee that the store
    // into prev.esp and prev.eip happens BEFORE the jmp. Optimally we would like
    // to use "=m" as a constraint but rustc/llvm doesn't seem to like that.
    asm!(
        "cli;
        push %ebp;
        mov %esp, $0;
        lea resume, $1;"
        : "=r"(prev.esp), "=r"(prev.eip) ::: "volatile");
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