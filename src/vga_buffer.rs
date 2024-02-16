#![allow(dead_code)]

use core::fmt;
use volatile::Volatile;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Meganta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

// #[derive(Default)]
pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
    row_num: usize,
}

impl Writer {
    // pub fn write_byte(&mut self, byte: u8) {
    //     match byte {
    //         b'\n' => self.new_line(),
    //         _ => {
    //             if self.column_position >= BUFFER_WIDTH {
    //                 self.new_line();
    //             }

    //             let row = BUFFER_HEIGHT - 1;
    //             let column = self.column_position;
    //             let color_code = self.color_code;

    //             self.buffer.chars[row][column].write(ScreenChar {
    //                 ascii_character: byte,
    //                 color_code: color_code,
    //             });
    //             self.column_position += 1;
    //         }
    //     }
    // }

    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => {
                self.new_line();
            }
            _ => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = self.row_num;
                let column = self.column_position;
                let color_code = self.color_code;

                self.buffer.chars[row][column].write(ScreenChar {
                    ascii_character: byte,
                    color_code: color_code,
                });
                self.column_position += 1;
            }
        }
    }

    fn new_line(&mut self) {
        if self.row_num < BUFFER_HEIGHT - 1 {
            self.row_num += 1;
            self.column_position = 0;
        } else {
            for r in 0..BUFFER_HEIGHT - 1 {
                for c in 0..BUFFER_WIDTH {
                    self.buffer.chars[r][c].write(self.buffer.chars[r + 1][c].read());
                }
            }

            // CLEAR THE LAST ROW
            let row = BUFFER_HEIGHT - 1;
            let blank_char = ScreenChar {
                ascii_character: 0,
                color_code: ColorCode(0),
            };
            for c in 0..BUFFER_WIDTH {
                self.buffer.chars[row][c].write(blank_char);
            }

            // REPOSITION THE COLUMN POINTER TO THE BEGINNING
            self.column_position = 0;
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            if (byte >= 0x20 && byte <= 0x7e) || byte == b'\n' {
                self.write_byte(byte);
            } else {
                self.write_byte(0xfe);
            }
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }

    // fn write_char(&mut self, c: char) -> fmt::Result {
    //     let mut x:u8 = c as u8;
    //     x += 0x20;

    //     if x <= 0x20 {
    //         self.write_string("less than 20\n");
    //     } else if x > 0x7e {
    //         self.write_string("greater than 7e\n");
    //     } else {
    //         self.write_string("found it\n");
    //     }
    //     Ok(())
    // }
}

use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::LightGreen, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
        row_num: 0,
    });
}

// pub fn print_something() -> () {
//     let x = WRITER.lock().buffer.chars[0][0].read().ascii_character;
//     WRITER.lock().write_char(x as char).unwrap();
// }

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {$crate::vga_buffer::_print(format_args!($($arg)*));};
}

#[macro_export]
macro_rules! println {
    () => { $crate::print!("\n"); };
    ($($arg:tt)*) => {
        $crate::print!("{}\n", format_args!($($arg)*));
    }
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}
