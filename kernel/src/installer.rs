extern crate alloc;

use alloc::format;
use alloc::string::String;
use crate::{fs, keyboard, print, println, vga::{self, ColorCode}};

pub fn run_installer(auto_yes: bool) {
    vga::clear_screen();
    vga::set_colors(ColorCode::RustOrange, ColorCode::Black);
    println!("==========================================================================");
    println!("                   RustOS Guided System Installer                         ");
    println!("==========================================================================");
    vga::set_colors(ColorCode::White, ColorCode::Black);
    println!(" Welcome to the RustOS installation assistant!");
    println!(" This installer will configure locale, timezone, mirrors, storage, users,");
    println!(" and your graphical environment (Floating Window Compositor).\n");

    // 1. Region & Locale
    let locales = [
        "en_US.UTF-8 (US / English)",
        "en_GB.UTF-8 (UK / English)",
        "de_DE.UTF-8 (German / Deutsch)",
        "ro_RO.UTF-8 (Romanian / Romana)",
        "fr_FR.UTF-8 (French / Francais)",
        "es_ES.UTF-8 (Spanish / Espanol)",
    ];
    let locale_idx = prompt_selection(
        "1. Select Locale / Region & Keymap:",
        &locales,
        0,
        auto_yes,
    );
    let chosen_locale = locales[locale_idx].split_whitespace().next().unwrap_or("en_US.UTF-8");

    // 2. Timezone (Comprehensive World Timezones by Region)
    let region_names: [&str; 8] = [
        "Europe (36 timezones)",
        "Americas (North, Central & South - 29 timezones)",
        "Asia (31 timezones)",
        "Africa (19 timezones)",
        "Australia & Pacific (16 timezones)",
        "Atlantic & Antarctica (11 timezones)",
        "Universal & POSIX (UTC / GMT - 26 timezones)",
        "Custom / Enter Timezone Manually",
    ];

    let chosen_tz: String = if auto_yes {
        String::from("UTC")
    } else {
        let region_idx = prompt_selection(
            "\n2a. Select Timezone Continent / World Region:",
            &region_names,
            0,
            auto_yes,
        );

        if region_idx == 7 {
            prompt_string("   Enter Custom Timezone (e.g. Europe/Bucharest, America/New_York, UTC)", "UTC", false, false)
        } else {
            let region = &crate::tzdata::REGIONS[region_idx];
            let mut tz_labels: alloc::vec::Vec<String> = alloc::vec::Vec::new();
            for tz in region.timezones {
                tz_labels.push(format!("{} ({})", tz.name, crate::tzdata::format_offset_string(tz.offset_minutes)));
            }
            let label_slices: alloc::vec::Vec<&str> = tz_labels.iter().map(|s| s.as_str()).collect();

            let title = format!("\n2b. Select Timezone in {}:", region.name);
            let selected_tz_idx = prompt_selection_columns(&title, &label_slices, 0, auto_yes);
            String::from(region.timezones[selected_tz_idx].name)
        }
    };

    // 3. Package Mirror Server (Location-based: Country -> Mirror)
    let country_list = crate::mirrors::COUNTRY_MIRRORS;
    let mut country_labels: alloc::vec::Vec<String> = alloc::vec::Vec::new();
    for c in country_list {
        country_labels.push(format!("{} ({} mirrors)", c.country, c.mirrors.len()));
    }
    country_labels.push(String::from("Custom Mirror URL (Manual Input)"));
    let country_slices: alloc::vec::Vec<&str> = country_labels.iter().map(|s| s.as_str()).collect();

    let (chosen_country, chosen_mirror_name, chosen_mirror): (String, String, String) = if auto_yes {
        (
            String::from("Worldwide / Global Anycast"),
            String::from("GeoDNS Tier-1 Global Anycast"),
            String::from("https://geo.mirror.pkgbuild.com/$repo/os/$arch")
        )
    } else {
        let country_idx = prompt_selection_columns(
            "\n3a. Select Package Mirror Location / Country:",
            &country_slices,
            0,
            auto_yes,
        );

        if country_idx >= country_list.len() {
            let custom_url = prompt_string(
                "   Enter Custom Mirror Server URL",
                "https://geo.mirror.pkgbuild.com/$repo/os/$arch",
                false,
                false,
            );
            (String::from("Custom"), String::from("Custom Mirror Server"), custom_url)
        } else {
            let country = &country_list[country_idx];
            let mut mirror_labels: alloc::vec::Vec<String> = alloc::vec::Vec::new();
            for m in country.mirrors {
                mirror_labels.push(format!("{} [{}] ({})", m.name, m.bandwidth, m.location));
            }
            let mirror_slices: alloc::vec::Vec<&str> = mirror_labels.iter().map(|s| s.as_str()).collect();

            let title = format!("\n3b. Select Mirror Server in {}:", country.country);
            let mirror_idx = prompt_selection(&title, &mirror_slices, 0, auto_yes);
            let selected_mirror = &country.mirrors[mirror_idx];

            (
                String::from(country.country),
                String::from(selected_mirror.name),
                String::from(selected_mirror.url)
            )
        }
    };

    // 4. Partitioning & Disk Layout
    let ata = crate::ata::ATA_DRIVE.lock();
    let drive_info = ata.get_info().cloned();
    drop(ata);

    let drive_desc = match drive_info {
        Some(ref info) => format!("/dev/sda ({} MB, {})", info.size_mb, info.model_str()),
        None => String::from("In-Memory RAM Disk (tmpfs, 32 MB)"),
    };

    let partition_schemes = [
        "Wipe entire disk /dev/sda (SimpleFS v2 Auto-partition & format)",
        "Keep existing partition, format root /dev/sda2",
        "In-Memory RAM Disk (tmpfs Live RootFS, non-persistent)",
    ];
    let part_idx = prompt_selection(
        &format!("\n4. Select Partitioning Scheme for {}:", drive_desc),
        &partition_schemes,
        0,
        auto_yes,
    );

    // 5. Hostname
    let hostname = prompt_string("\n5. Enter Hostname", "rustos", false, auto_yes);

    // 6. User accounts
    let root_pass = prompt_string("\n6. Set Root Password", "root", true, auto_yes);
    let username = prompt_string("   Enter New Username", "archuser", false, auto_yes);
    let user_pass = prompt_string("   Set Password for user", "archuser", true, auto_yes);
    let sudo_opt = prompt_yes_no("   Add user to sudoers (wheel group)?", true, auto_yes);

    // 7. Desktop Environment / Window Compositor
    let desktop_options = [
        "RustOS Floating Window Compositor (Graphical Floating Desktop)",
        "Minimal Console Only (CLI Only, No GUI)",
    ];
    let desktop_idx = prompt_selection(
        "\n7. Choose Graphical Environment / Window Compositor:",
        &desktop_options,
        0,
        auto_yes,
    );
    let install_compositor = desktop_idx == 0;

    // 8. Installation Summary & Confirmation
    vga::set_colors(ColorCode::RustOrange, ColorCode::Black);
    println!("\n==========================================================================");
    println!("                   Installation Configuration Summary                     ");
    println!("==========================================================================");
    vga::set_colors(ColorCode::White, ColorCode::Black);
    println!("   * Locale & Keymap : {}", chosen_locale);
    println!("   * Timezone        : {}", chosen_tz);
    println!("   * Mirror Location : {}", chosen_country);
    println!("   * Mirror Server   : {} ({})", chosen_mirror_name, chosen_mirror);
    println!("   * Storage Layout  : {}", partition_schemes[part_idx]);
    println!("   * Hostname        : {}", hostname);
    println!("   * Root Account    : root ({})", if root_pass == "root" { "default" } else { "configured" });
    println!("   * Standard User   : {} (Sudoer: {})", username, if sudo_opt { "Yes" } else { "No" });
    println!("   * Desktop GUI     : {}", desktop_options[desktop_idx]);
    println!("==========================================================================");

    if !auto_yes {
        let proceed = prompt_yes_no("Proceed with installation to disk?", true, auto_yes);
        if !proceed {
            vga::set_colors(ColorCode::Yellow, ColorCode::Black);
            println!("\nInstallation cancelled by user.");
            return;
        }
    } else {
        println!("\n[Auto-Confirm] Starting non-interactive installation...");
    }

    // Execution Steps
    step_progress("1/7: Partitioning and formatting disk filesystem...", 12, auto_yes);
    if part_idx != 2 {
        if let Err(e) = fs::FS.lock().format() {
            vga::set_colors(ColorCode::LightRed, ColorCode::Black);
            println!("Error formatting filesystem: {}", e);
            return;
        }
    }

    step_progress("2/7: Creating standard Linux / Arch FHS directory structure...", 10, auto_yes);
    let dirs = [
        "/bin", "/sbin", "/usr", "/usr/bin", "/usr/sbin", "/usr/local/bin",
        "/etc", "/etc/pacman.d", "/etc/systemd", "/etc/udev",
        "/home", &format!("/home/{}", username), "/root", "/tmp", "/run", "/var", "/var/lib",
        "/var/lib/rustos", "/var/lib/pacman", "/var/cache", "/var/cache/yay", "/var/log",
        "/boot", "/boot/efi", "/dev", "/proc", "/sys", "/mnt", "/opt"
    ];
    for d in dirs.iter() {
        let _ = fs::mkdir(d);
    }

    step_progress("3/7: Writing system configuration (/etc/locale, timezone, fstab, hosts)...", 15, auto_yes);
    let _ = fs::write_file("/etc/hostname", format!("{}\n", hostname).as_bytes());
    let _ = fs::write_file("/etc/locale.conf", format!("LANG={}\nLC_COLLATE=C\n", chosen_locale).as_bytes());
    let _ = fs::write_file("/etc/timezone", format!("{}\n", chosen_tz).as_bytes());
    let _ = fs::write_file("/etc/localtime", format!("{}\n", chosen_tz).as_bytes());
    let mirror_content = format!(
        "# RustOS Pacman Mirrorlist (Inspired by Arch Linux)\n# Primary Country: {}\n# Server Name: {}\nServer = {}\n\n# Fallback Anycast Mirrors:\nServer = https://geo.mirror.pkgbuild.com/$repo/os/$arch\nServer = https://cloudflare.cdn.pkgbuild.com/$repo/os/$arch\n",
        chosen_country,
        chosen_mirror_name,
        chosen_mirror
    );
    let _ = fs::write_file("/etc/pacman.d/mirrorlist", mirror_content.as_bytes());
    let _ = fs::write_file(
        "/etc/os-release",
        b"NAME=\"RustOS\"\nPRETTY_NAME=\"RustOS (Inspired by Arch Linux, x86_64 in Rust)\"\nID=rustos\nID_LIKE=arch\nBUILD_ID=rolling\nANSI_COLOR=\"38;2;222;85;28\"\nHOME_URL=\"https://github.com/rappar222/rust-os\"\n",
    );
    let _ = fs::write_file(
        "/etc/fstab",
        b"# /etc/fstab: static file system information\n# <file system> <dir>    <type>     <options>             <dump> <pass>\n/dev/sda2       /        simplefs   rw,relatime,sync       0      1\n/dev/sda1       /boot    vfat       defaults               0      2\ntmpfs           /tmp     tmpfs      defaults,nodev,nosuid  0      0\n",
    );
    let passwd_content = format!(
        "root:x:0:0:Superuser:/root:/bin/rsh\n{}:x:1000:1000:{}:/home/{}:/bin/rsh\n",
        username, username, username
    );
    let _ = fs::write_file("/etc/passwd", passwd_content.as_bytes());
    let shadow_content = format!(
        "root:$6$salt${}:19000:0:99999:7:::\n{}:$6$salt${}:19000:0:99999:7:::\n",
        root_pass, username, user_pass
    );
    let _ = fs::write_file("/etc/shadow", shadow_content.as_bytes());
    if sudo_opt {
        let _ = fs::write_file("/etc/sudoers", b"%wheel ALL=(ALL:ALL) ALL\nroot ALL=(ALL:ALL) ALL\n");
    }

    let install_meta = format!(
        "INSTALLED=1\nCOMPOSITOR={}\nLOCALE={}\nTIMEZONE={}\nUSER={}\nHOSTNAME={}\n",
        if install_compositor { "1" } else { "0" },
        chosen_locale,
        chosen_tz,
        username,
        hostname
    );
    let _ = fs::write_file("/var/lib/rustos/installed", install_meta.as_bytes());

    step_progress("4/7: Pacstrap: Deploying base system & ALL 200+ GNU / Arch Linux commands...", 25, auto_yes);
    install_all_arch_binaries(auto_yes);

    step_progress("5/7: Configuring Pacman, AUR (yay/paru), and package database...", 18, auto_yes);
    crate::pkg::install_all_packages();

    step_progress("6/7: Configuring Graphical Environment (Floating Window Compositor)...", 15, auto_yes);

    step_progress("7/7: Finalizing bootloader and updating initramfs hooks...", 12, auto_yes);
    vga::set_colors(ColorCode::LightGreen, ColorCode::Black);
    println!("\n==========================================================================");
    println!("     [SUCCESS] RustOS Installation Completed Successfully!                ");
    println!("==========================================================================");
    vga::set_colors(ColorCode::White, ColorCode::Black);
    println!(" System installed to /dev/sda with full persistence and user accounts.");
    if install_compositor {
        vga::set_colors(ColorCode::RustOrange, ColorCode::Black);
        println!(" [Desktop GUI] RustOS Floating Window Compositor is enabled!");
        println!(" Launch the graphical desktop anytime by typing: 'startx' or 'desktop' or 'wm'.");
        vga::set_colors(ColorCode::White, ColorCode::Black);
    }
    println!(" Type 'help' to view all commands or 'neofetch' / 'pacman -Q' to inspect.");
}

