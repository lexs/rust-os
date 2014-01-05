use core::container::Container;
use core::mem::size_of;

static GDT_SIZE: uint = 5;
type GdtTable = [GdtEntry, ..GDT_SIZE];

#[packed]
struct GdtEntry {
    limit_low: u16,
    base_low: u16,
    base_middle: u8,
    access: u8,
    granularity: u8,
    base_high: u8
}

#[packed]
struct GdtPtr {
    limit: u16,
    base: *GdtTable
}

impl GdtEntry {
    fn new(base: uint, limit: uint, access: u8, granularity: u8) -> GdtEntry {
        GdtEntry {
            base_low: (base & 0xFFFF) as u16,
            base_middle: ((base >> 16) & 0xFF) as u8,
            base_high: ((base >> 24) & 0xFF) as u8,
            limit_low: (limit & 0xFFFF) as u16,
            granularity: (((limit >> 16) & 0x0F) as u8) | granularity & 0xF0,
            access: access
        }
    }
}

impl GdtPtr {
    fn new(table: &GdtTable) -> GdtPtr {
        GdtPtr {
            limit: (size_of::<GdtEntry>() * table.len() - 1) as u16,
            base: table as *GdtTable
        }
    }
}

static mut entries: GdtTable = [
    GdtEntry {
        base_low: 0,
        base_middle: 0,
        base_high: 0,
        limit_low: 0,
        granularity: 0,
        access: 0
    }, ..GDT_SIZE
];

static mut table: GdtPtr = GdtPtr { limit: 0, base: 0 as *GdtTable };

pub fn init() {
    unsafe {
        entries[0] = GdtEntry::new(0, 0, 0, 0);
        entries[1] = GdtEntry::new(0, 0xFFFFFFFF, 0x9A, 0xCF);
        entries[2] = GdtEntry::new(0, 0xFFFFFFFF, 0x92, 0xCF);
        entries[3] = GdtEntry::new(0, 0xFFFFFFFF, 0xFA, 0xCF);
        entries[4] = GdtEntry::new(0, 0xFFFFFFFF, 0xF2, 0xCF);

        table = GdtPtr::new(&entries);

        gdt_flush(&table, 0x08, 0x10);
    }
}

unsafe fn gdt_flush(ptr: *GdtPtr, codeseg: u16, dataseg: u16) {
    asm!("lgdt ($0)" :: "r"(ptr));
    asm!("jmp $0, $$.g; .g:" :: "Ir"(codeseg));
    asm!("mov $0, %ax;  \
         mov %ax, %ds; \
         mov %ax, %es; \
         mov %ax, %fs; \
         mov %ax, %ss" :: "Ir"(dataseg) : "ax");
}
