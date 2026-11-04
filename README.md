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

### 4. Linux Utility Suite
- **Filesystem & Text**: `ls` (`-l`, `-a`, `-la`), `cd`, `pwd`, `mkdir`, `rmdir`, `touch`, `cat` (`-n`), `write`, `append`, `cp`, `mv`, `rm` (`-r`, `-rf`), `chmod`, `stat`, `head`, `tail`, `wc`, `grep`, `find`, `tree`, `nano`
- **System & Hardware**: `neofetch`, `fastfetch`, `hyfetch`, `pfetch`, `onefetch`, `uname` (`-a`, `-r`), `arch`, `whoami`, `id`, `hostname`, `hostnamectl`, `date`, `timedatectl`, `uptime`, `lscpu`, `lspci`, `lsblk`, `lsmod`, `free -h`, `df -h`, `du`, `mount`, `ps`, `top`, `htop`, `btop`, `systemctl`, `journalctl`, `dmesg`, `ip`, `ping`, `curl`
- **Shell & Environment**: `echo`, `env`, `export`, `unset`, `alias`, `which`, `history`, `man`, `source`, `calc`, `mem`, `alloc`, `disk`, `format`, `colors`, `rice`, `reboot`, `shutdown`

---

## Building & Running

### Build All Images (`rustos.vdi`, `rustos.iso`, BIOS & UEFI)
```bash
cargo run --package runner -- --no-run
```
This produces:
- `rustos.vdi`: VirtualBox Hard Disk image (with persistent `SimpleFS v2` storage)
- `rustos.iso`: Bootable hybrid BIOS/UEFI ISO image
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
