extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use spin::Mutex;
use x86_64::instructions::port::Port;

#[derive(Clone)]
pub struct CpuInfo {
    pub vendor: String,
    pub model_name: String,
    pub family: u8,
    pub model: u8,
    pub stepping: u8,
    pub cores: u8,
    pub flags: Vec<&'static str>,
}

pub fn get_cpu_info() -> CpuInfo {
    use core::arch::x86_64::__cpuid;

    // 1. Vendor ID (Leaf 0)
    let leaf0 = __cpuid(0);
    let mut vendor_bytes = [0u8; 12];
    vendor_bytes[0..4].copy_from_slice(&leaf0.ebx.to_le_bytes());
    vendor_bytes[4..8].copy_from_slice(&leaf0.edx.to_le_bytes());
    vendor_bytes[8..12].copy_from_slice(&leaf0.ecx.to_le_bytes());
    let vendor = String::from(core::str::from_utf8(&vendor_bytes).unwrap_or("Unknown").trim());

    // 2. Family, Model, Stepping, Features (Leaf 1)
    let leaf1 = __cpuid(1);
    let stepping = (leaf1.eax & 0xF) as u8;
    let base_model = ((leaf1.eax >> 4) & 0xF) as u8;
    let base_family = ((leaf1.eax >> 8) & 0xF) as u8;
    let ext_model = ((leaf1.eax >> 16) & 0xF) as u8;
    let ext_family = ((leaf1.eax >> 20) & 0xFF) as u8;

    let family = if base_family == 0xF {
        base_family + ext_family
    } else {
        base_family
    };
    let model = if base_family == 0x6 || base_family == 0xF {
        (ext_model << 4) | base_model
    } else {
        base_model
    };

    let logical_cpus = ((leaf1.ebx >> 16) & 0xFF) as u8;
    let cores = if logical_cpus > 0 { logical_cpus } else { 1 };

    let mut flags = Vec::new();
    if (leaf1.edx & (1 << 0)) != 0 { flags.push("fpu"); }
    if (leaf1.edx & (1 << 4)) != 0 { flags.push("tsc"); }
    if (leaf1.edx & (1 << 5)) != 0 { flags.push("msr"); }
    if (leaf1.edx & (1 << 6)) != 0 { flags.push("pae"); }
    if (leaf1.edx & (1 << 9)) != 0 { flags.push("apic"); }
    if (leaf1.edx & (1 << 23)) != 0 { flags.push("mmx"); }
    if (leaf1.edx & (1 << 24)) != 0 { flags.push("fxsr"); }
    if (leaf1.edx & (1 << 25)) != 0 { flags.push("sse"); }
    if (leaf1.edx & (1 << 26)) != 0 { flags.push("sse2"); }
    if (leaf1.ecx & (1 << 0)) != 0 { flags.push("sse3"); }
    if (leaf1.ecx & (1 << 9)) != 0 { flags.push("ssse3"); }
    if (leaf1.ecx & (1 << 19)) != 0 { flags.push("sse4_1"); }
    if (leaf1.ecx & (1 << 20)) != 0 { flags.push("sse4_2"); }
    if (leaf1.ecx & (1 << 28)) != 0 { flags.push("avx"); }
    if (leaf1.ecx & (1 << 31)) != 0 { flags.push("hypervisor"); }

    // 3. Processor Brand String (Leaves 0x80000002 .. 0x80000004)
    let ext_max = __cpuid(0x80000000).eax;
    let model_name = if ext_max >= 0x80000004 {
        let mut brand_bytes = [0u8; 48];
        for i in 0..3 {
            let leaf = __cpuid(0x80000002 + i as u32);
            let offset = i * 16;
            brand_bytes[offset..offset + 4].copy_from_slice(&leaf.eax.to_le_bytes());
            brand_bytes[offset + 4..offset + 8].copy_from_slice(&leaf.ebx.to_le_bytes());
            brand_bytes[offset + 8..offset + 12].copy_from_slice(&leaf.ecx.to_le_bytes());
            brand_bytes[offset + 12..offset + 16].copy_from_slice(&leaf.edx.to_le_bytes());
        }
        let len = brand_bytes.iter().position(|&b| b == 0).unwrap_or(48);
        String::from(core::str::from_utf8(&brand_bytes[..len]).unwrap_or("x86_64 Processor").trim())
    } else {
        format!("{} x86_64 Processor", vendor)
    };

    CpuInfo {
        vendor,
        model_name,
        family,
        model,
        stepping,
        cores,
        flags,
    }
}

pub struct PciDevice {
    pub bus: u8,
    pub slot: u8,
    pub func: u8,
    pub vendor_id: u16,
    pub device_id: u16,
    pub class_id: u8,
    pub subclass_id: u8,
}

