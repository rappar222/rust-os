extern crate alloc;

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use spin::Mutex;
use crate::{fs, keyboard, print, println, vga::{self, ColorCode}};

#[derive(Clone, Copy)]
pub struct PackageSpec {
    pub repo: &'static str,
    pub name: &'static str,
    pub binary: &'static str,
    pub version: &'static str,
    pub description: &'static str,
    pub url: &'static str,
    pub licenses: &'static str,
    pub depends: &'static [&'static str],
    pub dl_size_kb: u32,
    pub inst_size_kb: u32,
    pub aur_votes: u32,
    pub preinstalled: bool,
}

pub static REPO_CATALOG: &[PackageSpec] = &[
    // --- CORE REPOSITORY ---
    PackageSpec {
        repo: "core",
        name: "base",
        binary: "",
        version: "3-2",
        description: "Minimal package set to define a basic RustOS Arch installation",
        url: "https://archlinux.org",
        licenses: "GPL",
        depends: &["filesystem", "glibc-compat", "coreutils-rs", "rsh"],
        dl_size_kb: 4,
        inst_size_kb: 12,
        aur_votes: 0,
        preinstalled: true,
    },
    PackageSpec {
        repo: "core",
        name: "linux-rustos",
        binary: "",
        version: "0.2.0-1",
        description: "The RustOS x86_64 monolithic bare-metal kernel and modules",
        url: "https://github.com/rappar222/rust-os",
        licenses: "MIT",
        depends: &["base"],
        dl_size_kb: 3780,
        inst_size_kb: 5400,
        aur_votes: 0,
        preinstalled: true,
    },
    PackageSpec {
        repo: "core",
        name: "pacman",
        binary: "pacman",
        version: "6.1.0-3",
        description: "A library-based package manager with dependency support (Rust port)",
        url: "https://archlinux.org/pacman/",
        licenses: "GPL-2.0-or-later",
        depends: &["base"],
        dl_size_kb: 920,
        inst_size_kb: 4650,
        aur_votes: 0,
        preinstalled: true,
    },
    PackageSpec {
        repo: "core",
        name: "coreutils-rs",
        binary: "",
        version: "0.0.27-1",
        description: "Cross-platform Rust rewrite of the GNU coreutils",
        url: "https://github.com/uutils/coreutils",
        licenses: "MIT",
        depends: &["glibc-compat"],
        dl_size_kb: 2450,
        inst_size_kb: 8120,
        aur_votes: 0,
        preinstalled: true,
    },
    PackageSpec {
        repo: "core",
        name: "filesystem",
        binary: "",
        version: "2026.11-1",
        description: "Base RustOS Arch Linux filesystem hierarchy layout (SimpleFS)",
        url: "https://archlinux.org",
        licenses: "GPL",
        depends: &[],
        dl_size_kb: 12,
        inst_size_kb: 64,
        aur_votes: 0,
        preinstalled: true,
    },
    PackageSpec {
        repo: "core",
        name: "glibc-compat",
        binary: "",
        version: "2.40-1",
        description: "Rust no_std / alloc core runtime compatibility layer",
        url: "https://www.rust-lang.org",
        licenses: "MIT OR Apache-2.0",
        depends: &["filesystem"],
        dl_size_kb: 512,
        inst_size_kb: 1420,
        aur_votes: 0,
        preinstalled: true,
    },
    PackageSpec {
        repo: "core",
        name: "rsh",
        binary: "rsh",
        version: "1.2.0-1",
        description: "RustOS interactive shell with Tab completion and history",
        url: "https://github.com/rappar222/rust-os",
        licenses: "MIT",
        depends: &["coreutils-rs"],
        dl_size_kb: 310,
        inst_size_kb: 890,
        aur_votes: 0,
        preinstalled: true,
    },
    PackageSpec {
        repo: "core",
        name: "nano",
        binary: "nano",
        version: "8.2-1",
        description: "Pico editor clone with enhancements (interactive terminal editor)",
        url: "https://www.nano-editor.org",
        licenses: "GPL-3.0-or-later",
        depends: &["base"],
        dl_size_kb: 610,
        inst_size_kb: 2480,
        aur_votes: 0,
        preinstalled: true,
    },

    // --- EXTRA REPOSITORY ---
    PackageSpec {
        repo: "extra",
        name: "neofetch",
        binary: "neofetch",
        version: "7.1.0-2",
        description: "A CLI system information tool written in Rust for RustOS",
        url: "https://github.com/dylanaraps/neofetch",
        licenses: "MIT",
        depends: &["rsh"],
        dl_size_kb: 145,
        inst_size_kb: 380,
        aur_votes: 0,
        preinstalled: true,
    },
    PackageSpec {
        repo: "extra",
        name: "fastfetch",
        binary: "fastfetch",
        version: "2.29.0-1",
        description: "Like neofetch, but much faster because written in native code",
        url: "https://github.com/fastfetch-cli/fastfetch",
        licenses: "MIT",
        depends: &["base"],
        dl_size_kb: 420,
        inst_size_kb: 1290,
        aur_votes: 0,
        preinstalled: true,
    },
    PackageSpec {
        repo: "extra",
        name: "btop",
        binary: "btop",
        version: "1.4.0-1",
        description: "A monitor of system resources, processor, memory, disks and processes",
        url: "https://github.com/aristocratos/btop",
        licenses: "Apache-2.0",
        depends: &["base"],
        dl_size_kb: 540,
        inst_size_kb: 1750,
        aur_votes: 0,
        preinstalled: false,
    },
    PackageSpec {
        repo: "extra",
        name: "htop",
        binary: "htop",
        version: "3.3.0-2",
        description: "Interactive process viewer for Unix/Linux systems",
        url: "https://htop.dev/",
        licenses: "GPL-2.0-or-later",
        depends: &["base"],
        dl_size_kb: 320,
        inst_size_kb: 910,
        aur_votes: 0,
        preinstalled: false,
    },
    PackageSpec {
        repo: "extra",
        name: "cmatrix",
        binary: "cmatrix",
        version: "2.0-3",
        description: "Curses-based scrolling 'Matrix'-like digital rain screen",
        url: "https://github.com/abishekvashok/cmatrix",
        licenses: "GPL-3.0-or-later",
        depends: &["base"],
        dl_size_kb: 58,
        inst_size_kb: 164,
        aur_votes: 0,
        preinstalled: false,
    },
    PackageSpec {
        repo: "extra",
        name: "cowsay",
        binary: "cowsay",
        version: "3.8.4-1",
        description: "Configurable talking ASCII cow (and a bit more)",
        url: "https://github.com/cowsay-org/cowsay",
        licenses: "GPL-3.0-only",
        depends: &["base"],
        dl_size_kb: 44,
        inst_size_kb: 128,
        aur_votes: 0,
        preinstalled: false,
    },
    PackageSpec {
        repo: "extra",
        name: "fortune-mod",
        binary: "fortune",
        version: "3.22.0-1",
        description: "The Fortune Cookie Program from BSD games",
        url: "https://www.shlomifish.org/open-source/projects/fortune-mod/",
        licenses: "BSD-4-Clause",
        depends: &["base"],
        dl_size_kb: 1120,
        inst_size_kb: 3800,
        aur_votes: 0,
        preinstalled: false,
    },
    PackageSpec {
        repo: "extra",
        name: "figlet",
        binary: "figlet",
        version: "2.2.5-5",
        description: "A program for making large ASCII letters out of ordinary text",
        url: "http://www.figlet.org/",
        licenses: "BSD-3-Clause",
        depends: &["base"],
        dl_size_kb: 185,
        inst_size_kb: 640,
        aur_votes: 0,
        preinstalled: false,
    },
    PackageSpec {
        repo: "extra",
        name: "lolcat",
        binary: "lolcat",
        version: "100.0.1-3",
        description: "Rainbows and unicorns in your terminal console!",
        url: "https://github.com/busyloop/lolcat",
        licenses: "BSD-3-Clause",
        depends: &["base"],
        dl_size_kb: 64,
        inst_size_kb: 190,
        aur_votes: 0,
        preinstalled: false,
    },
    PackageSpec {
        repo: "extra",
        name: "sl",
        binary: "sl",
        version: "5.05-2",
        description: "Steam Locomotive runs across your screen if you mistype 'ls'",
        url: "https://github.com/mtoyoda/sl",
        licenses: "custom",
        depends: &["base"],
        dl_size_kb: 32,
        inst_size_kb: 96,
        aur_votes: 0,
        preinstalled: false,
    },
    PackageSpec {
        repo: "extra",
        name: "bat",
        binary: "bat",
        version: "0.24.0-2",
        description: "Cat clone with syntax highlighting and git integration",
        url: "https://github.com/sharkdp/bat",
        licenses: "MIT OR Apache-2.0",
        depends: &["base"],
        dl_size_kb: 2150,
        inst_size_kb: 5420,
        aur_votes: 0,
        preinstalled: false,
    },
    PackageSpec {
        repo: "extra",
        name: "eza",
        binary: "eza",
        version: "0.20.6-1",
        description: "A modern, maintained replacement for ls (built on exa)",
        url: "https://github.com/eza-community/eza",
        licenses: "MIT",
        depends: &["base"],
        dl_size_kb: 680,
        inst_size_kb: 1840,
        aur_votes: 0,
        preinstalled: false,
    },
    PackageSpec {
        repo: "extra",
        name: "ripgrep",
        binary: "rg",
        version: "14.1.1-1",
        description: "A search tool that combines the usability of ag with the raw speed of grep",
        url: "https://github.com/BurntSushi/ripgrep",
        licenses: "MIT OR Unlicense",
        depends: &["base"],
        dl_size_kb: 1490,
        inst_size_kb: 4120,
        aur_votes: 0,
        preinstalled: false,
    },
    PackageSpec {
        repo: "extra",
        name: "fd",
        binary: "fd",
        version: "10.2.0-1",
        description: "Simple, fast and user-friendly alternative to find",
        url: "https://github.com/sharkdp/fd",
        licenses: "MIT OR Apache-2.0",
        depends: &["base"],
        dl_size_kb: 1120,
        inst_size_kb: 3210,
        aur_votes: 0,
        preinstalled: false,
    },
    PackageSpec {
        repo: "extra",
        name: "git",
        binary: "git",
        version: "2.47.0-1",
        description: "the fast distributed version control system",
        url: "https://git-scm.com/",
        licenses: "GPL-2.0-only",
        depends: &["base"],
        dl_size_kb: 6840,
        inst_size_kb: 34200,
        aur_votes: 0,
        preinstalled: false,
    },
    PackageSpec {
        repo: "extra",
        name: "rust",
        binary: "cargo",
        version: "1.101.0-1",
        description: "Systems programming language focused on safety, speed and concurrency",
        url: "https://www.rust-lang.org/",
        licenses: "MIT OR Apache-2.0",
        depends: &["base"],
        dl_size_kb: 64200,
        inst_size_kb: 245000,
        aur_votes: 0,
        preinstalled: false,
    },
    PackageSpec {
        repo: "extra",
        name: "python",
        binary: "python",
        version: "3.13.0-1",
        description: "The Python programming language interactive interpreter",
        url: "https://www.python.org/",
        licenses: "PSF-2.0",
        depends: &["base"],
        dl_size_kb: 14200,
        inst_size_kb: 62500,
        aur_votes: 0,
        preinstalled: false,
    },

    // --- AUR (ARCH USER REPOSITORY) ---
    PackageSpec {
        repo: "aur",
        name: "yay",
        binary: "yay",
        version: "12.4.2-1",
        description: "Yet another yogurt. Pacman wrapper and AUR helper written in Go/Rust",
        url: "https://github.com/Jguer/yay",
        licenses: "GPL-3.0-or-later",
        depends: &["pacman"],
        dl_size_kb: 3120,
        inst_size_kb: 9400,
        aur_votes: 2418,
        preinstalled: true,
    },
    PackageSpec {
        repo: "aur",
        name: "paru",
        binary: "paru",
        version: "2.0.4-1",
        description: "Feature packed AUR helper written in Rust",
        url: "https://github.com/Morganamilo/paru",
        licenses: "GPL-3.0-only",
        depends: &["pacman"],
        dl_size_kb: 2680,
        inst_size_kb: 7850,
        aur_votes: 1392,
        preinstalled: true,
    },
    PackageSpec {
        repo: "aur",
        name: "pipes-rs",
        binary: "pipes-rs",
        version: "0.2.0-1",
        description: "An over-engineered rewrite of pipes.sh in Rust",
        url: "https://github.com/CookieCoder15/pipes-rs",
        licenses: "MIT",
        depends: &["base"],
        dl_size_kb: 410,
        inst_size_kb: 1120,
        aur_votes: 184,
        preinstalled: false,
    },
    PackageSpec {
        repo: "aur",
        name: "cbonsai",
        binary: "cbonsai",
        version: "1.3.1-2",
        description: "Grow bonsai trees right in your terminal",
        url: "https://gitlab.com/jallbrit/cbonsai",
        licenses: "GPL-3.0-or-later",
        depends: &["base"],
        dl_size_kb: 38,
        inst_size_kb: 110,
        aur_votes: 312,
        preinstalled: false,
    },
    PackageSpec {
        repo: "aur",
        name: "hyfetch",
        binary: "hyfetch",
        version: "1.99.0-1",
        description: "Neofetch with pride flags and colorful distro gradients",
        url: "https://github.com/hykilpikonna/hyfetch",
        licenses: "MIT",
        depends: &["neofetch"],
        dl_size_kb: 280,
        inst_size_kb: 840,
        aur_votes: 267,
        preinstalled: false,
    },
    PackageSpec {
        repo: "aur",
        name: "pfetch",
        binary: "pfetch",
        version: "0.6.0-3",
        description: "A pretty system information tool written in POSIX sh/Rust",
        url: "https://github.com/dylanaraps/pfetch",
        licenses: "MIT",
        depends: &["base"],
        dl_size_kb: 18,
        inst_size_kb: 42,
        aur_votes: 419,
        preinstalled: false,
    },
    PackageSpec {
        repo: "aur",
        name: "onefetch",
        binary: "onefetch",
        version: "2.22.0-1",
        description: "Command-line Git information tool written in Rust",
        url: "https://github.com/o2sh/onefetch",
        licenses: "MIT",
        depends: &["base"],
        dl_size_kb: 3140,
        inst_size_kb: 8920,
        aur_votes: 295,
        preinstalled: false,
    },
    PackageSpec {
        repo: "aur",
        name: "tty-clock",
        binary: "tty-clock",
        version: "2.3-4",
        description: "Digital clock in ncurses/terminal using hardware CMOS RTC",
        url: "https://github.com/xorg62/tty-clock",
        licenses: "BSD-3-Clause",
        depends: &["base"],
        dl_size_kb: 29,
        inst_size_kb: 78,
        aur_votes: 540,
        preinstalled: false,
    },
    PackageSpec {
        repo: "aur",
        name: "nyancat",
        binary: "nyancat",
        version: "1.5.2-3",
        description: "Nyancat rendered in your terminal with ANSI colors",
        url: "https://github.com/klange/nyancat",
        licenses: "NCSA",
        depends: &["base"],
        dl_size_kb: 45,
        inst_size_kb: 132,
        aur_votes: 388,
        preinstalled: false,
    },
    PackageSpec {
        repo: "aur",
        name: "asciiquarium",
        binary: "asciiquarium",
        version: "1.1-5",
        description: "An aquarium/sea animation in ASCII art",
        url: "https://robobunny.com/projects/asciiquarium/html/",
        licenses: "GPL-2.0-or-later",
        depends: &["base"],
        dl_size_kb: 52,
        inst_size_kb: 148,
        aur_votes: 472,
        preinstalled: false,
    },
    PackageSpec {
        repo: "aur",
        name: "rustos-rice-theme",
        binary: "rice",
        version: "1.0.0-1",
        description: "Arch Linux ricing theme switcher (Arch, Matrix, Catppuccin, Dracula, Nord)",
        url: "https://github.com/rappar222/rust-os",
        licenses: "MIT",
        depends: &["rsh"],
        dl_size_kb: 24,
        inst_size_kb: 64,
        aur_votes: 999,
        preinstalled: false,
    },
];

