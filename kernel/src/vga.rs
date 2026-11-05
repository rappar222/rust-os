use core::fmt::{self, Write};
use spin::Mutex;
use bootloader_api::info::{FrameBuffer, FrameBufferInfo, PixelFormat};
use crate::font::{FONT_DATA, FONT_HEIGHT, FONT_WIDTH};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ColorCode {
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
    LightMagenta = 13,
    Yellow = 14,
    White = 15,
    RustOrange = 16,
}

#[derive(Debug, Clone, Copy)]
pub struct RgbColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl ColorCode {
    pub fn to_rgb(self) -> RgbColor {
        match self {
            ColorCode::Black => RgbColor { r: 15, g: 15, b: 15 },
            ColorCode::Blue => RgbColor { r: 0, g: 0, b: 170 },
            ColorCode::Green => RgbColor { r: 0, g: 170, b: 0 },
            ColorCode::Cyan => RgbColor { r: 0, g: 170, b: 170 },
            ColorCode::Red => RgbColor { r: 170, g: 0, b: 0 },
            ColorCode::Magenta => RgbColor { r: 170, g: 0, b: 170 },
            ColorCode::Brown => RgbColor { r: 215, g: 106, b: 40 }, // Rich Rust Orange
            ColorCode::LightGray => RgbColor { r: 170, g: 170, b: 170 },
            ColorCode::DarkGray => RgbColor { r: 85, g: 85, b: 85 },
            ColorCode::LightBlue => RgbColor { r: 85, g: 85, b: 255 },
            ColorCode::LightGreen => RgbColor { r: 85, g: 255, b: 85 },
            ColorCode::LightCyan => RgbColor { r: 85, g: 255, b: 255 },
            ColorCode::LightRed => RgbColor { r: 255, g: 85, b: 85 },
            ColorCode::LightMagenta => RgbColor { r: 255, g: 85, b: 255 },
            ColorCode::Yellow => RgbColor { r: 245, g: 165, b: 65 }, // Amber / Warm Rust Highlight
            ColorCode::White => RgbColor { r: 255, g: 255, b: 255 },
            ColorCode::RustOrange => RgbColor { r: 222, g: 85, b: 28 }, // Official Rust reddish-orange #DE551C
        }
    }
}

pub struct ConsoleWriter {
    buffer: &'static mut [u8],
    info: FrameBufferInfo,
    col: usize,
    row: usize,
    max_cols: usize,
    max_rows: usize,
    fg_color: ColorCode,
    bg_color: ColorCode,
}

impl ConsoleWriter {
    pub fn new(framebuffer: &'static mut FrameBuffer) -> Self {
        let info = framebuffer.info();
        let buffer = framebuffer.buffer_mut();
        let max_cols = info.width / FONT_WIDTH;
        let max_rows = info.height / FONT_HEIGHT;

        let mut writer = Self {
            buffer,
            info,
            col: 0,
            row: 0,
            max_cols,
            max_rows,
            fg_color: ColorCode::LightCyan,
            bg_color: ColorCode::Black,
        };
        writer.clear_screen();
        writer
    }

    pub fn set_colors(&mut self, fg: ColorCode, bg: ColorCode) {
        self.fg_color = fg;
        self.bg_color = bg;
    }

    pub fn set_fg(&mut self, fg: ColorCode) {
        self.fg_color = fg;
    }

    pub fn set_bg(&mut self, bg: ColorCode) {
        self.bg_color = bg;
    }

    pub fn draw_pixel(&mut self, x: usize, y: usize, color: RgbColor) {
        if x >= self.info.width || y >= self.info.height {
            return;
        }

        let pixel_offset = (y * self.info.stride + x) * self.info.bytes_per_pixel;
        if pixel_offset + self.info.bytes_per_pixel > self.buffer.len() {
            return;
        }

        match self.info.pixel_format {
            PixelFormat::Rgb => {
                self.buffer[pixel_offset] = color.r;
                self.buffer[pixel_offset + 1] = color.g;
                self.buffer[pixel_offset + 2] = color.b;
            }
            PixelFormat::Bgr => {
                self.buffer[pixel_offset] = color.b;
                self.buffer[pixel_offset + 1] = color.g;
                self.buffer[pixel_offset + 2] = color.r;
            }
            PixelFormat::U8 => {
                let gray = ((color.r as u16 + color.g as u16 + color.b as u16) / 3) as u8;
                self.buffer[pixel_offset] = gray;
            }
            _ => {
                self.buffer[pixel_offset] = color.r;
                self.buffer[pixel_offset + 1] = color.g;
                self.buffer[pixel_offset + 2] = color.b;
            }
        }
    }