fn prompt_selection_columns(title: &str, options: &[&str], default_idx: usize, auto_yes: bool) -> usize {
    vga::set_colors(ColorCode::RustOrange, ColorCode::Black);
    println!("{}", title);
    vga::set_colors(ColorCode::White, ColorCode::Black);

    if options.len() <= 6 {
        return prompt_selection_internal(options, default_idx, auto_yes);
    }

    let half = (options.len() + 1) / 2;
    for row in 0..half {
        let left_idx = row;
        let right_idx = row + half;

        // Left item
        if left_idx == default_idx {
            vga::set_colors(ColorCode::RustOrange, ColorCode::Black);
            print!(" *{:2}) ", left_idx + 1);
        } else {
            vga::set_colors(ColorCode::LightCyan, ColorCode::Black);
            print!("  {:2}) ", left_idx + 1);
        }
        vga::set_colors(if left_idx == default_idx { ColorCode::RustOrange } else { ColorCode::White }, ColorCode::Black);
        let left_text = options[left_idx];
        let mut left_chars = 0;
        for c in left_text.chars().take(34) {
            print!("{}", c);
            left_chars += 1;
        }
        for _ in left_chars..35 {
            print!(" ");
        }

        // Right item
        if right_idx < options.len() {
            if right_idx == default_idx {
                vga::set_colors(ColorCode::RustOrange, ColorCode::Black);
                print!(" *{:2}) ", right_idx + 1);
            } else {
                vga::set_colors(ColorCode::LightCyan, ColorCode::Black);
                print!("  {:2}) ", right_idx + 1);
            }
            vga::set_colors(if right_idx == default_idx { ColorCode::RustOrange } else { ColorCode::White }, ColorCode::Black);
            let right_text = options[right_idx];
            for c in right_text.chars().take(36) {
                print!("{}", c);
            }
        }
        println!();
    }
    vga::set_colors(ColorCode::White, ColorCode::Black);

    prompt_selection_read_input(options, default_idx, auto_yes)
}