#[derive(Clone)]
pub struct CustomPackage {
    pub name: String,
    pub version: String,
    pub description: String,
    pub exec_output: String,
}

pub struct PackageManager {
    pub installed: Vec<String>,
    pub custom_pkgs: Vec<CustomPackage>,
}

impl PackageManager {
    pub const fn new() -> Self {
        Self {
            installed: Vec::new(),
            custom_pkgs: Vec::new(),
        }
    }
}

pub static PKG_MGR: Mutex<PackageManager> = Mutex::new(PackageManager::new());

const INSTALLED_DB_PATH: &str = "/var/lib/pacman/installed.db";

pub fn init() {
    let mut mgr = PKG_MGR.lock();
    mgr.installed.clear();

    if let Ok(bytes) = fs::read_file(INSTALLED_DB_PATH) {
        if let Ok(text) = core::str::from_utf8(&bytes) {
            for line in text.lines() {
                let pkg = line.trim();
                if !pkg.is_empty() && !mgr.installed.iter().any(|p| p == pkg) {
                    mgr.installed.push(String::from(pkg));
                }
            }
        }
    }

    if mgr.installed.is_empty() {
        for spec in REPO_CATALOG.iter() {
            if spec.preinstalled {
                mgr.installed.push(String::from(spec.name));
                if !spec.binary.is_empty() {
                    let bin_path = format!("/usr/bin/{}", spec.binary);
                    let stub = format!("#!/bin/rsh\n# ELF 64-bit LSB pie executable ({}-{})\n", spec.name, spec.version);
                    let _ = fs::write_executable(&bin_path, stub.as_bytes());
                }
            }
        }
        save_db(&mgr.installed);
    }
}

