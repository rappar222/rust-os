use core::fmt::{self, Write};
use spin::Mutex;
use uart_16550::SerialPort;
use x86_64::instructions::port::Port;

pub static SERIAL1: Mutex<SerialPort> = Mutex::new(unsafe { SerialPort::new(0x3F8) });

pub fn init() {
    SERIAL1.lock().init();
}

pub fn receive_char() -> Option<char> {
    let mut status_port: Port<u8> = Port::new(0x3FD);
    let status = unsafe { status_port.read() };
    if (status & 0x01) != 0 {
        let mut data_port: Port<u8> = Port::new(0x3F8);
        let byte = unsafe { data_port.read() };
        match byte {
            b'\r' => Some('\n'),
            0x7F => Some('\x08'), // DEL key as backspace
            b => Some(b as char),
        }
    } else {
        None
    }
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        SERIAL1
            .lock()
            .write_fmt(args)
            .expect("Printing to serial port failed");
    });
}

#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::serial::_print(format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(concat!($fmt, "\n"), $($arg)*));
}
