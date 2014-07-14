use core::prelude::*;
use core::mem::{transmute, size_of};
use core::ptr::copy_nonoverlapping_memory;

use arch::idt;
use memory::physical;

static PAGE_SIZE: u32 = 0x1000;
static PAGE_MASK: u32 = 0xFFFFF000;
static ENTRIES: u32 = 1024;

bitflags!(
    #[packed]
    flags Flags: u32 {
        static NONE     = 0,
        static PRESENT  = 1 << 0,
        static WRITE    = 1 << 1,
        static USER     = 1 << 2,
        #[allow(dead_code)]
        static ACCESSED = 1 << 5,
        static EXEC     = 1 << 7
    }
)

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
#[allow(dead_code)]
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
    let f = translate_flags(flags);

    unsafe {
        let mut current_addr = addr;
        while current_addr < addr + size {
            let table = (*current_directory).fetch_table(current_addr, f);

            let phys_addr = physical::allocate_frame();
            (*table).set(current_addr, phys_addr, f);
            klog!("Mapping virtual {:x} to physical {:x}", current_addr, phys_addr);

            current_addr += PAGE_SIZE;
        }
    }
}

fn translate_flags(flags: Flags) -> Flags {
    // TODO: Have external flags
    let mut t = flags.clone();

    if t.contains(EXEC) {
        klog!("mmap(): EXEC is currently ignored");
        t.remove(EXEC);
    }

    t.insert(PRESENT);
    t
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
    (*current_directory).set_page(TEMP1, src, PRESENT);
    (*current_directory).set_page(TEMP2, dst, PRESENT | WRITE);
    copy_nonoverlapping_memory(TEMP2 as *mut u8, TEMP1 as *const u8, PAGE_SIZE as uint);
}

fn page_fault(regs: &mut idt::Registers) {
    let address = read_faulting_address();
    let flags = Flags::from_bits_truncate(regs.err_code);

    let reserved = regs.err_code & 0x8 == 0;
    panic!("page fault! ( {}{}{}{}) at 0x{:x}",
        if flags.contains(PRESENT) { "present " } else { "non-present " },
        if flags.contains(WRITE) { "write " } else { " read " },
        if flags.contains(USER) { "user-mode " } else { "kernel-mode " },
        if reserved { "reserved " } else { "" },
        address);
}

impl Page {
    fn empty() -> Page { Page(0) }

    fn new(addr: u32, flags: Flags) -> Page {
        Page(addr | flags.bits())
    }

    fn addr(self) -> u32 {
        match self {
            Page(addr) => addr & PAGE_MASK
        }
    }

    fn flags(self) -> Flags {
        match self {
            Page(value) => Flags::from_bits_truncate(value)
        }
    }

    fn present(self) -> bool {
        self.flags().contains(PRESENT)
    }
}

impl<U> Table<U> {
    fn empty() -> Table<U> {
        Table { entries: [Page::empty(), ..ENTRIES as uint] }
    }

    fn get(&mut self, addr: u32) -> Page {
        let level = size_of::<U>() / size_of::<Page>();
        let index = (addr / (PAGE_SIZE * level as u32)) % ENTRIES;
        self.entries[index as uint]
    }

    fn set(&mut self, addr: u32, phys: u32, flags: Flags) {
        self.set_at(addr, Page::new(phys, flags));
    }

    fn set_at(&mut self, addr: u32, page: Page) {
        let level = size_of::<U>() / size_of::<Page>();
        let index = (addr / (PAGE_SIZE * level as u32)) % ENTRIES;
        self.entries[index as uint] = page;
        flush_tlb(addr);
    }
}

impl Table<Table<Page>> {
    fn get_page(&self, addr: u32) -> Page {
        let index = addr / (PAGE_SIZE * ENTRIES);
        match self.entries[index as uint] {
            p if p.present() => unsafe {
                let table = self.table_at(index);
                (*table).get(addr)
            },
            _ => Page::empty()
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
        match self.entries[index as uint] {
            p if p.present() => self.table_at(index),
            _ => unsafe {
                // Allocate table
                let table_physical = physical::allocate_frame();
                
                self.entries[index as uint] = Page::new(table_physical, flags);

                let table = self.table_at(index);
                // Flush table so we can write to its virtual address
                flush_tlb(table);

                *table = Table::empty();
                table
            }
        }
    }

    fn table_at(&self, index: u32) -> *mut PageTable {
        let self_addr: u32 = unsafe { transmute(self as *const PageDirectory) };
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
