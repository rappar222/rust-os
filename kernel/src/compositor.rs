extern crate alloc;

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use crate::allocator::get_heap_info;
use crate::font::{FONT_HEIGHT, FONT_WIDTH};
use crate::fs;
use crate::interrupts::uptime_seconds;
use crate::keyboard::{self, KEY_ALT_N, KEY_ALT_Q, KEY_ALT_TAB, KEY_DOWN, KEY_ESC, KEY_LEFT, KEY_RIGHT, KEY_UP};
use crate::mouse;
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
    FileManager,
    Paint,
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
    pub minimized: bool,
    pub maximized: bool,
    pub saved_geom: (usize, usize, usize, usize),
    // Terminal fields
    pub lines: Vec<String>,
    pub prompt: String,
    pub input_buf: String,
    // File Manager fields
    pub fm_path: String,
    pub fm_entries: Vec<String>,
    pub fm_preview: String,
    // Paint Canvas fields
    pub paint_color: RgbColor,
    pub paint_pixels: Vec<(u16, u16, RgbColor)>,
}

impl FloatingWindow {
    pub fn new_terminal(id: usize, x: usize, y: usize, w: usize, h: usize) -> Self {
        let mut lines = Vec::new();
        lines.push(String::from("RustOS Graphical Terminal (rsh)"));
        lines.push(String::from("Type Linux commands (ls, cat, neofetch, uname, pacman, uptime)"));
        lines.push(String::from("Mouse: Drag title bar to move  |  Drag bottom-right corner to resize"));
        lines.push(String::from("Window Buttons: [X] Close   [-] Minimize   [+] Maximize"));
        lines.push(String::from("------------------------------------------------------------------"));
        Self {
            id,
            title: format!("Terminal {} - rsh", id),
            win_type: WindowType::Terminal,
            x,
            y,
            width: w,
            height: h,
            minimized: false,
            maximized: false,
            saved_geom: (x, y, w, h),
            lines,
            prompt: String::from("[root@rustos ~]# "),
            input_buf: String::new(),
            fm_path: String::new(),
            fm_entries: Vec::new(),
            fm_preview: String::new(),
            paint_color: ColorCode::RustOrange.to_rgb(),
            paint_pixels: Vec::new(),
        }
    }

    pub fn new_monitor(id: usize, x: usize, y: usize, w: usize, h: usize) -> Self {
        Self {
            id,
            title: String::from("System Resource Monitor (btop)"),
            win_type: WindowType::Monitor,
            x,
            y,
            width: w,
            height: h,
            minimized: false,
            maximized: false,
            saved_geom: (x, y, w, h),
            lines: Vec::new(),
            prompt: String::new(),
            input_buf: String::new(),
            fm_path: String::new(),
            fm_entries: Vec::new(),
            fm_preview: String::new(),
            paint_color: ColorCode::RustOrange.to_rgb(),
            paint_pixels: Vec::new(),
        }
    }

    pub fn new_file_manager(id: usize, x: usize, y: usize, w: usize, h: usize) -> Self {
        let mut entries = Vec::new();
        entries.push(String::from("[DIR]  /bin"));
        entries.push(String::from("[DIR]  /etc"));
        entries.push(String::from("[DIR]  /home"));
        entries.push(String::from("[DIR]  /root"));
        entries.push(String::from("[DIR]  /usr"));
        entries.push(String::from("[DIR]  /var"));
        entries.push(String::from("[DIR]  /tmp"));
        entries.push(String::from("[FILE] /etc/os-release"));
        entries.push(String::from("[FILE] /etc/hostname"));
        entries.push(String::from("[FILE] /etc/fstab"));
        entries.push(String::from("[FILE] /var/lib/rustos/installed"));

        Self {
            id,
            title: String::from("File Browser - SimpleFS"),
            win_type: WindowType::FileManager,
            x,
            y,
            width: w,
            height: h,
            minimized: false,
            maximized: false,
            saved_geom: (x, y, w, h),
            lines: Vec::new(),
            prompt: String::new(),
            input_buf: String::new(),
            fm_path: String::from("/"),
            fm_entries: entries,
            fm_preview: String::from("Click an item below to browse files and view metadata."),
            paint_color: ColorCode::RustOrange.to_rgb(),
            paint_pixels: Vec::new(),
        }
    }

    pub fn new_paint(id: usize, x: usize, y: usize, w: usize, h: usize) -> Self {
        Self {
            id,
            title: String::from("Paint Canvas - Freehand Drawing"),
            win_type: WindowType::Paint,
            x,
            y,
            width: w,
            height: h,
            minimized: false,
            maximized: false,
            saved_geom: (x, y, w, h),
            lines: Vec::new(),
            prompt: String::new(),
            input_buf: String::new(),
            fm_path: String::new(),
            fm_entries: Vec::new(),
            fm_preview: String::new(),
            paint_color: ColorCode::RustOrange.to_rgb(),
            paint_pixels: Vec::new(),
        }
    }

