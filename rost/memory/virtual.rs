use core;
use core::mem::{transmute, size_of};
use core::ptr::copy_nonoverlapping_memory;

use arch::idt;
use memory::physical;

static PAGE_SIZE: u32 = 0x1000;
static PAGE_MASK: u32 = 0xFFFFF000;
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
static DIRECTORY_SECONDARY: u32 = 0xFFBFF000;
static PAGES: u32 = 0xFFC00000;

// Temporary virtual addresses useful for mapping in physical pages
static TEMP1: u32 = 0xFF7FF000;
static TEMP2: u32 = 0xFF7FE000;

static current_directory: *mut PageDirectory = DIRECTORY as *mut PageDirectory;

pub static mut kernel_directory: u32 = 0;

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
        (*directory).set(DIRECTORY, directory as u32, PRESENT | WRITE);

        idt::register_interrupt(14, page_fault);

        kernel_directory = directory as u32;
        switch_page_directory(kernel_directory);
        enable_paging();
    }
}

pub fn map(addr: u32, size: u32, flags: Flags) {
    unsafe {
        let mut current_addr = addr;
        while current_addr < addr + size {
            let table = (*current_directory).fetch_table(current_addr, flags);

            (*table).set(current_addr, physical::allocate_frame(), flags);

            current_addr += PAGE_SIZE;
        }

        *(addr as *mut u8) = 5;

        let page = (*current_directory).get_page(addr);
    }
}

pub fn clone_directory() -> u32 {
    unsafe {
        let directory_physical = new_directory();
        let directory = map_secondary_directory(directory_physical);

        // Link first 4MB
        (*directory).set_at(0, (*current_directory).get(0));

        // Copy everything up to the kernel
        let mut i = ENTRIES * PAGE_SIZE;
        while i < 0xC0000000 {
            let src_page = (*current_directory).get_page(i);
            if src_page.present() {
                let dst_page = physical::allocate_frame();
                (*directory).set_page(i, dst_page, src_page.flags());
                copy_page(src_page.addr(), dst_page);
            }

            i += PAGE_SIZE;
        }

        // Link all kernel space
        while i < DIRECTORY_SECONDARY {
            // FIXME: Force table initialization, this is quite dirty but lets us skip
            // kernel space synchronisation for now
            (*current_directory).fetch_table(i, PRESENT | WRITE);

            (*directory).set_at(i, (*current_directory).get(i));
            i += ENTRIES * PAGE_SIZE;
        }

        directory_physical
    }
}

unsafe fn map_secondary_directory(directory_physical: u32) -> *mut PageDirectory {
    (*current_directory).set(DIRECTORY_SECONDARY, directory_physical, PRESENT | WRITE);
    DIRECTORY_SECONDARY as *mut PageDirectory
}

unsafe fn new_directory() -> u32 {
    let directory_physical = physical::allocate_frame();

    (*current_directory).set_page(TEMP1, directory_physical, PRESENT | WRITE);

    let directory = TEMP1 as *mut PageDirectory;
    *directory = Table::empty();

    // Map the last page onto the directory itself
    (*directory).set(DIRECTORY, directory_physical, PRESENT | WRITE);

    directory_physical
}

unsafe fn copy_page(src: u32, dst: u32) {
    (*current_directory).set_page(TEMP1, src, PRESENT | WRITE);
    (*current_directory).set_page(TEMP2, dst, PRESENT | WRITE);
    copy_nonoverlapping_memory(dst as *mut (), src as *(), PAGE_SIZE as uint);
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

    fn addr(self) -> u32 {
        match self {
            Page(addr) => addr & PAGE_MASK
        }
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

    fn get(&mut self, addr: u32) -> Page {
        let level = size_of::<U>() / size_of::<Page>();
        let index = (addr / (PAGE_SIZE * level as u32)) % ENTRIES;
        self.entries[index]
    }

    fn set(&mut self, addr: u32, phys: u32, flags: Flags) {
        self.set_at(addr, Page::new(phys, flags));
    }

    fn set_at(&mut self, addr: u32, page: Page) {
        let level = size_of::<U>() / size_of::<Page>();
        let index = (addr / (PAGE_SIZE * level as u32)) % ENTRIES;
        self.entries[index] = page;
        flush_tlb(addr);
    }
}

impl Table<Table<Page>> {
    fn get_page(&self, addr: u32) -> Page {
        let index = addr / (PAGE_SIZE * ENTRIES);
        match self.entries[index] {
            p if p.present() => unsafe {
                let table = self.table_at(index);
                (*table).get(addr)
            },
            _ => Page(0)
        }
    }

    fn set_page(&mut self, addr: u32, phys: u32, flags: Flags) {
        unsafe {
            let table = self.fetch_table(addr, flags);
            (*table).set(addr, phys, flags);
        }
    }

    fn fetch_table(&mut self, addr: u32, flags: Flags) -> *mut PageTable {
        let index = addr / (PAGE_SIZE * ENTRIES);
        match self.entries[index] {
            p if p.present() => self.table_at(index),
            _ => unsafe {
                // Allocate table
                let table_physical = physical::allocate_frame();
                self.entries[index] = Page::new(table_physical, flags);

                let table = self.table_at(index);
                // Flush table so we can write to its virtual address
                flush_tlb(table);

                *table = Table::empty();
                table
            }
        }
    }

    fn table_at(&self, index: u32) -> *mut PageTable {
        let self_addr: u32 = unsafe { transmute(self as *PageDirectory) };
        let size = size_of::<PageTable>() as u32;
        let start = self_addr - (ENTRIES - 1) * size;
        (start + index * size) as *mut PageTable
    }
}

fn flush_tlb<T>(addr: T) {
    unsafe {
        asm!("invlpg ($0)" :: "r"(addr) : "memory" : "volatile");
    }
}

pub fn switch_page_directory(directory: u32) {
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
