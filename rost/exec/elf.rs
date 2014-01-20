use core::ptr::{copy_nonoverlapping_memory, set_memory};
use core::option::{Option, Some, None};

use core2::ptr::{offset, mut_offset};

use kernel::console;
use memory;

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
}

// Header flags
static PT_X: u32 = 0x1;
static PT_R: u32 = 0x2;
static PT_W: u32 = 0x4;

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

    let mut i: uint = 0;
    while i < header.e_phnum as uint {
        let program_header_offset = header.e_phoff as uint + i * header.e_phentsize as uint;
        let program_header = offset(buffer, program_header_offset as int) as *ProgramHeader;

        match (*program_header).p_type {
            PT_NULL => {}, // Ignore
            PT_LOAD => load_segment(buffer, program_header),
            _ => {
                console::write_str("Unsupported ELF segment\n");
                return;
            }
        }

        i += 1;
    }

    asm!("jmp *$0" :: "r"(header.e_entry) :: "volatile");
}

unsafe fn load_segment(buffer: *u8, header: *ProgramHeader) {
    let memsize = (*header).p_memsz; // Size in memory
    let filesize = (*header).p_filesz; // Size in file
    let mempos = (*header).p_vaddr as *mut u8; // Position in memory
    let filepos = (*header).p_offset; // Position in file

    memory::map(mempos as u32, memsize, memory::FLAG_PRESENT | translate_flags(header));

    copy_nonoverlapping_memory(mempos, offset(buffer, filepos as int), filesize as uint);
    set_memory(mut_offset(mempos, (filepos + filesize) as int), 0, (memsize - filesize) as uint);
}

unsafe fn translate_flags(header: *ProgramHeader) -> u32 {
    if (*header).p_flags & PT_W != 0 {
        memory::FLAG_WRITE
    } else {
        0
    }
}
