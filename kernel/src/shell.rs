extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::format;
use crate::{print, println, vga::{self, ColorCode}};
use crate::keyboard::read_char;
use crate::interrupts::uptime_seconds;
use crate::allocator::get_heap_info;

const BANNER: &str = r#"
  ____            _    ___  ____  
 |  _ \ _   _ ___| |_ / _ \/ ___| 
 | |_) | | | / __| __| | | \___ \ 
 |  _ <| |_| \__ \ |_| |_| |___) |
 |_| \_\\__,_|___/\__|\___/|____/ 
"#;

pub fn print_prompt() {
    vga::set_colors(ColorCode::LightGreen, ColorCode::Black);
    print!("rustos");
    vga::set_colors(ColorCode::White, ColorCode::Black);
    print!("> ");
    vga::set_colors(ColorCode::LightCyan, ColorCode::Black);
}

pub fn run_shell() -> ! {
    vga::set_colors(ColorCode::LightCyan, ColorCode::Black);
    println!("{}", BANNER);
    vga::set_colors(ColorCode::Yellow, ColorCode::Black);
    println!(" Welcome to RustOS v0.1.0 (x86_64 Bare Metal Terminal)");
    vga::set_colors(ColorCode::LightGray, ColorCode::Black);
    println!(" Type 'help' to see available commands.\n");

    let mut input_buffer = String::new();
    print_prompt();

    loop {
        let c = read_char();
        match c {
            '\n' => {
                println!();
                let cmd_line = input_buffer.trim();
                if !cmd_line.is_empty() {
                    execute_command(cmd_line);
                }
                input_buffer.clear();
                print_prompt();
            }
            '\x08' => {
                if !input_buffer.is_empty() {
                    input_buffer.pop();
                    vga::backspace();
                }
            }
            '\t' => {
                // Autocomplete could be added, or 4 spaces
            }
            printable => {
                if printable >= ' ' && printable <= '~' {
                    input_buffer.push(printable);
                    print!("{}", printable);
                }
            }
        }
    }
}

fn execute_command(cmd_line: &str) {
    let mut parts = cmd_line.split_whitespace();
    let command = match parts.next() {
        Some(cmd) => cmd,
        None => return,
    };
    let args: Vec<&str> = parts.collect();

    match command {
        "help" => cmd_help(),
        "clear" => cmd_clear(),
        "echo" => cmd_echo(&args),
        "info" => cmd_info(),
        "uptime" => cmd_uptime(),
        "colors" => cmd_colors(),
        "mem" => cmd_mem(),
        "alloc" => cmd_alloc(),
        "calc" => cmd_calc(&args),
        "disk" => cmd_disk(),
        "ls" | "dir" => cmd_ls(),
        "cat" => cmd_cat(&args),
        "write" => cmd_write(&args),
        "rm" => cmd_rm(&args),
        "format" => cmd_format(),
        "panic" => {
            vga::set_colors(ColorCode::LightRed, ColorCode::Black);
            panic!("Intentional panic requested via shell command.");
        }
        "reboot" => {
            crate::power::reboot();
        }
        "shutdown" | "exit" | "poweroff" => {
            crate::power::shutdown();
        }
        unknown => {
            vga::set_colors(ColorCode::LightRed, ColorCode::Black);
            println!("Unknown command: '{}'. Type 'help' for available commands.", unknown);
            vga::set_colors(ColorCode::LightCyan, ColorCode::Black);
        }
    }
}

