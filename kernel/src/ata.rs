use spin::Mutex;
use x86_64::instructions::port::{Port, PortReadOnly, PortWriteOnly};

const ATA_PRIMARY_DATA: u16 = 0x1F0;
const ATA_PRIMARY_SEC_COUNT: u16 = 0x1F2;
const ATA_PRIMARY_LBA_LO: u16 = 0x1F3;
const ATA_PRIMARY_LBA_MID: u16 = 0x1F4;
const ATA_PRIMARY_LBA_HI: u16 = 0x1F5;
const ATA_PRIMARY_DRIVE_HEAD: u16 = 0x1F6;
const ATA_PRIMARY_STATUS: u16 = 0x1F7;
const ATA_PRIMARY_COMMAND: u16 = 0x1F7;
const ATA_PRIMARY_DCR: u16 = 0x3F6;

const ATA_CMD_READ_SECTORS: u8 = 0x20;
const ATA_CMD_WRITE_SECTORS: u8 = 0x30;
const ATA_CMD_IDENTIFY: u8 = 0xEC;
const ATA_CMD_CACHE_FLUSH: u8 = 0xE7;

const STATUS_BSY: u8 = 0x80; // Busy
const STATUS_DRQ: u8 = 0x08; // Data Request
const STATUS_ERR: u8 = 0x01; // Error

#[derive(Debug, Clone)]
pub struct DriveInfo {
    pub present: bool,
    pub model: [u8; 40],
    pub sectors: u32,
    pub size_mb: u32,
}

impl DriveInfo {
    pub fn model_str(&self) -> &str {
        if let Ok(s) = core::str::from_utf8(&self.model) {
            s.trim()
        } else {
            "Unknown Drive"
        }
    }
}

pub struct AtaController {
    data_port: Port<u16>,
    sec_count_port: Port<u8>,
    lba_lo_port: Port<u8>,
    lba_mid_port: Port<u8>,
    lba_hi_port: Port<u8>,
    drive_head_port: Port<u8>,
    status_port: PortReadOnly<u8>,
    cmd_port: PortWriteOnly<u8>,
    drive_info: Option<DriveInfo>,
}

impl AtaController {
    pub const fn new() -> Self {
        Self {
            data_port: Port::new(ATA_PRIMARY_DATA),
            sec_count_port: Port::new(ATA_PRIMARY_SEC_COUNT),
            lba_lo_port: Port::new(ATA_PRIMARY_LBA_LO),
            lba_mid_port: Port::new(ATA_PRIMARY_LBA_MID),
            lba_hi_port: Port::new(ATA_PRIMARY_LBA_HI),
            drive_head_port: Port::new(ATA_PRIMARY_DRIVE_HEAD),
            status_port: PortReadOnly::new(ATA_PRIMARY_STATUS),
            cmd_port: PortWriteOnly::new(ATA_PRIMARY_COMMAND),
            drive_info: None,
        }
    }

