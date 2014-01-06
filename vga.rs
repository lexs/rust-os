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
        '\t' => unsafe { forward_cursor(4 - (cursor_x + 4) % 4); },
        '\u0008' => unsafe { erase(); },
        _ => {
            (*SCREEN)[cursor_y][cursor_x] = Character::make(c, White, Black);
            forward_cursor(1);
        }
    }
}

unsafe fn erase() {
    if cursor_x != 0 {
        cursor_x -= 1;
    } else if cursor_y != 0 {
        cursor_x = COLS - 1;
        cursor_y -= 1;
    }

    (*SCREEN)[cursor_y][cursor_x] = Character::make(' ', White, Black);
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
