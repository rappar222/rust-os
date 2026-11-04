# RustOS (x86_64 Bare-Metal Operating System in Rust)

**RustOS** is a custom, low-overhead `x86_64` bare-metal operating system written 100% in `#![no_std]` Rust. It is **inspired by Arch Linux** and features an interactive terminal shell (`rsh`), hierarchical persistent filesystem (`SimpleFS v2`), and native package management with support for **Arch Linux packages and the AUR** (`pacman`, `yay`, `paru`, `makepkg`).

---

## Key Features

### 1. Bare-Metal `x86_64` Rust Kernel (`#![no_std]`)
- **Bootloader & Long Mode**: Boots via BIOS or UEFI (`bootloader_api 0.11`) with 4-level paging and full physical memory mapping.
- **GDT, TSS & Interrupts**: Custom Global Descriptor Table, Task State Segment (with isolated double-fault stack), IDT exception handlers, 8259 PIC, and 100 Hz PIT hardware timer.
- **Dynamic Kernel Heap**: 4 MiB physical frame allocator + `linked_list_allocator` supporting `Box`, `Vec`, `String`, and dynamic structures in Ring 0.
- **Hardware Drivers**:
  - **VGA Graphical Framebuffer Console** (`kernel/src/vga.rs`): 8x16 bitmap font renderer, 16-color ANSI/VGA palette with official Rust Orange (`#DE551C`), hardware scrolling, and coordinate plotting (`put_char_at`).
  - **PS/2 Keyboard Driver** (`kernel/src/keyboard.rs`): Full i8042 controller initialization (works in both BIOS and UEFI), Shift/CapsLock state machine, arrow keys (`Up`/`Down`/`Left`/`Right`), and `Ctrl+C`/`Ctrl+L`/`Ctrl+D` shortcuts.
  - **ATA PIO Storage Driver** (`kernel/src/ata.rs`): 28-bit LBA sector read/write driver persisting data directly to disk (`rustos.vdi` / `rustos-bios.img`) with automatic in-memory fallback when booted from read-only ISO media.
  - **CMOS Real-Time Clock (RTC)** (`kernel/src/rtc.rs`): Reads real hardware UTC date and time from ports `0x70`/`0x71`.
  - **CPUID & PCI Bus Scanner** (`kernel/src/sysinfo.rs`): Queries real x86_64 processor brand/vendor/features (`cpuid`) and enumerates hardware devices on the PCI configuration space (`0xCF8`/`0xCFC`).

### 2. Interactive Terminal Shell (`rsh`) with Tab Completion & History
- **Arch-Inspired Prompt**: Rust-styled `[root@rustos ~]# ` prompt with dynamic working directory display (`~`, `/etc`, `/var/cache/yay`, etc.).
- **Smart `[TAB]` Completion**:
  - Completes shell commands and installed package binaries when typing the first word.
  - Completes `pacman` / `yay` / `paru` flags (`-S`, `-Syu`, `-Ss`, `-Si`, `-Q`, `-Qi`, `-Ql`, `-R`).
  - Completes package names from Arch repositories (`core`, `extra`) and AUR (`aur`) after `pacman -S` / `yay -S`.
  - Completes installed package names after `pacman -R` / `pacman -Qi`.
  - Completes file and directory paths in the current or specified directory.
- **Command History & Piping**: Navigate previous commands with `Up` / `Down` arrow keys, chain commands with `&&` or `;`, redirect output to files (`>`, `>>`), and pipe output (`|`) into `grep`, `wc`, `head`, `tail`, `cowsay`, `lolcat`, or `figlet`.

### 3. Arch Linux Packages, AUR (`yay` / `paru`), and `makepkg` in Rust
- **Arch Repositories via `pacman`**:
  - `pacman -Syu`: Synchronize repository definitions and verify system status.
  - `pacman -Ss <query>`: Search package catalogue inspired by Arch Linux repositories.
  - `pacman -S <pkg>`: Install package applets (`cmatrix`, `btop`, `htop`, `cowsay`, `fortune-mod`, `figlet`, `lolcat`, `sl`, `bat`, `eza`, `ripgrep`, `fd`, `git`, `rust`, `python`, `fastfetch`, `neofetch`, `nano`).
  - `pacman -Q` / `-Qi` / `-Ql` / `-R`: Query installed packages, inspect metadata, list owned files, or remove packages. Installed state is persisted in `/var/lib/pacman/installed.db`.
- **Arch User Repository (`yay` & `paru`)**:
  - `yay -Ss <query>` / `paru -Ss <query>`: Search across both standard and AUR packages.
  - `yay -S <pkg>` / `paru -S <pkg>`: Downloads the AUR `PKGBUILD` into `/var/cache/yay/`, compiles the package in Rust, and installs the binary into `/usr/bin` (`cbonsai`, `pipes-rs`, `hyfetch`, `pfetch`, `onefetch`, `tty-clock`, `nyancat`, `asciiquarium`, `rustos-rice-theme`).
