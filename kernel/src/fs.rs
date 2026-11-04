extern crate alloc;

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use spin::Mutex;
use crate::ata;

const FS_MAGIC: &[u8; 8] = b"RUSTFS02";
const SUPERBLOCK_LBA: u32 = 12000;
const METADATA_SECTORS: u32 = 8; // 8 sectors * 8 entries/sector = 64 entries
const DATA_START_LBA: u32 = 12016;
pub const MAX_FILES: usize = 64;
const MAX_SECTORS_PER_FILE: u32 = 16; // 8 KB per file

#[derive(Clone, Copy)]
#[repr(C)]
pub struct FileEntry {
    pub name: [u8; 48],
    pub start_sector: u32,
    pub size_bytes: u32,
    pub mode: u16,
    pub is_dir: u8,
    pub used: u8,
    pub _reserved: [u8; 4],
}

impl FileEntry {
    pub const fn empty() -> Self {
        Self {
            name: [0; 48],
            start_sector: 0,
            size_bytes: 0,
            mode: 0o644,
            is_dir: 0,
            used: 0,
            _reserved: [0; 4],
        }
    }

    pub fn name_str(&self) -> &str {
        let len = self
            .name
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(self.name.len());
        core::str::from_utf8(&self.name[..len]).unwrap_or("<invalid>")
    }

    pub fn basename(&self) -> &str {
        let full = self.name_str();
        if let Some(pos) = full.rfind('/') {
            if pos + 1 < full.len() {
                &full[pos + 1..]
            } else {
                full
            }
        } else {
            full
        }
    }

    pub fn mode_string(&self) -> String {
        let ftype = if self.is_dir == 1 { 'd' } else { '-' };
        let m = self.mode;
        let rwx = |shift: u16| -> (char, char, char) {
            let bits = (m >> shift) & 0b111;
            (
                if (bits & 4) != 0 { 'r' } else { '-' },
                if (bits & 2) != 0 { 'w' } else { '-' },
                if (bits & 1) != 0 { 'x' } else { '-' },
            )
        };
        let (ur, uw, ux) = rwx(6);
        let (gr, gw, gx) = rwx(3);
        let (or, ow, ox) = rwx(0);
        format!("{}{}{}{}{}{}{}{}{}{}", ftype, ur, uw, ux, gr, gw, gx, or, ow, ox)
    }
}

pub fn normalize_path(cwd: &str, input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return String::from(cwd);
    }

    let expanded = if trimmed == "~" {
        String::from("/root")
    } else if let Some(rest) = trimmed.strip_prefix("~/") {
        format!("/root/{}", rest)
    } else if trimmed.starts_with('/') {
        String::from(trimmed)
    } else if cwd == "/" {
        format!("/{}", trimmed)
    } else {
        format!("{}/{}", cwd, trimmed)
    };

    let mut stack: Vec<&str> = Vec::new();
    for part in expanded.split('/') {
        if part.is_empty() || part == "." {
            continue;
        } else if part == ".." {
            stack.pop();
        } else {
            stack.push(part);
        }
    }

    if stack.is_empty() {
        String::from("/")
    } else {
        let mut out = String::new();
        for seg in stack {
            out.push('/');
            out.push_str(seg);
        }
        out
    }
}

pub fn parent_and_basename(path: &str) -> (String, String) {
    if path == "/" {
        return (String::from("/"), String::from("/"));
    }
    if let Some(pos) = path.rfind('/') {
        if pos == 0 {
            (String::from("/"), String::from(&path[1..]))
        } else {
            (String::from(&path[..pos]), String::from(&path[pos + 1..]))
        }
    } else {
        (String::from("/"), String::from(path))
    }
}

pub struct SimpleFs {
    pub entries: [FileEntry; MAX_FILES],
    pub cache: [Option<Vec<u8>>; MAX_FILES],
    pub disk_backed: bool,
    pub initialized: bool,
}

impl SimpleFs {
    pub const fn new() -> Self {
        const NONE_VEC: Option<Vec<u8>> = None;
        Self {
            entries: [FileEntry::empty(); MAX_FILES],
            cache: [NONE_VEC; MAX_FILES],
            disk_backed: false,
            initialized: false,
        }
    }

