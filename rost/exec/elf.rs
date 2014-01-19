use core::mem::size_of;
use core::option::{Option, Some, None};

use core2::ptr::{offset, mut_offset};

use kernel::console;
use memory;

enum Ident {
    EI_MAG0 = 0,
    EI_MAG1 = 1,
    EI_MAG2 = 2,
    EI_MAG3 = 3,
    EI_CLASS = 4,
    EI_DATA = 5,
    EI_VERSION = 6,
    EI_OSABI = 7,
    EI_PAD = 8
}

#[packed]
struct ELFIdent {
    ei_mag: [u8, ..4],
    ei_class: u8,
    ei_data: u8,
    ei_version: u8,
    ei_osabi: u8,
    ei_abiversion: u8,
    ei_pad: [u8, ..7]
}

#[packed]
struct ELFHeader {
    e_ident: ELFIdent,
    e_type: u16,
    e_machine: u16,
    e_version: u32,
    e_entry: u32,
    e_phoff: u32,
    e_shoff: u32,
    e_flags: u32,
    e_ehsize: u16,
    e_phentsize: u16,
    e_phnum: u16,
    e_shentsize: u16,
    e_shnum: u16,
    e_shstrndx: u16
}

#[repr(u16)]
enum ObjectType {
    ET_NONE = 0,
    ET_REL = 1,
    ET_EXEC = 2
}

#[repr(u32)]
enum HeaderType {
    PT_NULL = 0,
    PT_LOAD = 1,
    PT_DYNAMIC = 2,
    PT_INTERP = 3,
    PT_NOTE = 4,
    PT_SHLIB = 5,
    PT_PHDR = 6,
    PT_TLS = 7,
    PT_LOOS = 0x60000000,
    PT_HIOS = 0x6fffffff,
    PT_LOPROC = 0x70000000,
    PT_HIPROC = 0x7fffffff
}

#[packed]
struct ProgramHeader {
    p_type: HeaderType,
    p_offset: u32,
    p_vaddr: u32,
    p_paddr: u32,
    p_filesz: u32,
    p_memsz: u32,
    p_flags: u32
}



pub fn probe(buffer: *u8) -> bool {
    match unsafe { read_header(buffer) } {
        Some(_) => true,
        None => false
    }
}

pub fn exec(buffer: *u8) {
    match unsafe { read_header(buffer) } {
        Some(header) => unsafe { start(buffer, header) },
        None => {}
    }
}

fn check_magic(ident: &ELFIdent) -> bool {
    let magic = "\u007fELF";
    ident.ei_mag[0] == magic[0]
        && ident.ei_mag[1] == magic[1]
        && ident.ei_mag[2] == magic[2]
        && ident.ei_mag[3] == magic[3]
}

unsafe fn read_header(buffer: *u8) -> Option<&ELFHeader> {
    let header = buffer as *ELFHeader;
    if check_magic(&(*header).e_ident) {
        Some(&(*header))
    } else {
        None
    }
}

unsafe fn start(buffer: *u8, header: &ELFHeader) {
    if header.e_type != ET_EXEC as u16 {
        // File is not excutable
        console::write_str("Not executable\n");
        return;
    }

    write("Headers count: ", header.e_phnum as u32);

    let mut i: uint = 0;
    while i < header.e_phnum as uint {
        let program_header_offset: uint = header.e_phoff as uint + i * header.e_phentsize as uint;
        let program_header = offset(buffer, program_header_offset as int) as *ProgramHeader;

        write("Offset: ", program_header_offset as u32);
        write("Header at ", program_header as u32);

        match (*program_header).p_type {
            PT_NULL => {}, // Ignore
            PT_LOAD => load_program_header(buffer, &(*program_header)),
            _ => {}
        }

        i += 1;
    }

    write("Entry is at ", header.e_entry);

    asm!("jmp *$0" :: "r"(header.e_entry) :: "volatile");
}

extern {
    fn memcpy(dest: *mut u8, src: *u8, n: int);
    fn memset(s: *mut u8, c: int, n: int);
}

fn write(msg: &str, value: u32) {
    console::write_str(msg);
    console::write_hex(value);
    console::write_newline();
}

unsafe fn load_program_header(buffer: *u8, header: &ProgramHeader) {
    console::write_str("load_program_header()\n");
    memory::map(header.p_vaddr, header.p_memsz, memory::FLAG_WRITE);
    let vaddr = header.p_vaddr as *mut u8;
    console::write_str("Loading data at ");
    console::write_hex(vaddr as u32);
    console::write_newline();
    memcpy(vaddr, offset(buffer, header.p_offset as int), header.p_filesz as int);

    if header.p_memsz > header.p_filesz {
        let difference = header.p_memsz - header.p_filesz;
        memset(mut_offset(vaddr, (header.p_offset + header.p_filesz) as int), 0, difference as int);
    }
}