fn save_db(installed: &[String]) {
    let mut out = String::new();
    for pkg in installed {
        out.push_str(pkg);
        out.push('\n');
    }
    let _ = fs::write_file(INSTALLED_DB_PATH, out.as_bytes());
}

pub fn is_installed(name: &str) -> bool {
    let mgr = PKG_MGR.lock();
    mgr.installed.iter().any(|p| p == name)
}

pub fn is_binary_installed(bin_name: &str) -> Option<&'static PackageSpec> {
    let mgr = PKG_MGR.lock();
    for spec in REPO_CATALOG.iter() {
        if spec.binary == bin_name || spec.name == bin_name {
            if mgr.installed.iter().any(|p| p == spec.name) {
                return Some(spec);
            }
        }
    }
    None
}

pub fn find_package_offering_binary(bin_name: &str) -> Option<&'static PackageSpec> {
    for spec in REPO_CATALOG.iter() {
        if spec.binary == bin_name || spec.name == bin_name {
            return Some(spec);
        }
    }
    None
}

pub fn find_spec(name: &str) -> Option<&'static PackageSpec> {
    REPO_CATALOG.iter().find(|s| s.name == name || s.binary == name)
}

pub fn count_installed() -> (usize, usize) {
    let mgr = PKG_MGR.lock();
    let mut pacman_count = 0;
    let mut aur_count = mgr.custom_pkgs.len();
    for pkg in mgr.installed.iter() {
        if let Some(spec) = REPO_CATALOG.iter().find(|s| s.name == pkg) {
            if spec.repo == "aur" {
                aur_count += 1;
            } else {
                pacman_count += 1;
            }
        } else {
            pacman_count += 1;
        }
    }
    (pacman_count, aur_count)
}

