extern crate alloc;

use alloc::vec::Vec;
use crate::fs;
use crate::{print, println, vga::{self, ColorCode}};

pub fn dispatch_arch_command(cmd: &str, args: &[&str], cwd: &str) -> bool {
    match cmd {
        "printf" => {
            let s = args.join(" ");
            let unescaped = s.replace("\\n", "\n").replace("\\t", "\t");
            print!("{}", unescaped);
        }
        "vdir" => {
            println!("total 8");
            for e in fs::list_dir(cwd) {
                println!("{} 1 root root {:6} Nov 04 19:45 {}", e.mode_string(), e.size_bytes, e.basename());
            }
        }
        "tac" => {
            if let Some(&f) = args.first() {
                let p = fs::normalize_path(cwd, f);
                if let Ok(bytes) = fs::read_file(&p) {
                    if let Ok(text) = core::str::from_utf8(&bytes) {
                        let lines: Vec<&str> = text.lines().collect();
                        for l in lines.iter().rev() {
                            println!("{}", l);
                        }
                    }
                }
            }
        }
        "shred" => {
            if let Some(&f) = args.first() {
                let p = fs::normalize_path(cwd, f);
                let _ = fs::write_file(&p, b"\x00\xFF\x00\xFF\xAA\x55");
                let _ = fs::delete(&p, false);
                println!("shred: {}: pass 3/3 (random) ... removed", f);
            }
        }
        "chown" | "chgrp" => {
            if args.len() >= 2 {
                println!("Ownership of '{}' updated to '{}'.", args[1], args[0]);
            }
        }
        "nl" => {
            if let Some(&f) = args.first() {
                let p = fs::normalize_path(cwd, f);
                if let Ok(bytes) = fs::read_file(&p) {
                    if let Ok(text) = core::str::from_utf8(&bytes) {
                        for (i, l) in text.lines().enumerate() {
                            println!("{:6}  {}", i + 1, l);
                        }
                    }
                }
            }
        }
        "egrep" | "fgrep" => {
            if args.len() >= 2 {
                let pat = args[0];
                let p = fs::normalize_path(cwd, args[1]);
                if let Ok(bytes) = fs::read_file(&p) {
                    if let Ok(text) = core::str::from_utf8(&bytes) {
                        for (i, l) in text.lines().enumerate() {
                            if l.contains(pat) {
                                println!("{}:{}:{}", args[1], i + 1, l);
                            }
                        }
                    }
                }
            }
        }
        "diff" | "cmp" => {
            if args.len() >= 2 {
                let p1 = fs::normalize_path(cwd, args[0]);
                let p2 = fs::normalize_path(cwd, args[1]);
                let d1 = fs::read_file(&p1).unwrap_or_default();
                let d2 = fs::read_file(&p2).unwrap_or_default();
                if d1 == d2 {
                    // identical, silent
                } else {
                    println!("--- {}", args[0]);
                    println!("+++ {}", args[1]);
                    println!("@@ -1 +1 @@");
                    println!("-{}", core::str::from_utf8(&d1).unwrap_or("<binary>").trim());
                    println!("+{}", core::str::from_utf8(&d2).unwrap_or("<binary>").trim());
                }
            }
        }
        "comm" => {
            println!("comm: comparison stream ready");
        }
        "sort" => {
            if let Some(&f) = args.first() {
                let p = fs::normalize_path(cwd, f);
                if let Ok(bytes) = fs::read_file(&p) {
                    if let Ok(text) = core::str::from_utf8(&bytes) {
                        let mut lines: Vec<&str> = text.lines().collect();
                        lines.sort();
                        for l in lines {
                            println!("{}", l);
                        }
                    }
                }
            }
        }
        "uniq" => {
            if let Some(&f) = args.first() {
                let p = fs::normalize_path(cwd, f);
                if let Ok(bytes) = fs::read_file(&p) {
                    if let Ok(text) = core::str::from_utf8(&bytes) {
                        let mut prev: Option<&str> = None;
                        for l in text.lines() {
                            if prev != Some(l) {
                                println!("{}", l);
                                prev = Some(l);
                            }
                        }
                    }
                }
            }
        }
        "tr" => {
            if args.len() >= 2 {
                println!("tr: translated pattern '{}' to '{}'", args[0], args[1]);
            }
        }
        "cut" => {
            if let Some(&f) = args.last() {
                let p = fs::normalize_path(cwd, f);
                if let Ok(bytes) = fs::read_file(&p) {
                    if let Ok(text) = core::str::from_utf8(&bytes) {
                        for l in text.lines() {
                            let part = l.split_whitespace().next().unwrap_or("");
                            println!("{}", part);
                        }
                    }
                }
            }
        }
        "paste" => {
            println!("paste: merged column streams");
        }
        "sed" => {
            println!("sed: stream editor pattern executed");
        }
        "awk" => {
            println!("awk: pattern scanning and processing language ready");
        }
        "less" | "more" => {
            if let Some(&f) = args.first() {
                let p = fs::normalize_path(cwd, f);
                if let Ok(bytes) = fs::read_file(&p) {
                    if let Ok(text) = core::str::from_utf8(&bytes) {
                        for l in text.lines().take(25) {
                            println!("{}", l);
                        }
                        vga::set_colors(ColorCode::DarkGray, ColorCode::Black);
                        println!("--More--(END)");
                    }
                }
            }
        }
        "pacstrap" => {
            let target = args.first().copied().unwrap_or("/mnt");
            vga::set_colors(ColorCode::LightGreen, ColorCode::Black);
            print!("==> ");
            vga::set_colors(ColorCode::White, ColorCode::Black);
            println!("Installing packages to '{}'...", target);
            crate::installer::run_installer(true);
        }
        "arch-chroot" => {
            let target = args.first().copied().unwrap_or("/mnt");
            vga::set_colors(ColorCode::LightGreen, ColorCode::Black);
            println!("==> Mounting /sys /proc /dev into chroot at '{}'...", target);
            println!("Entered chroot environment (type exit to leave).");
        }
        "genfstab" => {
            println!("# /etc/fstab: generated by genfstab");
            println!("/dev/sda2       /        simplefs   rw,relatime,sync       0 1");
            println!("/dev/sda1       /boot    vfat       defaults               0 2");
            println!("tmpfs           /tmp     tmpfs      defaults,nodev,nosuid  0 0");
        }
        "groups" => {
            println!("root bin daemon wheel");
        }
        "users" | "who" => {
            println!("root     tty1         Nov  4 19:40");
        }
        "w" => {
            println!(" 19:45:00 up 12 min,  1 user,  load average: 0.00, 0.01, 0.05");
            println!("USER     TTY      FROM             LOGIN@   IDLE   JCPU   PCPU WHAT");
            println!("root     tty1     -                19:40    0.00s  0.08s  0.00s -rsh");
        }
        "cal" => {
            let dt = crate::rtc::read_rtc();
            println!("     {} {:04}    ", dt.month_name(), dt.year);
            println!("Su Mo Tu We Th Fr Sa");
            println!(" 1  2  3  4  5  6  7");
            println!(" 8  9 10 11 12 13 14");
            println!("15 16 17 18 19 20 21");
            println!("22 23 24 25 26 27 28");
            println!("29 30 31");
        }
        "lsusb" => {
            println!("Bus 001 Device 001: ID 1d6b:0002 Linux Foundation 2.0 root hub");
            println!("Bus 001 Device 002: ID 80ee:0021 VirtualBox USB Tablet");
        }
        "modprobe" | "insmod" => {
            if let Some(&m) = args.first() {
                vga::set_colors(ColorCode::LightGreen, ColorCode::Black);
                println!("[OK] Kernel module '{}' loaded.", m);
            }
        }
        "rmmod" => {
            if let Some(&m) = args.first() {
                println!("Module '{}' unloaded.", m);
            }
        }
        "umount" => {
            if let Some(&m) = args.first() {
                println!("umount: {} unmounted.", m);
            }
        }
        "fdisk" | "parted" => {
            let ata = crate::ata::ATA_DRIVE.lock();
            let mb = ata.get_info().map(|i| i.size_mb).unwrap_or(32);
            drop(ata);
            println!("Disk /dev/sda: {} MiB, {} bytes", mb, mb as u64 * 1024 * 1024);
            println!("Units: sectors of 1 * 512 = 512 bytes");
            println!("Sector size (logical/physical): 512 bytes / 512 bytes");
            println!("Device     Boot Start      End  Sectors Size Id Type");
            println!("/dev/sda1  *     2048    12000     9953 4.8M  ef EFI (FAT-12/16/32)");
            println!("/dev/sda2       12000    65535    53536  26M  83 Linux");
        }
        "mkfs" | "mkfs.ext4" => {
            let target = args.first().copied().unwrap_or("/dev/sda2");
            println!("mke2fs 1.47.1 (RustOS SimpleFS)");
            println!("Creating filesystem with 65536 1k blocks on {}", target);
            println!("Allocating group tables: done");
            println!("Writing inode tables: done");
            println!("Writing superblocks and filesystem accounting information: done");
        }
        "fsck" => {
            println!("fsck from util-linux 2.40 (RustOS)");
            println!("/dev/sda2: clean, 24/64 files, 1420/65536 blocks");
        }
        "sync" => {
            let _ = crate::fs::FS.lock().save_metadata();
            println!("sync: caches written to persistent disk.");
        }
        "pstree" => {
            println!("systemd(1)─┬─ata-pio(2)");
            println!("           ├─i8042-kbd(3)");
            println!("           └─rsh(10)───pstree(42)");
        }
        "kill" | "killall" | "pkill" => {
            if let Some(&target) = args.first() {
                println!("kill: sent SIGTERM to target '{}'", target);
            }
        }
        "nice" | "renice" => {
            println!("nice: priority adjusted to 0");
        }
        "sleep" => {
            let secs: u64 = args.first().and_then(|s| s.parse().ok()).unwrap_or(1);
            crate::keyboard::sleep_ticks(secs * 100);
        }
        "loginctl" => {
            println!("SESSION  UID USER SEAT  TTY ");
            println!("      1    0 root seat0 tty1");
            println!("\n1 sessions listed.");
        }
        "udevadm" => {
            println!("udevadm: 18 hardware events triggered and settled.");
        }
        "traceroute" => {
            let host = args.first().copied().unwrap_or("archlinux.org");
            println!("traceroute to {} (95.217.163.246), 30 hops max, 60 byte packets", host);
            println!(" 1  gateway (10.0.2.2)  0.284 ms  0.211 ms  0.198 ms");
            println!(" 2  95.217.163.246  34.120 ms  34.205 ms  34.090 ms");
        }
        "route" | "netstat" | "ss" => {
            println!("Kernel IP routing table");
            println!("Destination     Gateway         Genmask         Flags Metric Ref    Use Iface");
            println!("0.0.0.0         10.0.2.2        0.0.0.0         UG    100    0        0 enp0s3");
            println!("10.0.2.0        0.0.0.0         255.255.255.0   U     100    0        0 enp0s3");
        }
        "dig" | "nslookup" | "host" => {
            let query = args.first().copied().unwrap_or("archlinux.org");
            println!("; <<>> DiG 9.20.2 <<>> {}", query);
            println!(";; ANSWER SECTION:");
            println!("{} 300 IN A 95.217.163.246", query);
            println!(";; Query time: 14 msec");
        }
        "unalias" => {
            println!("unalias: alias mapping removed");
        }
        "whereis" => {
            for &t in args {
                println!("{}: /usr/bin/{} /usr/share/man/man1/{}.1.gz", t, t, t);
            }
        }
        "apropos" | "whatis" => {
            let topic = args.first().copied().unwrap_or("arch");
            println!("{}: system reference utility (1)", topic);
        }
        "tar" => {
            println!("tar (child): archive stream processed");
        }
        "gzip" | "gunzip" | "bzip2" | "bunzip2" | "xz" | "unxz" | "zstd" | "unzstd" | "zip" | "unzip" => {
            if let Some(&f) = args.first() {
                println!("{}: processed compression stream for '{}'", cmd, f);
            }
        }
        "md5sum" | "sha1sum" | "sha256sum" | "sha512sum" => {
            if let Some(&f) = args.first() {
                let p = fs::normalize_path(cwd, f);
                let data = fs::read_file(&p).unwrap_or_default();
                let mut acc: u64 = 0xcbf29ce484222325;
                for b in data {
                    acc ^= b as u64;
                    acc = acc.wrapping_mul(0x100000001b3);
                }
                println!("{:016x}{:016x}  {}", acc, acc.rotate_left(16), f);
            }
        }
        "base64" => {
            if let Some(&f) = args.first() {
                let p = fs::normalize_path(cwd, f);
                if let Ok(bytes) = fs::read_file(&p) {
                    println!("[base64: {} bytes encoded]", bytes.len());
                }
            }
        }
        "xxd" | "hexdump" | "od" => {
            if let Some(&f) = args.first() {
                let p = fs::normalize_path(cwd, f);
                if let Ok(bytes) = fs::read_file(&p) {
                    for (i, chunk) in bytes.chunks(16).take(8).enumerate() {
                        print!("{:08x}: ", i * 16);
                        for b in chunk {
                            print!("{:02x} ", b);
                        }
                        println!();
                    }
                }
            }
        }
        "basename" => {
            if let Some(&f) = args.first() {
                let (_, base) = fs::parent_and_basename(f);
                println!("{}", base);
            }
        }
        "dirname" => {
            if let Some(&f) = args.first() {
                let (parent, _) = fs::parent_and_basename(f);
                println!("{}", parent);
            }
        }
        "realpath" | "readlink" => {
            if let Some(&f) = args.first() {
                println!("{}", fs::normalize_path(cwd, f));
            }
        }
        "seq" => {
            let max: usize = args.first().and_then(|s| s.parse().ok()).unwrap_or(10);
            for i in 1..=core::cmp::min(max, 50) {
                println!("{}", i);
            }
        }
        "yes" => {
            let word = args.first().copied().unwrap_or("y");
            for _ in 0..10 {
                println!("{}", word);
            }
        }
        "true" => {}
        "false" => {}
        "test" | "expr" => {
            println!("0");
        }
        "tee" => {
            if let Some(&f) = args.first() {
                let p = fs::normalize_path(cwd, f);
                let _ = fs::write_file(&p, b"tee stream output\n");
            }
        }
        "xargs" => {
            println!("xargs: executed command arguments");
        }
        "envsubst" => {
            println!("envsubst: substituted environment variables");
        }
        "timeout" => {
            if args.len() >= 2 {
                println!("timeout: running '{}' for {}s", args[1], args[0]);
            }
        }
        "watch" => {
            println!("Every 2.0s: {}", args.join(" "));
        }
        _ => return false,
    }
    true
}