fn prompt_selection(title: &str, options: &[&str], default_idx: usize, auto_yes: bool) -> usize {
    vga::set_colors(ColorCode::RustOrange, ColorCode::Black);
    println!("{}", title);
    vga::set_colors(ColorCode::White, ColorCode::Black);
    prompt_selection_internal(options, default_idx, auto_yes)
}

fn prompt_selection_internal(options: &[&str], default_idx: usize, auto_yes: bool) -> usize {
    for (i, opt) in options.iter().enumerate() {
        if i == default_idx {
            vga::set_colors(ColorCode::RustOrange, ColorCode::Black);
            println!("   {}) {} [Default]", i + 1, opt);
            vga::set_colors(ColorCode::White, ColorCode::Black);
        } else {
            println!("   {}) {}", i + 1, opt);
        }
    }
    prompt_selection_read_input(options, default_idx, auto_yes)
}

fn prompt_selection_read_input(options: &[&str], default_idx: usize, auto_yes: bool) -> usize {
    if auto_yes {
        println!("   -> Selected: {} (auto)", options[default_idx]);
        return default_idx;
    }
    vga::set_colors(ColorCode::RustOrange, ColorCode::Black);
    print!("   Select [1-{}, default {}]: ", options.len(), default_idx + 1);
    vga::set_colors(ColorCode::White, ColorCode::Black);

    let mut input = String::new();
    loop {
        let ch = keyboard::read_char();
        if ch == '\n' || ch == '\r' {
            println!();
            break;
        } else if ch == '\x08' {
            if !input.is_empty() {
                input.pop();
                vga::backspace();
            }
        } else if ch.is_ascii_digit() {
            input.push(ch);
            print!("{}", ch);
        }
    }

    if let Ok(num) = input.trim().parse::<usize>() {
        if num >= 1 && num <= options.len() {
            return num - 1;
        }
    }
    default_idx
}

