use spin::Mutex;
use x86_64::instructions::port::Port;

pub const KEY_UP: char = '\x11';
pub const KEY_DOWN: char = '\x12';
pub const KEY_LEFT: char = '\x13';
pub const KEY_RIGHT: char = '\x14';
pub const CTRL_C: char = '\x03';
pub const CTRL_L: char = '\x0C';
pub const CTRL_D: char = '\x04';
pub const KEY_ALT_TAB: char = '\x01';
pub const KEY_ALT_Q: char = '\x02';
pub const KEY_ALT_N: char = '\x0E';
pub const KEY_ESC: char = '\x1B';

const QUEUE_SIZE: usize = 256;

pub struct KeyQueue {
    buffer: [char; QUEUE_SIZE],
    read_pos: usize,
    write_pos: usize,
}

impl KeyQueue {
    pub const fn new() -> Self {
        Self {
            buffer: ['\0'; QUEUE_SIZE],
            read_pos: 0,
            write_pos: 0,
        }
    }

    pub fn push(&mut self, c: char) -> bool {
        let next_write = (self.write_pos + 1) % QUEUE_SIZE;
        if next_write == self.read_pos {
            false
        } else {
            self.buffer[self.write_pos] = c;
            self.write_pos = next_write;
            true
        }
    }

    pub fn pop(&mut self) -> Option<char> {
        if self.read_pos == self.write_pos {
            None
        } else {
            let c = self.buffer[self.read_pos];
            self.read_pos = (self.read_pos + 1) % QUEUE_SIZE;
            Some(c)
        }
    }
}

pub static KEY_QUEUE: Mutex<KeyQueue> = Mutex::new(KeyQueue::new());

pub fn pop_key() -> Option<char> {
    KEY_QUEUE.lock().pop()
}

pub fn poll_ps2() {
    let mut status_port: Port<u8> = Port::new(0x64);
    let status = unsafe { status_port.read() };
    // Bit 0 = Output Buffer Full (data available in 0x60)
    // Bit 5 = Mouse data (0 = keyboard data, 1 = mouse data)
    if (status & 0x01) != 0 && (status & 0x20) == 0 {
        let mut data_port: Port<u8> = Port::new(0x60);
        let scancode = unsafe { data_port.read() };
        handle_scancode(scancode);
    }
}

pub fn try_read_char() -> Option<char> {
    if let Some(c) = pop_key() {
        return Some(c);
    }
    poll_ps2();
    if let Some(c) = pop_key() {
        return Some(c);
    }
    if let Some(c) = crate::serial::receive_char() {
        return Some(c);
    }
    None
}

pub fn read_char() -> char {
    loop {
        if let Some(c) = try_read_char() {
            return c;
        }
        for _ in 0..100 {
            core::hint::spin_loop();
        }
    }
}

pub fn sleep_ticks(ticks: u64) {
    let start = crate::interrupts::ticks();
    while crate::interrupts::ticks().wrapping_sub(start) < ticks {
        poll_ps2();
        core::hint::spin_loop();
    }
}

pub fn init() {
    let mut cmd_port: Port<u8> = Port::new(0x64);
    let mut data_port: Port<u8> = Port::new(0x60);

    unsafe {
        // Drain any stale data in buffer
        while (cmd_port.read() & 0x01) != 0 {
            let _ = data_port.read();
        }

        // Enable First PS/2 port (Keyboard)
        cmd_port.write(0xAE);

        // Read Controller Configuration Byte (command 0x20)
        cmd_port.write(0x20);
        let mut timeout = 10_000;
        while (cmd_port.read() & 0x01) == 0 && timeout > 0 {
            timeout -= 1;
        }
        if (cmd_port.read() & 0x01) != 0 {
            let mut config = data_port.read();

            config |= 0x01;  // enable IRQ 1
            config &= !0x10; // enable clock
            config |= 0x40;  // enable scancode set 1 translation

            cmd_port.write(0x60);
            while (cmd_port.read() & 0x02) != 0 {}
            data_port.write(config);
        }

        // Send "Enable Scanning" command (0xF4) to the keyboard
        while (cmd_port.read() & 0x02) != 0 {}
        data_port.write(0xF4);

        // Drain any response (e.g. ACK 0xFA)
        for _ in 0..5000 {
            if (cmd_port.read() & 0x01) != 0 {
                let _ = data_port.read();
                break;
            }
        }
    }
}

pub struct KeyboardState {
    shift_pressed: bool,
    ctrl_pressed: bool,
    alt_pressed: bool,
    caps_lock: bool,
    extended: bool,
}

impl KeyboardState {
    pub const fn new() -> Self {
        Self {
            shift_pressed: false,
            ctrl_pressed: false,
            alt_pressed: false,
            caps_lock: false,
            extended: false,
        }
    }
}

