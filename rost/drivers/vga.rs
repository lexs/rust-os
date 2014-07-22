use core::prelude::*;
use core::intrinsics::volatile_store;

use arch::io;

bitflags!(
    #[packed]
    flags Color: u8 {
        static BRIGHT = 1 << 3,

        static BLACK = 0,

        static RED = 1 << 2,
        static GREEN = 1 << 1,
        static BLUE = 1 << 0,

        static CYAN = BLUE.bits | GREEN.bits,
        static MAGENTA = BLUE.bits | RED.bits,
        static BROWN = GREEN.bits | RED.bits,

        static WHITE = BLUE.bits | GREEN.bits | RED.bits
    }
)

#[packed]
struct Character {
    char: u8,
    attr: u8
}

impl Character {
    #[inline]
    fn make(c: char, fg: Color, bg: Color) -> Character {
        Character { char: c as u8, attr: fg.bits() | bg.bits() << 4 }
    }
}

pub static ROWS: uint = 25;
pub static COLS: uint = 80;

static screen: *mut Character = 0xb8000 as *mut Character;

static mut cursor_x: uint = 0;
static mut cursor_y: uint = 0;

pub fn puts(s: &str) {
    for c in s.chars() {
        unsafe { do_putch(c); }
    }

    unsafe { update_cursor() }
}

pub fn putch(c: char) {
    unsafe {
        if cursor_y > ROWS {
            clear_screen();
        }

        do_putch(c);
        update_cursor();
    }
}

pub fn clear_screen() {
    for x in range(0, COLS) {
        for y in range(0, ROWS) {
            unsafe { write(y, x, Character::make(' ', WHITE, BLACK)); }
        }
    }
    move_cursor(0, 0);
}

pub fn get_cursor() -> (uint, uint) {
    unsafe { (cursor_x, cursor_y) }
}

pub fn move_cursor(x: uint, y: uint) {
    unsafe {
        cursor_x = x;
        cursor_y = y;
        update_cursor();
    }
}

static mut fg: Color = WHITE;
static mut bg: Color = BLACK;

pub fn set_color(_fg: Color, _bg: Color) {
    unsafe {
        fg = _fg;
        bg = _bg;
    }
}

unsafe fn do_putch(c: char) {
    match c {
        '\n' => newline(),
        '\t' => tab(),
        '\u0008' => backspace(),
        _ => {
            write(cursor_y, cursor_x, Character::make(c, fg, bg));
            forward_cursor(1);
        }
    }
}

unsafe fn tab() {
    forward_cursor(4 - (cursor_x + 4) % 4);
}

unsafe fn backspace() {
    if cursor_x != 0 {
        cursor_x -= 1;
    } else if cursor_y != 0 {
        cursor_x = COLS - 1;
        cursor_y -= 1;
    }

    write(cursor_y, cursor_x, Character::make(' ', WHITE, BLACK));
}

#[inline]
unsafe fn write(y: uint, x: uint, c: Character) {
    let offset = y * COLS + x;
    volatile_store(screen.offset(offset as int), c);
}

unsafe fn forward_cursor(steps: uint) {
    cursor_x += steps;

    while cursor_x >= COLS {
        cursor_x -= COLS;
        cursor_y += 1;
    }
}

unsafe fn newline() {
    cursor_x = 0;
    cursor_y += 1;
}

unsafe fn update_cursor() {
    let position = cursor_y * COLS + cursor_x;

    io::write_port(0x3D4, 0x0F);
    io::write_port(0x3D5, position as u8);
    io::write_port(0x3D4, 0x0E);
    io::write_port(0x3D5, (position >> 8) as u8);
}