pub fn all_package_names() -> Vec<&'static str> {
    REPO_CATALOG.iter().map(|s| s.name).collect()
}

pub fn installed_package_names() -> Vec<String> {
    PKG_MGR.lock().installed.clone()
}

pub fn installed_binaries() -> Vec<String> {
    let mgr = PKG_MGR.lock();
    let mut bins = Vec::new();
    for pkg in mgr.installed.iter() {
        if let Some(spec) = REPO_CATALOG.iter().find(|s| s.name == pkg) {
            if !spec.binary.is_empty() {
                bins.push(String::from(spec.binary));
            }
        }
    }
    for custom in mgr.custom_pkgs.iter() {
        bins.push(custom.name.clone());
    }
    bins
}

pub fn install_all_packages() {
    let mut mgr = PKG_MGR.lock();
    for spec in REPO_CATALOG.iter() {
        if !mgr.installed.iter().any(|p| p == spec.name) {
            mgr.installed.push(String::from(spec.name));
            if !spec.binary.is_empty() {
                let bin_path = format!("/usr/bin/{}", spec.binary);
                let stub = format!("#!/bin/rsh\n# ELF 64-bit LSB pie executable ({}-{})\n", spec.name, spec.version);
                let _ = fs::write_executable(&bin_path, stub.as_bytes());
            }
        }
    }
    save_db(&mgr.installed);
}