impl PciDevice {
    pub fn class_name(&self) -> &'static str {
        match (self.class_id, self.subclass_id) {
            (0x01, 0x01) => "IDE interface",
            (0x01, 0x06) => "SATA controller",
            (0x01, 0x08) => "Non-Volatile memory controller",
            (0x02, 0x00) => "Ethernet controller",
            (0x03, 0x00) => "VGA compatible controller",
            (0x04, 0x00) => "Multimedia video controller",
            (0x04, 0x01) => "Multimedia audio controller",
            (0x04, 0x03) => "Audio device",
            (0x06, 0x00) => "Host bridge",
            (0x06, 0x01) => "ISA bridge",
            (0x06, 0x04) => "PCI bridge",
            (0x06, 0x80) => "Bridge",
            (0x08, 0x80) => "System peripheral",
            (0x0C, 0x03) => "USB controller",
            (0x0C, 0x05) => "SMBus",
            _ => "PCI Device",
        }
    }

    pub fn vendor_name(&self) -> &'static str {
        match self.vendor_id {
            0x8086 => "Intel Corporation",
            0x1022 => "Advanced Micro Devices, Inc. [AMD]",
            0x10EC => "Realtek Semiconductor Co., Ltd.",
            0x1234 => "QEMU / Bochs Virtual Graphics",
            0x80EE => "InnoTek Systemberatung GmbH (VirtualBox)",
            0x1AF4 => "Red Hat, Inc. (VirtIO)",
            0x106B => "Apple Inc.",
            0x15AD => "VMware",
            _ => "Generic Hardware Vendor",
        }
    }

    pub fn device_desc(&self) -> &'static str {
        match (self.vendor_id, self.device_id) {
            (0x8086, 0x1237) => "440FX - 82441FX PMC [Natoma]",
            (0x8086, 0x7000) => "82371SB PIIX3 ISA [Natoma/Triton II]",
            (0x8086, 0x7010) => "82371SB PIIX3 IDE [Natoma/Triton II]",
            (0x8086, 0x7111) => "82371AB/EB/MB PIIX4 IDE",
            (0x8086, 0x7113) => "82371AB/EB/MB PIIX4 ACPI",
            (0x8086, 0x100E) => "82540EM Gigabit Ethernet Controller",
            (0x8086, 0x2415) => "82801AA AC'97 Audio Controller",
            (0x8086, 0x2668) => "82801FB/FBM/FR/FW/FRW High Definition Audio",
            (0x8086, 0x2829) => "82801HM/HEM (ICH8M/ICH8M-E) SATA Controller [AHCI]",
            (0x1022, 0x2000) => "79c970 [PCnet32 LANCE]",
            (0x80EE, 0xBEEF) => "VirtualBox Graphics Adapter (VBoxVGA)",
            (0x80EE, 0xCAFE) => "VirtualBox Guest Service",
            (0x1234, 0x1111) => "QEMU Standard VGA",
            (0x106B, 0x003F) => "KeyLargo/Intrepid USB",
            _ => "Device",
        }
    }
}

fn pci_read_u32(bus: u8, slot: u8, func: u8, offset: u8) -> u32 {
    let address: u32 = 0x8000_0000
        | ((bus as u32) << 16)
        | ((slot as u32) << 11)
        | ((func as u32) << 8)
        | ((offset as u32) & 0xFC);

    let mut addr_port: Port<u32> = Port::new(0xCF8);
    let mut data_port: Port<u32> = Port::new(0xCFC);

    unsafe {
        addr_port.write(address);
        data_port.read()
    }
}

pub fn scan_pci() -> Vec<PciDevice> {
    let mut devices = Vec::new();
    for bus in 0..4 {
        for slot in 0..32 {
            for func in 0..8 {
                let reg0 = pci_read_u32(bus, slot, func, 0x00);
                let vendor_id = (reg0 & 0xFFFF) as u16;
                if vendor_id == 0xFFFF || vendor_id == 0x0000 {
                    if func == 0 {
                        break;
                    }
                    continue;
                }
                let device_id = ((reg0 >> 16) & 0xFFFF) as u16;
                let reg2 = pci_read_u32(bus, slot, func, 0x08);
                let class_id = ((reg2 >> 24) & 0xFF) as u8;
                let subclass_id = ((reg2 >> 16) & 0xFF) as u8;

                devices.push(PciDevice {
                    bus,
                    slot,
                    func,
                    vendor_id,
                    device_id,
                    class_id,
                    subclass_id,
                });

                if func == 0 {
                    let reg3 = pci_read_u32(bus, slot, 0, 0x0C);
                    let header_type = ((reg3 >> 16) & 0xFF) as u8;
                    if (header_type & 0x80) == 0 {
                        break;
                    }
                }
            }
        }
    }
    devices
}

pub static DMESG_LOG: Mutex<Vec<(u64, String)>> = Mutex::new(Vec::new());

pub fn klog(msg: &str) {
    let ticks = crate::interrupts::ticks();
    let mut log = DMESG_LOG.lock();
    if log.len() < 128 {
        log.push((ticks, String::from(msg)));
    }
}

pub fn get_dmesg() -> Vec<(u64, String)> {
    DMESG_LOG.lock().clone()
}
