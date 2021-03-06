use core::fmt;
use volatile::Volatile;

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

pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                let color_code = self.color_code;
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });
                self.column_position += 1;
                // 刷新光标
                unsafe { update_cursor(row, col) }
            }
        }
    }

    pub fn delete(&mut self) {
        // 删除一个字符
        let row = BUFFER_HEIGHT - 1;
        if self.column_position == 0 {
            // 删除一行的操作
            self.del_line();
        }
        let col = self.column_position - 1;
        let color_code = self.color_code;
        self.buffer.chars[row][col].write(ScreenChar {
            ascii_character: b' ',
            color_code,
        });
        self.column_position = self.column_position - 1;
        // 刷新光标
        unsafe { update_cursor(row, col) }
    }

    // 删除末尾行
    fn del_line(&mut self) {
        for row in 0..BUFFER_HEIGHT - 1 {
            for col in 0..BUFFER_WIDTH {
                let brow = BUFFER_HEIGHT - 1 - row;
                if row < BUFFER_WIDTH {
                    let character = self.buffer.chars[brow - 1][col].read();
                    self.buffer.chars[brow][col].write(character);
                }
            }
        }
        self.column_position = BUFFER_WIDTH;
    }

    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
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

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // 可以是能打印的ASCII码字节，也可以是换行符
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // 不包含在上述范围之内的字节
                _ => self.write_byte(0xfe),
            }
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! del {
    () => {
        $crate::vga_buffer::_del()
    };
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}

#[doc(hidden)]
pub fn _del() {
    use x86_64::instructions::interrupts;
    // 临时暂停中断
    interrupts::without_interrupts(|| {
        WRITER.lock().delete();
    })
}

/**
 * 移动光标
 */
#[inline]
pub unsafe fn update_cursor(x: usize, y: usize) {
    let pos: u16 = x as u16 * BUFFER_WIDTH as u16 + y as u16;
    llvm_asm!("outb %al,%dx"
        :
        :"{dx}"(0x3D4),"{al}"(0x0F)
        :
    );
    llvm_asm!("outb %al,%dx"
        :
        :"{dx}"(0x3D5),"{al}"(pos & 0xFF)
        :
    );
    llvm_asm!("outb %al,%dx"
        :
        :"{dx}"(0x3D4),"{al}"(0x0E)
        :
    );
    llvm_asm!("outb %al,%dx"
        :
        :"{dx}"(0x3D5),"{al}"( (pos>>8) & 0xFF)
        :
    );
}
