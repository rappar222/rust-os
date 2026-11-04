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
pub mod power;
pub mod shell;

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

    println!("[OK] Display subsystem initialized");

    // 3. Initialize GDT & TSS (with separate double-fault stack)
    gdt::init();
    println!("[OK] GDT & TSS initialized");

    // 4. Initialize IDT, 8259 PIC, and PIT Timer (100 Hz)
    interrupts::init();
    keyboard::init();
    println!("[OK] IDT, PIC, PIT timer (100Hz), and PS/2 keyboard initialized");

    // 5. Initialize Memory & Dynamic Heap
    let phys_offset = boot_info
        .physical_memory_offset
        .into_option()
        .expect("Physical memory offset missing from bootloader");

    let mem_stats = memory::inspect_memory(&boot_info.memory_regions);
    println!(
        "Memory: Detected {} MB usable / {} MB total",
        mem_stats.total_usable_bytes / (1024 * 1024),
        mem_stats.total_memory_bytes / (1024 * 1024)
    );

    match memory::init_heap_from_boot_info(&mut boot_info.memory_regions, phys_offset) {
        Ok((heap_start, heap_size)) => {
            println!(
                "[OK] Dynamic heap initialized: {} MB @ {:#x}",
                heap_size / (1024 * 1024),
                heap_start
            );
        }
        Err(e) => {
            println!("[WARN] Failed to initialize heap: {}", e);
        }
    }

    // 6. Initialize ATA Hard Drive & Persistent Storage
    match ata::init() {
        Ok(drive_info) => {
            println!(
                "[OK] Hard Drive detected: {} ({} MB)",
                drive_info.model_str(),
                drive_info.size_mb
            );
            match fs::init() {
                Ok(()) => println!("[OK] Persistent storage (SimpleFS) mounted"),
                Err(e) => println!("[WARN] Filesystem error: {}", e),
            }
        }
        Err(e) => {
            println!("[INFO] Storage: {}", e);
        }
    }

    println!("[OK] System initialized successfully. Launching interactive terminal shell...");

    // 6. Launch the interactive terminal shell
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