fn cmd_help() {
    vga::set_colors(ColorCode::Yellow, ColorCode::Black);
    println!("Available RustOS Commands:");
    vga::set_colors(ColorCode::White, ColorCode::Black);

    let commands = [
        ("help", "Display this list of commands"),
        ("clear", "Clear the terminal screen"),
        ("echo <text>", "Print text to standard output"),
        ("info", "Display system and kernel information"),
        ("uptime", "Display system uptime and PIT timer ticks"),
        ("colors", "Display 16-color VGA terminal palette"),
        ("mem", "Display kernel heap and memory status"),
        ("alloc", "Test dynamic heap memory allocation (Vec, Box, String)"),
        ("disk", "Display ATA hard drive hardware info & capacity"),
        ("ls", "List files stored in persistent disk storage"),
        ("cat <file>", "Read and display file contents from disk"),
        ("write <file> <text>", "Write text into a file on disk"),
        ("rm <file>", "Delete a file from disk"),
        ("format", "Reformat the disk filesystem"),
        ("calc <expr>", "Evaluate basic arithmetic expressions (e.g. calc 15 * 4 + 2)"),
        ("panic", "Trigger a kernel panic to test crash handling"),
        ("reboot", "Reboot the machine"),
        ("shutdown", "Power off the machine"),
    ];

    for (name, desc) in commands.iter() {
        vga::set_colors(ColorCode::LightGreen, ColorCode::Black);
        print!("  {:14}", name);
        vga::set_colors(ColorCode::LightGray, ColorCode::Black);
        println!("- {}", desc);
    }
    vga::set_colors(ColorCode::LightCyan, ColorCode::Black);
}

fn cmd_clear() {
    vga::clear_screen();
}

fn cmd_echo(args: &[&str]) {
    vga::set_colors(ColorCode::White, ColorCode::Black);
    let output = args.join(" ");
    println!("{}", output);
    vga::set_colors(ColorCode::LightCyan, ColorCode::Black);
}

fn cmd_info() {
    vga::set_colors(ColorCode::LightGreen, ColorCode::Black);
    println!("--------------------------------------------------");
    println!(" RustOS - Modern Bare-Metal x86_64 Operating System");
    println!("--------------------------------------------------");
    vga::set_colors(ColorCode::White, ColorCode::Black);
    println!(" Architecture : x86_64 (64-bit Long Mode)");
    println!(" Kernel Model : Monolithic / Freestanding no_std");
    println!(" CPU Paging   : 4-Level Paging Enabled");
    println!(" Interrupts   : IDT + 8259 Dual PIC (IRQ 0-15)");
    println!(" Drivers      : VGA Framebuffer, PS/2 Keyboard, UART 16550 Serial");
    println!(" Compiler     : rustc 1.101.0-nightly");
    vga::set_colors(ColorCode::LightCyan, ColorCode::Black);
}

fn cmd_uptime() {
    let secs = uptime_seconds();
    let ticks = crate::interrupts::ticks();
    let minutes = secs / 60;
    let remaining_secs = secs % 60;
    let hours = minutes / 60;
    let remaining_mins = minutes % 60;

    vga::set_colors(ColorCode::White, ColorCode::Black);
    println!("Uptime: {:02}h {:02}m {:02}s ({} timer ticks @ 100Hz)", hours, remaining_mins, remaining_secs, ticks);
    vga::set_colors(ColorCode::LightCyan, ColorCode::Black);
}

fn cmd_colors() {
    println!("16-Color Terminal Palette:");
    let colors = [
        (ColorCode::Black, "Black"),
        (ColorCode::Blue, "Blue"),
        (ColorCode::Green, "Green"),
        (ColorCode::Cyan, "Cyan"),
        (ColorCode::Red, "Red"),
        (ColorCode::Magenta, "Magenta"),
        (ColorCode::Brown, "Brown"),
        (ColorCode::LightGray, "LightGray"),
        (ColorCode::DarkGray, "DarkGray"),
        (ColorCode::LightBlue, "LightBlue"),
        (ColorCode::LightGreen, "LightGreen"),
        (ColorCode::LightCyan, "LightCyan"),
        (ColorCode::LightRed, "LightRed"),
        (ColorCode::LightMagenta, "LightMagenta"),
        (ColorCode::Yellow, "Yellow"),
        (ColorCode::White, "White"),
    ];

    for (code, name) in colors.iter() {
        vga::set_colors(*code, ColorCode::Black);
        print!("  [■■■] ");
        vga::set_colors(ColorCode::White, ColorCode::Black);
        println!("{:12} (code {})", name, *code as u8);
    }
    vga::set_colors(ColorCode::LightCyan, ColorCode::Black);
}