static KB_STATE: Mutex<KeyboardState> = Mutex::new(KeyboardState::new());

pub fn is_alt_pressed() -> bool {
    KB_STATE.lock().alt_pressed
}

pub fn is_ctrl_pressed() -> bool {
    KB_STATE.lock().ctrl_pressed
}

// US QWERTY Scancode Set 1 (unmodified)
static SCANCODE_TABLE: [char; 58] = [
    '\0', '\x1B', '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', '-', '=', '\x08',
    '\t', 'q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p', '[', ']', '\n',
    '\0', // Left Control (0x1D)
    'a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l', ';', '\'', '`',
    '\0', // Left Shift (0x2A)
    '\\', 'z', 'x', 'c', 'v', 'b', 'n', 'm', ',', '.', '/',
    '\0', // Right Shift (0x36)
    '*',
    '\0', // Left Alt (0x38)
    ' ',  // Space (0x39)
];

// US QWERTY Scancode Set 1 (Shifted)
static SCANCODE_TABLE_SHIFT: [char; 58] = [
    '\0', '\x1B', '!', '@', '#', '$', '%', '^', '&', '*', '(', ')', '_', '+', '\x08',
    '\t', 'Q', 'W', 'E', 'R', 'T', 'Y', 'U', 'I', 'O', 'P', '{', '}', '\n',
    '\0', // Left Control
    'A', 'S', 'D', 'F', 'G', 'H', 'J', 'K', 'L', ':', '"', '~',
    '\0', // Left Shift
    '|', 'Z', 'X', 'C', 'V', 'B', 'N', 'M', '<', '>', '?',
    '\0', // Right Shift
    '*',
    '\0', // Left Alt
    ' ',  // Space
];

pub fn handle_scancode(scancode: u8) {
    let mut state = KB_STATE.lock();

    if scancode == 0xE0 {
        state.extended = true;
        return;
    }

    let is_break = (scancode & 0x80) != 0;
    let code = scancode & 0x7F;

    if state.extended {
        state.extended = false;
        if !is_break {
            match code {
                0x48 => { KEY_QUEUE.lock().push(KEY_UP); }
                0x50 => { KEY_QUEUE.lock().push(KEY_DOWN); }
                0x4B => { KEY_QUEUE.lock().push(KEY_LEFT); }
                0x4D => { KEY_QUEUE.lock().push(KEY_RIGHT); }
                0x53 => { KEY_QUEUE.lock().push('\x08'); } // Delete -> Backspace
                _ => {}
            }
        }
        return;
    }

    match code {
        0x1D => {
            // Left or Right Control
            state.ctrl_pressed = !is_break;
        }
        0x38 => {
            // Left or Right Alt
            state.alt_pressed = !is_break;
        }
        0x2A | 0x36 => {
            // Left Shift or Right Shift
            state.shift_pressed = !is_break;
        }
        0x3A => {
            // Caps Lock toggle on press
            if !is_break {
                state.caps_lock = !state.caps_lock;
            }
        }
        _ => {
            if !is_break && (code as usize) < SCANCODE_TABLE.len() {
                let mut ch = if state.shift_pressed {
                    SCANCODE_TABLE_SHIFT[code as usize]
                } else {
                    SCANCODE_TABLE[code as usize]
                };

                if state.alt_pressed {
                    match ch.to_ascii_lowercase() {
                        '\t' => {
                            KEY_QUEUE.lock().push(KEY_ALT_TAB);
                            return;
                        }
                        'q' => {
                            KEY_QUEUE.lock().push(KEY_ALT_Q);
                            return;
                        }
                        'n' => {
                            KEY_QUEUE.lock().push(KEY_ALT_N);
                            return;
                        }
                        _ => {}
                    }
                }

                if state.ctrl_pressed {
                    match ch.to_ascii_lowercase() {
                        'c' => {
                            KEY_QUEUE.lock().push(CTRL_C);
                            return;
                        }
                        'l' => {
                            KEY_QUEUE.lock().push(CTRL_L);
                            return;
                        }
                        'd' => {
                            KEY_QUEUE.lock().push(CTRL_D);
                            return;
                        }
                        _ => {}
                    }
                }

                if state.caps_lock && ch.is_ascii_alphabetic() {
                    if state.shift_pressed {
                        ch = ch.to_ascii_lowercase();
                    } else {
                        ch = ch.to_ascii_uppercase();
                    }
                }

                if ch != '\0' {
                    KEY_QUEUE.lock().push(ch);
                }
            }
        }
    }
}

pub fn handle_keyboard_interrupt() {
    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };
    handle_scancode(scancode);
}
