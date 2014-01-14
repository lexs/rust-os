use core::mem::size_of;

use console;
use idt;

#[packed]
struct Page(u32);

#[packed]
struct PageTable {
    pages: [Page, ..1024]
}

#[packed]
struct PageDirectory {
    tables: [u32, ..1024]
}

static mut kernel_directory: *mut PageDirectory = 0 as *mut PageDirectory;

static NO_FLAGS: u32 = 0;
static FLAG_PRESENT: u32 = 1 << 0;
static FLAG_WRITE: u32 = 1 << 1;
static FLAG_USER: u32 = 1 << 2;

pub fn init() {
    unsafe {
        kernel_directory = alloc_s::<PageDirectory>();
        *kernel_directory = PageDirectory::new();
    }

    // Identity map all currently used memory
    let mut i = 0;
    while i < unsafe { placement_address } {
        let page = unsafe { (*kernel_directory).get_page(i) };
        page.set(i, FLAG_PRESENT | FLAG_WRITE);
        i += 0x1000;
    }

    idt::register_isr_handler(14, page_fault);

    switch_page_directory(unsafe { kernel_directory });
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
        PageTable { pages: [Page::empty(), ..1024] }
    }
}

impl PageDirectory {
    fn new() -> PageDirectory {
        PageDirectory {
            tables: [0, ..1024]
        }
    }

    unsafe fn get_table(&mut self, address: u32) -> *mut PageTable {
        let table_index = address / (4096 * 1024);

        if to_addr(self.tables[table_index]) == 0 {
            let table = alloc_s::<PageTable>();
            *table = PageTable::empty();

            self.tables[table_index] = table as u32 | FLAG_PRESENT | FLAG_WRITE | FLAG_USER;
            table
        } else {
            to_addr(self.tables[table_index]) as *mut PageTable
        }
    }

    unsafe fn get_page(&mut self, address: u32) -> &mut Page {
        let table = self.get_table(address);

        let page_index = address / 4096;
        &mut (*table).pages[page_index % 1024]
    }

    unsafe fn get_physical(&mut self, address: u32) -> u32 {
        let page = self.get_page(address);
        page.addr() + (address % 1024)
    }
}

fn switch_page_directory(directory: *mut PageDirectory) {
    unsafe {
        let address = (*directory).get_physical(directory as u32);
        write_cr3(address);
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

unsafe fn alloc_s<T>() -> *mut T {
    let size = size_of::<T>();
    alloc::<T>(size as u32)
}

static mut placement_address: u32 = 0;
unsafe fn alloc<T>(size: u32) -> *mut T {
    if placement_address == 0 {
        extern { static kernel_end: u32; }
        placement_address = (&kernel_end as *u32) as u32;
    }

    if placement_address & !0xfff != 0 {
        placement_address &= !0xfff;
        placement_address += 0x1000;
    }

    let address = placement_address;
    placement_address += size;
    address as *mut T
}
