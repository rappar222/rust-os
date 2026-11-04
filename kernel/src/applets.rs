extern crate alloc;

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use crate::{
    allocator, fs, interrupts, keyboard, pkg, print, println, rtc, sysinfo,
    vga::{self, ColorCode},
};

pub fn cmd_neofetch(rainbow: bool) {
    let cpu = sysinfo::get_cpu_info();
    let (width, height, cols, rows) = vga::get_resolution();
    let (pac_pkgs, aur_pkgs) = pkg::count_installed();
    let (_, heap_size, _) = allocator::get_heap_info();
    let secs = interrupts::uptime_seconds();
    let mins = secs / 60;
    let hrs = mins / 60;

    let logo = [
        "               ______               ",
        "            .-~~~~~~~~-.            ",
        "          /   _  __  _   \\          ",
        "      |==|   | ||  || |   |==|      ",
        "      |==|   |_||__||_|   |==|      ",
        "      |  |    ________    |  |      ",
        "      |==|   |  ____  \\   |==|      ",
        "      |==|   | |   _) |   |==|      ",
        "      |  |   |  __  <     |  |      ",
        "      |==|   | |  \\  \\    |==|      ",
        "      |==|   |_|   \\__\\   |==|      ",
        "      |  |    ________    |  |      ",
        "      |==|    _  __  _    |==|      ",
        "      |==|   | ||  || |   |==|      ",
        "          \\  |_||__||_|  /          ",
        "           `-..........-`           ",
    ];

    let info_lines = [
        ("root", "@rustos"),
        ("--------------", ""),
        ("OS", ": RustOS x86_64"),
        ("Host", ": Bare-Metal / Virtual Machine (x86_64)"),
        ("Kernel", ": 0.2.0-rustos (no_std)"),
        ("Uptime", &format!(": {}h {}m {}s", hrs, mins % 60, secs % 60)),
        ("Packages", &format!(": {} (pacman), {} (aur)", pac_pkgs, aur_pkgs)),
        ("Shell", ": rsh 1.2.0 (Tab-completion + History)"),
        ("Resolution", &format!(": {}x{} ({}x{} chars)", width, height, cols, rows)),
        ("Terminal", ": RustOS Framebuffer Console"),
        ("CPU", &format!(": {} ({})", cpu.model_name, cpu.cores)),
        ("Memory", &format!(": {} KiB / 128 MiB", heap_size / 1024)),
        ("Filesystem", ": SimpleFS (ATA PIO Persistent)"),
        ("Date", &{
            let dt = rtc::read_rtc();
            format!(": {:04}-{:02}-{:02} {:02}:{:02}:{:02} UTC", dt.year, dt.month, dt.day, dt.hour, dt.minute, dt.second)
        }),
    ];

    let pride_colors = [
        ColorCode::LightRed,
        ColorCode::Yellow,
        ColorCode::LightGreen,
        ColorCode::LightCyan,
        ColorCode::LightBlue,
        ColorCode::LightMagenta,
    ];

    println!();
    for i in 0..logo.len() {
        let logo_color = if rainbow {
            pride_colors[i % pride_colors.len()]
        } else {
            ColorCode::RustOrange
        };
        vga::set_colors(logo_color, ColorCode::Black);
        print!("{}", logo[i]);

        if i < info_lines.len() {
            let (k, v) = info_lines[i];
            if i == 0 {
                vga::set_colors(ColorCode::RustOrange, ColorCode::Black);
                print!("{}", k);
                vga::set_colors(ColorCode::White, ColorCode::Black);
                println!("{}", v);
            } else if i == 1 {
                vga::set_colors(ColorCode::DarkGray, ColorCode::Black);
                println!("{}", k);
            } else {
                vga::set_colors(ColorCode::RustOrange, ColorCode::Black);
                print!("{}", k);
                vga::set_colors(ColorCode::White, ColorCode::Black);
                println!("{}", v);
            }
        } else if i == info_lines.len() {
            let palette = [
                ColorCode::Black,
                ColorCode::Red,
                ColorCode::Green,
                ColorCode::Yellow,
                ColorCode::Blue,
                ColorCode::Magenta,
                ColorCode::Cyan,
                ColorCode::White,
            ];
            for &c in palette.iter() {
                vga::set_colors(c, ColorCode::Black);
                print!("███");
            }
            println!();
        } else if i == info_lines.len() + 1 {
            let palette2 = [
                ColorCode::DarkGray,
                ColorCode::LightRed,
                ColorCode::LightGreen,
                ColorCode::Yellow,
                ColorCode::LightBlue,
                ColorCode::LightMagenta,
                ColorCode::LightCyan,
                ColorCode::White,
            ];
            for &c in palette2.iter() {
                vga::set_colors(c, ColorCode::Black);
                print!("███");
            }
            println!();
        } else {
            println!();
        }
    }
    println!();
}

