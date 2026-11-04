use core::sync::atomic::{AtomicU64, Ordering};
use pic8259::ChainedPics;
use spin::{Lazy, Mutex};
use x86_64::instructions::port::Port;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};
use crate::{gdt, println};

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: Mutex<ChainedPics> =
    Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard = PIC_1_OFFSET + 1,
}

impl InterruptIndex {
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

static TICKS: AtomicU64 = AtomicU64::new(0);

pub fn ticks() -> u64 {
    TICKS.load(Ordering::Relaxed)
}

pub fn uptime_seconds() -> u64 {
    ticks() / 100
}

static IDT: Lazy<InterruptDescriptorTable> = Lazy::new(|| {
    let mut idt = InterruptDescriptorTable::new();

    // CPU Exceptions
    idt.breakpoint.set_handler_fn(breakpoint_handler);
    unsafe {
        idt.double_fault
            .set_handler_fn(double_fault_handler)
            .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
    }
    idt.page_fault.set_handler_fn(page_fault_handler);
    idt.general_protection_fault.set_handler_fn(general_protection_fault_handler);

    // Fallback default handler for all hardware PIC interrupts (32..48)
    for i in PIC_1_OFFSET..PIC_2_OFFSET + 8 {
        idt[i].set_handler_fn(default_irq_handler);
    }

    // Hardware PIC interrupts
    idt[InterruptIndex::Timer.as_u8()].set_handler_fn(timer_interrupt_handler);
    idt[InterruptIndex::Keyboard.as_u8()].set_handler_fn(keyboard_interrupt_handler);

    idt
});

pub fn init_idt() {
    IDT.load();
}

pub fn init_pit() {
    // Configure PIT Channel 0 to generate 100 Hz interrupts
    // 1193182 Hz / 100 = 11931 = 0x2E9B
    const PIT_DIVISOR: u16 = 11931;
    let mut command_port: Port<u8> = Port::new(0x43);
    let mut data_port: Port<u8> = Port::new(0x40);

    unsafe {
        command_port.write(0x36); // Channel 0, lobyte/hibyte, rate generator
        data_port.write((PIT_DIVISOR & 0xFF) as u8);
        data_port.write((PIT_DIVISOR >> 8) as u8);
    }
}

pub fn init() {
    init_idt();
    unsafe {
        PICS.lock().initialize();
        PICS.lock().write_masks(0b1111_1000, 0b1111_1111);
    };
    init_pit();
    x86_64::instructions::interrupts::enable();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("[EXCEPTION] BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("[EXCEPTION] DOUBLE FAULT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn general_protection_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    println!(
        "[EXCEPTION] GENERAL PROTECTION FAULT (code: {:#x})\n{:#?}",
        error_code, stack_frame
    );
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;

    println!("[EXCEPTION] PAGE FAULT");
    println!("Accessed Address: {:?}", Cr2::read());
    println!("Error Code: {:?}", error_code);
    println!("{:#?}", stack_frame);
    panic!("Execution halted due to unhandled page fault.");
}

extern "x86-interrupt" fn default_irq_handler(_stack_frame: InterruptStackFrame) {
    unsafe {
        // Send EOI to both PICs safely
        let mut port_pic1: Port<u8> = Port::new(0x20);
        let mut port_pic2: Port<u8> = Port::new(0xA0);
        port_pic2.write(0x20);
        port_pic1.write(0x20);
    }
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    TICKS.fetch_add(1, Ordering::Relaxed);

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    crate::keyboard::handle_keyboard_interrupt();

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}
