#![allow(dead_code)]

use core::prelude::*;

use arch::irq;
use arch::idt;
use arch::io;
use drivers::vga;

static KEYMAP: &'static str = "\
\x00\x1B1234567890-=\x08\tqwertyuiop[]\n?asdfghjkl;'`?\\zxcvbnm,./?*? ?????????????789-456+1230.?????";
static KEYMAP_SHIFTED: &'static str = "\
\x00\x1B!@#$%^&*()_+\x08\tQWERTYUIOP{}\n?ASDFGHJKL:\"~?|ZXCVBNM<>??*? ?????????????789-456+1230.?????";

static LEFT_SHIFT: u8 = 0x2a;
static RIGHT_SHIFT: u8 = 0x36;
static CAPS_LOCK: u8 = 0x3a;
static NUMBER_LOCK: u8 = 0x45;
static SCROLL_LOCK: u8 = 0x46;

static mut shifted: bool = false;
static mut caps_lock: bool = false;

pub fn init() {
    irq::register_handler(1, keyboard_handler);
}

fn keyboard_handler(_: &mut idt::Registers) {
    let status: u8 = io::read_port(0x64);
    if status & 0x1 == 0 {
        return;
    }

    let scancode: u8 = io::read_port(0x60);

    // Top bit means key released
    if scancode & 0x80 != 0 {
        key_up(!0x80);
    } else {
        key_down(scancode);
    }
}

fn key_up(scancode: u8) {
    match scancode {
        LEFT_SHIFT | RIGHT_SHIFT => unsafe { shifted = false; },
        _ => {}
    }
}

fn key_down(scancode: u8) {
    match scancode {
        LEFT_SHIFT | RIGHT_SHIFT => unsafe { shifted = true; },
        CAPS_LOCK => unsafe { caps_lock = !caps_lock; },
        _ => { write(scancode); }
    }
}

fn write(scancode: u8) {
    if scancode > KEYMAP.len() as u8 { return; }

    let c: char = unsafe {
        if shifted ^ caps_lock {
            KEYMAP_SHIFTED.char_at(scancode as uint)
        } else {
            KEYMAP.char_at(scancode as uint)
        }
    };

    vga::putch(c);
}