pub fn cmd_pfetch() {
    let cpu = sysinfo::get_cpu_info();
    let (pac_pkgs, aur_pkgs) = pkg::count_installed();
    let secs = interrupts::uptime_seconds();
    let small_logo = [
        "   _.._   ",
        " /|==||\\  ",
        "| |__|| | ",
        "|  ____  |",
        "| | _) | |",
        "| |__\\ | |",
        " \\|==||/  ",
        "   \"\"\"\"   ",
    ];
    let lines = [
        format!("root@rustos"),
        format!("os     RustOS x86_64"),
        format!("kernel 0.2.0-rustos"),
        format!("uptime {}m {}s", secs / 60, secs % 60),
        format!("pkgs   {} ({} pacman, {} aur)", pac_pkgs + aur_pkgs, pac_pkgs, aur_pkgs),
        format!("cpu    {}", cpu.model_name),
        format!("memory 4M / 128M"),
        format!("shell  rsh 1.2.0"),
    ];
    println!();
    for i in 0..small_logo.len() {
        vga::set_colors(ColorCode::RustOrange, ColorCode::Black);
        print!("{}  ", small_logo[i]);
        vga::set_colors(ColorCode::White, ColorCode::Black);
        println!("{}", lines[i]);
    }
    println!();
}

pub fn cmd_onefetch() {
    let gear = [
        "        _~^~^~_        ",
        "    \\) /  o o  \\ (/    ",
        "      '_   -   _'      ",
        "      / '-----' \\      ",
    ];
    let info = [
        "Project   : RustOS (rappar222/rust-os)",
        "HEAD      : main (x86_64 bare-metal kernel)",
        "Language  : Rust (100.0%) [no_std + alloc]",
        "License   : MIT / Apache-2.0",
    ];
    println!();
    for i in 0..gear.len() {
        vga::set_colors(ColorCode::LightRed, ColorCode::Black);
        print!("{}   ", gear[i]);
        vga::set_colors(ColorCode::White, ColorCode::Black);
        println!("{}", info[i]);
    }
    println!();
}