    fn wait_ready(&mut self) -> Result<u8, &'static str> {
        for _ in 0..100_000 {
            let status = unsafe { self.status_port.read() };
            if (status & STATUS_BSY) == 0 {
                return Ok(status);
            }
        }
        Err("ATA disk busy timeout")
    }

    fn wait_drq(&mut self) -> Result<u8, &'static str> {
        for _ in 0..100_000 {
            let status = unsafe { self.status_port.read() };
            if (status & STATUS_ERR) != 0 {
                return Err("ATA disk error");
            }
            if (status & STATUS_BSY) == 0 && (status & STATUS_DRQ) != 0 {
                return Ok(status);
            }
        }
        Err("ATA wait DRQ timeout")
    }

    pub fn identify(&mut self) -> Result<DriveInfo, &'static str> {
        let mut dcr_port: Port<u8> = Port::new(ATA_PRIMARY_DCR);
        unsafe {
            // Disable ATA interrupts (run in polling PIO mode)
            dcr_port.write(0x02);

            // Select Master drive
            self.drive_head_port.write(0xA0);
            self.sec_count_port.write(0);
            self.lba_lo_port.write(0);
            self.lba_mid_port.write(0);
            self.lba_hi_port.write(0);
            self.cmd_port.write(ATA_CMD_IDENTIFY);
        }

        let status = unsafe { self.status_port.read() };
        if status == 0 || status == 0xFF {
            return Err("No ATA drive detected on Primary bus");
        }

        self.wait_ready()?;
        self.wait_drq()?;

        let mut buffer = [0u16; 256];
        for word in buffer.iter_mut() {
            *word = unsafe { self.data_port.read() };
        }

        // Model string: words 27..46 (byte-swapped)
        let mut model = [0u8; 40];
        for i in 0..20 {
            let w = buffer[27 + i];
            model[i * 2] = (w >> 8) as u8;
            model[i * 2 + 1] = (w & 0xFF) as u8;
        }

        // Total sectors: words 60 and 61
        let sectors = ((buffer[61] as u32) << 16) | (buffer[60] as u32);
        let size_mb = sectors / 2048; // 512 bytes * 2048 = 1 MB

        let info = DriveInfo {
            present: true,
            model,
            sectors,
            size_mb,
        };
        self.drive_info = Some(info.clone());
        Ok(info)
    }

    pub fn read_sector(&mut self, lba: u32, buf: &mut [u8; 512]) -> Result<(), &'static str> {
        self.wait_ready()?;

        unsafe {
            self.drive_head_port.write(0xE0 | (((lba >> 24) & 0x0F) as u8));
            self.sec_count_port.write(1);
            self.lba_lo_port.write((lba & 0xFF) as u8);
            self.lba_mid_port.write(((lba >> 8) & 0xFF) as u8);
            self.lba_hi_port.write(((lba >> 16) & 0xFF) as u8);
            self.cmd_port.write(ATA_CMD_READ_SECTORS);
        }

        self.wait_drq()?;

        for chunk in buf.chunks_exact_mut(2) {
            let word = unsafe { self.data_port.read() };
            chunk[0] = (word & 0xFF) as u8;
            chunk[1] = (word >> 8) as u8;
        }

        Ok(())
    }

    pub fn write_sector(&mut self, lba: u32, buf: &[u8; 512]) -> Result<(), &'static str> {
        self.wait_ready()?;

        unsafe {
            self.drive_head_port.write(0xE0 | (((lba >> 24) & 0x0F) as u8));
            self.sec_count_port.write(1);
            self.lba_lo_port.write((lba & 0xFF) as u8);
            self.lba_mid_port.write(((lba >> 8) & 0xFF) as u8);
            self.lba_hi_port.write(((lba >> 16) & 0xFF) as u8);
            self.cmd_port.write(ATA_CMD_WRITE_SECTORS);
        }

        self.wait_drq()?;

        for chunk in buf.chunks_exact(2) {
            let word = (chunk[0] as u16) | ((chunk[1] as u16) << 8);
            unsafe { self.data_port.write(word) };
        }

        // Flush cache
        self.wait_ready()?;
        unsafe { self.cmd_port.write(ATA_CMD_CACHE_FLUSH) };
        self.wait_ready()?;

        Ok(())
    }

    pub fn get_info(&self) -> Option<&DriveInfo> {
        self.drive_info.as_ref()
    }
}

pub static ATA_DRIVE: Mutex<AtaController> = Mutex::new(AtaController::new());

pub fn init() -> Result<DriveInfo, &'static str> {
    ATA_DRIVE.lock().identify()
}

pub fn read_sector(lba: u32, buf: &mut [u8; 512]) -> Result<(), &'static str> {
    ATA_DRIVE.lock().read_sector(lba, buf)
}

pub fn write_sector(lba: u32, buf: &[u8; 512]) -> Result<(), &'static str> {
    ATA_DRIVE.lock().write_sector(lba, buf)
}