    pub fn init(&mut self) -> Result<(), &'static str> {
        let mut buf = [0u8; 512];
        match ata::read_sector(SUPERBLOCK_LBA, &mut buf) {
            Ok(()) => {
                self.disk_backed = true;
                if &buf[0..8] == FS_MAGIC {
                    // Read 8 sectors of 8 entries each (64 bytes per entry)
                    for sec in 0..METADATA_SECTORS {
                        let mut sec_buf = [0u8; 512];
                        ata::read_sector(SUPERBLOCK_LBA + 1 + sec, &mut sec_buf)?;
                        for slot in 0..8 {
                            let idx = (sec as usize * 8) + slot;
                            let offset = slot * 64;
                            let slice = &sec_buf[offset..offset + 64];
                            unsafe {
                                let ptr = slice.as_ptr() as *const FileEntry;
                                self.entries[idx] = *ptr;
                            }
                        }
                    }
                    self.initialized = true;
                    Ok(())
                } else {
                    self.format()
                }
            }
            Err(_) => {
                // Fallback to in-memory filesystem if booted without writable IDE drive
                self.disk_backed = false;
                self.format()
            }
        }
    }

    pub fn format(&mut self) -> Result<(), &'static str> {
        for i in 0..MAX_FILES {
            self.entries[i] = FileEntry::empty();
            self.cache[i] = None;
        }

        if self.disk_backed {
            let mut buf = [0u8; 512];
            buf[0..8].copy_from_slice(FS_MAGIC);
            let _ = ata::write_sector(SUPERBLOCK_LBA, &buf);
        }

        self.initialized = true;

        // Create standard Arch Linux FHS directories
        let default_dirs = [
            "/bin",
            "/etc",
            "/etc/pacman.d",
            "/home",
            "/root",
            "/tmp",
            "/usr",
            "/usr/bin",
            "/var",
            "/var/lib",
            "/var/lib/pacman",
            "/var/cache",
            "/var/cache/yay",
        ];
        for d in default_dirs.iter() {
            let _ = self.mkdir_internal(d, false);
        }

        // Populate standard Arch Linux configuration files
        let _ = self.write_file_internal(
            "/etc/os-release",
            b"NAME=\"RustOS Arch Edition\"\nPRETTY_NAME=\"RustOS (Arch Linux Inspired, x86_64)\"\nID=rustos\nID_LIKE=arch\nBUILD_ID=rolling\nANSI_COLOR=\"38;2;23;147;209\"\nHOME_URL=\"https://github.com/rappar222/rust-os\"\n",
            0o644,
            false,
        );

        let _ = self.write_file_internal("/etc/hostname", b"rustos\n", 0o644, false);

        let _ = self.write_file_internal(
            "/etc/pacman.conf",
            b"# /etc/pacman.conf - RustOS Pacman Configuration\n[options]\nHoldPkg     = pacman glibc-compat base\nArchitecture = x86_64\nColor\nCheckSpace\nParallelDownloads = 5\n\n[core]\nInclude = /etc/pacman.d/mirrorlist\n\n[extra]\nInclude = /etc/pacman.d/mirrorlist\n\n[community]\nInclude = /etc/pacman.d/mirrorlist\n\n[aur]\nServer = https://aur.archlinux.org/packages\n",
            0o644,
            false,
        );

        let _ = self.write_file_internal(
            "/etc/pacman.d/mirrorlist",
            b"## Arch Linux & RustOS Repository Mirrorlist\nServer = https://geo.mirror.pkgbuild.com/$repo/os/$arch\nServer = https://mirror.rackspace.com/archlinux/$repo/os/$arch\nServer = https://mirrors.kernel.org/archlinux/$repo/os/$arch\n",
            0o644,
            false,
        );

        let _ = self.write_file_internal(
            "/etc/passwd",
            b"root:x:0:0:Superuser:/root:/bin/rsh\narch:x:1000:1000:Arch User:/home/arch:/bin/rsh\n",
            0o644,
            false,
        );

        let _ = self.write_file_internal(
            "/etc/fstab",
            b"# <file system> <dir> <type>     <options> <dump> <pass>\n/dev/sda1       /     simplefs   defaults  0      1\ntmpfs           /tmp  tmpfs      defaults  0      0\n",
            0o644,
            false,
        );

        let _ = self.write_file_internal(
            "/root/.bashrc",
            b"# ~/.bashrc for RustOS rsh\nexport EDITOR=nano\nexport TERM=xterm-256color\nalias ll='ls -l'\nalias la='ls -a'\nalias pacs='pacman -S'\nalias pacu='pacman -Syu'\n",
            0o644,
            false,
        );

        let _ = self.write_file_internal(
            "/root/welcome.txt",
            b"Welcome to RustOS (Arch Linux Edition)!\n\nQuick Tips:\n  - Press TAB to autocomplete commands, flags, packages, and filenames.\n  - Press UP / DOWN arrows to cycle through command history.\n  - Run 'neofetch' or 'fastfetch' for system overview.\n  - Run 'pacman -Ss <query>' or 'pacman -S <pkg>' to install official packages.\n  - Run 'yay -S <pkg>' or 'paru -S <pkg>' to build & install AUR packages!\n  - Run 'makepkg -si' in /root to build the sample PKGBUILD!\n",
            0o644,
            false,
        );

        let _ = self.write_file_internal(
            "/root/PKGBUILD",
            b"# Maintainer: rappar222 <rappar2018@gmail.com>\npkgname=hello-rustos-git\npkgver=1.0.0\npkgrel=1\npkgdesc=\"Custom Arch package built with makepkg on RustOS\"\narch=('x86_64')\nurl=\"https://github.com/rappar222/rust-os\"\nlicense=('MIT')\nexec_output=\"Hello from custom AUR PKGBUILD compiled natively inside RustOS!\"\n",
            0o644,
            false,
        );

        self.save_metadata()?;
        Ok(())
    }

    pub fn save_metadata(&self) -> Result<(), &'static str> {
        if !self.disk_backed {
            return Ok(());
        }
        for sec in 0..METADATA_SECTORS {
            let lba = SUPERBLOCK_LBA + 1 + sec;
            let mut sec_buf = [0u8; 512];
            for slot in 0..8 {
                let idx = (sec as usize * 8) + slot;
                let offset = slot * 64;
                let bytes = unsafe {
                    core::slice::from_raw_parts(
                        &self.entries[idx] as *const FileEntry as *const u8,
                        64,
                    )
                };
                sec_buf[offset..offset + 64].copy_from_slice(bytes);
            }
            ata::write_sector(lba, &sec_buf)?;
        }
        Ok(())
    }

    pub fn is_dir(&self, path: &str) -> bool {
        if path == "/" {
            return true;
        }
        for entry in self.entries.iter() {
            if entry.used == 1 && entry.name_str() == path && entry.is_dir == 1 {
                return true;
            }
        }
        false
    }

    pub fn exists(&self, path: &str) -> bool {
        if path == "/" {
            return true;
        }
        self.entries
            .iter()
            .any(|e| e.used == 1 && e.name_str() == path)
    }

    fn mkdir_internal(&mut self, path: &str, flush: bool) -> Result<(), &'static str> {
        if path == "/" || self.is_dir(path) {
            return Ok(());
        }
        if path.len() >= 47 {
            return Err("Path too long (max 47 chars)");
        }
        if self.exists(path) {
            return Err("File already exists");
        }

        let idx = self
            .entries
            .iter()
            .position(|e| e.used == 0)
            .ok_or("Filesystem table full")?;

        let mut entry = FileEntry::empty();
        entry.name[..path.len()].copy_from_slice(path.as_bytes());
        entry.start_sector = 0;
        entry.size_bytes = 4096;
        entry.mode = 0o755;
        entry.is_dir = 1;
        entry.used = 1;
        self.entries[idx] = entry;

        if flush {
            self.save_metadata()?;
        }
        Ok(())
    }

    pub fn mkdir(&mut self, path: &str) -> Result<(), &'static str> {
        self.mkdir_internal(path, true)
    }

    pub fn list_dir(&self, dir_path: &str) -> Vec<FileEntry> {
        let mut results = Vec::new();
        let prefix = if dir_path == "/" {
            String::from("/")
        } else {
            format!("{}/", dir_path)
        };

        for entry in self.entries.iter() {
            if entry.used == 1 {
                let full_name = entry.name_str();
                if let Some(rest) = full_name.strip_prefix(&prefix) {
                    // Direct child only if `rest` does not contain another '/'
                    if !rest.is_empty() && !rest.contains('/') {
                        results.push(*entry);
                    }
                }
            }
        }
        results
    }

    pub fn list_all(&self) -> Vec<FileEntry> {
        self.entries
            .iter()
            .filter(|e| e.used == 1)
            .copied()
            .collect()
    }

    pub fn read_file(&mut self, path: &str) -> Result<Vec<u8>, &'static str> {
        for i in 0..MAX_FILES {
            if self.entries[i].used == 1 && self.entries[i].name_str() == path {
                if self.entries[i].is_dir == 1 {
                    return Err("Is a directory");
                }
                if let Some(ref cached) = self.cache[i] {
                    return Ok(cached.clone());
                }
                if !self.disk_backed {
                    return Ok(Vec::new());
                }
                let size_bytes = self.entries[i].size_bytes;
                let start_sector = self.entries[i].start_sector;
                let mut data = Vec::new();
                let sectors = (size_bytes + 511) / 512;
                for s in 0..sectors {
                    let mut buf = [0u8; 512];
                    ata::read_sector(start_sector + s, &mut buf)?;
                    let remaining = size_bytes as usize - data.len();
                    let to_take = core::cmp::min(remaining, 512);
                    data.extend_from_slice(&buf[..to_take]);
                }
                self.cache[i] = Some(data.clone());
                return Ok(data);
            }
        }
        Err("No such file or directory")
    }

    fn write_file_internal(
        &mut self,
        path: &str,
        data: &[u8],
        mode: u16,
        flush: bool,
    ) -> Result<(), &'static str> {
        if path.len() >= 47 {
            return Err("Path too long (max 47 chars)");
        }
        let sectors_needed = (data.len() as u32 + 511) / 512;
        if sectors_needed > MAX_SECTORS_PER_FILE {
            return Err("File too large (max 8 KB)");
        }

        let mut target_idx = None;
        for (i, entry) in self.entries.iter().enumerate() {
            if entry.used == 1 && entry.name_str() == path {
                if entry.is_dir == 1 {
                    return Err("Is a directory");
                }
                target_idx = Some(i);
                break;
            }
        }

        if target_idx.is_none() {
            for (i, entry) in self.entries.iter().enumerate() {
                if entry.used == 0 {
                    target_idx = Some(i);
                    break;
                }
            }
        }

        let idx = target_idx.ok_or("Filesystem full (max 64 entries)")?;
        let start_lba = DATA_START_LBA + (idx as u32 * MAX_SECTORS_PER_FILE);

        if self.disk_backed {
            for (s, chunk) in data.chunks(512).enumerate() {
                let mut sec_buf = [0u8; 512];
                sec_buf[..chunk.len()].copy_from_slice(chunk);
                let _ = ata::write_sector(start_lba + s as u32, &sec_buf);
            }
        }

        let mut entry = FileEntry::empty();
        entry.name[..path.len()].copy_from_slice(path.as_bytes());
        entry.start_sector = start_lba;
        entry.size_bytes = data.len() as u32;
        entry.mode = mode;
        entry.is_dir = 0;
        entry.used = 1;
        self.entries[idx] = entry;
        self.cache[idx] = Some(Vec::from(data));

        if flush {
            self.save_metadata()?;
        }
        Ok(())
    }

    pub fn write_file(&mut self, path: &str, data: &[u8]) -> Result<(), &'static str> {
        self.write_file_internal(path, data, 0o644, true)
    }

    pub fn write_executable(&mut self, path: &str, data: &[u8]) -> Result<(), &'static str> {
        self.write_file_internal(path, data, 0o755, true)
    }

    pub fn chmod(&mut self, path: &str, mode: u16) -> Result<(), &'static str> {
        for entry in self.entries.iter_mut() {
            if entry.used == 1 && entry.name_str() == path {
                entry.mode = mode;
                self.save_metadata()?;
                return Ok(());
            }
        }
        Err("No such file or directory")
    }

    pub fn delete(&mut self, path: &str, recursive: bool) -> Result<(), &'static str> {
        if path == "/" {
            return Err("Cannot remove root directory '/'");
        }
        // Check if directory has children
        if self.is_dir(path) {
            let children = self.list_dir(path);
            if !children.is_empty() && !recursive {
                return Err("Directory not empty (use -r)");
            }
            if recursive {
                let prefix = format!("{}/", path);
                for i in 0..MAX_FILES {
                    if self.entries[i].used == 1 && self.entries[i].name_str().starts_with(&prefix) {
                        self.entries[i].used = 0;
                        self.cache[i] = None;
                    }
                }
            }
        }

        for i in 0..MAX_FILES {
            if self.entries[i].used == 1 && self.entries[i].name_str() == path {
                self.entries[i].used = 0;
                self.cache[i] = None;
                self.save_metadata()?;
                return Ok(());
            }
        }
        Err("No such file or directory")
    }
}