pub fn cmd_nano(cwd: &str, args: &[&str]) {
    if args.is_empty() {
        vga::set_colors(ColorCode::LightRed, ColorCode::Black);
        println!("Usage: nano <filename>");
        return;
    }

    let path = fs::normalize_path(cwd, args[0]);
    let mut lines: Vec<String> = Vec::new();

    if let Ok(data) = fs::read_file(&path) {
        if let Ok(text) = core::str::from_utf8(&data) {
            for l in text.lines() {
                lines.push(String::from(l));
            }
        }
    }
    if lines.is_empty() {
        lines.push(String::new());
    }

    let redraw = |lines: &[String], status: &str| {
        vga::clear_screen();
        vga::set_colors(ColorCode::Black, ColorCode::White);
        println!("  GNU nano 8.2 (RustOS)          File: {:36} ", path);
        vga::set_colors(ColorCode::White, ColorCode::Black);
        println!();
        for (idx, line) in lines.iter().enumerate() {
            vga::set_colors(ColorCode::DarkGray, ColorCode::Black);
            print!("{:3} | ", idx + 1);
            vga::set_colors(ColorCode::White, ColorCode::Black);
            println!("{}", line);
        }
        println!("\n------------------------------------------------------------------------");
        vga::set_colors(ColorCode::Yellow, ColorCode::Black);
        println!("Status: {}", status);
        vga::set_colors(ColorCode::LightCyan, ColorCode::Black);
        println!("[ Commands: Type ':wq' + Enter to Save & Exit | ':q' + Enter to Quit | ESC to Save & Exit ]");
        vga::set_colors(ColorCode::White, ColorCode::Black);
        if let Some(last) = lines.last() {
            print!("> {}", last);
        }
    };

    redraw(&lines, "Editing buffer...");

    loop {
        let c = keyboard::read_char();
        match c {
            '\x1B' => {
                // ESC saves and exits immediately
                let mut full_text = String::new();
                for l in &lines {
                    full_text.push_str(l);
                    full_text.push('\n');
                }
                let _ = fs::write_file(&path, full_text.as_bytes());
                vga::clear_screen();
                vga::set_colors(ColorCode::LightGreen, ColorCode::Black);
                println!("[nano] Saved {} ({} lines, {} bytes).", path, lines.len(), full_text.len());
                return;
            }
            keyboard::CTRL_C => {
                vga::clear_screen();
                vga::set_colors(ColorCode::Yellow, ColorCode::Black);
                println!("[nano] Exited without saving.");
                return;
            }
            '\n' => {
                let last_line = lines.last().cloned().unwrap_or_default();
                if last_line.trim() == ":wq" || last_line.trim() == ":x" {
                    lines.pop();
                    let mut full_text = String::new();
                    for l in &lines {
                        full_text.push_str(l);
                        full_text.push('\n');
                    }
                    let _ = fs::write_file(&path, full_text.as_bytes());
                    vga::clear_screen();
                    vga::set_colors(ColorCode::LightGreen, ColorCode::Black);
                    println!("[nano] Saved {} ({} bytes).", path, full_text.len());
                    return;
                } else if last_line.trim() == ":q" || last_line.trim() == ":q!" {
                    vga::clear_screen();
                    vga::set_colors(ColorCode::Yellow, ColorCode::Black);
                    println!("[nano] Closed {}.", path);
                    return;
                }
                lines.push(String::new());
                redraw(&lines, "New line added");
            }
            '\x08' => {
                if let Some(last) = lines.last_mut() {
                    if !last.is_empty() {
                        last.pop();
                        vga::backspace();
                    } else if lines.len() > 1 {
                        lines.pop();
                        redraw(&lines, "Line merged");
                    }
                }
            }
            ch if ch >= ' ' && ch <= '~' => {
                if let Some(last) = lines.last_mut() {
                    last.push(ch);
                    print!("{}", ch);
                }
            }
            _ => {}
        }
    }
}

pub fn try_run_applet(cmd: &str, args: &[&str], cwd: &str) -> bool {
    // Check custom makepkg packages first
    if pkg::try_run_custom_pkg(cmd) {
        return true;
    }

    // Check if command belongs to a known repository/AUR package
    let spec = match pkg::find_package_offering_binary(cmd) {
        Some(s) => s,
        None => return false,
    };

    if !pkg::is_installed(spec.name) {
        vga::set_colors(ColorCode::LightRed, ColorCode::Black);
        println!("rsh: {}: command not found", cmd);
        vga::set_colors(ColorCode::Yellow, ColorCode::Black);
        if spec.repo == "aur" {
            println!(
                " -> '{}' is available in the AUR ({}/{} {})",
                cmd, spec.repo, spec.name, spec.version
            );
            println!(" -> Install it with: yay -S {}", spec.name);
        } else {
            println!(
                " -> '{}' may be found in package '{}/{}' ({})",
                cmd, spec.repo, spec.name, spec.version
            );
            println!(" -> Install it with: pacman -S {}", spec.name);
        }
        return true;
    }

    match cmd {
        "neofetch" | "fastfetch" => cmd_neofetch(false),
        "hyfetch" => cmd_neofetch(true),
        "pfetch" => cmd_pfetch(),
        "onefetch" => cmd_onefetch(),
        "nano" => cmd_nano(cwd, args),
        "cowsay" => applet_cowsay(args),
        "fortune" | "fortune-mod" => applet_fortune(),
        "figlet" => applet_figlet(args),
        "lolcat" => applet_lolcat(cwd, args),
        "cmatrix" => applet_cmatrix(),
        "pipes-rs" => applet_pipes(),
        "cbonsai" => applet_cbonsai(),
        "btop" | "htop" => applet_btop(),
        "sl" => applet_sl(),
        "bat" => applet_bat(cwd, args),
        "eza" | "exa" => applet_eza(cwd, args),
        "rg" | "ripgrep" => applet_rg(cwd, args),
        "fd" => applet_fd(args),
        "git" => applet_git(args),
        "cargo" | "rust" => applet_cargo(args),
        "python" => applet_python(),
        "tty-clock" => applet_tty_clock(),
        "nyancat" => applet_nyancat(),
        "asciiquarium" => applet_asciiquarium(),
        "rice" | "rustos-rice-theme" => applet_rice(args),
        _ => return false,
    }
    true
}