    pub fn read_pixel(&self, x: usize, y: usize) -> RgbColor {
        if x >= self.info.width || y >= self.info.height {
            return RgbColor { r: 0, g: 0, b: 0 };
        }
        let pixel_offset = (y * self.info.stride + x) * self.info.bytes_per_pixel;
        if pixel_offset + 3 > self.buffer.len() {
            return RgbColor { r: 0, g: 0, b: 0 };
        }
        match self.info.pixel_format {
            PixelFormat::Bgr => RgbColor {
                b: self.buffer[pixel_offset],
                g: self.buffer[pixel_offset + 1],
                r: self.buffer[pixel_offset + 2],
            },
            _ => RgbColor {
                r: self.buffer[pixel_offset],
                g: self.buffer[pixel_offset + 1],
                b: self.buffer[pixel_offset + 2],
            },
        }
    }

    pub fn fill_rect(&mut self, x: usize, y: usize, width: usize, height: usize, color: RgbColor) {
        let max_x = core::cmp::min(x + width, self.info.width);
        let max_y = core::cmp::min(y + height, self.info.height);
        if max_x <= x || max_y <= y {
            return;
        }

        let bpp = self.info.bytes_per_pixel;
        let stride = self.info.stride;
        let count = max_x - x;

        if bpp == 4 {
            let pixel_val: u32 = match self.info.pixel_format {
                PixelFormat::Bgr => {
                    (color.b as u32) | ((color.g as u32) << 8) | ((color.r as u32) << 16) | 0xFF000000
                }
                _ => {
                    (color.r as u32) | ((color.g as u32) << 8) | ((color.b as u32) << 16) | 0xFF000000
                }
            };

            for cy in y..max_y {
                let row_start = (cy * stride + x) * 4;
                if row_start + count * 4 <= self.buffer.len() {
                    let ptr = unsafe { self.buffer.as_mut_ptr().add(row_start) as *mut u32 };
                    let slice = unsafe { core::slice::from_raw_parts_mut(ptr, count) };
                    slice.fill(pixel_val);
                }
            }
        } else {
            for cy in y..max_y {
                for cx in x..max_x {
                    self.draw_pixel(cx, cy, color);
                }
            }
        }
    }

    pub fn clear_screen(&mut self) {
        let bg = self.bg_color.to_rgb();
        let w = self.info.width;
        let h = self.info.height;
        self.fill_rect(0, 0, w, h, bg);
        self.col = 0;
        self.row = 0;
        self.draw_cursor();
    }

    pub fn draw_char_at(&mut self, col: usize, row: usize, c: char, fg: ColorCode, bg: ColorCode) {
        if col >= self.max_cols || row >= self.max_rows {
            return;
        }

        let char_idx = (c as usize) & 0xFF;
        let glyph = FONT_DATA[char_idx];
        let fg_rgb = fg.to_rgb();
        let bg_rgb = bg.to_rgb();

        let base_x = col * FONT_WIDTH;
        let base_y = row * FONT_HEIGHT;

        for (dy, &row_bits) in glyph.iter().enumerate() {
            for dx in 0..FONT_WIDTH {
                let pixel_x = base_x + dx;
                let pixel_y = base_y + dy;
                let is_fg = (row_bits & (0x80 >> dx)) != 0;
                let color = if is_fg { fg_rgb } else { bg_rgb };
                self.draw_pixel(pixel_x, pixel_y, color);
            }
        }
    }

    fn erase_cursor(&mut self) {
        self.draw_char_at(self.col, self.row, ' ', self.fg_color, self.bg_color);
    }

    fn draw_cursor(&mut self) {
        if self.col < self.max_cols && self.row < self.max_rows {
            let base_x = self.col * FONT_WIDTH;
            let cursor_y = self.row * FONT_HEIGHT + FONT_HEIGHT - 2;
            let fg_rgb = self.fg_color.to_rgb();

            for dy in 0..2 {
                for dx in 0..FONT_WIDTH {
                    self.draw_pixel(base_x + dx, cursor_y + dy, fg_rgb);
                }
            }
        }
    }

