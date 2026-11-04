extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use spin::Mutex;
use crate::ata;

const FS_MAGIC: &[u8; 8] = b"RUSTFS01";
const SUPERBLOCK_LBA: u32 = 12000;
const DATA_START_LBA: u32 = 12010;
const MAX_FILES: usize = 32;
const MAX_SECTORS_PER_FILE: u32 = 16; // 8 KB per file

#[derive(Clone, Copy)]
#[repr(C)]
pub struct FileEntry {
    pub name: [u8; 32],
    pub start_sector: u32,
    pub size_bytes: u32,
    pub used: u8,
    pub _reserved: [u8; 7],
}

impl FileEntry {
    pub const fn empty() -> Self {
        Self {
            name: [0; 32],
            start_sector: 0,
            size_bytes: 0,
            used: 0,
            _reserved: [0; 7],
        }
    }

    pub fn name_str(&self) -> &str {
        let len = self.name.iter().position(|&b| b == 0).unwrap_or(self.name.len());
        core::str::from_utf8(&self.name[..len]).unwrap_or("<invalid utf8>")
    }
}

pub struct SimpleFs {
    entries: [FileEntry; MAX_FILES],
    initialized: bool,
}

impl SimpleFs {
    pub const fn new() -> Self {
        Self {
            entries: [FileEntry::empty(); MAX_FILES],
            initialized: false,
        }
    }

    pub fn init(&mut self) -> Result<(), &'static str> {
        let mut buf = [0u8; 512];
        ata::read_sector(SUPERBLOCK_LBA, &mut buf)?;

        if &buf[0..8] == FS_MAGIC {
            // Read file entries (32 entries * 48 bytes = 1536 bytes = 3 sectors)
            for i in 0..MAX_FILES {
                let sector_idx = SUPERBLOCK_LBA + 1 + (i as u32 / 10);
                let entry_offset = (i % 10) * 48;
                let mut sec_buf = [0u8; 512];
                ata::read_sector(sector_idx, &mut sec_buf)?;

                let entry_slice = &sec_buf[entry_offset..entry_offset + 48];
                unsafe {
                    let entry_ptr = entry_slice.as_ptr() as *const FileEntry;
                    self.entries[i] = *entry_ptr;
                }
            }
            self.initialized = true;
            Ok(())
        } else {
            // Format new filesystem on disk
            self.format()
        }
    }

    pub fn format(&mut self) -> Result<(), &'static str> {
        let mut buf = [0u8; 512];
        buf[0..8].copy_from_slice(FS_MAGIC);
        ata::write_sector(SUPERBLOCK_LBA, &buf)?;

        for entry in self.entries.iter_mut() {
            *entry = FileEntry::empty();
        }

        // Write default initial files
        self.initialized = true;
        self.save_metadata()?;

        let welcome_data = b"Welcome to RustOS Persistent Storage!\nThis file is stored in real physical disk sectors (ATA PIO).\nChanges you write here survive reboots!\n";
        let _ = self.write_file("welcome.txt", welcome_data);

        let os_data = b"OS: RustOS\nArchitecture: x86_64\nKernel: Custom Monolithic Bare-Metal\nFilesystem: SimpleFS\n";
        let _ = self.write_file("os_info.txt", os_data);

        Ok(())
    }

    fn save_metadata(&self) -> Result<(), &'static str> {
        for sector_idx in 0..4 {
            let lba = SUPERBLOCK_LBA + 1 + sector_idx;
            let mut sec_buf = [0u8; 512];
            for i in 0..10 {
                let file_idx = (sector_idx as usize * 10) + i;
                if file_idx < MAX_FILES {
                    let entry_offset = i * 48;
                    let bytes = unsafe {
                        core::slice::from_raw_parts(
                            &self.entries[file_idx] as *const FileEntry as *const u8,
                            48,
                        )
                    };
                    sec_buf[entry_offset..entry_offset + 48].copy_from_slice(bytes);
                }
            }
            ata::write_sector(lba, &sec_buf)?;
        }
        Ok(())
    }

    pub fn list_files(&self) -> Vec<(String, u32)> {
        let mut list = Vec::new();
        for entry in self.entries.iter() {
            if entry.used == 1 {
                list.push((String::from(entry.name_str()), entry.size_bytes));
            }
        }
        list
    }

    pub fn read_file(&self, name: &str) -> Result<Vec<u8>, &'static str> {
        for entry in self.entries.iter() {
            if entry.used == 1 && entry.name_str() == name {
                let mut data = Vec::new();
                let sectors = (entry.size_bytes + 511) / 512;
                for s in 0..sectors {
                    let mut buf = [0u8; 512];
                    ata::read_sector(entry.start_sector + s, &mut buf)?;
                    let remaining = entry.size_bytes as usize - data.len();
                    let to_take = core::cmp::min(remaining, 512);
                    data.extend_from_slice(&buf[..to_take]);
                }
                return Ok(data);
            }
        }
        Err("File not found")
    }

    pub fn write_file(&mut self, name: &str, data: &[u8]) -> Result<(), &'static str> {
        if name.len() > 30 {
            return Err("Filename too long (max 30 characters)");
        }
        let sectors_needed = (data.len() as u32 + 511) / 512;
        if sectors_needed > MAX_SECTORS_PER_FILE {
            return Err("File too large (max 8 KB)");
        }

        // Find existing or free entry
        let mut target_idx = None;
        for (i, entry) in self.entries.iter().enumerate() {
            if entry.used == 1 && entry.name_str() == name {
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

        let idx = target_idx.ok_or("Filesystem full (maximum 32 files)")?;
        let start_lba = DATA_START_LBA + (idx as u32 * MAX_SECTORS_PER_FILE);

        // Write data sectors to disk
        for (s, chunk) in data.chunks(512).enumerate() {
            let mut sec_buf = [0u8; 512];
            sec_buf[..chunk.len()].copy_from_slice(chunk);
            ata::write_sector(start_lba + s as u32, &sec_buf)?;
        }

        // Update directory entry
        let mut entry = FileEntry::empty();
        entry.name[..name.len()].copy_from_slice(name.as_bytes());
        entry.start_sector = start_lba;
        entry.size_bytes = data.len() as u32;
        entry.used = 1;
        self.entries[idx] = entry;

        self.save_metadata()?;
        Ok(())
    }

    pub fn delete_file(&mut self, name: &str) -> Result<(), &'static str> {
        for entry in self.entries.iter_mut() {
            if entry.used == 1 && entry.name_str() == name {
                entry.used = 0;
                self.save_metadata()?;
                return Ok(());
            }
        }
        Err("File not found")
    }
}

pub static FS: Mutex<SimpleFs> = Mutex::new(SimpleFs::new());

pub fn init() -> Result<(), &'static str> {
    FS.lock().init()
}

pub fn list_files() -> Vec<(String, u32)> {
    FS.lock().list_files()
}

pub fn read_file(name: &str) -> Result<Vec<u8>, &'static str> {
    FS.lock().read_file(name)
}

pub fn write_file(name: &str, data: &[u8]) -> Result<(), &'static str> {
    FS.lock().write_file(name, data)
}

pub fn delete_file(name: &str) -> Result<(), &'static str> {
    FS.lock().delete_file(name)
}