fn applet_cowsay(args: &[&str]) {
    let msg = if args.is_empty() {
        String::from("I use RustOS (Arch btw)!")
    } else {
        args.join(" ")
    };
    let border = "-".repeat(msg.len() + 2);
    vga::set_colors(ColorCode::White, ColorCode::Black);
    println!(" {}", border);
    println!("< {} >", msg);
    println!(" {}", border);
    println!("        \\   ^__^");
    println!("         \\  (oo)\\_______");
    println!("            (__)\\       )\\/\\");
    println!("                ||----w |");
    println!("                ||     ||");
}

fn applet_fortune() {
    let quotes = [
        "\"Talk is cheap. Show me the Rust code.\" - Linus Torvalds (probably)",
        "\"Memory safety without garbage collection: the OS developer's dream.\"",
        "\"I use Arch (and RustOS) btw.\"",
        "\"There are only 10 types of people: those who understand binary and those who don't.\"",
        "\"In Rust we trust: 0 segfaults, 0 data races, 100% bare-metal speed.\"",
        "\"A kernel panic a day keeps the C pointers away.\"",
    ];
    let idx = (interrupts::ticks() as usize) % quotes.len();
    vga::set_colors(ColorCode::Yellow, ColorCode::Black);
    println!("{}", quotes[idx]);
}

fn applet_figlet(args: &[&str]) {
    let text = if args.is_empty() { "RUSTOS" } else { args[0] };
    vga::set_colors(ColorCode::LightCyan, ColorCode::Black);
    println!("==========================================");
    println!("   >>>  {}  <<<", text.to_uppercase());
    println!("==========================================");
}

fn applet_lolcat(cwd: &str, args: &[&str]) {
    let palette = [
        ColorCode::LightRed,
        ColorCode::Yellow,
        ColorCode::LightGreen,
        ColorCode::LightCyan,
        ColorCode::LightBlue,
        ColorCode::LightMagenta,
    ];
    let text = if args.is_empty() {
        String::from("RustOS Arch Edition - Rainbow Terminal Output!")
    } else {
        let p = fs::normalize_path(cwd, args[0]);
        if let Ok(bytes) = fs::read_file(&p) {
            String::from(core::str::from_utf8(&bytes).unwrap_or(""))
        } else {
            args.join(" ")
        }
    };

    for (i, ch) in text.chars().enumerate() {
        let c = palette[(i / 2) % palette.len()];
        vga::set_colors(c, ColorCode::Black);
        print!("{}", ch);
    }
    println!();
}

fn applet_cmatrix() {
    vga::clear_screen();
    let (_, _, max_cols, max_rows) = vga::get_resolution();
    let cols = core::cmp::min(max_cols, 100);
    let rows = core::cmp::min(max_rows, 40);
    let mut drops = [0usize; 100];
    for i in 0..cols {
        drops[i] = (i * 7 + 3) % rows;
    }

    let chars = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ@#$%&*+=-~";
    let mut tick: usize = 0;

    while tick < 300 {
        if let Some(c) = keyboard::try_read_char() {
            if c == 'q' || c == keyboard::CTRL_C || c == '\x1B' {
                break;
            }
        }
        for x in (0..cols).step_by(2) {
            let y = drops[x];
            let ch = chars[(x + y + tick) % chars.len()] as char;
            vga::put_char_at(x, y, ch, ColorCode::White, ColorCode::Black);
            let prev_y = if y == 0 { rows - 1 } else { y - 1 };
            let prev_ch = chars[(x * 3 + prev_y) % chars.len()] as char;
            vga::put_char_at(x, prev_y, prev_ch, ColorCode::LightGreen, ColorCode::Black);
            let tail_y = (y + rows - 8) % rows;
            vga::put_char_at(x, tail_y, ' ', ColorCode::Black, ColorCode::Black);
            drops[x] = (y + 1) % rows;
        }
        keyboard::sleep_ticks(3);
        tick += 1;
    }
    vga::clear_screen();
}

