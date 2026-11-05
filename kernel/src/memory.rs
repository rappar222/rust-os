use bootloader_api::info::{MemoryRegionKind, MemoryRegions};

pub struct MemoryStats {
    pub total_usable_bytes: u64,
    pub total_memory_bytes: u64,
    pub usable_regions: usize,
    pub total_regions: usize,
}

pub fn inspect_memory(regions: &MemoryRegions) -> MemoryStats {
    let mut total_usable: u64 = 0;
    let mut total_mem: u64 = 0;
    let mut usable_count = 0;
    let mut total_count = 0;

    for region in regions.iter() {
        let size = region.end - region.start;
        total_mem += size;
        total_count += 1;
        if region.kind == MemoryRegionKind::Usable {
            total_usable += size;
            usable_count += 1;
        }
    }

    MemoryStats {
        total_usable_bytes: total_usable,
        total_memory_bytes: total_mem,
        usable_regions: usable_count,
        total_regions: total_count,
    }
}

pub fn init_heap_from_boot_info(
    regions: &'static mut MemoryRegions,
    phys_offset: u64,
) -> Result<(usize, usize), &'static str> {
    const HEAP_SIZE: usize = 32 * 1024 * 1024; // 32 MiB heap

    // Find the largest usable memory region
    let mut best_region_idx = None;
    let mut max_size = 0;

    for (i, region) in regions.iter().enumerate() {
        if region.kind == MemoryRegionKind::Usable {
            let size = region.end - region.start;
            if size > max_size {
                max_size = size;
                best_region_idx = Some(i);
            }
        }
    }

    let idx = best_region_idx.ok_or("No usable memory region found for heap allocation")?;
    let target_region = &mut regions[idx];

    if (target_region.end - target_region.start) < (HEAP_SIZE as u64) {
        return Err("Usable memory region is smaller than requested heap size");
    }

    // Allocate from the beginning of this region
    let phys_start = target_region.start;
    target_region.start += HEAP_SIZE as u64;

    let virt_start = (phys_offset + phys_start) as usize;
    crate::allocator::init(virt_start, HEAP_SIZE);

    Ok((virt_start, HEAP_SIZE))
}
