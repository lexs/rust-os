use core::container::Container;

use io;
use util::range;

#[repr(u8)]
enum Color {
    Black,
    Blue,
    Green,
    Cyan,
    Red,
    Pink,
    Brown,
    LightGray,
    DarkGray,
    LightBlue,
    LightGreen,
    LightCyan,
    LightRed,
    LightPink,
    Yellow,
    White
}

#[packed]
struct Character {
    char: u8,
    attr: u8
}

impl Character {
    #[inline]
    fn make(c: char, fg: Color, bg: Color) -> Character {
        Character { char: c as u8, attr: fg as u8 | bg as u8 << 4 }
    }
}

pub static ROWS: uint = 25;
pub static COLS: uint = 80;

type Screen = [[Character, ..COLS], ..ROWS];
static SCREEN: *mut Screen = 0xb8000 as *mut Screen;

static mut cursor_x: uint = 0;
static mut cursor_y: uint = 0;

pub fn puts(s: &str) {
    range(0, s.len(), |i| {
        unsafe { do_putch(s[i] as char); }
    });

    unsafe { update_cursor() }
}

pub fn putch(c: char) {
    unsafe {
        do_putch(c);
        update_cursor();
    }
}

pub fn clear_screen() {
    range(0, COLS, |x| {
        range(0, ROWS, |y| {
            unsafe {
                (*SCREEN)[y][x] = Character::make(' ', White, Black);
            }
        })
    });
}

pub fn move_cursor(x: uint, y: uint) {
    unsafe {
        cursor_x = x;
        cursor_y = y;
        update_cursor();
    }
}

unsafe fn do_putch(c: char) {
    match c {
        '\n' => newline(),
        _ => {
            (*SCREEN)[cursor_y][cursor_x] = Character::make(c, White, Black);
            forward_cursor();
        }
    }
}

unsafe fn forward_cursor() {
    cursor_x += 1;

    if cursor_x >= COLS {
        newline();
    }
}

unsafe fn newline() {
    cursor_x = 0;
    cursor_y += 1;
}

unsafe fn update_cursor() {
    let position = cursor_y * COLS + cursor_x;

    io::out(0x3D4, 0x0F);
    io::out(0x3D5, position as u8);
    io::out(0x3D4, 0x0E);
    io::out(0x3D5, (position >> 8) as u8);
}