fn applet_pipes() {
    vga::clear_screen();
    let (_, _, max_cols, max_rows) = vga::get_resolution();
    let cols = core::cmp::min(max_cols, 100);
    let rows = core::cmp::min(max_rows, 40);
    let colors = [
        ColorCode::LightCyan,
        ColorCode::LightGreen,
        ColorCode::LightMagenta,
        ColorCode::Yellow,
        ColorCode::LightRed,
        ColorCode::LightBlue,
    ];
    let mut x = cols / 2;
    let mut y = rows / 2;
    let mut dir = 0usize; // 0=R, 1=D, 2=L, 3=U
    let mut step = 0usize;

    while step < 500 {
        if let Some(c) = keyboard::try_read_char() {
            if c == 'q' || c == keyboard::CTRL_C || c == '\x1B' {
                break;
            }
        }
        let color = colors[(step / 25) % colors.len()];
        let glyph = if dir % 2 == 0 { '-' } else { '|' };
        vga::put_char_at(x, y, glyph, color, ColorCode::Black);

        if (step + x + y) % 5 == 0 {
            dir = (dir + 1) % 4;
            vga::put_char_at(x, y, '+', ColorCode::White, ColorCode::Black);
        }
        match dir {
            0 => x = (x + 1) % cols,
            1 => y = (y + 1) % rows,
            2 => x = if x == 0 { cols - 1 } else { x - 1 },
            _ => y = if y == 0 { rows - 1 } else { y - 1 },
        }
        keyboard::sleep_ticks(2);
        step += 1;
    }
    vga::clear_screen();
}

fn applet_cbonsai() {
    vga::set_colors(ColorCode::LightGreen, ColorCode::Black);
    println!("          ,.,.");
    println!("        ,@@@@@@@,          ,.,.");
    println!("     ,@@@@@@@@@@@@@,    ,@@@@@@@@,");
    println!("    ,@@@@@@@@@@@@@@@,  ,@@@@@@@@@@,");
    println!("    `@@@@@@@@@@@@@@@'  `@@@@@@@@@@'");
    vga::set_colors(ColorCode::Brown, ColorCode::Black);
    println!("          \\\\|//          \\\\|//");
    println!("           \\|/            \\|/");
    println!("            \\\\            //");
    println!("             \\\\          //");
    println!("              \\\\________//");
    println!("               |        |");
    println!("               |        |");
    vga::set_colors(ColorCode::LightGray, ColorCode::Black);
    println!("            .--------------.");
    println!("            |   [RustOS]   |");
    println!("            '--------------'");
}

fn applet_btop() {
    let cpu = sysinfo::get_cpu_info();
    let (_, heap_size, _) = allocator::get_heap_info();
    let secs = interrupts::uptime_seconds();

    vga::set_colors(ColorCode::LightCyan, ColorCode::Black);
    println!("┌─ btop++ (RustOS Resource Monitor) ───────────────────────────────────┐");
    vga::set_colors(ColorCode::White, ColorCode::Black);
    println!("│ CPU: {:44} [|||||     14.2%] │", cpu.model_name);
    println!("│ MEM: {:4} KB / 128 MB                             [||||      18.5%] │", heap_size / 1024);
    println!("│ DSK: /dev/sda1 (SimpleFS 32 MB)                   [||         6.1%] │");
    println!("│ UPT: {}s ({} ticks @ 100Hz)                                         │", secs, interrupts::ticks());
    vga::set_colors(ColorCode::LightCyan, ColorCode::Black);
    println!("├─ PID ── USER ──── CPU% ── MEM% ── STATE ── COMMAND ──────────────────┤");
    vga::set_colors(ColorCode::LightGreen, ColorCode::Black);
    println!("│    1    root      0.1%    1.2%    RUN      /sbin/init (kernel_main)  │");
    println!("│    2    root      0.0%    0.4%    SLEEP    [kthreadd/pit-timer]      │");
    println!("│    3    root      0.0%    0.5%    SLEEP    [irq/1-i8042-keyboard]    │");
    println!("│    4    root      0.2%    0.8%    SLEEP    [ata_pio_poller]          │");
    println!("│   12    root      0.8%    2.1%    RUN      /bin/rsh (interactive)    │");
    println!("│   42    root      1.4%    1.5%    RUN      /usr/bin/btop             │");
    vga::set_colors(ColorCode::LightCyan, ColorCode::Black);
    println!("└──────────────────────────────────────────────────────────────────────┘");
}