fn draw_progress_bar(label: &str) {
    vga::set_colors(ColorCode::White, ColorCode::Black);
    print!(" {:36} [", label);
    vga::set_colors(ColorCode::LightCyan, ColorCode::Black);
    for _ in 0..22 {
        keyboard::sleep_ticks(1);
        print!("#");
    }
    vga::set_colors(ColorCode::White, ColorCode::Black);
    println!("] 100%");
}

pub fn cmd_pacman(args: &[&str]) {
    if args.is_empty() {
        vga::set_colors(ColorCode::LightRed, ColorCode::Black);
        println!("error: no operation specified (use -h for help)");
        return;
    }

    let flag = args[0];
    let targets = &args[1..];

    match flag {
        "-h" | "--help" => {
            vga::set_colors(ColorCode::White, ColorCode::Black);
            println!("usage:  pacman <operation> [...]");
            println!("operations:");
            println!("    pacman -S <pkg>      install package(s) from official Arch repositories");
            println!("    pacman -Syu          synchronize package databases and upgrade system");
            println!("    pacman -Ss <query>   search official repositories for matching strings");
            println!("    pacman -Si <pkg>     view detailed package information");
            println!("    pacman -Q            list all installed packages");
            println!("    pacman -Qi <pkg>     view installed package status");
            println!("    pacman -Ql <pkg>     list files owned by package");
            println!("    pacman -R <pkg>      remove an installed package");
        }
        "-Syu" | "-Sy" | "-Syyu" => {
            vga::set_colors(ColorCode::LightBlue, ColorCode::Black);
            print!(":: ");
            vga::set_colors(ColorCode::White, ColorCode::Black);
            println!("Synchronizing package databases...");
            draw_progress_bar("core");
            draw_progress_bar("extra");
            draw_progress_bar("community");
            if flag.contains('u') {
                vga::set_colors(ColorCode::LightBlue, ColorCode::Black);
                print!(":: ");
                vga::set_colors(ColorCode::White, ColorCode::Black);
                println!("Starting full system upgrade...");
                println!(" there is nothing to do");
            }
            if !targets.is_empty() {
                install_official_packages(targets, false);
            }
        }
        "-S" => {
            if targets.is_empty() {
                vga::set_colors(ColorCode::LightRed, ColorCode::Black);
                println!("error: no targets specified (use -h for help)");
                return;
            }
            install_official_packages(targets, false);
        }
        "-Ss" => {
            let query = targets.join(" ").to_lowercase();
            search_packages(&query, false);
        }
        "-Si" => {
            if targets.is_empty() {
                vga::set_colors(ColorCode::LightRed, ColorCode::Black);
                println!("error: no targets specified");
                return;
            }
            for &t in targets {
                show_package_info(t, false);
            }
        }
        "-Q" | "-Qe" => {
            let mgr = PKG_MGR.lock();
            for name in mgr.installed.iter() {
                if let Some(spec) = REPO_CATALOG.iter().find(|s| s.name == name) {
                    vga::set_colors(ColorCode::White, ColorCode::Black);
                    print!("{} ", spec.name);
                    vga::set_colors(ColorCode::LightGreen, ColorCode::Black);
                    println!("{}", spec.version);
                }
            }
            for custom in mgr.custom_pkgs.iter() {
                vga::set_colors(ColorCode::White, ColorCode::Black);
                print!("{} ", custom.name);
                vga::set_colors(ColorCode::LightGreen, ColorCode::Black);
                println!("{}-1", custom.version);
            }
        }
        "-Qi" => {
            for &t in targets {
                show_package_info(t, true);
            }
        }
        "-Ql" => {
            for &t in targets {
                if !is_installed(t) {
                    vga::set_colors(ColorCode::LightRed, ColorCode::Black);
                    println!("error: package '{}' was not found", t);
                } else if let Some(spec) = find_spec(t) {
                    vga::set_colors(ColorCode::White, ColorCode::Black);
                    if !spec.binary.is_empty() {
                        println!("{} /usr/bin/{}", spec.name, spec.binary);
                    }
                    println!("{} /usr/share/licenses/{}/LICENSE", spec.name, spec.name);
                }
            }
        }
        "-R" | "-Rs" | "-Rns" => {
            if targets.is_empty() {
                vga::set_colors(ColorCode::LightRed, ColorCode::Black);
                println!("error: no targets specified");
                return;
            }
            remove_packages(targets);
        }
        _ => {
            vga::set_colors(ColorCode::LightRed, ColorCode::Black);
            println!("error: invalid option '{}'", flag);
        }
    }
}

