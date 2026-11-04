#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

extern crate alloc;

pub mod font;
pub mod vga;
pub mod serial;
pub mod gdt;
pub mod interrupts;
pub mod keyboard;
pub mod memory;
pub mod allocator;
pub mod ata;
pub mod fs;
pub mod rtc;
pub mod sysinfo;
pub mod pkg;
pub mod applets;
pub mod power;
pub mod shell;
pub mod installer;
pub mod compositor;
pub mod arch_commands;

use alloc::format;
use bootloader_api::{entry_point, BootInfo, BootloaderConfig};
use core::panic::PanicInfo;

pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(bootloader_api::config::Mapping::Dynamic);
    config
};

entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    // 1. Initialize Serial port (COM1 0x3F8) for immediate logging
    serial::init();
    serial_println!("=== RustOS Boot Sequence Started ===");

    // 2. Initialize Graphical Framebuffer Console
    if let Some(framebuffer) = boot_info.framebuffer.as_mut() {
        vga::init(framebuffer);
    } else {
        serial_println!("[WARN] Framebuffer not available, running in headless serial mode");
    }

    println!(":: running early hook [udev]");
    println!(":: Triggering uevents...");

    // 3. Initialize GDT & TSS (with separate double-fault stack)
    gdt::init();

    // 4. Initialize IDT, 8259 PIC, PIT Timer (100 Hz), and PS/2 Keyboard
    interrupts::init();
    keyboard::init();

    // 5. Initialize Memory & Dynamic Heap
    let phys_offset = boot_info
        .physical_memory_offset
        .into_option()
        .expect("Physical memory offset missing from bootloader");

    let mem_stats = memory::inspect_memory(&boot_info.memory_regions);

    match memory::init_heap_from_boot_info(&mut boot_info.memory_regions, phys_offset) {
        Ok((heap_start, heap_size)) => {
            sysinfo::klog(&format!(
                "Linux version 6.12.0-arch1-1-rustos (root@archiso) (rustc 1.101.0-nightly)"
            ));
            sysinfo::klog(&format!(
                "Memory: {}K/{}K available (Heap {}MB @ {:#x})",
                mem_stats.total_usable_bytes / 1024,
                mem_stats.total_memory_bytes / 1024,
                heap_size / (1024 * 1024),
                heap_start
            ));
        }
        Err(e) => {
            println!("[WARN] Failed to initialize heap: {}", e);
        }
    }

    let cpu = sysinfo::get_cpu_info();
    sysinfo::klog(&format!(
        "smpboot: CPU0: {} {} (family: {:#x}, model: {:#x}, stepping: {:#x})",
        cpu.vendor,
        cpu.model_name,
        cpu.family,
        cpu.model,
        cpu.stepping
    ));

    let pci_devs = sysinfo::scan_pci();
    sysinfo::klog(&format!("PCI: Probed bus 00, detected {} hardware devices", pci_devs.len()));

    // 6. Initialize ATA Hard Drive & Persistent Storage (with in-memory Arch FHS fallback)
    match ata::init() {
        Ok(drive_info) => {
            sysinfo::klog(&format!(
                "ata1.00: ATA-7: {}, {} MB ({} sectors)",
                drive_info.model_str(),
                drive_info.size_mb,
                drive_info.sectors
            ));
            println!(
                ":: Mounting '/dev/sda2' ({}, {} MB) on real root...",
                drive_info.model_str(),
                drive_info.size_mb
            );
        }
        Err(e) => {
            sysinfo::klog(&format!("ata1: no writable disk ({}), mounting archiso tmpfs root", e));
            println!(":: No writable ATA drive ({}), mounting in-memory Arch rootfs...", e);
        }
    }

    if let Err(e) = fs::init() {
        println!("[WARN] Filesystem init warning: {}", e);
    } else {
        sysinfo::klog("EXT4-fs / SimpleFS (sda2): mounted filesystem with ordered data mode");
    }

    // 7. Initialize Package Database (inspired by Arch Linux pacman + AUR)
    pkg::init();
    sysinfo::klog("systemd[1]: Reached target Multi-User System (RustOS).");

    let dt = rtc::read_rtc();
    println!(
        "[  OK  ] Started RustOS Userland ({} {:04}-{:02}-{:02} {:02}:{:02}:{:02} UTC).",
        dt.weekday_name(),
        dt.year,
        dt.month,
        dt.day,
        dt.hour,
        dt.minute,
        dt.second
    );

    // 8. Launch the interactive Arch-style terminal shell
    shell::run_shell();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    vga::set_colors(vga::ColorCode::LightRed, vga::ColorCode::Black);
    println!("\n================ KERNEL PANIC ================");
    println!("{}", info);
    println!("==============================================");

    serial_println!("\n================ KERNEL PANIC ================");
    serial_println!("{}", info);
    serial_println!("==============================================");

    loop {
        x86_64::instructions::interrupts::disable();
        x86_64::instructions::hlt();
    }
}