fn applet_sl() {
    let train = [
        "      ====        ________                ___________ ",
        "  _D _|  |_______/        \\__I_I_____===__|_________| ",
        "   |(_)---  |   H\\________/ |   |        =|___ ___|   ",
        "   /     |  |   H  |  |     |   |         ||_| |_||   ",
        "  |      |  |   H  |__--------------------| [___] |   ",
        "  | ________|___H__/__|_____/[][]~\\_______|       |   ",
        "  |/ |   |-----------I_____I [][] []  D   |=======|__ ",
        "__/ =| o |=-~~\\  /~~\\  /~~\\  /~~\\ ____Y___________|__ ",
        " |/-=|___|=    ||    ||    ||    |_____/~\\___/        ",
        "  \\_/      \\O=====O=====O=====O_/      \\_/            ",
    ];
    vga::clear_screen();
    for offset in (0..40).rev().step_by(4) {
        vga::clear_screen();
        let pad = " ".repeat(offset);
        vga::set_colors(ColorCode::White, ColorCode::Black);
        println!("\n\n");
        for line in train.iter() {
            println!("{}{}", pad, line);
        }
        keyboard::sleep_ticks(6);
    }
    vga::clear_screen();
}

fn applet_bat(cwd: &str, args: &[&str]) {
    if args.is_empty() {
        vga::set_colors(ColorCode::LightRed, ColorCode::Black);
        println!("Usage: bat <file>");
        return;
    }
    let path = fs::normalize_path(cwd, args[0]);
    match fs::read_file(&path) {
        Ok(bytes) => {
            let text = core::str::from_utf8(&bytes).unwrap_or("<binary>");
            vga::set_colors(ColorCode::DarkGray, ColorCode::Black);
            println!("───────┬────────────────────────────────────────────────────────");
            print!("       │ ");
            vga::set_colors(ColorCode::White, ColorCode::Black);
            println!("File: {}", path);
            vga::set_colors(ColorCode::DarkGray, ColorCode::Black);
            println!("───────┼────────────────────────────────────────────────────────");
            for (i, line) in text.lines().enumerate() {
                vga::set_colors(ColorCode::DarkGray, ColorCode::Black);
                print!(" {:4}  │ ", i + 1);
                if line.trim_start().starts_with('#') {
                    vga::set_colors(ColorCode::LightGreen, ColorCode::Black);
                } else if line.contains('=') {
                    vga::set_colors(ColorCode::LightCyan, ColorCode::Black);
                } else {
                    vga::set_colors(ColorCode::White, ColorCode::Black);
                }
                println!("{}", line);
            }
            vga::set_colors(ColorCode::DarkGray, ColorCode::Black);
            println!("───────┴────────────────────────────────────────────────────────");
        }
        Err(e) => {
            vga::set_colors(ColorCode::LightRed, ColorCode::Black);
            println!("[bat error]: {}: {}", path, e);
        }
    }
}

