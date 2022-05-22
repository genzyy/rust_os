use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;

lazy_static! {
        // statics are initialized at compile time so we cannot use vga_buffer address.
        // lazy_static code will be initialized when it is being accessed for first time.
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::LightGreen, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) }   // use of static mut is discouraged because it can lead to data races and data overwrite.
    });
}

// to mutate a variable when there are immutable references to it, we use interior mutability.
// To get a basic lock and mutual exclusion without any lock ability and threads, we can use spinlock -> a basic kind of mutex.
// It keeps the thread in a tight loop constantly looking for mutex, and once mutex is free, thread can take it and it does not block mutex.

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

// The standard color palette in VGA text mode.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
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

// A combination of a foreground and a background color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        return ColorCode((background as u8) << 4 | (foreground as u8));
    }
}

// A screen character in the VGA text buffer, consisting of an ASCII character and a `ColorCode`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
// #[repr(transparent)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

// Height of text buffer -> 25 lines.
const BUFFER_HEIGHT: usize = 25;
// Width of text buffer -> 80 columns.
const BUFFER_WIDTH: usize = 80;

// A structure that represents vga buffer where we can write content.
#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    column_position: usize,      // keeps track of current position of carret
    color_code: ColorCode, // color with which current buffer needs to be written on vga buffer
    buffer: &'static mut Buffer, // reference to that buffer -> 'static tells that the reference is valid for the whole program.
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line()
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                // this gurantees that compiler will never optimize
                // this or remove it as its redundant.
                let color_code = self.color_code;
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });
                self.column_position += 1;
            }
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                //printable ASCII byte or newline
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // not in the printable ASCII range
                _ => self.write_byte(0xfe),
            }
        }
    }

    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character: ScreenChar = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }

        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };

        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }

    pub fn print_something() {
        use core::fmt::Write;
        let mut writer: Writer = Writer {
            column_position: 0,
            color_code: ColorCode::new(Color::Yellow, Color::Black),
            buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
        };

        writer.write_byte(b'H');
        writer.write_string("ello! ");
        write!(writer, "The numbers are {} and {}", 42, 2.0 / 3.0).unwrap();
    }
}

// macros define how the given argument should be formatted and printed or returned.
// macros are preprocessed before compilation and are different from functions.
// functions are compiled while macros are preprocessed.

// make this macro available to the whole crate.
#[macro_export]
// _print function comes from io crate and format_args macro formats all arguments
// passed to custom macro.
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => (crate::print!("\n"));
    ($($arg:tt)*) => (crate::print!("{}\n", format_args!($($arg)*)))
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}

// custom macro.
// macro_rules! hello {
//     () => {
//         return print!("\n");
//     };
//     ($($arg:tt)*) => (print!("{}\n", format_args!($($arg)*)))
// }