fn cmd_mem() {
    let (start, size, initialized) = get_heap_info();
    vga::set_colors(ColorCode::White, ColorCode::Black);
    println!("Kernel Heap Status:");
    println!("  Initialized : {}", initialized);
    println!("  Base Address: {:#018x}", start);
    println!("  Total Size  : {} KB ({} MB)", size / 1024, size / (1024 * 1024));
    vga::set_colors(ColorCode::LightCyan, ColorCode::Black);
}

fn cmd_alloc() {
    vga::set_colors(ColorCode::LightGreen, ColorCode::Black);
    println!("Testing dynamic kernel memory allocations:");

    // 1. Box test
    let heap_value = Box::new(0xCAFE_BABE_u64);
    println!("  1. Box allocation: value = {:#x} at {:p}", *heap_value, heap_value);

    // 2. Vector test
    let mut vec = Vec::new();
    for i in 0..10 {
        vec.push(i * i);
    }
    println!("  2. Vec allocation: 10 squares = {:?}", vec);

    // 3. String formatting test
    let test_str = format!("RustOS dynamic string allocation ({})", 2026);
    println!("  3. String format: \"{}\"", test_str);

    vga::set_colors(ColorCode::Yellow, ColorCode::Black);
    println!("All allocations and deallocations succeeded!");
    vga::set_colors(ColorCode::LightCyan, ColorCode::Black);
}

fn cmd_calc(args: &[&str]) {
    if args.is_empty() {
        vga::set_colors(ColorCode::LightRed, ColorCode::Black);
        println!("Usage: calc <number> <operator> <number> ...");
        println!("Example: calc 42 * 10 + 5");
        vga::set_colors(ColorCode::LightCyan, ColorCode::Black);
        return;
    }

    let full_expr = args.join(" ");
    match simple_eval(&full_expr) {
        Ok(result) => {
            vga::set_colors(ColorCode::LightGreen, ColorCode::Black);
            println!("Result: {}", result);
        }
        Err(err) => {
            vga::set_colors(ColorCode::LightRed, ColorCode::Black);
            println!("Error: {}", err);
        }
    }
    vga::set_colors(ColorCode::LightCyan, ColorCode::Black);
}

fn simple_eval(expr: &str) -> Result<i64, &'static str> {
    let tokens: Vec<&str> = expr.split_whitespace().collect();
    if tokens.is_empty() {
        return Err("Empty expression");
    }

    let mut current_val: i64 = tokens[0].parse().map_err(|_| "Invalid number")?;
    let mut i = 1;

    while i < tokens.len() {
        let op = tokens[i];
        if i + 1 >= tokens.len() {
            return Err("Missing operand after operator");
        }
        let next_val: i64 = tokens[i + 1].parse().map_err(|_| "Invalid number")?;

        match op {
            "+" => current_val = current_val.checked_add(next_val).ok_or("Addition overflow")?,
            "-" => current_val = current_val.checked_sub(next_val).ok_or("Subtraction underflow")?,
            "*" | "x" => current_val = current_val.checked_mul(next_val).ok_or("Multiplication overflow")?,
            "/" => {
                if next_val == 0 {
                    return Err("Division by zero");
                }
                current_val /= next_val;
            }
            "%" => {
                if next_val == 0 {
                    return Err("Modulo by zero");
                }
                current_val %= next_val;
            }
            _ => return Err("Unsupported operator. Use +, -, *, /, %"),
        }
        i += 2;
    }

    Ok(current_val)
}

fn cmd_disk() {
    vga::set_colors(ColorCode::White, ColorCode::Black);
    let ata = crate::ata::ATA_DRIVE.lock();
    if let Some(info) = ata.get_info() {
        println!("ATA Hard Drive Information:");
        println!("  Model    : {}", info.model_str());
        println!("  Capacity : {} MB ({} GB)", info.size_mb, info.size_mb / 1024);
        println!("  Sectors  : {} (512 bytes/sector)", info.sectors);
    } else {
        vga::set_colors(ColorCode::LightRed, ColorCode::Black);
        println!("No ATA hard drive detected or initialized.");
    }
    vga::set_colors(ColorCode::LightCyan, ColorCode::Black);
}