    pub fn new_neofetch(id: usize, x: usize, y: usize, w: usize, h: usize) -> Self {
        let mut lines = Vec::new();
        lines.push(String::from("          ######          root@rustos"));
        lines.push(String::from("       ############       -------------------------"));
        lines.push(String::from("     ###   ####   ###     OS: RustOS x86_64"));
        lines.push(String::from("    ##      ##      ##    WM: RustOS Floating Window Manager"));
        lines.push(String::from("   ##   ##      ##   ##   Kernel: 0.2.0-rustos (no_std)"));
        lines.push(String::from("   ##  ####    ####  ##   Host: VirtualBox / Bare-Metal x86_64"));
        lines.push(String::from("   ##   ##      ##   ##   Drivers: PS/2 Mouse + Keyboard, ATA"));
        lines.push(String::from("    ##      ##      ##    Packages: 18 (pacman), 2 (aur)"));
        lines.push(String::from("     ###   ####   ###     Shell: rsh 1.2.0"));
        lines.push(String::from("       ############       Resolution: 1280x720 (Framebuffer)"));
        lines.push(String::from("          ######          Mouse: Hardware Pointer Active"));
        Self {
            id,
            title: String::from("System Info - neofetch"),
            win_type: WindowType::Neofetch,
            x,
            y,
            width: w,
            height: h,
            minimized: false,
            maximized: false,
            saved_geom: (x, y, w, h),
            lines,
            prompt: String::new(),
            input_buf: String::new(),
            fm_path: String::new(),
            fm_entries: Vec::new(),
            fm_preview: String::new(),
            paint_color: ColorCode::RustOrange.to_rgb(),
            paint_pixels: Vec::new(),
        }
    }
}

