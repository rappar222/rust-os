use linked_list_allocator::LockedHeap;
use spin::Mutex;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

static HEAP_INFO: Mutex<HeapInfo> = Mutex::new(HeapInfo {
    start: 0,
    size: 0,
    initialized: false,
});

pub struct HeapInfo {
    pub start: usize,
    pub size: usize,
    pub initialized: bool,
}

pub fn init(heap_start: usize, heap_size: usize) {
    unsafe {
        ALLOCATOR.lock().init(heap_start as *mut u8, heap_size);
    }
    let mut info = HEAP_INFO.lock();
    info.start = heap_start;
    info.size = heap_size;
    info.initialized = true;
}

pub fn get_heap_info() -> (usize, usize, bool) {
    let info = HEAP_INFO.lock();
    (info.start, info.size, info.initialized)
}
