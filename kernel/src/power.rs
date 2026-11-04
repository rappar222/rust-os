use x86_64::instructions::port::Port;

pub fn reboot() -> ! {
    crate::println!("Rebooting system...");
    
    // 1. Try standard 8042 keyboard controller reset
    let mut kbd_ctrl: Port<u8> = Port::new(0x64);
    unsafe {
        // Wait for keyboard controller buffer to clear
        while (kbd_ctrl.read() & 0x02) != 0 {}
        kbd_ctrl.write(0xFE); // Pulse reset line
    }

    // 2. Fallback: Triple fault via empty IDT
    use x86_64::structures::idt::InterruptDescriptorTable;
    static EMPTY_IDT: InterruptDescriptorTable = InterruptDescriptorTable::new();
    EMPTY_IDT.load();
    unsafe {
        core::arch::asm!("int 3");
    }

    loop {
        x86_64::instructions::hlt();
    }
}

pub fn shutdown() -> ! {
    crate::println!("Powering off system...");

    // 1. QEMU ACPI poweroff (standard in modern QEMU: outw(0x604, 0x2000))
    let mut acpi_port: Port<u16> = Port::new(0x604);
    unsafe {
        acpi_port.write(0x2000);
    }

    // 2. QEMU isa-debug-exit device (outb(0xf4, 0x00))
    let mut debug_exit: Port<u8> = Port::new(0xf4);
    unsafe {
        debug_exit.write(0x00);
    }

    // 3. VirtualBox poweroff
    let mut vbox_port: Port<u16> = Port::new(0x4004);
    unsafe {
        vbox_port.write(0x3400);
    }

    crate::println!("System halted. You can safely close QEMU.");
    loop {
        x86_64::instructions::interrupts::disable();
        x86_64::instructions::hlt();
    }
}
