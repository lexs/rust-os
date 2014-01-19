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

static mut kernel_directory: *mut PageTable = 0xFFFFF000 as *mut PageTable;
static PAGES : u32 = 0xFFC00000;

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
        (*directory).set_entry(dir_index(kernel_directory as u32), directory, FLAG_PRESENT | FLAG_WRITE);

        idt::register_isr_handler(14, page_fault);

        switch_page_directory(unsafe { directory });
    }
}

pub fn map(addr: u32, size: u32, flags: u32) {
    unsafe {
        // FIXME: We assume the table doesn't exist and it can hold the whole size
        let directory_index = dir_index(addr);

        let table_physical = physical::allocate_frame() as *mut PageTable;
        (*kernel_directory).set_entry(directory_index, table_physical, FLAG_PRESENT | flags);

        let table = page_table(directory_index);
        // Flush table so we can write to its virtual address
        flush_tlb(table);

        *table = PageTable::empty();

        let mut current_addr = addr;
        while current_addr < addr + size {
            let page = (*table).get_page(current_addr);

            page.set(physical::allocate_frame(), FLAG_PRESENT | FLAG_WRITE);
            flush_tlb(current_addr);

            current_addr += PAGE_SIZE;
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

    fn addr(self) -> u32 {
        match self {
            Page(value) => to_addr(value)
        }
    }

    fn set(&mut self, address: u32, flags: u32) {
        //assert!(address & 0xfff == 0);
        *self = Page(address | flags);
    }

    fn clear(&mut self) {
        *self = Page(0);
    }
}

impl PageTable {
    fn empty() -> PageTable {
        PageTable { entries: [Page::empty(), ..NUM_ENTRIES] }
    }

    unsafe fn set_entry<T>(&mut self, index: u32, entry: *mut T, flags: u32) {
        self.entries[index] = Page(entry as u32 | flags);
    }

    unsafe fn get_page<'a>(&'a mut self, addr: u32) -> &'a mut Page {
        &'a mut self.entries[table_index(addr)]
    }
}

fn page_table(index: u32) -> *mut PageTable {
    let size = size_of::<PageTable>() as u32;
    (PAGES + index * size) as *mut PageTable
}

fn dir_index(addr: u32) -> u32 {
    addr / (PAGE_SIZE * NUM_ENTRIES)
}

fn table_index(addr: u32) -> u32 {
    (addr / PAGE_SIZE) % NUM_ENTRIES
}

fn flush_tlb<T>(addr: T) {
    unsafe {
        asm!("invlpg ($0)" :: "r"(addr) : "volatile", "memory");
    }
}

fn switch_page_directory(directory: *mut PageTable) {
    unsafe {
        write_cr3(directory as u32);
        // Set the paging bit in CR0 to 1
        write_cr0(read_cr0() | 0x80000000);
    }
}

#[inline]
fn to_addr(value: u32) -> u32 {
    value & !0xfff
}

fn read_faulting_address() -> u32 {
    unsafe {
        let mut value;
        asm!("mov %cr2, $0" : "=r"(value));
        value
    }
}

#[inline]
unsafe fn write_cr3(value: u32) {
    asm!("mov $0, %cr3" :: "r"(value) :: "volatile");
}

unsafe fn read_cr0() -> u32 {
    let mut value;
    asm!("mov %cr0, $0" : "=r"(value));
    value
}

unsafe fn write_cr0(value: u32) {
    asm!("mov $0, %cr0" :: "r"(value) :: "volatile");
}