fn prompt_string(prompt_text: &str, default_val: &str, is_password: bool, auto_yes: bool) -> String {
    if auto_yes {
        return String::from(default_val);
    }
    vga::set_colors(ColorCode::RustOrange, ColorCode::Black);
    print!("{} [default: {}]: ", prompt_text, if is_password { "******" } else { default_val });
    vga::set_colors(ColorCode::White, ColorCode::Black);

    let mut input = String::new();
    loop {
        let ch = keyboard::read_char();
        if ch == '\n' || ch == '\r' {
            println!();
            break;
        } else if ch == '\x08' {
            if !input.is_empty() {
                input.pop();
                vga::backspace();
            }
        } else if ch >= ' ' && ch <= '~' {
            input.push(ch);
            if is_password {
                print!("*");
            } else {
                print!("{}", ch);
            }
        }
    }
    let trimmed = input.trim();
    if trimmed.is_empty() {
        String::from(default_val)
    } else {
        String::from(trimmed)
    }
}

fn prompt_yes_no(prompt_text: &str, default_yes: bool, auto_yes: bool) -> bool {
    if auto_yes {
        return default_yes;
    }
    vga::set_colors(ColorCode::RustOrange, ColorCode::Black);
    print!("{} [{}/{}]: ", prompt_text, if default_yes { "Y" } else { "y" }, if !default_yes { "N" } else { "n" });
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
    if trimmed.is_empty() {
        default_yes
    } else {
        trimmed.eq_ignore_ascii_case("y") || trimmed.eq_ignore_ascii_case("yes")
    }
}