- **Custom Package Compilation (`makepkg`)**:
  - Edit `/root/PKGBUILD` with `nano PKGBUILD` and run `makepkg -si` to build and install your own custom executable package directly inside RustOS.

### 4. Interactive Guided Installer (`archinstall` style)
- **Guided Setup (`archinstall` / `rustos-install` / `install`)**:
  - Interactive multi-step wizard modeled after the Arch Linux guided installer (`archinstall`).
  - **Locale & Keymap**: Choose between English (US/UK), German, Romanian, French, Spanish, Italian.
  - **Timezone**: Select between UTC, Europe/Bucharest, America/New_York, Europe/London, Europe/Berlin, Asia/Tokyo.
  - **Package Mirror**: Select Tier-1 Arch & RustOS package mirrors (Worldwide geo, Kernel.org, Cloudflare CDN).
  - **Disk Partitioning**: Wipe and auto-partition `/dev/sda` with SimpleFS v2 persistence, use existing partitions, or RAM disk.
  - **System Hostname & Users**: Set hostname, root password, standard user account, and sudoers (`wheel` group) privileges.
  - **Desktop Selection**: Choose to install the RustOS Floating Window Compositor or minimal CLI console.
  - Automated non-interactive mode supported via `-y` / `--yes`.

### 5. Custom Floating Window Compositor (`startx` / `wm` / `desktop`)
- **Overlapping Floating Windows**:
  - Available after installation (or enabled via `startx` / `wm` / `desktop`).
  - **Interactive Floating Terminal**: Live `rsh` shell in a floating window supporting commands, scrolling, and history.
  - **Live System Monitor**: Floating `btop` dashboard with live CPU stats, memory, disk I/O, and kernel tasks.
  - **System Information Window**: Floating `neofetch` window with the official Rust gear ASCII logo and specifications.
- **Window Chrome & Controls**:
  - Rust Orange (`#DE551C`) title bars for focused windows with `[X]` `[-]` `[+]` control buttons and 1px border.
  - Top taskbar panel showing RustOS badge, active window tabs, live RTC UTC clock, and memory stats.
  - Bottom status bar with keybinding hints.
- **Keyboard Shortcuts**:
  - `Alt + Tab`: Cycle focus through windows and bring focused window to the top of the Z-stack.
  - `Alt + Arrows`: Move focused window smoothly across the screen canvas.
  - `Alt + N`: Spawn a new floating terminal window.
  - `Alt + Q`: Close the active window.
  - `Esc`: Exit GUI compositor and return cleanly to the console shell.

### 6. Linux Utility Suite
- **Filesystem & Text**: `ls` (`-l`, `-a`, `-la`), `cd`, `pwd`, `mkdir`, `rmdir`, `touch`, `cat` (`-n`), `write`, `append`, `cp`, `mv`, `rm` (`-r`, `-rf`), `chmod`, `stat`, `head`, `tail`, `wc`, `grep`, `find`, `tree`, `nano`
- **System & Hardware**: `neofetch`, `fastfetch`, `hyfetch`, `pfetch`, `onefetch`, `uname` (`-a`, `-r`), `arch`, `whoami`, `id`, `hostname`, `hostnamectl`, `date`, `timedatectl`, `uptime`, `lscpu`, `lspci`, `lsblk`, `lsmod`, `free -h`, `df -h`, `du`, `mount`, `ps`, `top`, `htop`, `btop`, `systemctl`, `journalctl`, `dmesg`, `ip`, `ping`, `curl`
- **Shell & Environment**: `echo`, `env`, `export`, `unset`, `alias`, `which`, `history`, `man`, `source`, `calc`, `mem`, `alloc`, `disk`, `format`, `colors`, `rice`, `reboot`, `shutdown`

---

## Building & Running

### Build All Images (`rustos.vdi`, `rustos-uefi.vdi`, `rustos.iso`)
```bash
cargo run --package runner -- --no-run
```
This produces:
- `rustos.vdi`: VirtualBox Hard Disk image (BIOS MBR, fixed UUID `bf3f79da-beec-4105-abad-2a6221b124a4`)
- `rustos-uefi.vdi`: VirtualBox Hard Disk image (UEFI GPT)
- `rustos.iso`: Hybrid bootable ISO image (boots in both BIOS and UEFI mode)
- `target/bin/rustos-bios.img`: Raw 32 MiB BIOS disk image
- `target/bin/rustos-uefi.img`: Raw UEFI GPT disk image

### Run in QEMU
```bash
cargo run --package runner
```
Or headless in your current terminal:
```bash
cargo run --package runner -- --headless
```
