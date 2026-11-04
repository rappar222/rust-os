use anyhow::{Context, Result};
use std::env;
use std::path::PathBuf;
use std::process::{Command, ExitStatus};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();

    let is_release = args.iter().any(|arg| arg == "--release");
    let no_run = args.iter().any(|arg| arg == "--no-run" || arg == "--build-only");
    let headless = args.iter().any(|arg| arg == "--headless" || arg == "--nographic");

    println!("==================================================");
    println!("        RustOS Builder & Runner System           ");
    println!("==================================================");

    // 1. Determine workspace root and target paths
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .context("Failed to find workspace root")?
        .to_path_buf();

    let profile = if is_release { "release" } else { "debug" };
    println!("[*] Building kernel for x86_64-unknown-none ({profile})...");

    // 2. Build the kernel binary
    let mut cargo_build = Command::new("cargo");
    cargo_build
        .arg("build")
        .arg("--package")
        .arg("kernel")
        .arg("--target")
        .arg("x86_64-unknown-none")
        .current_dir(&workspace_root);

    if is_release {
        cargo_build.arg("--release");
    }

    let status = cargo_build.status().context("Failed to invoke cargo build for kernel")?;
    if !status.success() {
        anyhow::bail!("Kernel build failed with status: {status}");
    }
    println!("[+] Kernel built successfully");

    // 3. Locate kernel executable
    let kernel_binary = workspace_root
        .join("target")
        .join("x86_64-unknown-none")
        .join(profile)
        .join("kernel");

    if !kernel_binary.exists() {
        anyhow::bail!("Kernel binary not found at {:?}", kernel_binary);
    }

    // 4. Generate BIOS bootable disk image
    let out_dir = workspace_root.join("target").join("bin");
    std::fs::create_dir_all(&out_dir).context("Failed to create target/bin directory")?;

    let bios_image = out_dir.join("rustos-bios.img");
    println!("[*] Creating BIOS bootable disk image: {:?}", bios_image);

    let bios_boot = bootloader::BiosBoot::new(&kernel_binary);
    bios_boot
        .create_disk_image(&bios_image)
        .context("Failed to create BIOS disk image")?;

    println!("[+] BIOS disk image ready ({} bytes)", std::fs::metadata(&bios_image)?.len());

    if no_run {
        println!("[+] Build complete. Skipping execution as requested.");
        return Ok(());
    }

    // 5. Launch QEMU
    println!("[*] Launching QEMU emulator...");
    let mut qemu = Command::new("qemu-system-x86_64");
    qemu.arg("-drive")
        .arg(format!("format=raw,file={}", bios_image.display()))
        .arg("-m")
        .arg("512M")
        .arg("-device")
        .arg("isa-debug-exit,iobase=0xf4,iosize=0x04");

    if headless {
        qemu.arg("-display").arg("none").arg("-serial").arg("stdio");
    } else {
        qemu.arg("-serial").arg("mon:stdio");
    }

    // Pass any extra arguments that look like QEMU options
    for arg in &args {
        if arg.starts_with("-") && arg != "--release" && arg != "--no-run" && arg != "--build-only" && arg != "--headless" && arg != "--nographic" {
            qemu.arg(arg);
        }
    }

    println!("[+] Running: {:?}", qemu);
    let status = qemu.status().context("Failed to run QEMU emulator")?;

    handle_qemu_exit(status);
    Ok(())
}

fn handle_qemu_exit(status: ExitStatus) {
    match status.code() {
        Some(0) => println!("\n[+] QEMU exited cleanly."),
        Some(33) => println!("\n[+] QEMU exited via ACPI / debug-exit (success)."),
        Some(code) => println!("\n[*] QEMU exited with code: {}", code),
        None => println!("\n[*] QEMU terminated by signal."),
    }
}