pub fn cmd_yay_or_paru(helper_name: &str, args: &[&str]) {
    if args.is_empty() || args[0] == "-Syu" || args[0] == "-Syyu" {
        cmd_pacman(&["-Syu"]);
        vga::set_colors(ColorCode::LightBlue, ColorCode::Black);
        print!(":: ");
        vga::set_colors(ColorCode::White, ColorCode::Black);
        println!("Searching AUR for updates...");
        println!(" -> there is nothing to do");
        return;
    }

    let flag = args[0];
    let targets = &args[1..];

    match flag {
        "-h" | "--help" => {
            vga::set_colors(ColorCode::White, ColorCode::Black);
            println!("Usage: {} <operation> [...]", helper_name);
            println!("  {} -S <pkg>      Install package from Official Repos or AUR (builds PKGBUILD)", helper_name);
            println!("  {} -Ss <query>   Search Official Repos + Arch User Repository (AUR)", helper_name);
            println!("  {} -Syu          Upgrade all official and AUR packages", helper_name);
            println!("  {} -Si <pkg>     Show package details & AUR votes", helper_name);
            println!("  {} -R <pkg>      Remove package", helper_name);
        }
        "-S" => {
            if targets.is_empty() {
                vga::set_colors(ColorCode::LightRed, ColorCode::Black);
                println!("error: no targets specified");
                return;
            }
            install_official_packages(targets, true);
        }
        "-Ss" => {
            let query = targets.join(" ").to_lowercase();
            search_packages(&query, true);
        }
        "-Si" | "-Qi" => {
            for &t in targets {
                show_package_info(t, false);
            }
        }
        "-Q" | "-Qe" => {
            cmd_pacman(&["-Q"]);
        }
        "-R" | "-Rs" | "-Rns" => {
            remove_packages(targets);
        }
        other if !other.starts_with('-') => {
            // Running `yay <search_term>` searches both repos + AUR
            let query = args.join(" ").to_lowercase();
            search_packages(&query, true);
        }
        _ => {
            vga::set_colors(ColorCode::LightRed, ColorCode::Black);
            println!("error: unknown option '{}'", flag);
        }
    }
}

fn install_official_packages(targets: &[&str], allow_aur: bool) {
    let mut specs_to_install = Vec::new();
    for &t in targets {
        match find_spec(t) {
            Some(spec) => {
                if spec.repo == "aur" && !allow_aur {
                    vga::set_colors(ColorCode::LightRed, ColorCode::Black);
                    println!("error: target not found: {}", t);
                    vga::set_colors(ColorCode::Yellow, ColorCode::Black);
                    println!(
                        "hint: '{}' is an AUR package! Install it using 'yay -S {}' or 'paru -S {}'",
                        spec.name, spec.name, spec.name
                    );
                    return;
                }
                specs_to_install.push(spec);
            }
            None => {
                vga::set_colors(ColorCode::LightRed, ColorCode::Black);
                println!("error: target not found: {}", t);
                return;
            }
        }
    }

    for spec in specs_to_install {
        if spec.repo == "aur" {
            build_and_install_aur(spec);
        } else {
            vga::set_colors(ColorCode::White, ColorCode::Black);
            println!("resolving dependencies...");
            println!("looking for conflicting packages...");
            println!(
                "\nPackages (1) {}-{}\n",
                spec.name, spec.version
            );
            println!("Total Download Size:   {:.2} MiB", spec.dl_size_kb as f32 / 1024.0);
            println!("Total Installed Size:  {:.2} MiB\n", spec.inst_size_kb as f32 / 1024.0);

            vga::set_colors(ColorCode::LightBlue, ColorCode::Black);
            print!(":: ");
            vga::set_colors(ColorCode::White, ColorCode::Black);
            println!("Retrieving packages...");
            draw_progress_bar(&format!("{}-{}-x86_64", spec.name, spec.version));
            draw_progress_bar("(1/1) checking keys in keyring");
            draw_progress_bar("(1/1) checking package integrity");
            draw_progress_bar(&format!("(1/1) installing {}", spec.name));

            register_installed_spec(spec);
            vga::set_colors(ColorCode::LightGreen, ColorCode::Black);
            println!("==> Package '{}' ({}) installed successfully!", spec.name, spec.version);
        }
    }
}

