use core::option::{Option, Some, None};
use core::mem::size_of;

use kernel::console;
use arch::idt;
use memory::physical;

static PAGE_SIZE: u32 = 0x1000;
static NUM_ENTRIES: u32 = 1024;

#[packed]
struct Page(u32);

#[packed]
struct PageTable {
    entries: [Page, ..NUM_ENTRIES]
}

static DIRECTORY: u32 = 0xFFFFF000;
static PAGES: u32 = 0xFFC00000;
static mut kernel_directory: *mut PageTable = DIRECTORY as *mut PageTable;


pub static FLAG_PRESENT: u32 = 1 << 0;
pub static FLAG_WRITE: u32 = 1 << 1;
pub static FLAG_USER: u32 = 1 << 2;

pub fn init() {
    unsafe {
        let directory = physical::allocate_frame() as *mut PageTable;
        *directory = PageTable::empty();

        let table = physical::allocate_frame() as *mut PageTable;
        *table = PageTable::empty();

        // Identity map table the whole table, 4MB
        let mut i = 0;
        while i < PAGE_SIZE * NUM_ENTRIES {
            let page = (*table).get_page(i);
            page.set(i, FLAG_PRESENT | FLAG_WRITE);
            i += PAGE_SIZE;
        }

        (*directory).set_entry(0, table, FLAG_PRESENT | FLAG_WRITE);

        // Map the directory itself as the last entry
        (*directory).set_entry(table_index(kernel_directory as u32), directory, FLAG_PRESENT | FLAG_WRITE);

        idt::register_isr_handler(14, page_fault);

        switch_page_directory(directory);
    }
}

pub fn map(addr: u32, size: u32, flags: u32) {
    unsafe {
        let mut current_addr = addr;
        while current_addr < addr + size {
            let table = get_table(current_addr, flags);
            let page = (*table).get_page(current_addr);

            page.set(physical::allocate_frame(), flags);
            flush_tlb(current_addr);

            current_addr += PAGE_SIZE;
        }
    }
}

unsafe fn get_table(addr: u32, flags: u32) -> *mut PageTable {
    let table_index = table_index(addr);
    match (*kernel_directory).get_entry(table_index) {
        Some(_) => page_table(table_index),
        None => {
            // Create table
            let table_physical = physical::allocate_frame() as *mut PageTable;
            (*kernel_directory).set_entry(table_index, table_physical, flags);
            
            let table = page_table(table_index);
            // Flush table so we can write to its virtual address
            flush_tlb(table);

            *table = PageTable::empty();
            table
        }
    }
}

fn page_fault(regs: &idt::Registers) {
    let address = read_faulting_address();

    let present = regs.err_code & 0x1 != 0;
    let rw = regs.err_code & 0x2 == 0;
    let user = regs.err_code & 0x4 == 0;
    let reserved = regs.err_code & 0x8 == 0;

    console::write_str("Page fault! ( ");
    if present { console::write_str("present "); }
    if rw { console::write_str("read-only "); }
    if user { console::write_str("user-mode "); }
    if reserved { console::write_str("reserved "); }
    console::write_str(") at ");
    console::write_hex(address);
    console::write_newline();

    loop {}
}

impl Page {
    fn empty() -> Page { Page(0) }

    fn present(self) -> bool {
        match self {
            Page(value) => value & FLAG_PRESENT != 0
        }
    }

    fn set(&mut self, address: u32, flags: u32) {
        //assert!(address & 0xfff == 0);
        *self = Page(address | flags);
    }
}

impl PageTable {
    fn empty() -> PageTable {
        PageTable { entries: [Page::empty(), ..NUM_ENTRIES] }
    }

    fn get_entry(&mut self, index: u32) -> Option<Page> {
        if self.entries[index].present() { Some(self.entries[index]) } else { None }
    }

    unsafe fn set_entry<T>(&mut self, index: u32, entry: *mut T, flags: u32) {
        self.entries[index] = Page(entry as u32 | flags);
    }

    unsafe fn get_page<'a>(&'a mut self, addr: u32) -> &'a mut Page {
        &'a mut self.entries[page_index(addr)]
    }
}

fn page_table(index: u32) -> *mut PageTable {
    let size = size_of::<PageTable>() as u32;
    (PAGES + index * size) as *mut PageTable
}

fn table_index(addr: u32) -> u32 {
    addr / (PAGE_SIZE * NUM_ENTRIES)
}

fn page_index(addr: u32) -> u32 {
    (addr / PAGE_SIZE) % NUM_ENTRIES
}

fn flush_tlb<T>(addr: T) {
    unsafe {
        asm!("invlpg ($0)" :: "r"(addr) : "volatile", "memory");
    }
}

fn switch_page_directory(directory: *mut PageTable) {
    unsafe fn read_cr0() -> u32 {
        let mut value;
        asm!("mov %cr0, $0" : "=r"(value));
        value
    }

    unsafe fn write_cr0(value: u32) {
        asm!("mov $0, %cr0" :: "r"(value) :: "volatile");
    }

    unsafe {
        asm!("mov $0, %cr3" :: "r"(directory) :: "volatile");
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