    pub fn scroll(&mut self) {
        let bytes_per_pixel = self.info.bytes_per_pixel;
        let stride = self.info.stride;
        let line_bytes = stride * bytes_per_pixel;
        let font_line_bytes = FONT_HEIGHT * line_bytes;

        let total_valid_bytes = (self.max_rows * FONT_HEIGHT) * line_bytes;

        // Shift rows up by 1 font line
        if total_valid_bytes > font_line_bytes {
            self.buffer.copy_within(font_line_bytes..total_valid_bytes, 0);
        }

        // Clear the bottom line
        let bottom_start = (self.max_rows - 1) * FONT_HEIGHT;
        let bg_rgb = self.bg_color.to_rgb();
        for y in bottom_start..self.info.height {
            for x in 0..self.info.width {
                self.draw_pixel(x, y, bg_rgb);
            }
        }
    }

    pub fn backspace(&mut self) {
        self.erase_cursor();
        if self.col > 0 {
            self.col -= 1;
            self.draw_char_at(self.col, self.row, ' ', self.fg_color, self.bg_color);
        }
        self.draw_cursor();
    }

    pub fn write_char(&mut self, c: char) {
        self.erase_cursor();
        match c {
            '\n' => {
                self.col = 0;
                if self.row + 1 >= self.max_rows {
                    self.scroll();
                } else {
                    self.row += 1;
                }
            }
            '\r' => {
                self.col = 0;
            }
            '\t' => {
                let next_tab = (self.col + 4) & !3;
                let spaces = next_tab - self.col;
                for _ in 0..spaces {
                    self.write_char(' ');
                }
            }
            '\x08' => {
                self.backspace();
                return;
            }
            printable => {
                if self.col >= self.max_cols {
                    self.col = 0;
                    if self.row + 1 >= self.max_rows {
                        self.scroll();
                    } else {
                        self.row += 1;
                    }
                }
                self.draw_char_at(self.col, self.row, printable, self.fg_color, self.bg_color);
                self.col += 1;
            }
        }
        self.draw_cursor();
    }

    pub fn write_str(&mut self, s: &str) {
        for c in s.chars() {
            self.write_char(c);
        }
    }
}

impl Write for ConsoleWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_str(s);
        Ok(())
    }
}

pub static WRITER: Mutex<Option<ConsoleWriter>> = Mutex::new(None);

pub fn init(framebuffer: &'static mut FrameBuffer) {
    let mut writer = WRITER.lock();
    *writer = Some(ConsoleWriter::new(framebuffer));
}

pub fn set_colors(fg: ColorCode, bg: ColorCode) {
    if let Some(ref mut writer) = *WRITER.lock() {
        writer.set_colors(fg, bg);
    }
}

pub fn clear_screen() {
    if let Some(ref mut writer) = *WRITER.lock() {
        writer.clear_screen();
    }
}

pub fn backspace() {
    if let Some(ref mut writer) = *WRITER.lock() {
        writer.backspace();
    }
    crate::serial::_print(format_args!("\x08 \x08"));
}

pub fn get_resolution() -> (usize, usize, usize, usize) {
    if let Some(ref writer) = *WRITER.lock() {
        (
            writer.info.width,
            writer.info.height,
            writer.max_cols,
            writer.max_rows,
        )
    } else {
        (1280, 720, 160, 45)
    }
}

pub fn put_char_at(col: usize, row: usize, c: char, fg: ColorCode, bg: ColorCode) {
    if let Some(ref mut writer) = *WRITER.lock() {
        writer.draw_char_at(col, row, c, fg, bg);
    }
}

pub fn draw_pixel(x: usize, y: usize, color: RgbColor) {
    if let Some(ref mut writer) = *WRITER.lock() {
        writer.draw_pixel(x, y, color);
    }
}

pub fn fill_rect(x: usize, y: usize, width: usize, height: usize, color: RgbColor) {
    if let Some(ref mut writer) = *WRITER.lock() {
        writer.fill_rect(x, y, width, height, color);
    }
}