pub static FS: Mutex<SimpleFs> = Mutex::new(SimpleFs::new());

pub fn init() -> Result<(), &'static str> {
    FS.lock().init()
}

pub fn is_dir(path: &str) -> bool {
    FS.lock().is_dir(path)
}

pub fn exists(path: &str) -> bool {
    FS.lock().exists(path)
}

pub fn mkdir(path: &str) -> Result<(), &'static str> {
    FS.lock().mkdir(path)
}

pub fn list_dir(dir_path: &str) -> Vec<FileEntry> {
    FS.lock().list_dir(dir_path)
}

pub fn list_all() -> Vec<FileEntry> {
    FS.lock().list_all()
}

pub fn read_file(path: &str) -> Result<Vec<u8>, &'static str> {
    FS.lock().read_file(path)
}

pub fn write_file(path: &str, data: &[u8]) -> Result<(), &'static str> {
    FS.lock().write_file(path, data)
}

pub fn write_executable(path: &str, data: &[u8]) -> Result<(), &'static str> {
    FS.lock().write_executable(path, data)
}

pub fn chmod(path: &str, mode: u16) -> Result<(), &'static str> {
    FS.lock().chmod(path, mode)
}

pub fn delete(path: &str, recursive: bool) -> Result<(), &'static str> {
    FS.lock().delete(path, recursive)
}

