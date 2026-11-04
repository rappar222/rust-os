# RustOS

An interactive, terminal-based 64-bit operating system written in **Rust** from scratch, featuring a custom monolithic bare-metal kernel, hardware interrupt handling, dynamic heap allocation, high-resolution framebuffer console, serial logging, and a built-in shell.

```
  ____            _    ___  ____  
 |  _ \ _   _ ___| |_ / _ \/ ___| 
 | |_) | | | / __| __| | | \___ \ 
 |  _ <| |_| \__ \ |_| |_| |___) |
 |_| \_\\__,_|___/\__|\___/|____/ 
```

---

## Architecture Overview

```
                      +-----------------------------+
                      |      RustOS Shell           |
                      | (help, mem, calc, alloc...) |
                      +--------------+--------------+
                                     |
               +---------------------+---------------------+
               |                                           |
      +--------v--------+                         +--------v--------+
      |  VGA / Console  |                         |  Dynamic Heap   |
      |   Framebuffer   |                         |  (alloc crate)  |
      |   (8x16 font)   |                         | (Box, Vec, Str) |
      +--------+--------+                         +--------+--------+
               |                                           |
+--------------v-------------------------------------------v--------------+
|                          RustOS Kernel Core                             |
|  - GDT & TSS (Double Fault IST Stack)                                   |
|  - IDT (Exceptions: Page Fault, GPF, Breakpoint)                        |
|  - 8259 Dual PIC + PIT 100Hz Timer + PS/2 Keyboard Driver               |
|  - UART 16550 Serial Driver (COM1 0x3F8)                                |
|  - Power Management (ACPI shutdown & 8042 reboot)                       |
+------------------------------------+------------------------------------+
                                     |
+------------------------------------v------------------------------------+
|                         x86_64 Bare-Metal Hardware                      |
+-------------------------------------------------------------------------+
```

---

## Key Features

- **Freestanding `#![no_std]` Kernel**: Runs directly on bare metal without any OS dependency or C runtime.
- **Custom Terminal / Console**:
  - Linear framebuffer text rendering using an embedded **8x16 bitmap font**.
  - **16-color ANSI VGA palette** support.
  - Smooth line scrolling, tab expansion, backspace, and hardware-style cursor.
- **Dual Output**:
  - Every `print!` and `println!` mirrors to both the **VGA display** and **UART 16550 Serial Port (COM1)**.
- **CPU & Interrupt Architecture**:
  - **GDT (Global Descriptor Table)** and **TSS (Task State Segment)** with an Interrupt Stack Table (IST) preventing triple-fault kernel crashes during stack overflows.
  - **IDT (Interrupt Descriptor Table)** handling CPU exceptions (Breakpoint, Double Fault, Page Fault with CR2 address inspection, General Protection Fault).
  - **8259 Dual PIC** handling hardware IRQs.
  - **PIT Timer** calibrated to 100 Hz (10 ms resolution) for system uptime.
  - **PS/2 Keyboard Controller** with scancode set 1 decoding, Shift, Caps Lock, backspace, enter, and non-blocking input ring buffer.
- **Dynamic Memory Allocation**:
  - Integrates Rust's official `alloc` crate on bare metal.
  - Linked-list heap allocator initialized from bootloader-detected usable RAM regions.
  - Full support for `Box`, `Vec`, `String`, `format!`, and closures.
- **ATA Hard Drive & Persistent Storage**:
  - **ATA PIO Hard Drive Driver** on primary IDE/SATA bus.
  - **SimpleFS Filesystem**: Real persistent file storage saved directly to hard disk sectors across reboots.
  - Shell commands for disk inspection and file management (`disk`, `ls`, `cat`, `write`, `rm`, `format`).
- **Interactive Shell**:
  - Command parsing and line editing in real time.
  - Built-in utilities for inspecting hardware, memory, uptime, math evaluation, and power control.

---

## Shell Commands