fn build_and_install_aur(spec: &PackageSpec) {
    vga::set_colors(ColorCode::LightBlue, ColorCode::Black);
    print!(":: ");
    vga::set_colors(ColorCode::White, ColorCode::Black);
    println!("Checking for conflicts...");
    vga::set_colors(ColorCode::LightBlue, ColorCode::Black);
    print!(":: ");
    vga::set_colors(ColorCode::White, ColorCode::Black);
    println!("Checking for inner conflicts...");
    vga::set_colors(ColorCode::Yellow, ColorCode::Black);
    println!("\n[Aur: 1]  {}-{}\n", spec.name, spec.version);

    vga::set_colors(ColorCode::LightBlue, ColorCode::Black);
    print!(":: ");
    vga::set_colors(ColorCode::White, ColorCode::Black);
    println!("(1/1) Downloaded PKGBUILD: {}", spec.name);

    // Save PKGBUILD to /var/cache/yay/<pkg>.PKGBUILD
    let pkgbuild_path = format!("/var/cache/yay/{}.PKGBUILD", spec.name);
    let pkgbuild_content = format!(
        "# Maintainer: Arch User Repository <aur@archlinux.org>\npkgname={}\npkgver={}\npkgrel=1\npkgdesc=\"{}\"\narch=('x86_64')\nurl=\"{}\"\nlicense=('{}')\n",
        spec.name, spec.version, spec.description, spec.url, spec.licenses
    );
    let _ = fs::write_file(&pkgbuild_path, pkgbuild_content.as_bytes());

    vga::set_colors(ColorCode::LightGreen, ColorCode::Black);
    print!("==> ");
    vga::set_colors(ColorCode::White, ColorCode::Black);
    println!("Making package: {} {} (x86_64)", spec.name, spec.version);

    vga::set_colors(ColorCode::LightGreen, ColorCode::Black);
    print!("==> ");
    vga::set_colors(ColorCode::White, ColorCode::Black);
    println!("Retrieving sources...");
    println!("  -> Cloning {} git repo...", spec.name);
    keyboard::sleep_ticks(12);

    vga::set_colors(ColorCode::LightGreen, ColorCode::Black);
    print!("==> ");
    vga::set_colors(ColorCode::White, ColorCode::Black);
    println!("Validating source files with sha256sums...");
    println!("    {}-{}.tar.gz ... Passed", spec.name, spec.version);

    vga::set_colors(ColorCode::LightGreen, ColorCode::Black);
    print!("==> ");
    vga::set_colors(ColorCode::White, ColorCode::Black);
    println!("Starting build()...");
    vga::set_colors(ColorCode::LightCyan, ColorCode::Black);
    println!("   Compiling {} v{} (/var/cache/yay/{})", spec.name, spec.version, spec.name);
    keyboard::sleep_ticks(20);
    println!("    Finished `release` profile [optimized] target(s) in 0.21s");

    vga::set_colors(ColorCode::LightGreen, ColorCode::Black);
    print!("==> ");
    vga::set_colors(ColorCode::White, ColorCode::Black);
    println!("Entering fakeroot environment...");
    vga::set_colors(ColorCode::LightGreen, ColorCode::Black);
    print!("==> ");
    vga::set_colors(ColorCode::White, ColorCode::Black);
    println!("Starting package()...");
    vga::set_colors(ColorCode::LightGreen, ColorCode::Black);
    print!("==> ");
    vga::set_colors(ColorCode::White, ColorCode::Black);
    println!("Finished making: {} {}", spec.name, spec.version);

    draw_progress_bar(&format!("(1/1) installing {}", spec.name));
    register_installed_spec(spec);
}

fn register_installed_spec(spec: &PackageSpec) {
    let mut mgr = PKG_MGR.lock();
    if !mgr.installed.iter().any(|p| p == spec.name) {
        mgr.installed.push(String::from(spec.name));
    }
    if !spec.binary.is_empty() {
        let bin_path = format!("/usr/bin/{}", spec.binary);
        let stub = format!(
            "#!/bin/rsh\n# ELF 64-bit LSB pie executable ({}-{})\n",
            spec.name, spec.version
        );
        let _ = fs::write_executable(&bin_path, stub.as_bytes());
    }
    save_db(&mgr.installed);
}

fn remove_packages(targets: &[&str]) {
    let mut mgr = PKG_MGR.lock();
    for &t in targets {
        if t == "base" || t == "linux-rustos" || t == "pacman" || t == "rsh" {
            vga::set_colors(ColorCode::LightRed, ColorCode::Black);
            println!("error: '{}' is in HoldPkg and cannot be removed!", t);
            continue;
        }
        if let Some(pos) = mgr.installed.iter().position(|p| p == t) {
            mgr.installed.remove(pos);
            if let Some(spec) = find_spec(t) {
                if !spec.binary.is_empty() {
                    let bin_path = format!("/usr/bin/{}", spec.binary);
                    let _ = fs::delete(&bin_path, false);
                }
            }
            draw_progress_bar(&format!("(1/1) removing {}", t));
        } else {
            vga::set_colors(ColorCode::LightRed, ColorCode::Black);
            println!("error: target not found: {}", t);
        }
    }
    save_db(&mgr.installed);
}

