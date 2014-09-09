use core::prelude::*;
use core::mem::swap;

static ESCAPE: char = '\x1b';

static LOW: char = 'A';
static HIGH: char = 'z';

static CUU: char = 'A'; // Cursor up
static CUD: char = 'B'; // Cursor down
static CUF: char = 'C'; // Cursor forward
static CUB: char = 'D'; // Cursor back

static SGR: char = 'm'; // Select Graphic Rendition

#[deriving(FromPrimitive)]
#[repr(u8)]
pub enum Color {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White
}

bitflags!(
    #[packed]
    flags Flags: u8 {
        static BRIGHT = 1,
        static UNDERLINE = 1 << 1,
        static BLINK = 1 << 2
    }
)

enum Status {
    Idle,
    Pending,
    Escaped
}

pub trait Device {
    fn write(&mut self, c: char);
    fn set_cursor(&mut self, x: uint, y: uint);
    fn get_cursor(&self) -> (uint, uint);
    fn set_color(&mut self, fg: Color, bg: Color, flags: Flags);
}

pub struct Ansi {
    status: Status,
    buffer: [char, ..100],
    buffer_pos: uint,
    state: State
}

struct State {
    fg: Color,
    bg: Color,
    flags: Flags
}

impl State {
    fn default() -> State {
        State {
            fg: White,
            bg: Black,
            flags: Flags::empty()
        }
    }
}

impl Ansi {
    pub fn new() -> Ansi {
        Ansi {
            status: Idle,
            buffer: ['\0', ..100],
            buffer_pos: 0,
            state: State::default()
        }
    }

    pub fn put(&mut self, c: char, device: &mut Device) {
        match (self.status, c) {
            (Idle, ESCAPE) => {
                self.put_buf(c);
                self.status = Pending;
            },
            (Idle, c) => device.write(c),
            (Pending, '[') => {
                self.put_buf(c);
                self.status = Escaped;
            },
            (Pending, c) => {
                // We were not escaped
                self.dump_buf(c, |c| device.write(c));
                self.status = Idle;
            },
            (Escaped, LOW..HIGH) => {
                self.handle_code(c, device);
                self.clear_buf();
                self.status = Idle;
            },
            (Escaped, c) => self.put_buf(c)
        }
    }

    fn handle_code(&mut self, c: char, device: &mut Device) {
        let buffer = self.buffer.slice(0, self.buffer_pos);
        let mut args = match buffer.split(|&c| c == '[').nth(1) {
            None => return,
            Some(options) => options.split(|&c| c == ';')
        };

        match c {
            SGR => {
                let mut had_args = false;
                for arg in args {
                    had_args = true;

                    self.state.handle_sgr(arg);
                }

                if !had_args {
                    self.state.handle_sgr(['0']);
                }

                device.set_color(self.state.fg, self.state.bg, self.state.flags);

            },
            CUU..CUB => {
                let (direction_x, direction_y) = match c {
                    CUU => (-1, 0),
                    CUD => (1, 0),
                    CUF => (0, 1),
                    CUB => (0, -1),
                    _ => unreachable!()
                };

                let distance = self.first_or(&mut args, 1);
                let (x, y) = device.get_cursor();
                device.set_cursor(direction_x * distance * x, direction_y * distance * y);
            }
            _ => klog!("Unsuppored ansi code: {}", c)
        }
    }

    fn first_or<'a, T: Iterator<&'a [char]>>(&self, args: &mut T, default: uint) -> uint {
        args.nth(0).and_then(from_str).unwrap_or(default)
    }

    fn put_buf(&mut self, c: char) {
        self.buffer[self.buffer_pos] = c;
        self.buffer_pos += 1;
        kassert!(self.buffer_pos < 100);
    }

    fn dump_buf(&mut self, c: char, write: |c: char|) {
        for &c in self.buf().iter() {
            write(c);
        }
        write(c);
        self.buffer_pos = 0;
    }

    fn clear_buf(&mut self) {
        self.buffer_pos = 0;
    }

    fn buf<'a>(&'a self) -> &'a [char] {
        self.buffer.slice(0, self.buffer_pos)
    }
}

static SGR_RESET: uint = 0;
static SGR_BOLD: uint = 1;
static SGR_UNDERLINE: uint = 4;
static SGR_BLINK: uint = 5;
static SGR_REVERSE: uint = 7;
static SGR_FG_LOW: uint = 30;
static SGR_FG_HIGH: uint = 37;
static SGR_FG_RESET: uint = 39;
static SGR_BG_LOW: uint = 40;
static SGR_BG_HIGH: uint = 47;
static SGR_BG_RESET: uint = 49;

impl State {
    fn handle_sgr(&mut self, arg: &[char]) {
        let value = match from_str(arg) {
            None => return,
            Some(value) => value
        };

        match value {
            SGR_RESET               => *self = State::default(),
            SGR_BOLD                => self.flags.insert(BRIGHT),
            SGR_UNDERLINE           => self.flags.insert(UNDERLINE),
            SGR_BLINK               => self.flags.insert(BLINK),
            SGR_REVERSE             => swap(&mut self.fg, &mut self.bg),
            SGR_FG_LOW..SGR_FG_HIGH => self.fg = fg(value - SGR_FG_LOW),
            SGR_FG_RESET            => self.fg = White,
            SGR_BG_LOW..SGR_BG_HIGH => self.bg = bg(value - SGR_BG_LOW),
            SGR_BG_RESET            => self.bg = Black,
            _                       => ()
        }
    }
}

fn fg(color: uint) -> Color {
    FromPrimitive::from_uint(color).unwrap_or(White)
}

fn bg(color: uint) -> Color {
    FromPrimitive::from_uint(color).unwrap_or(Black)
}

fn from_str(text: &[char]) -> Option<uint> {
    let mut value = 0;
    for &c in text.iter() {
        value = value * 10 + match c.to_digit(10) {
            None => return None,
            Some(digit) => digit
        }
    }
    Some(value)
}
