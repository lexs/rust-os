use core;
use core::mem::size_of;

use arch::idt;
use memory::physical;

static PAGE_SIZE: u32 = 0x1000;
static ENTRIES: u32 = 1024;

define_flags!(Flags: u32 {
    NONE     = 0,
    PRESENT  = 1 << 0,
    WRITE    = 1 << 1,
    USER     = 1 << 2,
    ACCESSED = 1 << 5
})

#[packed]
struct Page(u32);

#[packed]
struct Table<U> {
    entries: [Page, ..ENTRIES]
}

type PageTable = Table<Page>;
type PageDirectory = Table<PageTable>;

static DIRECTORY: u32 = 0xFFFFF000;
static PAGES: u32 = 0xFFC00000;
static mut kernel_directory: *mut PageDirectory = DIRECTORY as *mut PageDirectory;

pub fn init() {
    unsafe {
        let directory = physical::allocate_frame() as *mut PageDirectory;
        *directory = Table::empty();

        let table = physical::allocate_frame() as *mut PageTable;
        *table = Table::empty();

        // Identity map table the whole table, 4MB
        let mut i = 0;
        while i < PAGE_SIZE * ENTRIES {
            (*table).set(i, i, PRESENT | WRITE | USER);
            i += PAGE_SIZE;
        }

        (*directory).set(0, table as u32, PRESENT | WRITE | USER);

        // Map the directory itself as the last entry
        (*directory).set(kernel_directory as u32, directory as u32, PRESENT | WRITE);

        idt::register_interrupt(14, page_fault);

        switch_page_directory(directory);
        enable_paging();
    }
}

pub fn map(addr: u32, size: u32, flags: Flags) {
    unsafe {
        let mut current_addr = addr;
        while current_addr < addr + size {
            let table = (*kernel_directory).fetch_table(current_addr, flags);

            (*table).set(current_addr, physical::allocate_frame(), flags);
            flush_tlb(current_addr);

            current_addr += PAGE_SIZE;
        }
    }
}

fn page_fault(regs: &mut idt::Registers) {
    let address = read_faulting_address();

    let present = regs.err_code & 0x1 != 0;
    let rw = regs.err_code & 0x2 == 0;
    let user = regs.err_code & 0x4 == 0;
    let reserved = regs.err_code & 0x8 == 0;

    kprintln!("Page fault! ( {}{}{}{}) at {x}",
        if present { "present " } else { "" },
        if rw { "read-only " } else { "" },
        if user { "user-mode " } else { "" },
        if reserved { "reserved " } else { "" },
        address);

    loop {}
}

impl Page {
    fn empty() -> Page { Page(0) }

    fn new(addr: u32, flags: Flags) -> Page {
        Page(addr | flags.to_int())
    }

    fn flags(self) -> Flags {
        match self {
            Page(value) => Flags::from_int(value)
        }
    }

    fn present(self) -> bool {
        self.flags() & PRESENT
    }
}

impl<U> Table<U> {
    fn empty() -> Table<U> {
        Table { entries: [Page::empty(), ..ENTRIES] }
    }

    fn set(&mut self, addr: u32, phys: u32, flags: Flags) {
        let len = size_of::<U>() / size_of::<Page>();
        let index = (addr / PAGE_SIZE / len as u32) % ENTRIES;
        self.entries[index] = Page::new(phys, flags);
    }
}

impl Table<Table<Page>> {
    fn fetch_table(&mut self, addr: u32, flags: Flags) -> *mut PageTable {
        let index = addr / (PAGE_SIZE * ENTRIES);
        match self.entries[index] {
            p @ Page(table_physical) if p.present() => {
                (table_physical & DIRECTORY) as *mut PageTable
            },
            _ => unsafe {
                // Allocate table
                let table_physical = physical::allocate_frame();
                (*kernel_directory).set(addr, table_physical, flags);

                let table = (table_physical & DIRECTORY) as *mut PageTable;
                // Flush table so we can write to its virtual address
                flush_tlb(table);

                *table = Table::empty();
                table
            }
        }
    }
}

fn flush_tlb<T>(addr: T) {
    unsafe {
        asm!("invlpg ($0)" :: "r"(addr) : "memory" : "volatile");
    }
}

fn switch_page_directory(directory: *mut PageDirectory) {
    unsafe {
        asm!("mov $0, %cr3" :: "r"(directory) :: "volatile");
    }
}

fn enable_paging() {
    unsafe {
        // Set the paging bit in CR0 to 1
        write_cr0(read_cr0() | 0x80000000);
    }
}

fn read_faulting_address() -> u32 {
    unsafe {
        let mut value;
        asm!("mov %cr2, $0" : "=r"(value));
        value
    }
}

unsafe fn read_cr0() -> u32 {
    let mut value;
    asm!("mov %cr0, $0" : "=r"(value));
    value
}

unsafe fn write_cr0(value: u32) {
    asm!("mov $0, %cr0" :: "r"(value) :: "volatile");
}