pub fn stat(path: &str) -> Option<FileEntry> {
    let fs = FS.lock();
    for entry in fs.entries.iter() {
        if entry.used == 1 && entry.name_str() == path {
            return Some(*entry);
        }
    }
    None
}

pub fn append_file(path: &str, extra: &[u8]) -> Result<(), &'static str> {
    let mut existing = read_file(path).unwrap_or_default();
    existing.extend_from_slice(extra);
    write_file(path, &existing)
}

pub fn touch(path: &str) -> Result<(), &'static str> {
    if exists(path) {
        Ok(())
    } else {
        write_file(path, b"")
    }
}

pub fn copy_file(src: &str, dst: &str) -> Result<(), &'static str> {
    let data = read_file(src)?;
    let mode = stat(src).map(|e| e.mode).unwrap_or(0o644);
    FS.lock().write_file_internal(dst, &data, mode, true)
}

pub fn move_path(src: &str, dst: &str) -> Result<(), &'static str> {
    copy_file(src, dst)?;
    delete(src, false)
}

pub fn stats() -> (usize, usize, bool) {
    let fs = FS.lock();
    let total_kb = 26 * 1024;
    let mut used_bytes = 0usize;
    for e in fs.entries.iter() {
        if e.used == 1 {
            used_bytes += (e.size_bytes as usize).max(512);
        }
    }
    let used_kb = (used_bytes / 1024).max(32);
    (total_kb, used_kb, fs.disk_backed)
}