fn applet_eza(cwd: &str, args: &[&str]) {
    let target = if let Some(&arg) = args.iter().find(|&&a| !a.starts_with('-')) {
        fs::normalize_path(cwd, arg)
    } else {
        String::from(cwd)
    };
    let entries = fs::list_dir(&target);
    vga::set_colors(ColorCode::LightGray, ColorCode::Black);
    println!("Permissions  Size  User   Name");
    for e in entries {
        let (_, base) = fs::parent_and_basename(e.name_str());
        vga::set_colors(ColorCode::Yellow, ColorCode::Black);
        print!("{} ", e.mode_string());
        vga::set_colors(ColorCode::LightGreen, ColorCode::Black);
        print!("{:5} ", e.size_bytes);
        vga::set_colors(ColorCode::LightCyan, ColorCode::Black);
        print!("root   ");
        if e.is_dir == 1 {
            vga::set_colors(ColorCode::LightBlue, ColorCode::Black);
            println!("📁 {}/", base);
        } else {
            vga::set_colors(ColorCode::White, ColorCode::Black);
            println!("📄 {}", base);
        }
    }
}

fn applet_rg(_cwd: &str, args: &[&str]) {
    if args.is_empty() {
        vga::set_colors(ColorCode::LightRed, ColorCode::Black);
        println!("Usage: rg <pattern>");
        return;
    }
    let pattern = args[0];
    let all = fs::list_all();
    for entry in all {
        if entry.is_dir == 0 {
            if let Ok(bytes) = fs::read_file(entry.name_str()) {
                if let Ok(text) = core::str::from_utf8(&bytes) {
                    for (idx, line) in text.lines().enumerate() {
                        if line.contains(pattern) {
                            vga::set_colors(ColorCode::LightMagenta, ColorCode::Black);
                            print!("{}", entry.name_str());
                            vga::set_colors(ColorCode::LightGreen, ColorCode::Black);
                            print!(":{}:", idx + 1);
                            vga::set_colors(ColorCode::White, ColorCode::Black);
                            println!("{}", line);
                        }
                    }
                }
            }
        }
    }
}

fn applet_fd(args: &[&str]) {
    let query = if args.is_empty() { "" } else { args[0] };
    for entry in fs::list_all() {
        if query.is_empty() || entry.name_str().contains(query) {
            if entry.is_dir == 1 {
                vga::set_colors(ColorCode::LightBlue, ColorCode::Black);
                println!("{}/", entry.name_str());
            } else {
                vga::set_colors(ColorCode::White, ColorCode::Black);
                println!("{}", entry.name_str());
            }
        }
    }
}

fn applet_git(args: &[&str]) {
    let sub = if args.is_empty() { "status" } else { args[0] };
    vga::set_colors(ColorCode::White, ColorCode::Black);
    match sub {
        "--version" | "-v" => println!("git version 2.47.0 (RustOS x86_64)"),
        "status" => {
            println!("On branch main");
            println!("Your branch is up to date with 'origin/main'.");
            println!("nothing to commit, working tree clean");
        }
        "log" => {
            vga::set_colors(ColorCode::Yellow, ColorCode::Black);
            println!("commit 5a80bc0 (HEAD -> main, origin/main)");
            vga::set_colors(ColorCode::White, ColorCode::Black);
            println!("Author: rappar222 <rappar2018@gmail.com>");
            println!("\n    feat: Arch Linux pacman + AUR support, Tab completion, FHS directories");
        }
        "branch" => {
            vga::set_colors(ColorCode::LightGreen, ColorCode::Black);
            println!("* main");
        }
        _ => println!("git: '{}' executed.", sub),
    }
}

fn applet_cargo(args: &[&str]) {
    let sub = if args.is_empty() { "--version" } else { args[0] };
    vga::set_colors(ColorCode::LightGreen, ColorCode::Black);
    match sub {
        "--version" | "-V" => println!("cargo 1.101.0-nightly (RustOS x86_64-unknown-none)"),
        "build" | "run" => {
            println!("   Compiling app v0.1.0 (/root)");
            keyboard::sleep_ticks(15);
            println!("    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.15s");
            if sub == "run" {
                vga::set_colors(ColorCode::White, ColorCode::Black);
                println!("     Running `target/debug/app`");
                println!("Hello from Rust inside RustOS!");
            }
        }
        _ => println!("cargo: '{}' completed.", sub),
    }
}