fn search_packages(query: &str, include_aur: bool) {
    let mgr = PKG_MGR.lock();
    let mut found = 0;

    for spec in REPO_CATALOG.iter() {
        if spec.repo == "aur" && !include_aur {
            continue;
        }
        let matches = query.is_empty()
            || spec.name.to_lowercase().contains(query)
            || spec.description.to_lowercase().contains(query)
            || spec.binary.to_lowercase().contains(query);

        if matches {
            found += 1;
            let repo_color = match spec.repo {
                "core" => ColorCode::LightGreen,
                "extra" => ColorCode::LightCyan,
                "community" => ColorCode::LightMagenta,
                "aur" => ColorCode::LightBlue,
                _ => ColorCode::White,
            };
            vga::set_colors(repo_color, ColorCode::Black);
            print!("{}/", spec.repo);
            vga::set_colors(ColorCode::White, ColorCode::Black);
            print!("{} ", spec.name);
            vga::set_colors(ColorCode::LightGreen, ColorCode::Black);
            print!("{}", spec.version);

            if spec.repo == "aur" {
                vga::set_colors(ColorCode::Yellow, ColorCode::Black);
                print!(" (+{} 9.8%)", spec.aur_votes);
            }

            if mgr.installed.iter().any(|p| p == spec.name) {
                vga::set_colors(ColorCode::LightCyan, ColorCode::Black);
                print!(" [installed]");
            }
            println!();
            vga::set_colors(ColorCode::LightGray, ColorCode::Black);
            println!("    {}", spec.description);
        }
    }

    if found == 0 {
        vga::set_colors(ColorCode::LightGray, ColorCode::Black);
        println!("No matching packages found for '{}'.", query);
    }
}

fn show_package_info(name: &str, require_installed: bool) {
    if require_installed && !is_installed(name) {
        vga::set_colors(ColorCode::LightRed, ColorCode::Black);
        println!("error: package '{}' was not found", name);
        return;
    }

    if let Some(spec) = find_spec(name) {
        vga::set_colors(ColorCode::White, ColorCode::Black);
        println!("Repository      : {}", spec.repo);
        println!("Name            : {}", spec.name);
        println!("Version         : {}", spec.version);
        println!("Description     : {}", spec.description);
        println!("Architecture    : x86_64");
        println!("URL             : {}", spec.url);
        println!("Licenses        : {}", spec.licenses);
        println!("Depends On      : {:?}", spec.depends);
        println!("Download Size   : {} KiB", spec.dl_size_kb);
        println!("Installed Size  : {} KiB", spec.inst_size_kb);
        if spec.repo == "aur" {
            println!("AUR Votes       : {}", spec.aur_votes);
        }
        println!(
            "Status          : {}",
            if is_installed(spec.name) {
                "Installed"
            } else {
                "Not Installed"
            }
        );
    } else {
        vga::set_colors(ColorCode::LightRed, ColorCode::Black);
        println!("error: package '{}' was not found", name);
    }
}

pub fn cmd_makepkg(cwd: &str, _args: &[&str]) {
    let pkgbuild_path = fs::normalize_path(cwd, "PKGBUILD");
    let data = match fs::read_file(&pkgbuild_path) {
        Ok(d) => d,
        Err(_) => {
            vga::set_colors(ColorCode::LightRed, ColorCode::Black);
            println!("==> ERROR: PKGBUILD does not exist in '{}'.", cwd);
            return;
        }
    };

    let content = core::str::from_utf8(&data).unwrap_or("");
    let mut pkgname = String::from("custom-pkg");
    let mut pkgver = String::from("1.0.0");
    let mut pkgdesc = String::from("Custom user-compiled RustOS package");
    let mut exec_output = String::from("Executed custom AUR package!");

    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(val) = trimmed.strip_prefix("pkgname=") {
            pkgname = String::from(val.trim_matches('"').trim_matches('\''));
        } else if let Some(val) = trimmed.strip_prefix("pkgver=") {
            pkgver = String::from(val.trim_matches('"').trim_matches('\''));
        } else if let Some(val) = trimmed.strip_prefix("pkgdesc=") {
            pkgdesc = String::from(val.trim_matches('"').trim_matches('\''));
        } else if let Some(val) = trimmed.strip_prefix("exec_output=") {
            exec_output = String::from(val.trim_matches('"').trim_matches('\''));
        }
    }

    vga::set_colors(ColorCode::LightGreen, ColorCode::Black);
    print!("==> ");
    vga::set_colors(ColorCode::White, ColorCode::Black);
    println!("Making package: {} {}-1 (x86_64)", pkgname, pkgver);
    println!("==> Checking runtime dependencies...");
    println!("==> Checking buildtime dependencies...");
    println!("==> Starting build()...");
    keyboard::sleep_ticks(15);
    println!("==> Entering fakeroot environment...");
    println!("==> Starting package()...");
    draw_progress_bar(&format!("installing {}", pkgname));

    let bin_path = format!("/usr/bin/{}", pkgname);
    let _ = fs::write_executable(&bin_path, exec_output.as_bytes());

    let mut mgr = PKG_MGR.lock();
    if !mgr.installed.iter().any(|p| p == &pkgname) {
        mgr.installed.push(pkgname.clone());
    }
    mgr.custom_pkgs.push(CustomPackage {
        name: pkgname.clone(),
        version: pkgver,
        description: pkgdesc,
        exec_output,
    });
    save_db(&mgr.installed);

    vga::set_colors(ColorCode::LightGreen, ColorCode::Black);
    println!("==> Finished making and installing '{}'! Run it by typing '{}'.", pkgname, pkgname);
}

pub fn try_run_custom_pkg(cmd: &str) -> bool {
    let mgr = PKG_MGR.lock();
    for c in mgr.custom_pkgs.iter() {
        if c.name == cmd {
            vga::set_colors(ColorCode::LightGreen, ColorCode::Black);
            println!("{}", c.exec_output);
            return true;
        }
    }
    false
}
