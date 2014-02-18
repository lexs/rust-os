use core;
use core::ptr::{copy_nonoverlapping_memory, set_memory};
use core::option::{Option, Some, None};

use core2::ptr::{offset, mut_offset};

use memory;
use exec::tasking;

// FIXME: Where should the stack go?
static STACK_POSITION: u32 = 0x5600000;
static STACK_SIZE: u32 = 8 * 1024;

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
    PT_LOOS = 0x60000000, // OS-specific
    PT_HIOS = 0x6fffffff, // OS-specific
    PT_GNU_STACK = 0x60000000 + 0x474e551
}

define_flags!(HeaderFlags: u32 {
    PT_X = 0x1,
    PT_R = 0x2,
    PT_W = 0x4
})

#[packed]
struct ProgramHeader {
    p_type: HeaderType,
    p_offset: u32,
    p_vaddr: u32,
    p_paddr: u32,
    p_filesz: u32,
    p_memsz: u32,
    p_flags: HeaderFlags
}

pub fn probe(buffer: *u8) -> bool {
    let header = buffer as *ELFHeader;
    unsafe { check_magic(&(*header).e_ident) }
}

pub fn exec(buffer: *u8) {
    unsafe {
        let header = buffer as *ELFHeader;

        setup(buffer, header).map(|entry| {
            memory::map(STACK_POSITION, STACK_SIZE, memory::PRESENT | memory::USER | memory::WRITE);

            let stack_top = STACK_POSITION + STACK_SIZE;
            tasking::user_mode(entry, stack_top)
        });
    }
}

unsafe fn check_magic(ident: *ELFIdent) -> bool {
    static MAGIC: &'static str = "\u007fELF";
    let ref ei_mag = (*ident).ei_mag;

    ei_mag[0] == MAGIC[0]
        && ei_mag[1] == MAGIC[1]
        && ei_mag[2] == MAGIC[2]
        && ei_mag[3] == MAGIC[3]
}

unsafe fn setup(buffer: *u8, header: *ELFHeader) -> Option<u32> {
    if (*header).e_type != ET_EXEC as u16 {
        // File is not excutable
        kprintln!("Not executable");
        return None;
    }

    let header_count = (*header).e_phnum as int;
    let header_size = (*header).e_phentsize as int;
    let header_base = offset(buffer, (*header).e_phoff as int);

    let mut i: int = 0;
    while i < header_count {
        let program_header = offset(header_base, i * header_size) as *ProgramHeader;

        // Does this program need an executable stack
        let mut exec_stack = true;

        match (*program_header).p_type {
            PT_NULL => {}, // Ignore
            PT_LOAD => load_segment(buffer, program_header),
            PT_GNU_STACK => {
                // We don't need an executable stack if the exec flag is not set
                if (*program_header).p_flags & !PT_X {
                    exec_stack = false;
                }
            },
            other => {
                kprintln!("Unsupported ELF segment: {}", other as u32);
                return None;
            }
        }

        i += 1;
    }

    Some((*header).e_entry)
}

unsafe fn load_segment(buffer: *u8, header: *ProgramHeader) {
    let mem_size = (*header).p_memsz as uint; // Size in memory
    let file_size = (*header).p_filesz as uint; // Size in file
    let mem_pos = (*header).p_vaddr as *mut u8; // Position in memory
    let file_pos = (*header).p_offset as int; // Position in file

    memory::map(mem_pos as u32, mem_size as u32, memory::PRESENT | memory::USER | translate_flags(header));

    copy_nonoverlapping_memory(mem_pos, offset(buffer, file_pos as int), file_size as uint);
    set_memory(mut_offset(mem_pos, file_pos + file_size as int), 0, mem_size - file_size);
}

unsafe fn translate_flags(header: *ProgramHeader) -> memory::Flags {
    if (*header).p_flags & PT_W {
        memory::WRITE
    } else {
        memory::NONE
    }
}