pub struct Compositor {
    pub windows: Vec<FloatingWindow>,
    pub next_win_id: usize,
    pub screen_w: usize,
    pub screen_h: usize,
    pub running: bool,
    pub start_menu_open: bool,
    pub drag_win_idx: Option<usize>,
    pub drag_offset_x: usize,
    pub drag_offset_y: usize,
    pub resize_win_idx: Option<usize>,
    pub mouse_x: usize,
    pub mouse_y: usize,
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
            start_menu_open: false,
            drag_win_idx: None,
            drag_offset_x: 0,
            drag_offset_y: 0,
            resize_win_idx: None,
            mouse_x: screen_w / 2,
            mouse_y: screen_h / 2,
        };

        // Create default desktop windows
        let mon = FloatingWindow::new_monitor(1, 680, 50, 560, 400);
        let fm = FloatingWindow::new_file_manager(2, 220, 260, 580, 380);
        let term = FloatingWindow::new_terminal(3, 50, 45, 610, 390);

        comp.windows.push(mon);
        comp.windows.push(fm);
        comp.windows.push(term); // Top-most active window
        comp.next_win_id = 4;
        comp
    }

    pub fn active_index(&self) -> Option<usize> {
        self.windows
            .iter()
            .enumerate()
            .rev()
            .find(|(_, w)| !w.minimized)
            .map(|(idx, _)| idx)
    }

    pub fn focus_window(&mut self, idx: usize) {
        if idx < self.windows.len() {
            let mut win = self.windows.remove(idx);
            win.minimized = false;
            self.windows.push(win);
        }
    }

    pub fn cycle_focus(&mut self) {
        if self.windows.len() > 1 {
            let win = self.windows.remove(0);
            self.windows.push(win);
        }
    }

    pub fn close_window(&mut self, idx: usize) {
        if idx < self.windows.len() {
            self.windows.remove(idx);
        }
    }

    pub fn toggle_maximize(&mut self, idx: usize) {
        if idx < self.windows.len() {
            let win = &mut self.windows[idx];
            if win.maximized {
                // Restore
                win.x = win.saved_geom.0;
                win.y = win.saved_geom.1;
                win.width = win.saved_geom.2;
                win.height = win.saved_geom.3;
                win.maximized = false;
            } else {
                // Maximize
                win.saved_geom = (win.x, win.y, win.width, win.height);
                win.x = 10;
                win.y = 28;
                win.width = self.screen_w.saturating_sub(20);
                win.height = self.screen_h.saturating_sub(54);
                win.maximized = true;
            }
        }
    }

    pub fn spawn_terminal(&mut self) {
        let count = self.windows.len();
        let offset_x = 60 + (count % 8) * 35;
        let offset_y = 50 + (count % 6) * 35;
        let win = FloatingWindow::new_terminal(self.next_win_id, offset_x, offset_y, 610, 390);
        self.next_win_id += 1;
        self.windows.push(win);
    }

    pub fn spawn_monitor(&mut self) {
        let win = FloatingWindow::new_monitor(self.next_win_id, 620, 60, 580, 420);
        self.next_win_id += 1;
        self.windows.push(win);
    }

    pub fn spawn_file_manager(&mut self) {
        let win = FloatingWindow::new_file_manager(self.next_win_id, 160, 100, 600, 420);
        self.next_win_id += 1;
        self.windows.push(win);
    }

    pub fn spawn_paint(&mut self) {
        let win = FloatingWindow::new_paint(self.next_win_id, 240, 120, 640, 440);
        self.next_win_id += 1;
        self.windows.push(win);
    }

    pub fn spawn_neofetch(&mut self) {
        let win = FloatingWindow::new_neofetch(self.next_win_id, 200, 140, 560, 360);
        self.next_win_id += 1;
        self.windows.push(win);
    }

    pub fn render(&mut self) {
        let bg_color = RgbColor { r: 18, g: 20, b: 26 };
        let rust_orange = ColorCode::RustOrange.to_rgb();
        let dark_panel = RgbColor { r: 28, g: 32, b: 40 };
        let white = RgbColor { r: 245, g: 245, b: 245 };
        let light_gray = RgbColor { r: 175, g: 180, b: 190 };

        // 1. Desktop Wallpaper Background
        vga::fill_rect(0, 0, self.screen_w, self.screen_h, bg_color);

        // Elegant desktop grid dots
        for y in (36..self.screen_h.saturating_sub(24)).step_by(40) {
            for x in (40..self.screen_w).step_by(40) {
                vga::draw_pixel(x, y, RgbColor { r: 35, g: 39, b: 48 });
            }
        }

        // 2. Top Taskbar (y: 0..26)
        vga::fill_rect(0, 0, self.screen_w, 26, dark_panel);
        vga::draw_rect_outline(0, 25, self.screen_w, 1, 1, RgbColor { r: 48, g: 54, b: 66 });

        // Start Button [RustOS] (x: 0..96)
        let start_bg = if self.start_menu_open {
            RgbColor { r: 245, g: 105, b: 45 }
        } else {
            rust_orange
        };
        vga::fill_rect(0, 0, 96, 26, start_bg);
        vga::draw_string(10, 5, "[RustOS]", white, None);

        // Window Tabs in Top Taskbar
        let active_opt = self.active_index();
        let mut tab_x = 106;
        for (idx, win) in self.windows.iter().enumerate() {
            let is_active = Some(idx) == active_opt && !win.minimized;
            let tab_title = if win.title.len() > 14 {
                &win.title[..14]
            } else {
                &win.title
            };
            let tab_text = if win.minimized {
                format!("(-)[{}]", tab_title)
            } else {
                format!("[{}]", tab_title)
            };
            let tab_width = tab_text.len() * FONT_WIDTH + 8;

            if is_active {
                vga::fill_rect(tab_x, 2, tab_width, 22, rust_orange);
                vga::draw_string(tab_x + 4, 5, &tab_text, white, None);
            } else {
                vga::fill_rect(tab_x, 2, tab_width, 22, RgbColor { r: 38, g: 42, b: 52 });
                vga::draw_string(tab_x + 4, 5, &tab_text, light_gray, None);
            }
            tab_x += tab_width + 4;
        }

        // Tray Area: Memory & Live UTC Clock
        let (_heap_start, heap_size, _) = get_heap_info();
        let mem_str = format!("RAM: {} KB", heap_size / 1024);
        let time_str = rtc::read_rtc().format_time();
        let status_right = format!("{}  |  {}", mem_str, time_str);
        let right_x = self.screen_w.saturating_sub(status_right.len() * FONT_WIDTH + 14);
        vga::draw_string(right_x, 5, &status_right, white, None);

        // 3. Render Windows in Z-Order (Bottom to Top)
        for (idx, win) in self.windows.iter().enumerate() {
            if win.minimized {
                continue;
            }
            let is_active = Some(idx) == active_opt;
            self.render_window(win, is_active);
        }

        // 4. Start Menu Pop-up (if open)
        if self.start_menu_open {
            self.render_start_menu();
        }

        // 5. Bottom Statusbar (y: 698..720)
        let btm_y = self.screen_h.saturating_sub(22);
        vga::fill_rect(0, btm_y, self.screen_w, 22, dark_panel);
        vga::draw_rect_outline(0, btm_y, self.screen_w, 1, 1, RgbColor { r: 48, g: 54, b: 66 });
        let legend = "Mouse: Click to focus/drag/resize | [RustOS] Menu | [Alt+Tab] Switch | [Alt+N] Terminal | [Esc] Exit";
        vga::draw_string(14, btm_y + 3, legend, rust_orange, None);

        // 6. Draw Hardware Mouse Cursor Sprite on Top of Everything
        vga::draw_mouse_cursor(self.mouse_x, self.mouse_y);
    }

    fn render_start_menu(&self) {
        let menu_x = 0;
        let menu_y = 26;
        let menu_w = 230;
        let menu_h = 190;
        let menu_bg = RgbColor { r: 25, g: 28, b: 36 };
        let border_color = ColorCode::RustOrange.to_rgb();
        let text_white = RgbColor { r: 245, g: 245, b: 245 };
        let text_gold = RgbColor { r: 245, g: 165, b: 65 };

        vga::fill_rect(menu_x, menu_y, menu_w, menu_h, menu_bg);
        vga::draw_rect_outline(menu_x, menu_y, menu_w, menu_h, 2, border_color);

        vga::draw_string(menu_x + 12, menu_y + 10, "RustOS Applications", text_gold, None);
        vga::fill_rect(menu_x + 10, menu_y + 26, menu_w - 20, 1, border_color);

        let items = [
            "1. Terminal (rsh)",
            "2. System Monitor",
            "3. File Browser",
            "4. Paint Canvas",
            "5. Neofetch Info",
            "6. Exit GUI to Shell",
        ];

        for (i, item) in items.iter().enumerate() {
            let item_y = menu_y + 35 + i * 24;
            vga::draw_string(menu_x + 14, item_y, item, text_white, None);
        }
    }

    fn render_window(&self, win: &FloatingWindow, is_active: bool) {
        let title_bar_h = 22;
        let rust_orange = ColorCode::RustOrange.to_rgb();
        let inactive_title_bg = RgbColor { r: 42, g: 46, b: 56 };
        let active_border = rust_orange;
        let inactive_border = RgbColor { r: 60, g: 65, b: 76 };
        let client_bg = RgbColor { r: 10, g: 12, b: 16 };
        let text_white = RgbColor { r: 245, g: 245, b: 245 };

        // Window Border
        let border_color = if is_active { active_border } else { inactive_border };
        vga::draw_rect_outline(win.x, win.y, win.width, win.height, 1, border_color);

        // Title Bar
        let title_bg = if is_active { rust_orange } else { inactive_title_bg };
        vga::fill_rect(win.x + 1, win.y + 1, win.width.saturating_sub(2), title_bar_h, title_bg);

        // Window Control Buttons [X] [-] [+]
        vga::draw_string(win.x + 6, win.y + 3, "[X]", text_white, None);
        vga::draw_string(win.x + 36, win.y + 3, "[-]", text_white, None);
        vga::draw_string(win.x + 64, win.y + 3, "[+]", text_white, None);

        // Window Title
        let title_text_x = win.x + 94;
        let title_max_chars = (win.width.saturating_sub(100)) / FONT_WIDTH;
        let display_title = if win.title.len() > title_max_chars {
            &win.title[..title_max_chars]
        } else {
            &win.title
        };
        vga::draw_string(title_text_x, win.y + 3, display_title, text_white, None);

        // Window Client Area
        let client_x = win.x + 1;
        let client_y = win.y + title_bar_h + 1;
        let client_w = win.width.saturating_sub(2);
        let client_h = win.height.saturating_sub(title_bar_h + 2);
        vga::fill_rect(client_x, client_y, client_w, client_h, client_bg);

        // Render Specific Application Content
        match win.win_type {
            WindowType::Terminal => self.render_terminal_content(win, client_x, client_y, client_w, client_h),
            WindowType::Monitor => self.render_monitor_content(win, client_x, client_y, client_w, client_h),
            WindowType::FileManager => self.render_file_manager_content(win, client_x, client_y, client_w, client_h),
            WindowType::Paint => self.render_paint_content(win, client_x, client_y, client_w, client_h),
            WindowType::Neofetch => self.render_neofetch_content(win, client_x, client_y, client_w, client_h),
        }

        // Bottom-Right Corner Resize Grip (///)
        let grip_x = win.x + win.width - 14;
        let grip_y = win.y + win.height - 14;
        vga::draw_string(grip_x, grip_y, "//", if is_active { rust_orange } else { inactive_border }, None);
    }

    fn render_terminal_content(&self, win: &FloatingWindow, cx: usize, cy: usize, cw: usize, ch: usize) {
        let text_pad_x = cx + 8;
        let mut curr_y = cy + 8;
        let max_cols = (cw.saturating_sub(16)) / FONT_WIDTH;
        let max_visible = (ch.saturating_sub(24)) / FONT_HEIGHT;
        let text_white = RgbColor { r: 240, g: 240, b: 240 };
        let rust_orange = ColorCode::RustOrange.to_rgb();

        let total_lines = win.lines.len();
        let start_line = total_lines.saturating_sub(max_visible);

        for line in &win.lines[start_line..] {
            if curr_y + FONT_HEIGHT > cy + ch {
                break;
            }
            let display_str = if line.len() > max_cols {
                &line[..max_cols]
            } else {
                line
            };
            let color = if line.starts_with("RustOS") || line.starts_with("root@") {
                rust_orange
            } else {
                text_white
            };
            vga::draw_string(text_pad_x, curr_y, display_str, color, None);
            curr_y += FONT_HEIGHT + 2;
        }

        // Terminal Prompt & Cursor
        if curr_y + FONT_HEIGHT <= cy + ch {
            let prompt_full = format!("{}{}_", win.prompt, win.input_buf);
            let display_prompt = if prompt_full.len() > max_cols {
                &prompt_full[..max_cols]
            } else {
                &prompt_full
            };
            vga::draw_string(text_pad_x, curr_y, display_prompt, rust_orange, None);
        }
    }

    fn render_monitor_content(&self, _win: &FloatingWindow, cx: usize, cy: usize, cw: usize, _ch: usize) {
        let white = RgbColor { r: 240, g: 240, b: 240 };
        let rust_orange = ColorCode::RustOrange.to_rgb();
        let green = RgbColor { r: 75, g: 220, b: 90 };
        let cyan = RgbColor { r: 80, g: 200, b: 240 };

        let pad_x = cx + 12;
        let mut py = cy + 10;

        vga::draw_string(pad_x, py, "SYSTEM RESOURCES & HARDWARE ACTIVITY", rust_orange, None);
        py += 22;

        // Visual CPU Meter
        vga::draw_string(pad_x, py, "CPU Usage: [||||||||||||||||||||........] 42% (x86_64 @ 100 Hz)", white, None);
        vga::fill_rect(pad_x, py + 16, (cw - 24).min(380), 8, RgbColor { r: 35, g: 40, b: 50 });
        vga::fill_rect(pad_x, py + 16, 160, 8, green);
        py += 32;

        // Visual RAM Meter
        let (_start, heap_size, _) = get_heap_info();
        let ram_text = format!("Heap Allocator: [||||||||||||||||||||||||....] {} KB / 524288 KB", heap_size / 1024);
        vga::draw_string(pad_x, py, &ram_text, white, None);
        vga::fill_rect(pad_x, py + 16, (cw - 24).min(380), 8, RgbColor { r: 35, g: 40, b: 50 });
        vga::fill_rect(pad_x, py + 16, 220, 8, cyan);
        py += 34;

        // Storage & Device Activity
        vga::draw_string(pad_x, py, "Storage: /dev/sda2 (SimpleFS v2 Persistent)  [MOUNTED /]", white, None);
        py += 20;
        let up_secs = uptime_seconds();
        vga::draw_string(pad_x, py, &format!("Kernel Uptime: {}h {}m {}s  |  IRQ: PIC 8259 Active", up_secs / 3600, (up_secs % 3600) / 60, up_secs % 60), white, None);
        py += 24;

        // Active Tasks
        vga::draw_string(pad_x, py, "Active Processes:", rust_orange, None);
        py += 18;
        let tasks = [
            "PID 1  root   [RUNNING]  kernel_main (Ring 0 core)",
            "PID 2  root   [ACTIVE]   rustos_compositor (Floating GUI)",
            "PID 3  root   [POLL]     ps2_mouse_driver (Auxiliary port)",
            "PID 4  root   [READY]    ps2_keyboard_driver (i8042)",
            "PID 5  root   [SYNC]     cmos_rtc_service (UTC clock)",
        ];
        for t in tasks.iter() {
            vga::draw_string(pad_x + 8, py, t, white, None);
            py += 18;
        }
    }

    fn render_file_manager_content(&self, win: &FloatingWindow, cx: usize, cy: usize, _cw: usize, ch: usize) {
        let white = RgbColor { r: 240, g: 240, b: 240 };
        let rust_orange = ColorCode::RustOrange.to_rgb();
        let yellow = ColorCode::Yellow.to_rgb();

        let pad_x = cx + 12;
        let mut py = cy + 10;

        vga::draw_string(pad_x, py, &format!("Location: /dev/sda2{}", win.fm_path), rust_orange, None);
        py += 20;

        vga::fill_rect(cx + 10, py, _cw - 20, 1, RgbColor { r: 50, g: 55, b: 68 });
        py += 8;

        for (idx, entry) in win.fm_entries.iter().enumerate() {
            if py + FONT_HEIGHT + 4 > cy + ch - 40 {
                break;
            }
            let is_dir = entry.starts_with("[DIR]");
            let color = if is_dir { yellow } else { white };
            vga::draw_string(pad_x, py, &format!("{}. {}", idx + 1, entry), color, None);
            py += 18;
        }

        // Preview Footer
        let foot_y = cy + ch - 26;
        vga::fill_rect(cx + 10, foot_y, _cw - 20, 1, RgbColor { r: 50, g: 55, b: 68 });
        vga::draw_string(pad_x, foot_y + 6, &win.fm_preview, white, None);
    }

    fn render_paint_content(&self, win: &FloatingWindow, cx: usize, cy: usize, cw: usize, ch: usize) {
        let white = RgbColor { r: 245, g: 245, b: 245 };
        let rust_orange = ColorCode::RustOrange.to_rgb();
        let green = RgbColor { r: 85, g: 255, b: 85 };
        let cyan = RgbColor { r: 85, g: 255, b: 255 };
        let red = RgbColor { r: 255, g: 85, b: 85 };
        let yellow = RgbColor { r: 255, g: 220, b: 50 };

        // Canvas Area
        let canvas_h = ch.saturating_sub(40);
        vga::fill_rect(cx + 4, cy + 4, cw.saturating_sub(8), canvas_h, RgbColor { r: 20, g: 22, b: 28 });

        // Draw User Strokes
        for &(px, py, color) in &win.paint_pixels {
            let abs_x = cx + 4 + px as usize;
            let abs_y = cy + 4 + py as usize;
            if abs_x < cx + cw - 6 && abs_y < cy + canvas_h + 2 {
                vga::fill_rect(abs_x, abs_y, 3, 3, color);
            }
        }

        // Palette Controls Bar at Bottom
        let bar_y = cy + ch - 32;
        vga::fill_rect(cx + 4, bar_y, cw.saturating_sub(8), 28, RgbColor { r: 35, g: 39, b: 48 });
        vga::draw_string(cx + 12, bar_y + 6, "Palette:", white, None);

        let colors = [
            ("Orange", rust_orange),
            ("White", white),
            ("Green", green),
            ("Cyan", cyan),
            ("Red", red),
            ("Yellow", yellow),
        ];

        let mut pal_x = cx + 80;
        for (label, col) in colors.iter() {
            let is_sel = win.paint_color.r == col.r && win.paint_color.g == col.g && win.paint_color.b == col.b;
            let btn_w = label.len() * FONT_WIDTH + 10;
            vga::fill_rect(pal_x, bar_y + 3, btn_w, 20, *col);
            let text_col = if col.r > 200 && col.g > 200 { RgbColor { r: 10, g: 10, b: 10 } } else { white };
            vga::draw_string(pal_x + 4, bar_y + 6, label, text_col, None);
            if is_sel {
                vga::draw_rect_outline(pal_x, bar_y + 3, btn_w, 20, 2, white);
            }
            pal_x += btn_w + 6;
        }

        // Clear Button
        vga::fill_rect(pal_x + 10, bar_y + 3, 60, 20, RgbColor { r: 180, g: 40, b: 40 });
        vga::draw_string(pal_x + 16, bar_y + 6, "CLEAR", white, None);
    }

    fn render_neofetch_content(&self, win: &FloatingWindow, cx: usize, cy: usize, cw: usize, ch: usize) {
        let text_pad_x = cx + 12;
        let mut curr_y = cy + 12;
        let max_cols = (cw.saturating_sub(20)) / FONT_WIDTH;
        let text_white = RgbColor { r: 240, g: 240, b: 240 };
        let rust_orange = ColorCode::RustOrange.to_rgb();

        for line in &win.lines {
            if curr_y + FONT_HEIGHT > cy + ch {
                break;
            }
            let display_str = if line.len() > max_cols {
                &line[..max_cols]
            } else {
                line
            };
            let color = if line.contains("######") || line.contains("##") {
                rust_orange
            } else {
                text_white
            };
            vga::draw_string(text_pad_x, curr_y, display_str, color, None);
            curr_y += FONT_HEIGHT + 2;
        }
    }

    pub fn handle_mouse_events(&mut self) -> bool {
        let (mx, my, left_held, clicked, released, moved) = mouse::get_mouse_event();
        self.mouse_x = mx;
        self.mouse_y = my;

        let mut need_redraw = moved;

        // 1. Handle Active Window Dragging
        if let Some(drag_idx) = self.drag_win_idx {
            if left_held {
                if drag_idx < self.windows.len() {
                    let win = &mut self.windows[drag_idx];
                    let new_x = (mx as isize - self.drag_offset_x as isize)
                        .max(5)
                        .min((self.screen_w.saturating_sub(win.width + 5)) as isize);
                    let new_y = (my as isize - self.drag_offset_y as isize)
                        .max(28)
                        .min((self.screen_h.saturating_sub(win.height + 25)) as isize);
                    win.x = new_x as usize;
                    win.y = new_y as usize;
                    need_redraw = true;
                }
            } else {
                self.drag_win_idx = None;
            }
        }

        // 2. Handle Active Window Resizing
        if let Some(res_idx) = self.resize_win_idx {
            if left_held {
                if res_idx < self.windows.len() {
                    let win = &mut self.windows[res_idx];
                    let new_w = (mx as isize - win.x as isize).clamp(240, (self.screen_w - win.x - 10) as isize) as usize;
                    let new_h = (my as isize - win.y as isize).clamp(140, (self.screen_h - win.y - 25) as isize) as usize;
                    win.width = new_w;
                    win.height = new_h;
                    need_redraw = true;
                }
            } else {
                self.resize_win_idx = None;
            }
        }

        // 3. Handle Mouse Left Click
        if clicked {
            // Check Start Button Click (x: 0..96, y: 0..26)
            if my <= 26 && mx <= 96 {
                self.start_menu_open = !self.start_menu_open;
                return true;
            }

            // Check Start Menu Item Click
            if self.start_menu_open && mx <= 230 && my >= 26 && my <= 216 {
                let item_idx = (my - 35) / 24;
                match item_idx {
                    0 => self.spawn_terminal(),
                    1 => self.spawn_monitor(),
                    2 => self.spawn_file_manager(),
                    3 => self.spawn_paint(),
                    4 => self.spawn_neofetch(),
                    5 => self.running = false,
                    _ => {}
                }
                self.start_menu_open = false;
                return true;
            } else if self.start_menu_open {
                self.start_menu_open = false;
                need_redraw = true;
            }

            // Check Taskbar Window Tabs Click (y: 0..26, x > 100)
            if my <= 26 && mx > 100 {
                let mut tab_x = 106;
                for idx in 0..self.windows.len() {
                    let win = &self.windows[idx];
                    let tab_title = if win.title.len() > 14 { &win.title[..14] } else { &win.title };
                    let tab_w = (tab_title.len() + 4) * FONT_WIDTH + 8;
                    if mx >= tab_x && mx <= tab_x + tab_w {
                        self.focus_window(idx);
                        return true;
                    }
                    tab_x += tab_w + 4;
                }
            }

            // Check Windows Click (from top to bottom in Z-order)
            let mut clicked_win_idx = None;
            for (idx, win) in self.windows.iter().enumerate().rev() {
                if win.minimized {
                    continue;
                }
                if mx >= win.x && mx <= win.x + win.width && my >= win.y && my <= win.y + win.height {
                    clicked_win_idx = Some(idx);
                    break;
                }
            }

            if let Some(hit_idx) = clicked_win_idx {
                self.focus_window(hit_idx);
                let top_idx = self.windows.len() - 1;
                let win = &mut self.windows[top_idx];

                // Check Window Buttons
                if my >= win.y && my <= win.y + 22 {
                    // Close button [X]
                    if mx >= win.x + 6 && mx <= win.x + 30 {
                        self.close_window(top_idx);
                        return true;
                    }
                    // Minimize button [-]
                    if mx >= win.x + 36 && mx <= win.x + 60 {
                        win.minimized = true;
                        return true;
                    }
                    // Maximize button [+]
                    if mx >= win.x + 64 && mx <= win.x + 88 {
                        self.toggle_maximize(top_idx);
                        return true;
                    }
                    // Title Bar Dragging
                    self.drag_win_idx = Some(top_idx);
                    self.drag_offset_x = mx - win.x;
                    self.drag_offset_y = my - win.y;
                    return true;
                }

                // Check Bottom-Right Resize Grip
                if mx >= win.x + win.width - 18 && my >= win.y + win.height - 18 {
                    self.resize_win_idx = Some(top_idx);
                    return true;
                }

                // Check Paint Canvas Click
                if win.win_type == WindowType::Paint {
                    let client_x = win.x + 1;
                    let client_y = win.y + 23;
                    let bar_y = win.y + win.height - 32;

                    // Clicked on palette or clear button
                    if my >= bar_y {
                        let rust_orange = ColorCode::RustOrange.to_rgb();
                        let white = RgbColor { r: 245, g: 245, b: 245 };
                        let green = RgbColor { r: 85, g: 255, b: 85 };
                        let cyan = RgbColor { r: 85, g: 255, b: 255 };
                        let red = RgbColor { r: 255, g: 85, b: 85 };
                        let yellow = RgbColor { r: 255, g: 220, b: 50 };

                        if mx >= client_x + 80 && mx <= client_x + 130 {
                            win.paint_color = rust_orange;
                        } else if mx >= client_x + 136 && mx <= client_x + 185 {
                            win.paint_color = white;
                        } else if mx >= client_x + 191 && mx <= client_x + 240 {
                            win.paint_color = green;
                        } else if mx >= client_x + 246 && mx <= client_x + 290 {
                            win.paint_color = cyan;
                        } else if mx >= client_x + 296 && mx <= client_x + 335 {
                            win.paint_color = red;
                        } else if mx >= client_x + 341 && mx <= client_x + 395 {
                            win.paint_color = yellow;
                        } else if mx >= client_x + 400 && mx <= client_x + 470 {
                            win.paint_pixels.clear();
                        }
                    } else if my > client_y && my < bar_y {
                        // Drawing inside canvas
                        let draw_x = (mx - client_x - 4) as u16;
                        let draw_y = (my - client_y - 4) as u16;
                        win.paint_pixels.push((draw_x, draw_y, win.paint_color));
                    }
                }

                // Check File Manager Click
                if win.win_type == WindowType::FileManager {
                    let pad_y = win.y + 45;
                    if my >= pad_y && my <= pad_y + (win.fm_entries.len() * 18) {
                        let click_idx = (my - pad_y) / 18;
                        if click_idx < win.fm_entries.len() {
                            let entry = &win.fm_entries[click_idx];
                            win.fm_preview = format!("Selected: {} (SimpleFS v2 inode)", entry);
                        }
                    }
                }

                return true;
            }
        }

        // 4. Handle Left Button Released
        if released {
            self.drag_win_idx = None;
            self.resize_win_idx = None;
        }

        need_redraw
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
                if let Some(idx) = self.active_index() {
                    self.close_window(idx);
                }
                self.render();
            }
            KEY_ALT_N => {
                self.spawn_terminal();
                self.render();
            }
            KEY_UP => {
                if keyboard::is_alt_pressed() {
                    if let Some(idx) = self.active_index() {
                        let win = &mut self.windows[idx];
                        win.y = win.y.saturating_sub(24).max(28);
                        self.render();
                    }
                }
            }
            KEY_DOWN => {
                if keyboard::is_alt_pressed() {
                    if let Some(idx) = self.active_index() {
                        let win = &mut self.windows[idx];
                        win.y = (win.y + 24).min(self.screen_h - win.height - 25);
                        self.render();
                    }
                }
            }
            KEY_LEFT => {
                if keyboard::is_alt_pressed() {
                    if let Some(idx) = self.active_index() {
                        let win = &mut self.windows[idx];
                        win.x = win.x.saturating_sub(24).max(5);
                        self.render();
                    }
                }
            }
            KEY_RIGHT => {
                if keyboard::is_alt_pressed() {
                    if let Some(idx) = self.active_index() {
                        let win = &mut self.windows[idx];
                        win.x = (win.x + 24).min(self.screen_w - win.width - 5);
                        self.render();
                    }
                }
            }
            '\n' | '\r' => {
                if let Some(idx) = self.active_index() {
                    let win = &mut self.windows[idx];
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
                if let Some(idx) = self.active_index() {
                    let win = &mut self.windows[idx];
                    if win.win_type == WindowType::Terminal && !win.input_buf.is_empty() {
                        win.input_buf.pop();
                        self.render();
                    }
                }
            }
            printable if printable >= ' ' && printable <= '~' => {
                if let Some(idx) = self.active_index() {
                    let win = &mut self.windows[idx];
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
            out.push(String::from("RustOS Graphical Terminal Shell (rsh)"));
            out.push(String::from("Commands: ls, cat, uname, whoami, uptime, pacman, yay, neofetch, clear, exit"));
            out.push(String::from("Mouse: Click title bar & drag to move  |  Drag bottom-right '//' to resize"));
            out.push(String::from("Controls: Click [X] to close, [-] to minimize, [+] to maximize window"));
        }
        "clear" => {
            out.push(String::from("--- Terminal Buffer Reset ---"));
        }
        "neofetch" | "fastfetch" => {
            out.push(String::from("OS: RustOS x86_64"));
            out.push(String::from("WM: RustOS Floating Window Manager (Mouse Enabled)"));
            out.push(String::from("Kernel: 0.2.0-rustos (no_std)"));
            out.push(String::from("Uptime: Active"));
            out.push(String::from("Shell: rsh 1.2.0"));
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
                out.push(String::from("rustos-wm 1.0.0-1"));
                out.push(String::from("coreutils 9.4-1"));
            } else {
                out.push(String::from("Pacman v6.1.0 - Package Manager for RustOS"));
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
            out.push(String::from("Exiting terminal..."));
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
        // 1. Poll PS/2 Mouse & Keyboard hardware ports
        keyboard::poll_ps2();
        mouse::poll_mouse();

        // 2. Process Mouse Events (Dragging, Resizing, Clicking, Menu, Drawing)
        if comp.handle_mouse_events() {
            comp.render();
        }

        // 3. Process Keyboard Events
        if let Some(ch) = keyboard::try_read_char() {
            comp.handle_key(ch);
        }

        // 4. Periodic Taskbar Clock & Stats Update (every 50 ticks = 0.5s)
        let now = crate::interrupts::ticks();
        if now.wrapping_sub(last_tick) > 50 {
            last_tick = now;
            let dark_panel = RgbColor { r: 28, g: 32, b: 40 };
            let white = RgbColor { r: 245, g: 245, b: 245 };
            let (_heap_start, heap_size, _) = get_heap_info();
            let mem_str = format!("RAM: {} KB", heap_size / 1024);
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
    crate::println!(":: Exited RustOS Window Manager. Returned to console shell.");
    vga::set_colors(ColorCode::White, ColorCode::Black);
}