| Command | Description |
| :--- | :--- |
| `help` | Lists all available shell commands with syntax |
| `info` | Displays architecture, kernel version, paging, and CPU mode |
| `clear` | Clears the terminal screen and resets cursor position |
| `echo <text>` | Prints arguments to the terminal |
| `uptime` | Shows system uptime and 100 Hz timer tick count |
| `colors` | Renders color swatch blocks for all 16 VGA colors |
| `mem` | Displays heap base address, allocated size, and RAM status |
| `alloc` | Runs dynamic memory tests allocating `Vec`, `Box`, and `String` |
| `disk` | Displays ATA hard drive hardware specifications & capacity |
| `ls` / `dir` | Lists files saved on persistent disk storage |
| `cat <file>` | Reads and displays file content from the hard drive |
| `write <file> <text>` | Saves text into a file on the persistent hard drive |
| `rm <file>` | Deletes a file from the hard drive |
| `format` | Reformats the persistent SimpleFS filesystem |
| `calc <expr>` | Mathematical expression evaluator (e.g. `calc 42 * 10 + 5`) |
| `panic` | Triggers an intentional kernel panic to demonstrate crash dumps |
| `reboot` | Reboots the computer via the 8042 PS/2 controller |
| `shutdown` | Powers down the machine via ACPI / QEMU exit port |

---

## Directory Structure

```
RustOS/
├── .cargo/
│   └── config.toml          # Cargo configuration
├── kernel/                  # The bare-metal kernel
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs          # Kernel entry point (_start) & boot coordination
│       ├── vga.rs           # Framebuffer terminal console & color management
│       ├── font.rs          # Embedded 8x16 console bitmap font (256 glyphs)
│       ├── serial.rs        # UART 16550 serial port driver (COM1)
│       ├── gdt.rs           # Global Descriptor Table & TSS double fault stack
│       ├── interrupts.rs    # IDT, PIC remapping, PIT timer, exception handlers
│       ├── keyboard.rs      # PS/2 scancode decoder and input ring buffer
│       ├── memory.rs        # Physical memory map inspection
│       ├── allocator.rs     # Dynamic heap allocator implementation
│       ├── power.rs         # ACPI shutdown and hardware reboot
│       └── shell.rs         # Interactive terminal shell & command processor
├── runner/                  # Host build script & QEMU launcher
│   ├── Cargo.toml
│   └── src/
│       └── main.rs          # Compiles kernel and builds bootable BIOS image
├── Makefile                 # Quick developer commands (build, run, clean)
├── rust-toolchain.toml      # Nightly toolchain pinning & x86_64 target
└── README.md
```

---

## Getting Started

### Prerequisites

- **Rust** (nightly toolchain is automatically managed via `rust-toolchain.toml`)
- **QEMU** (`qemu-system-x86_64`)

### Build Bootable Images (ISO, BIOS, UEFI)

```bash
cargo run --package runner -- --no-run
# or
make build
# or specifically for the ISO:
make iso
```

This compiles the kernel for `x86_64-unknown-none` and packages ready-to-use formats:
- **`rustos.vdi`**: Native VirtualBox Hard Disk (attach as primary SATA/IDE hard drive)
- **`rustos.iso`**: Bootable El Torito CD-ROM ISO
- **`target/bin/rustos-bios.img`**: Raw MBR disk image (BIOS systems & raw USB flash drive)
- **`target/bin/rustos-uefi.img`**: GPT disk image (UEFI hardware systems)

### Running in VirtualBox

1. In VirtualBox, create a new VM:
   - **Type**: `Linux` or `Other`
   - **Version**: **`Other/Unknown (64-bit)`** or **`Linux (64-bit)`** *(must be 64-bit!)*
2. Under **Storage**:
   - Add a Hard Disk: choose **`rustos.vdi`** from the project root.
3. Start the VM! RustOS will boot directly from the virtual hard drive with persistent storage enabled.

### Run in QEMU Emulator

To run with graphical window and serial output:
```bash
cargo run
# or
make run
```

To run directly from the CD-ROM `.iso` image:
```bash
make run-iso
```

To run in headless serial mode (ideal for terminal-only / remote servers):
```bash
make run-headless
```

---

## License

This project is licensed under the terms of the [LICENSE](LICENSE) file.