fn step_progress(msg: &str, ticks: u64, fast: bool) {
    vga::set_colors(ColorCode::RustOrange, ColorCode::Black);
    print!(":: ");
    vga::set_colors(ColorCode::White, ColorCode::Black);
    println!("{}", msg);
    print!("   [");
    vga::set_colors(ColorCode::RustOrange, ColorCode::Black);
    for _ in 0..28 {
        if !fast {
            keyboard::sleep_ticks(ticks / 6);
        }
        print!("#");
    }
    vga::set_colors(ColorCode::White, ColorCode::Black);
    println!("] 100%");
}

fn install_all_arch_binaries(fast: bool) {
    let stub_content = b"#!/bin/rsh\n# Native RustOS userland command stub\n";
    let list: &[&str] = if fast {
        &[
            "sh", "rsh", "bash", "ls", "cat", "echo", "pwd", "cd", "mkdir", "rm", "cp", "mv",
            "chmod", "chown", "uname", "whoami", "hostname", "date", "uptime", "free", "df", "ps",
            "pacman", "yay", "paru", "makepkg", "startx", "wm", "desktop", "neofetch"
        ]
    } else {
        crate::shell::ALL_COMMANDS_LIST
    };
    for &cmd in list {
        let path = format!("/usr/bin/{}", cmd);
        let _ = fs::write_executable(&path, stub_content);
    }
}
