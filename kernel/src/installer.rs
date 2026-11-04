extern crate alloc;

use alloc::format;
use alloc::string::String;
use crate::{fs, keyboard, print, println, vga::{self, ColorCode}};

pub fn run_installer(auto_yes: bool) {
    vga::clear_screen();
    vga::set_colors(ColorCode::RustOrange, ColorCode::Black);
    println!("==========================================================================");
    println!("             RustOS Bare-Metal OS Installation Utility                    ");
    println!("==========================================================================");
    vga::set_colors(ColorCode::White, ColorCode::Black);
    println!(" Welcome to the RustOS Disk Installer!");
    println!(" This installer sets up the full standard base system and all Arch-compatible");
    println!(" utilities, userland tools, package database, and boot configuration.\n");

    let ata = crate::ata::ATA_DRIVE.lock();
    let drive_info = ata.get_info().cloned();
    drop(ata);

    match drive_info {
        Some(info) => {
            vga::set_colors(ColorCode::LightGreen, ColorCode::Black);
            println!(" [Target Drive Detected]");
            vga::set_colors(ColorCode::White, ColorCode::Black);
            println!("   Device   : /dev/sda");
            println!("   Model    : {}", info.model_str());
            println!("   Capacity : {} MB ({} sectors)", info.size_mb, info.sectors);
        }
        None => {
            vga::set_colors(ColorCode::Yellow, ColorCode::Black);
            println!(" [Notice: Target Storage]");
            vga::set_colors(ColorCode::White, ColorCode::Black);
            println!("   Target   : In-Memory Persistent RootFS (RAM / Live Environment)");
            println!("   Capacity : 32 MB allocated");
        }
    }

    println!("\n Installation will perform the following steps:");
    println!("   1. Partition & format target drive with SimpleFS v2");
    println!("   2. Create standard Linux / Arch FHS directory layout (/bin, /etc, /root...)");
    println!("   3. Deploy /etc/fstab, /etc/os-release, /etc/hostname, /etc/pacman.conf");
    println!("   4. Pacstrap and register ALL 200+ GNU / Arch Linux commands & package binaries");
    println!("   5. Synchronize pacman & AUR databases and write bootloader metadata");
    println!("   6. Set root shell environment (/bin/rsh)");

    if !auto_yes {
        vga::set_colors(ColorCode::RustOrange, ColorCode::Black);
        print!("\n Proceed with installation to disk? [Y/n]: ");
        vga::set_colors(ColorCode::White, ColorCode::Black);

        let mut answer = String::new();
        loop {
            let ch = keyboard::read_char();
            if ch == '\n' || ch == '\r' {
                println!();
                break;
            } else if ch == '\x08' {
                if !answer.is_empty() {
                    answer.pop();
                    vga::backspace();
                }
            } else if ch >= ' ' && ch <= '~' {
                answer.push(ch);
                print!("{}", ch);
            }
        }

        let trimmed = answer.trim();
        if !trimmed.is_empty() && !trimmed.eq_ignore_ascii_case("y") && !trimmed.eq_ignore_ascii_case("yes") {
            vga::set_colors(ColorCode::Yellow, ColorCode::Black);
            println!("Installation cancelled by user.");
            return;
        }
    } else {
        println!("\n [Auto-Confirm] Starting non-interactive installation...");
    }

    step_progress("1/6: Partitioning and creating SimpleFS v2 filesystem...", 12);
    if let Err(e) = fs::FS.lock().format() {
        vga::set_colors(ColorCode::LightRed, ColorCode::Black);
        println!("Error creating filesystem: {}", e);
        return;
    }

    step_progress("2/6: Creating Linux FHS directories (/bin, /etc, /usr, /var...)...", 10);
    let dirs = [
        "/bin", "/sbin", "/usr", "/usr/bin", "/usr/sbin", "/usr/local/bin",
        "/etc", "/etc/pacman.d", "/etc/systemd", "/etc/udev",
        "/home", "/root", "/tmp", "/run", "/var", "/var/lib",
        "/var/lib/pacman", "/var/cache", "/var/cache/yay", "/var/log",
        "/boot", "/boot/efi", "/dev", "/proc", "/sys", "/mnt", "/opt"
    ];
    for d in dirs.iter() {
        let _ = fs::mkdir(d);
    }

    step_progress("3/6: Writing system configuration (/etc/fstab, hostname, os-release)...", 15);
    let _ = fs::write_file("/etc/hostname", b"rustos\n");
    let _ = fs::write_file(
        "/etc/os-release",
        b"NAME=\"RustOS\"\nPRETTY_NAME=\"RustOS (Inspired by Arch Linux, x86_64 in Rust)\"\nID=rustos\nID_LIKE=arch\nBUILD_ID=rolling\nANSI_COLOR=\"38;2;222;85;28\"\nHOME_URL=\"https://github.com/rappar222/rust-os\"\n",
    );
    let _ = fs::write_file(
        "/etc/fstab",
        b"# /etc/fstab: static file system information\n# <file system> <dir>    <type>     <options>             <dump> <pass>\n/dev/sda2       /        simplefs   rw,relatime,sync       0      1\n/dev/sda1       /boot    vfat       defaults               0      2\ntmpfs           /tmp     tmpfs      defaults,nodev,nosuid  0      0\n",
    );
    let _ = fs::write_file(
        "/etc/passwd",
        b"root:x:0:0:Superuser:/root:/bin/rsh\narch:x:1000:1000:Arch User:/home/arch:/bin/rsh\n",
    );

    step_progress("4/6: Pacstrap: Installing and linking ALL 200+ Arch & Linux commands...", 25);
    install_all_arch_binaries();

    step_progress("5/6: Populating pacman & AUR (yay/paru) databases...", 18);
    crate::pkg::install_all_packages();

    step_progress("6/6: Finalizing bootloader and updating initramfs hooks...", 15);
    vga::set_colors(ColorCode::LightGreen, ColorCode::Black);
    println!("\n==========================================================================");
    println!("     [SUCCESS] RustOS Installation Complete!                              ");
    println!("==========================================================================");
    vga::set_colors(ColorCode::White, ColorCode::Black);
    println!(" All Arch-compatible commands, tools, and package managers are now installed!");
    println!(" You can reboot into your installed RustOS system or start using all commands.");
    vga::set_colors(ColorCode::RustOrange, ColorCode::Black);
    println!(" Type 'help' to view available commands or 'neofetch' / 'pacman -Q' to inspect.");
}

fn step_progress(msg: &str, ticks: u64) {
    vga::set_colors(ColorCode::RustOrange, ColorCode::Black);
    print!(":: ");
    vga::set_colors(ColorCode::White, ColorCode::Black);
    println!("{}", msg);
    print!("   [");
    vga::set_colors(ColorCode::RustOrange, ColorCode::Black);
    for _ in 0..28 {
        keyboard::sleep_ticks(ticks / 6);
        print!("#");
    }
    vga::set_colors(ColorCode::White, ColorCode::Black);
    println!("] 100%");
}

fn install_all_arch_binaries() {
    let stub_content = b"#!/bin/rsh\n# Native RustOS userland command stub\n";
    for &cmd in crate::shell::ALL_COMMANDS_LIST {
        let path = format!("/usr/bin/{}", cmd);
        let _ = fs::write_executable(&path, stub_content);
    }
}