fn applet_python() {
    vga::set_colors(ColorCode::White, ColorCode::Black);
    println!("Python 3.13.0 (main, Nov 4 2026) [RustOS Bare-Metal x86_64]");
    println!("Type \"exit()\" or \"quit()\" to return to rsh.");
    let mut line = String::new();
    loop {
        vga::set_colors(ColorCode::Yellow, ColorCode::Black);
        print!(">>> ");
        vga::set_colors(ColorCode::White, ColorCode::Black);
        line.clear();
        loop {
            let c = keyboard::read_char();
            if c == '\n' {
                println!();
                break;
            } else if c == keyboard::CTRL_C || c == keyboard::CTRL_D {
                println!();
                return;
            } else if c == '\x08' {
                if !line.is_empty() {
                    line.pop();
                    vga::backspace();
                }
            } else if c >= ' ' && c <= '~' {
                line.push(c);
                print!("{}", c);
            }
        }
        let t = line.trim();
        if t == "exit()" || t == "quit()" || t == "exit" {
            return;
        } else if let Some(inner) = t.strip_prefix("print(").and_then(|s| s.strip_suffix(')')) {
            println!("{}", inner.trim_matches('"').trim_matches('\''));
        } else if !t.is_empty() {
            println!("{}", t);
        }
    }
}

fn applet_tty_clock() {
    let dt = rtc::read_rtc();
    vga::set_colors(ColorCode::LightCyan, ColorCode::Black);
    println!("\n   ┌──────────────────────────────────────┐");
    println!("   │      HARDWARE RTC DIGITAL CLOCK      │");
    println!("   │                                      │");
    vga::set_colors(ColorCode::LightGreen, ColorCode::Black);
    println!("   │           {:02} : {:02} : {:02} UTC           │", dt.hour, dt.minute, dt.second);
    vga::set_colors(ColorCode::Yellow, ColorCode::Black);
    println!("   │           {:04}-{:02}-{:02} ({})           │", dt.year, dt.month, dt.day, dt.month_name());
    vga::set_colors(ColorCode::LightCyan, ColorCode::Black);
    println!("   └──────────────────────────────────────┘\n");
}

fn applet_nyancat() {
    vga::set_colors(ColorCode::LightMagenta, ColorCode::Black);
    println!("~=~=~=~=~=~=~=~=~=  +------,");
    vga::set_colors(ColorCode::LightCyan, ColorCode::Black);
    println!("~=~=~=~=~=~=~=~=~=  |   /\\_/\\ ");
    vga::set_colors(ColorCode::LightGreen, ColorCode::Black);
    println!("~=~=~=~=~=~=~=~=~= ~|__( ^ .^)");
    vga::set_colors(ColorCode::Yellow, ColorCode::Black);
    println!("~=~=~=~=~=~=~=~=~=   \"\"  \"\" ");
}

fn applet_asciiquarium() {
    vga::set_colors(ColorCode::LightCyan, ColorCode::Black);
    println!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
    vga::set_colors(ColorCode::Yellow, ColorCode::Black);
    println!("      ><(((º>              ><((('>          ><(((º>");
    vga::set_colors(ColorCode::LightGreen, ColorCode::Black);
    println!("                  <º)))><                         <º)))><");
    vga::set_colors(ColorCode::Green, ColorCode::Black);
    println!("  \\|/       \\\\|//          \\|/        \\\\|//         \\|/");
    println!("^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^");
}

fn applet_rice(args: &[&str]) {
    let theme = if args.is_empty() { "arch" } else { args[0] };
    match theme {
        "matrix" => {
            vga::set_colors(ColorCode::LightGreen, ColorCode::Black);
            println!("Switched terminal theme to: Matrix Hacker Green");
        }
        "dracula" => {
            vga::set_colors(ColorCode::LightMagenta, ColorCode::Black);
            println!("Switched terminal theme to: Dracula Purple");
        }
        "gruvbox" => {
            vga::set_colors(ColorCode::Yellow, ColorCode::Black);
            println!("Switched terminal theme to: Gruvbox Warm Retro");
        }
        _ => {
            vga::set_colors(ColorCode::LightCyan, ColorCode::Black);
            println!("Available themes: rice [arch | matrix | dracula | gruvbox]");
        }
    }
}
