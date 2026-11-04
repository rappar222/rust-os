use anyhow::{Context, Result};
use std::env;
use std::io::{Read, Seek};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();

    let is_release = args.iter().any(|arg| arg == "--release");
    let no_run = args.iter().any(|arg| arg == "--no-run" || arg == "--build-only");
    let headless = args.iter().any(|arg| arg == "--headless" || arg == "--nographic");
    let boot_iso = args.iter().any(|arg| arg == "--iso" || arg == "--cdrom");

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

    // 4. Create target/bin directory
    let out_dir = workspace_root.join("target").join("bin");
    std::fs::create_dir_all(&out_dir).context("Failed to create target/bin directory")?;

    // 5. Generate BIOS bootable disk image (.img)
    let bios_image = out_dir.join("rustos-bios.img");
    println!("[*] Creating BIOS bootable disk image: {:?}", bios_image);
    let bios_boot = bootloader::BiosBoot::new(&kernel_binary);
    bios_boot
        .create_disk_image(&bios_image)
        .context("Failed to create BIOS disk image")?;

    // Extend disk image to 32 MB to provide hard drive storage for SimpleFS
    let disk_file = std::fs::OpenOptions::new()
        .write(true)
        .open(&bios_image)?;
    disk_file.set_len(32 * 1024 * 1024)?; // 32 MiB virtual disk

    println!(
        "[+] BIOS disk image ready ({} bytes)",
        std::fs::metadata(&bios_image)?.len()
    );

    // 6. Generate UEFI bootable disk image (.img)
    let uefi_image = out_dir.join("rustos-uefi.img");
    println!("[*] Creating UEFI bootable disk image: {:?}", uefi_image);
    let uefi_boot = bootloader::UefiBoot::new(&kernel_binary);
    uefi_boot
        .create_disk_image(&uefi_image)
        .context("Failed to create UEFI disk image")?;
    println!(
        "[+] UEFI disk image ready ({} bytes)",
        std::fs::metadata(&uefi_image)?.len()
    );

    // 7. Generate bootable .ISO using xorriso and UEFI image
    let iso_path = out_dir.join("rustos.iso");
    create_iso(&uefi_image, &out_dir, &iso_path)?;
    let root_iso = workspace_root.join("rustos.iso");
    let _ = std::fs::copy(&iso_path, &root_iso);

    // 8. Generate VirtualBox VDI hard disk image
    let vdi_image = out_dir.join("rustos.vdi");
    let root_vdi = workspace_root.join("rustos.vdi");
    println!("[*] Creating VirtualBox VDI image: {:?}", root_vdi);
    let qemu_img_status = Command::new("qemu-img")
        .arg("convert")
        .arg("-f")
        .arg("raw")
        .arg("-O")
        .arg("vdi")
        .arg(&bios_image)
        .arg(&vdi_image)
        .status();

    if let Ok(s) = qemu_img_status {
        if s.success() {
            let _ = std::fs::copy(&vdi_image, &root_vdi);
            println!(
                "[+] VirtualBox VDI ready: {:?} ({} bytes)",
                root_vdi,
                std::fs::metadata(&root_vdi)?.len()
            );
        }
    }

    if no_run {
        println!("[+] Build complete. Available images:");
        println!("    - VirtualBox Hard Disk : {:?}", root_vdi);
        println!("    - Bootable ISO Image   : {:?}", root_iso);
        println!("    - Raw BIOS Disk Image  : {:?}", bios_image);
        println!("    - Raw UEFI Disk Image  : {:?}", uefi_image);
        return Ok(());
    }

    // 8. Launch QEMU
    println!("[*] Launching QEMU emulator...");
    let mut qemu = Command::new("qemu-system-x86_64");

    if boot_iso {
        println!("[*] Booting from CD-ROM ISO: {:?}", iso_path);
        qemu.arg("-cdrom").arg(&iso_path);
    } else {
        println!("[*] Booting from BIOS disk image: {:?}", bios_image);
        qemu.arg("-drive").arg(format!("format=raw,file={}", bios_image.display()));
    }

    qemu.arg("-m")
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
        if arg.starts_with("-")
            && arg != "--release"
            && arg != "--no-run"
            && arg != "--build-only"
            && arg != "--headless"
            && arg != "--nographic"
            && arg != "--iso"
            && arg != "--cdrom"
        {
            qemu.arg(arg);
        }
    }

    println!("[+] Running: {:?}", qemu);
    let status = qemu.status().context("Failed to run QEMU emulator")?;

    handle_qemu_exit(status);
    Ok(())
}

fn create_iso(uefi_image: &Path, out_dir: &Path, iso_path: &Path) -> Result<()> {
    println!("[*] Creating bootable ISO image: {:?}", iso_path);

    // 1. Temporary staging directory for ISO structure
    let iso_root = out_dir.join("iso_root");
    let _ = std::fs::remove_dir_all(&iso_root);
    std::fs::create_dir_all(iso_root.join("boot"))?;

    // 2. Extract the EFI system FAT partition from the GPT UEFI image
    let efi_fat_img = iso_root.join("boot").join("efi.img");
    {
        let device = std::fs::File::open(uefi_image)
            .context("Failed to open UEFI image for partition extraction")?;
        let disk = gpt::GptConfig::new()
            .open_from_device(Box::new(device))
            .context("Failed to parse GPT header from UEFI image")?;

        let partition = disk
            .partitions()
            .values()
            .next()
            .context("No partition found in UEFI disk image")?;

        let start_bytes = partition.bytes_start(*disk.logical_block_size())?;
        let len_bytes = partition.bytes_len(*disk.logical_block_size())?;

        let mut src = std::fs::File::open(uefi_image)?;
        src.seek(std::io::SeekFrom::Start(start_bytes))?;
        let mut dest = std::fs::File::create(&efi_fat_img)?;
        std::io::copy(&mut src.take(len_bytes), &mut dest)?;
    }

    // 3. Package into bootable ISO using xorriso
    let xorriso_check = Command::new("which").arg("xorriso").output();
    let has_xorriso = xorriso_check.map(|o| o.status.success()).unwrap_or(false);

    if has_xorriso {
        let mut xorriso = Command::new("xorriso");
        xorriso
            .arg("-as")
            .arg("mkisofs")
            .arg("-iso-level")
            .arg("3")
            .arg("-full-iso9660-filenames")
            .arg("-volid")
            .arg("RUSTOS")
            .arg("-eltorito-alt-boot")
            .arg("-e")
            .arg("boot/efi.img")
            .arg("-no-emul-boot")
            .arg("-isohybrid-gpt-basdat")
            .arg("-output")
            .arg(iso_path)
            .arg(&iso_root);

        let status = xorriso.status().context("Failed to execute xorriso")?;
        if status.success() {
            println!(
                "[+] Bootable ISO ready: {:?} ({} bytes)",
                iso_path,
                std::fs::metadata(iso_path)?.len()
            );
        } else {
            eprintln!("[!] Warning: xorriso exited with status {}", status);
        }
    } else {
        println!("[!] xorriso not found on host system; skipping .iso packaging.");
    }

    // Clean up temporary iso_root staging
    let _ = std::fs::remove_dir_all(&iso_root);
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