fn cmd_ls() {
    vga::set_colors(ColorCode::Yellow, ColorCode::Black);
    println!("Files on Persistent Storage (SimpleFS):");
    let files = crate::fs::list_files();
    if files.is_empty() {
        vga::set_colors(ColorCode::LightGray, ColorCode::Black);
        println!("  (No files found. Use 'write <filename> <text>' to create one)");
    } else {
        for (name, size) in files {
            vga::set_colors(ColorCode::LightGreen, ColorCode::Black);
            print!("  {:20}", name);
            vga::set_colors(ColorCode::LightGray, ColorCode::Black);
            println!("{} bytes", size);
        }
    }
    vga::set_colors(ColorCode::LightCyan, ColorCode::Black);
}

fn cmd_cat(args: &[&str]) {
    if args.is_empty() {
        vga::set_colors(ColorCode::LightRed, ColorCode::Black);
        println!("Usage: cat <filename>");
        vga::set_colors(ColorCode::LightCyan, ColorCode::Black);
        return;
    }

    let filename = args[0];
    match crate::fs::read_file(filename) {
        Ok(data) => {
            vga::set_colors(ColorCode::White, ColorCode::Black);
            if let Ok(text) = core::str::from_utf8(&data) {
                print!("{}", text);
                if !text.ends_with('\n') {
                    println!();
                }
            } else {
                println!("[Binary data: {} bytes]", data.len());
            }
        }
        Err(err) => {
            vga::set_colors(ColorCode::LightRed, ColorCode::Black);
            println!("Error: {}", err);
        }
    }
    vga::set_colors(ColorCode::LightCyan, ColorCode::Black);
}

fn cmd_write(args: &[&str]) {
    if args.len() < 2 {
        vga::set_colors(ColorCode::LightRed, ColorCode::Black);
        println!("Usage: write <filename> <text content>");
        vga::set_colors(ColorCode::LightCyan, ColorCode::Black);
        return;
    }

    let filename = args[0];
    let content = args[1..].join(" ");
    match crate::fs::write_file(filename, content.as_bytes()) {
        Ok(()) => {
            vga::set_colors(ColorCode::LightGreen, ColorCode::Black);
            println!("File '{}' saved to disk ({} bytes).", filename, content.len());
        }
        Err(err) => {
            vga::set_colors(ColorCode::LightRed, ColorCode::Black);
            println!("Error writing file: {}", err);
        }
    }
    vga::set_colors(ColorCode::LightCyan, ColorCode::Black);
}

fn cmd_rm(args: &[&str]) {
    if args.is_empty() {
        vga::set_colors(ColorCode::LightRed, ColorCode::Black);
        println!("Usage: rm <filename>");
        vga::set_colors(ColorCode::LightCyan, ColorCode::Black);
        return;
    }

    let filename = args[0];
    match crate::fs::delete_file(filename) {
        Ok(()) => {
            vga::set_colors(ColorCode::LightGreen, ColorCode::Black);
            println!("File '{}' deleted from disk.", filename);
        }
        Err(err) => {
            vga::set_colors(ColorCode::LightRed, ColorCode::Black);
            println!("Error: {}", err);
        }
    }
    vga::set_colors(ColorCode::LightCyan, ColorCode::Black);
}

fn cmd_format() {
    vga::set_colors(ColorCode::Yellow, ColorCode::Black);
    println!("Formatting SimpleFS on disk...");
    match crate::fs::FS.lock().format() {
        Ok(()) => {
            vga::set_colors(ColorCode::LightGreen, ColorCode::Black);
            println!("Filesystem formatted successfully.");
        }
        Err(err) => {
            vga::set_colors(ColorCode::LightRed, ColorCode::Black);
            println!("Format error: {}", err);
        }
    }
    vga::set_colors(ColorCode::LightCyan, ColorCode::Black);
}