pub fn draw_rect_outline(x: usize, y: usize, width: usize, height: usize, border: usize, color: RgbColor) {
    if let Some(ref mut writer) = *WRITER.lock() {
        let max_x = core::cmp::min(x + width, writer.info.width);
        let max_y = core::cmp::min(y + height, writer.info.height);
        if max_x <= x || max_y <= y {
            return;
        }
        // Top border
        let top_end = core::cmp::min(y + border, max_y);
        for cy in y..top_end {
            for cx in x..max_x {
                writer.draw_pixel(cx, cy, color);
            }
        }
        // Bottom border
        let btm_start = max_y.saturating_sub(border).max(y);
        for cy in btm_start..max_y {
            for cx in x..max_x {
                writer.draw_pixel(cx, cy, color);
            }
        }
        // Left and right vertical borders
        let left_end = core::cmp::min(x + border, max_x);
        let right_start = max_x.saturating_sub(border).max(x);
        for cy in y..max_y {
            for cx in x..left_end {
                writer.draw_pixel(cx, cy, color);
            }
            for cx in right_start..max_x {
                writer.draw_pixel(cx, cy, color);
            }
        }
    }
}

static CURSOR_BACK_BUFFER: Mutex<[RgbColor; 12 * 18]> = Mutex::new([RgbColor { r: 0, g: 0, b: 0 }; 12 * 18]);
static LAST_CURSOR_POS: Mutex<Option<(usize, usize)>> = Mutex::new(None);

pub fn reset_mouse_cursor() {
    *LAST_CURSOR_POS.lock() = None;
}

pub fn update_mouse_cursor(new_x: usize, new_y: usize) {
    if let Some(ref mut writer) = *WRITER.lock() {
        let max_w = writer.info.width;
        let max_h = writer.info.height;

        let mut last_pos = LAST_CURSOR_POS.lock();
        if let Some((old_x, old_y)) = *last_pos {
            // If cursor has not moved, do NOT touch video memory!
            if old_x == new_x && old_y == new_y {
                return;
            }

            // 1. Restore previous cursor background pixels without redrawing whole screen
            let back = CURSOR_BACK_BUFFER.lock();
            for cy in 0..18 {
                let py = old_y + cy;
                if py >= max_h { break; }
                for cx in 0..12 {
                    let px = old_x + cx;
                    if px >= max_w { break; }
                    let saved_color = back[cy * 12 + cx];
                    writer.draw_pixel(px, py, saved_color);
                }
            }
        }

        // 2. Save background pixels at new position
        {
            let mut back = CURSOR_BACK_BUFFER.lock();
            for cy in 0..18 {
                let py = new_y + cy;
                if py >= max_h { break; }
                for cx in 0..12 {
                    let px = new_x + cx;
                    if px >= max_w { break; }
                    back[cy * 12 + cx] = writer.read_pixel(px, py);
                }
            }
        }
        *last_pos = Some((new_x, new_y));

        // 3. Draw mouse cursor sprite on top
        for (cy, row) in crate::mouse::CURSOR_SPRITE.iter().enumerate() {
            let py = new_y + cy;
            if py >= max_h {
                break;
            }
            for (cx, &val) in row.iter().enumerate() {
                let px = new_x + cx;
                if px >= max_w {
                    break;
                }
                if val == 1 {
                    writer.draw_pixel(px, py, RgbColor { r: 0, g: 0, b: 0 });
                } else if val == 2 {
                    writer.draw_pixel(px, py, RgbColor { r: 255, g: 255, b: 255 });
                }
            }
        }
    }
}

pub fn draw_mouse_cursor(x: usize, y: usize) {
    update_mouse_cursor(x, y);
}

pub fn draw_string(px: usize, py: usize, text: &str, fg: RgbColor, bg: Option<RgbColor>) {
    if let Some(ref mut writer) = *WRITER.lock() {
        let mut curr_x = px;
        for c in text.chars() {
            let glyph = FONT_DATA[(c as usize) & 0xFF];
            for (dy, &row_bits) in glyph.iter().enumerate() {
                for dx in 0..FONT_WIDTH {
                    let is_fg = (row_bits & (0x80 >> dx)) != 0;
                    if is_fg {
                        writer.draw_pixel(curr_x + dx, py + dy, fg);
                    } else if let Some(bg_color) = bg {
                        writer.draw_pixel(curr_x + dx, py + dy, bg_color);
                    }
                }
            }
            curr_x += FONT_WIDTH;
        }
    }
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        if let Some(ref mut writer) = *WRITER.lock() {
            let _ = writer.write_fmt(args);
        }
        // Mirror to serial port
        crate::serial::_print(args);
    });
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($fmt:expr) => ($crate::print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::print!(concat!($fmt, "\n"), $($arg)*));
}
