use spin::Mutex;
use x86_64::instructions::port::Port;

const QUEUE_SIZE: usize = 128;

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
            // Buffer full, drop key
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

pub fn read_char() -> char {
    loop {
        // 1. Check PS/2 hardware keyboard queue
        if let Some(c) = pop_key() {
            return c;
        }

        // 2. Check serial port input (COM1)
        if let Some(c) = crate::serial::receive_char() {
            return c;
        }

        x86_64::instructions::hlt();
    }
}

pub struct KeyboardState {
    shift_pressed: bool,
    caps_lock: bool,
    extended: bool,
}

impl KeyboardState {
    pub const fn new() -> Self {
        Self {
            shift_pressed: false,
            caps_lock: false,
            extended: false,
        }
    }
}

static KB_STATE: Mutex<KeyboardState> = Mutex::new(KeyboardState::new());

// US QWERTY Scancode Set 1 (unmodified)
static SCANCODE_TABLE: [char; 58] = [
    '\0', '\x1B', '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', '-', '=', '\x08',
    '\t', 'q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p', '[', ']', '\n',
    '\0', // Left Control
    'a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l', ';', '\'', '`',
    '\0', // Left Shift
    '\\', 'z', 'x', 'c', 'v', 'b', 'n', 'm', ',', '.', '/',
    '\0', // Right Shift
    '*',
    '\0', // Left Alt
    ' ',  // Space
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
        // Extended keys (arrows, etc.) can be handled if needed
        return;
    }

    match code {
        0x2A | 0x36 => {
            // Left Shift (0x2A) or Right Shift (0x36)
            state.shift_pressed = !is_break;
        }
        0x3A => {
            // Caps Lock toggle on press
            if !is_break {
                state.caps_lock = !state.caps_lock;
            }
        }
        _ => {
            if !is_break {
                if (code as usize) < SCANCODE_TABLE.len() {
                    let mut ch = if state.shift_pressed {
                        SCANCODE_TABLE_SHIFT[code as usize]
                    } else {
                        SCANCODE_TABLE[code as usize]
                    };

                    // Handle Caps Lock for letters
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
}

pub fn handle_keyboard_interrupt() {
    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };
    handle_scancode(scancode);
}
