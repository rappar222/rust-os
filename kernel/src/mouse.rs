use spin::Mutex;
use x86_64::instructions::port::Port;

pub struct MouseState {
    pub x: usize,
    pub y: usize,
    pub left_button: bool,
    pub right_button: bool,
    pub middle_button: bool,
    pub left_clicked: bool,
    pub left_released: bool,
    pub moved: bool,
    packet_cycle: u8,
    packet_bytes: [u8; 3],
    screen_width: usize,
    screen_height: usize,
}

impl MouseState {
    pub const fn new() -> Self {
        Self {
            x: 640,
            y: 360,
            left_button: false,
            right_button: false,
            middle_button: false,
            left_clicked: false,
            left_released: false,
            moved: false,
            packet_cycle: 0,
            packet_bytes: [0; 3],
            screen_width: 1280,
            screen_height: 720,
        }
    }
}

pub static MOUSE_STATE: Mutex<MouseState> = Mutex::new(MouseState::new());

// 12x18 Arrow Mouse Cursor Sprite
// 0 = Transparent, 1 = Black border, 2 = White interior
pub const CURSOR_WIDTH: usize = 12;
pub const CURSOR_HEIGHT: usize = 18;
pub const CURSOR_SPRITE: [[u8; CURSOR_WIDTH]; CURSOR_HEIGHT] = [
    [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [1, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [1, 2, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0],
    [1, 2, 2, 2, 1, 0, 0, 0, 0, 0, 0, 0],
    [1, 2, 2, 2, 2, 1, 0, 0, 0, 0, 0, 0],
    [1, 2, 2, 2, 2, 2, 1, 0, 0, 0, 0, 0],
    [1, 2, 2, 2, 2, 2, 2, 1, 0, 0, 0, 0],
    [1, 2, 2, 2, 2, 2, 2, 2, 1, 0, 0, 0],
    [1, 2, 2, 2, 2, 2, 2, 2, 2, 1, 0, 0],
    [1, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 0],
    [1, 2, 2, 1, 2, 2, 1, 0, 0, 0, 0, 0],
    [1, 2, 1, 0, 1, 2, 2, 1, 0, 0, 0, 0],
    [1, 1, 0, 0, 1, 2, 2, 1, 0, 0, 0, 0],
    [1, 0, 0, 0, 0, 1, 2, 2, 1, 0, 0, 0],
    [0, 0, 0, 0, 0, 1, 2, 2, 1, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
];

fn mouse_wait_write() {
    let mut status: Port<u8> = Port::new(0x64);
    for _ in 0..100_000 {
        if (unsafe { status.read() } & 0x02) == 0 {
            return;
        }
        core::hint::spin_loop();
    }
}

fn mouse_wait_read() -> bool {
    let mut status: Port<u8> = Port::new(0x64);
    for _ in 0..100_000 {
        if (unsafe { status.read() } & 0x01) != 0 {
            return true;
        }
        core::hint::spin_loop();
    }
    false
}

fn mouse_write(cmd: u8) {
    mouse_wait_write();
    unsafe { Port::<u8>::new(0x64).write(0xD4) };
    mouse_wait_write();
    unsafe { Port::<u8>::new(0x60).write(cmd) };
}

fn mouse_read() -> u8 {
    if mouse_wait_read() {
        unsafe { Port::<u8>::new(0x60).read() }
    } else {
        0
    }
}

pub fn init(width: usize, height: usize) {
    {
        let mut state = MOUSE_STATE.lock();
        state.screen_width = width;
        state.screen_height = height;
        state.x = width / 2;
        state.y = height / 2;
    }

    unsafe {
        // 1. Enable auxiliary mouse port on 8042
        mouse_wait_write();
        Port::<u8>::new(0x64).write(0xA8);

        // 2. Read Controller Configuration Byte
        mouse_wait_write();
        Port::<u8>::new(0x64).write(0x20);
        let mut status: u8 = 0x47; // Default sane config: translation on, both IRQs on
        if mouse_wait_read() {
            status = Port::<u8>::new(0x60).read();
        }

        // Enable interrupt for mouse (bit 1) and keyboard (bit 0), enable translation (bit 6)
        // Enable clocks for both keyboard (bit 4) and mouse (bit 5)
        status |= 0x43;
        status &= !(0x10 | 0x20);

        mouse_wait_write();
        Port::<u8>::new(0x64).write(0x60);
        mouse_wait_write();
        Port::<u8>::new(0x60).write(status);

        // 3. Set default settings
        mouse_write(0xF6);
        let _ = mouse_read(); // ACK (0xFA)

        // 4. Enable packet streaming
        mouse_write(0xF4);
        let _ = mouse_read(); // ACK (0xFA)
    }
}

pub fn handle_mouse_byte(byte: u8) {
    let mut state = MOUSE_STATE.lock();
    let cycle = state.packet_cycle;

    match cycle {
        0 => {
            // Byte 0: Bit 3 must be 1 for valid packet sync
            if (byte & 0x08) != 0 {
                state.packet_bytes[0] = byte;
                state.packet_cycle = 1;
            }
        }
        1 => {
            state.packet_bytes[1] = byte;
            state.packet_cycle = 2;
        }
        2 => {
            state.packet_bytes[2] = byte;
            state.packet_cycle = 0;

            let flags = state.packet_bytes[0];
            let raw_x = state.packet_bytes[1];
            let raw_y = state.packet_bytes[2];

            // Sign extend deltas
            let mut dx = raw_x as isize;
            if (flags & 0x10) != 0 {
                dx -= 256;
            }

            let mut dy = raw_y as isize;
            if (flags & 0x20) != 0 {
                dy -= 256;
            }

            // Check overflow bits
            if (flags & 0xC0) != 0 {
                return;
            }

            let max_x = state.screen_width.saturating_sub(CURSOR_WIDTH);
            let max_y = state.screen_height.saturating_sub(CURSOR_HEIGHT);

            let new_x = (state.x as isize + dx).max(0).min(max_x as isize) as usize;
            // Screen Y is inverted from PS/2 mouse coordinates (dy > 0 is up)
            let new_y = (state.y as isize - dy).max(0).min(max_y as isize) as usize;

            if new_x != state.x || new_y != state.y {
                state.x = new_x;
                state.y = new_y;
                state.moved = true;
            }

            let left = (flags & 0x01) != 0;
            let right = (flags & 0x02) != 0;
            let middle = (flags & 0x04) != 0;

            if left && !state.left_button {
                state.left_clicked = true;
            } else if !left && state.left_button {
                state.left_released = true;
            }

            state.left_button = left;
            state.right_button = right;
            state.middle_button = middle;
        }
        _ => {
            state.packet_cycle = 0;
        }
    }
}

pub fn poll_mouse() {
    let mut status_port: Port<u8> = Port::new(0x64);
    let status = unsafe { status_port.read() };
    // Bit 0 = data available, Bit 5 = mouse data (auxiliary device)
    if (status & 0x01) != 0 && (status & 0x20) != 0 {
        let mut data_port: Port<u8> = Port::new(0x60);
        let byte = unsafe { data_port.read() };
        handle_mouse_byte(byte);
    }
}

pub fn get_mouse_event() -> (usize, usize, bool, bool, bool, bool) {
    let mut state = MOUSE_STATE.lock();
    let x = state.x;
    let y = state.y;
    let left_btn = state.left_button;
    let clicked = state.left_clicked;
    let released = state.left_released;
    let moved = state.moved;

    // Reset one-shot event triggers
    state.left_clicked = false;
    state.left_released = false;
    state.moved = false;

    (x, y, left_btn, clicked, released, moved)
}
