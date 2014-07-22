use core::prelude::*;
use core::fmt;
use core::fmt::FormatWriter;

use drivers::{vga, ansi};

static mut ansi: Option<ansi::Ansi> = None;

pub struct Console;
pub struct AnsiConsole;

impl Console {
    pub fn print(&mut self, c: char) {
        vga::putch(c);
    }
}

impl AnsiConsole {
    pub fn print(&mut self, c: char) {
        match unsafe { ansi.as_mut() } {
            None => return Console.print(c),
            Some(ansi) => self.print_ansi(c, ansi)
        };
    }

    fn print_ansi(&mut self, c: char, ansi: &mut ansi::Ansi) {
        ansi.put(c, self);
    }
}

impl fmt::FormatWriter for Console {
    fn write(&mut self, bytes: &[u8]) -> fmt::Result {
        for &c in bytes.iter() {
            vga::putch(c as char);
        }
        Ok(())
    }
}

impl fmt::FormatWriter for AnsiConsole {
    fn write(&mut self, bytes: &[u8]) -> fmt::Result {
        let ansi = match unsafe { ansi.as_mut() } {
            None => return Console.write(bytes),
            Some(ansi) => ansi
        };

        for &c in bytes.iter() {
            self.print_ansi(c as char, ansi);
        }
        Ok(())
    }
}

impl ansi::Device for AnsiConsole {
    fn write(&mut self, c: char) {
        Console.print(c);
    }

    fn set_cursor(&mut self, x: uint, y: uint) {
        vga::move_cursor(x, y);
    }

    fn get_cursor(&self) -> (uint, uint) {
        vga::get_cursor()
    }

    fn set_color(&mut self, fg: ansi::Color, bg: ansi::Color, flags: ansi::Flags) {
        vga::set_color(translate_fg(fg, flags), translate_bg(bg));
    }
}

pub fn init() {
    unsafe {
        ansi = Some(ansi::Ansi::new())
    }
}

fn translate_fg(color: ansi::Color, flags: ansi::Flags) -> vga::Color {
    translate_color(color, flags)
}

fn translate_bg(color: ansi::Color) -> vga::Color {
    translate_color(color, ansi::Flags::empty())
}

fn translate_color(color: ansi::Color, flags: ansi::Flags) -> vga::Color {
    let vga_color = match color {
        ansi::Black     => vga::BLACK,
        ansi::Blue      => vga::BLUE,
        ansi::Green     => vga::GREEN,
        ansi::Cyan      => vga::CYAN,
        ansi::Red       => vga::RED,
        ansi::Magenta   => vga::MAGENTA,
        ansi::Yellow    => vga::BROWN,
        ansi::White     => vga::WHITE
    };

    if flags.contains(ansi::BRIGHT) {
        vga_color | vga::BRIGHT
    } else {
        vga_color
    }
}
