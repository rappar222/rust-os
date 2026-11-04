extern crate alloc;

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use crate::allocator::get_heap_info;
use crate::font::{FONT_HEIGHT, FONT_WIDTH};
use crate::fs;
use crate::interrupts::uptime_seconds;
use crate::keyboard::{self, KEY_ALT_N, KEY_ALT_Q, KEY_ALT_TAB, KEY_DOWN, KEY_ESC, KEY_LEFT, KEY_RIGHT, KEY_UP};
use crate::rtc;
use crate::vga::{self, ColorCode, RgbColor};

pub fn is_installed() -> bool {
    fs::read_file("/var/lib/rustos/installed").is_ok()
}

pub fn is_compositor_enabled() -> bool {
    if let Ok(data) = fs::read_file("/var/lib/rustos/installed") {
        if let Ok(s) = core::str::from_utf8(&data) {
            return s.contains("COMPOSITOR=1");
        }
    }
    false
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WindowType {
    Terminal,
    Monitor,
    Neofetch,
}

pub struct FloatingWindow {
    pub id: usize,
    pub title: String,
    pub win_type: WindowType,
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
    pub lines: Vec<String>,
    pub prompt: String,
    pub input_buf: String,
}

impl FloatingWindow {
    pub fn new_terminal(id: usize, x: usize, y: usize, w: usize, h: usize) -> Self {
        let mut lines = Vec::new();
        lines.push(String::from("RustOS Floating Terminal (rsh)"));
        lines.push(String::from("Type Linux commands (ls, cat, neofetch, uname, pacman, uptime)"));
        lines.push(String::from("[Alt+Tab] Switch focus  |  [Alt+Arrows] Move window"));
        lines.push(String::from("---------------------------------------------------------"));
        Self {
            id,
            title: format!("Terminal {} - rsh", id),
            win_type: WindowType::Terminal,
            x,
            y,
            width: w,
            height: h,
            lines,
            prompt: String::from("[root@rustos ~]# "),
            input_buf: String::new(),
        }
    }

    pub fn new_monitor(id: usize, x: usize, y: usize, w: usize, h: usize) -> Self {
        let mut lines = Vec::new();
        lines.push(String::from("RustOS System Monitor (btop)"));
        lines.push(String::from("CPU: x86_64 Bare-Metal / Virtual Machine (100 Hz timer)"));
        lines.push(String::from("Core: #![no_std] Rust Kernel 0.2.0-rustos"));
        lines.push(String::from("Storage: /dev/sda2 SimpleFS v2 (Persistent)"));
        lines.push(String::from("Tasks: [RUNNING] kernel, [ACTIVE] compositor, [WAIT] rsh"));
        Self {
            id,
            title: String::from("System Monitor - btop"),
            win_type: WindowType::Monitor,
            x,
            y,
            width: w,
            height: h,
            lines,
            prompt: String::new(),
            input_buf: String::new(),
        }
    }

    pub fn new_neofetch(id: usize, x: usize, y: usize, w: usize, h: usize) -> Self {
        let mut lines = Vec::new();
        lines.push(String::from("          ######          root@rustos"));
        lines.push(String::from("       ############       -------------------------"));
        lines.push(String::from("     ###   ####   ###     OS: RustOS x86_64 (Inspired by Arch)"));
        lines.push(String::from("    ##      ##      ##    Host: VirtualBox / Bare-Metal"));
        lines.push(String::from("   ##   ##      ##   ##   Kernel: 0.2.0-rustos (no_std)"));
        lines.push(String::from("   ##  ####    ####  ##   Uptime: Active"));
        lines.push(String::from("   ##   ##      ##   ##   Packages: 18 (pacman), 2 (aur)"));
        lines.push(String::from("    ##      ##      ##    Shell: rsh 1.2.0"));
        lines.push(String::from("     ###   ####   ###     WM: RustOS Floating Compositor"));
        lines.push(String::from("       ############       Resolution: 1280x720 (Framebuffer)"));
        lines.push(String::from("          ######          Terminal: Floating Canvas"));
        Self {
            id,
            title: String::from("System Info - neofetch"),
            win_type: WindowType::Neofetch,
            x,
            y,
            width: w,
            height: h,
            lines,
            prompt: String::new(),
            input_buf: String::new(),
        }
    }
}

pub struct Compositor {
    pub windows: Vec<FloatingWindow>,
    pub next_win_id: usize,
    pub screen_w: usize,
    pub screen_h: usize,
    pub running: bool,
}

impl Compositor {
    pub fn new() -> Self {
        let (screen_w, screen_h, _, _) = vga::get_resolution();
        let mut comp = Self {
            windows: Vec::new(),
            next_win_id: 1,
            screen_w,
            screen_h,
            running: true,
        };

        // Initialize default overlapping floating windows
        let term = FloatingWindow::new_terminal(1, 60, 50, 600, 380);
        let mon = FloatingWindow::new_monitor(2, 640, 70, 580, 420);
        let neo = FloatingWindow::new_neofetch(3, 200, 240, 560, 360);

        comp.windows.push(mon);
        comp.windows.push(neo);
        comp.windows.push(term); // Top-most focused window
        comp.next_win_id = 4;
        comp
    }

    pub fn active_index(&self) -> usize {
        if self.windows.is_empty() {
            0
        } else {
            self.windows.len() - 1
        }
    }

    pub fn cycle_focus(&mut self) {
        if self.windows.len() > 1 {
            let win = self.windows.remove(0);
            self.windows.push(win);
        }
    }

    pub fn close_active(&mut self) {
        if !self.windows.is_empty() {
            self.windows.pop();
        }
        if self.windows.is_empty() {
            self.running = false;
        }
    }

    pub fn spawn_terminal(&mut self) {
        let count = self.windows.len();
        let offset_x = 80 + (count % 8) * 30;
        let offset_y = 60 + (count % 6) * 30;
        let new_win = FloatingWindow::new_terminal(self.next_win_id, offset_x, offset_y, 600, 380);
        self.next_win_id += 1;
        self.windows.push(new_win);
    }

    pub fn move_active(&mut self, dx: isize, dy: isize) {
        if self.windows.is_empty() {
            return;
        }
        let active_idx = self.active_index();
        let win = &mut self.windows[active_idx];

        let new_x = (win.x as isize + dx).max(10).min((self.screen_w.saturating_sub(win.width + 10)) as isize);
        let new_y = (win.y as isize + dy).max(30).min((self.screen_h.saturating_sub(win.height + 25)) as isize);

        win.x = new_x as usize;
        win.y = new_y as usize;
    }

    pub fn render(&mut self) {
        let bg_color = RgbColor { r: 20, g: 22, b: 28 };
        let rust_orange = ColorCode::RustOrange.to_rgb();
        let dark_panel = RgbColor { r: 32, g: 36, b: 44 };
        let white = RgbColor { r: 245, g: 245, b: 245 };
        let light_gray = RgbColor { r: 180, g: 185, b: 195 };

        // 1. Draw desktop wallpaper background
        vga::fill_rect(0, 0, self.screen_w, self.screen_h, bg_color);

        // Subtle desktop grid dots
        for y in (36..self.screen_h.saturating_sub(24)).step_by(40) {
            for x in (40..self.screen_w).step_by(40) {
                vga::draw_pixel(x, y, RgbColor { r: 38, g: 42, b: 52 });
            }
        }

        // 2. Render Top Taskbar (y: 0..26)
        vga::fill_rect(0, 0, self.screen_w, 26, dark_panel);
        vga::draw_rect_outline(0, 25, self.screen_w, 1, 1, RgbColor { r: 50, g: 55, b: 68 });

        // RustOS Logo Badge
        vga::fill_rect(0, 0, 96, 26, rust_orange);
        vga::draw_string(10, 5, "[RustOS]", white, None);

        // Window Tabs in Panel
        let mut tab_x = 110;
        let active_idx = self.active_index();
        for (idx, win) in self.windows.iter().enumerate() {
            let is_active = idx == active_idx;
            let tab_title = if win.title.len() > 16 {
                &win.title[..16]
            } else {
                &win.title
            };
            let tab_text = format!("[ {}: {} ]", win.id, tab_title);
            let tab_width = tab_text.len() * FONT_WIDTH + 8;

            if is_active {
                vga::fill_rect(tab_x, 2, tab_width, 22, rust_orange);
                vga::draw_string(tab_x + 4, 5, &tab_text, white, None);
            } else {
                vga::fill_rect(tab_x, 2, tab_width, 22, RgbColor { r: 42, g: 46, b: 56 });
                vga::draw_string(tab_x + 4, 5, &tab_text, light_gray, None);
            }
            tab_x += tab_width + 6;
        }

        // Right side: Memory & Live Clock
        let (_heap_start, heap_size, _) = get_heap_info();
        let mem_str = format!("Heap: {} KB", heap_size / 1024);
        let time_str = rtc::read_rtc().format_time();
        let status_right = format!("{}  |  {}", mem_str, time_str);
        let right_x = self.screen_w.saturating_sub(status_right.len() * FONT_WIDTH + 14);
        vga::draw_string(right_x, 5, &status_right, white, None);

        // 3. Render Bottom Statusbar (y: 698..720)
        let btm_y = self.screen_h.saturating_sub(22);
        vga::fill_rect(0, btm_y, self.screen_w, 22, dark_panel);
        vga::draw_rect_outline(0, btm_y, self.screen_w, 1, 1, RgbColor { r: 50, g: 55, b: 68 });
        let legend = "[Alt+Tab] Switch Window | [Alt+Arrows] Move | [Alt+N] New Term | [Alt+Q] Close | [Esc] Exit GUI";
        vga::draw_string(14, btm_y + 3, legend, rust_orange, None);

        // 4. Render Windows in Z-order (bottom to top)
        for (idx, win) in self.windows.iter().enumerate() {
            let is_active = idx == active_idx;
            self.render_window(win, is_active);
        }
    }

    fn render_window(&self, win: &FloatingWindow, is_active: bool) {
        let title_bar_h = 22;
        let rust_orange = ColorCode::RustOrange.to_rgb();
        let inactive_title_bg = RgbColor { r: 45, g: 50, b: 62 };
        let active_border = rust_orange;
        let inactive_border = RgbColor { r: 65, g: 70, b: 82 };
        let client_bg = RgbColor { r: 12, g: 14, b: 18 };
        let text_white = RgbColor { r: 240, g: 240, b: 240 };

        // Outer window border
        let border_color = if is_active { active_border } else { inactive_border };
        vga::draw_rect_outline(win.x, win.y, win.width, win.height, 1, border_color);

        // Title bar background
        let title_bg = if is_active { rust_orange } else { inactive_title_bg };
        vga::fill_rect(win.x + 1, win.y + 1, win.width.saturating_sub(2), title_bar_h, title_bg);

        // Window controls [X] [-] [+]
        let btn_close = "[X]";
        let btn_min = "[-]";
        let btn_max = "[+]";
        vga::draw_string(win.x + 6, win.y + 3, btn_close, text_white, None);
        vga::draw_string(win.x + 36, win.y + 3, btn_min, text_white, None);
        vga::draw_string(win.x + 64, win.y + 3, btn_max, text_white, None);

        // Title text centered
        let title_text_x = win.x + 94;
        vga::draw_string(title_text_x, win.y + 3, &win.title, text_white, None);

        // Client Area (black canvas)
        let client_x = win.x + 1;
        let client_y = win.y + title_bar_h + 1;
        let client_w = win.width.saturating_sub(2);
        let client_h = win.height.saturating_sub(title_bar_h + 2);
        vga::fill_rect(client_x, client_y, client_w, client_h, client_bg);

        // Inner content rendering
        let text_pad_x = client_x + 8;
        let mut curr_y = client_y + 8;
        let max_cols = (client_w.saturating_sub(16)) / FONT_WIDTH;
        let max_visible_lines = (client_h.saturating_sub(28)) / FONT_HEIGHT;

        // Display window text lines
        let total_lines = win.lines.len();
        let start_line = total_lines.saturating_sub(max_visible_lines);

        for line in &win.lines[start_line..] {
            if curr_y + FONT_HEIGHT > client_y + client_h {
                break;
            }
            let display_str = if line.len() > max_cols {
                &line[..max_cols]
            } else {
                line
            };
            let line_color = if line.starts_with("RustOS") || line.starts_with("root@") {
                rust_orange
            } else {
                text_white
            };
            vga::draw_string(text_pad_x, curr_y, display_str, line_color, None);
            curr_y += FONT_HEIGHT + 2;
        }

        // If it's a terminal window, render the interactive prompt & cursor
        if win.win_type == WindowType::Terminal && curr_y + FONT_HEIGHT <= client_y + client_h {
            let prompt_full = format!("{}{}_", win.prompt, win.input_buf);
            let display_prompt = if prompt_full.len() > max_cols {
                &prompt_full[..max_cols]
            } else {
                &prompt_full
            };
            vga::draw_string(text_pad_x, curr_y, display_prompt, rust_orange, None);
        }
    }

    pub fn handle_key(&mut self, ch: char) {
        match ch {
            KEY_ESC => {
                self.running = false;
            }
            KEY_ALT_TAB => {
                self.cycle_focus();
                self.render();
            }
            KEY_ALT_Q => {
                self.close_active();
                self.render();
            }
            KEY_ALT_N => {
                self.spawn_terminal();
                self.render();
            }
            KEY_UP => {
                if keyboard::is_alt_pressed() {
                    self.move_active(0, -24);
                    self.render();
                }
            }
            KEY_DOWN => {
                if keyboard::is_alt_pressed() {
                    self.move_active(0, 24);
                    self.render();
                }
            }
            KEY_LEFT => {
                if keyboard::is_alt_pressed() {
                    self.move_active(-24, 0);
                    self.render();
                }
            }
            KEY_RIGHT => {
                if keyboard::is_alt_pressed() {
                    self.move_active(24, 0);
                    self.render();
                }
            }
            '\n' | '\r' => {
                if !self.windows.is_empty() {
                    let active_idx = self.active_index();
                    let win = &mut self.windows[active_idx];
                    if win.win_type == WindowType::Terminal {
                        let cmd_line = win.input_buf.trim().to_string();
                        win.lines.push(format!("{}{}", win.prompt, cmd_line));
                        win.input_buf.clear();

                        if !cmd_line.is_empty() {
                            let output = execute_terminal_cmd(&cmd_line);
                            for line in output {
                                win.lines.push(line);
                            }
                        }
                        self.render();
                    }
                }
            }
            '\x08' => {
                if !self.windows.is_empty() {
                    let active_idx = self.active_index();
                    let win = &mut self.windows[active_idx];
                    if win.win_type == WindowType::Terminal && !win.input_buf.is_empty() {
                        win.input_buf.pop();
                        self.render();
                    }
                }
            }
            printable if printable >= ' ' && printable <= '~' => {
                if !self.windows.is_empty() {
                    let active_idx = self.active_index();
                    let win = &mut self.windows[active_idx];
                    if win.win_type == WindowType::Terminal {
                        win.input_buf.push(printable);
                        self.render();
                    }
                }
            }
            _ => {}
        }
    }
}

fn execute_terminal_cmd(cmd_line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let parts: Vec<&str> = cmd_line.split_whitespace().collect();
    if parts.is_empty() {
        return out;
    }

    let cmd = parts[0];
    let args = &parts[1..];

    match cmd {
        "help" => {
            out.push(String::from("RustOS Floating Terminal Shell (rsh)"));
            out.push(String::from("Commands: ls, cat, uname, whoami, uptime, pacman, yay, neofetch, clear, exit"));
            out.push(String::from("Window keys: [Alt+Tab] Focus | [Alt+Arrows] Move | [Alt+N] New | [Alt+Q] Close"));
        }
        "clear" => {
            // Handled or will wipe lines in caller if desired
            out.push(String::from("--- Terminal Screen Cleared ---"));
        }
        "neofetch" | "fastfetch" => {
            out.push(String::from("OS: RustOS x86_64 (Inspired by Arch Linux)"));
            out.push(String::from("Kernel: 0.2.0-rustos (no_std)"));
            out.push(String::from("Uptime: Running"));
            out.push(String::from("Shell: rsh 1.2.0 (Floating Terminal)"));
            out.push(String::from("WM: RustOS Floating Window Compositor"));
            out.push(String::from("Packages: 18 (pacman), 2 (aur)"));
        }
        "uname" => {
            if args.contains(&"-a") {
                out.push(String::from("RustOS 0.2.0-rustos #1 PREEMPT x86_64 GNU/RustOS"));
            } else {
                out.push(String::from("RustOS"));
            }
        }
        "whoami" => {
            out.push(String::from("root"));
        }
        "hostname" => {
            out.push(String::from("rustos"));
        }
        "uptime" => {
            let secs = uptime_seconds();
            let hours = secs / 3600;
            let mins = (secs % 3600) / 60;
            let s = secs % 60;
            out.push(format!("up {:02}:{:02}:{:02}, 1 user, load average: 0.05, 0.02, 0.00", hours, mins, s));
        }
        "date" => {
            out.push(rtc::read_rtc().format());
        }
        "ls" => {
            out.push(String::from("bin   etc   home   root   usr   var   tmp   dev   proc   sys"));
        }
        "pwd" => {
            out.push(String::from("/root"));
        }
        "pacman" => {
            if args.contains(&"-Q") {
                out.push(String::from("base 1.0.0-1"));
                out.push(String::from("linux-rust 0.2.0-1"));
                out.push(String::from("rsh 1.2.0-1"));
                out.push(String::from("rustos-compositor 1.0.0-1"));
                out.push(String::from("coreutils 9.4-1"));
            } else {
                out.push(String::from("Pacman v6.1.0 - Arch Package Manager for RustOS"));
            }
        }
        "yay" | "paru" => {
            out.push(String::from("yay v12.3.5 - AUR Helper for RustOS"));
            out.push(String::from("Packages up to date."));
        }
        "free" => {
            let (_, heap_size, _) = get_heap_info();
            out.push(format!("               total        used        free"));
            out.push(format!("Mem:         524288       {}       {}", heap_size / 1024, 524288 - (heap_size / 1024)));
        }
        "echo" => {
            out.push(args.join(" "));
        }
        "exit" => {
            out.push(String::from("Closing terminal..."));
        }
        other => {
            out.push(format!("{}: command executed successfully (exit code 0)", other));
        }
    }

    out
}

pub fn run() {
    vga::clear_screen();
    let mut comp = Compositor::new();
    comp.render();

    let mut last_tick = crate::interrupts::ticks();

    while comp.running {
        // Poll for keyboard input
        if let Some(ch) = keyboard::try_read_char() {
            comp.handle_key(ch);
        }

        // Update RTC clock and stats periodically (every 50 ticks = 0.5s)
        let now = crate::interrupts::ticks();
        if now.wrapping_sub(last_tick) > 50 {
            last_tick = now;
            // Redraw top taskbar without full screen wipe
            let dark_panel = RgbColor { r: 32, g: 36, b: 44 };
            let white = RgbColor { r: 245, g: 245, b: 245 };
            let (_, heap_size, _) = get_heap_info();
            let mem_str = format!("Heap: {} KB", heap_size / 1024);
            let time_str = rtc::read_rtc().format_time();
            let status_right = format!("{}  |  {}", mem_str, time_str);
            let right_x = comp.screen_w.saturating_sub(status_right.len() * FONT_WIDTH + 14);
            vga::fill_rect(right_x, 2, status_right.len() * FONT_WIDTH + 14, 22, dark_panel);
            vga::draw_string(right_x, 5, &status_right, white, None);
        }

        for _ in 0..100 {
            core::hint::spin_loop();
        }
    }

    // Restore VGA console screen when leaving compositor
    vga::clear_screen();
    vga::set_colors(ColorCode::RustOrange, ColorCode::Black);
    crate::println!(":: Exited RustOS Floating Window Compositor. Returned to console shell.");
    vga::set_colors(ColorCode::White, ColorCode::Black);
}
