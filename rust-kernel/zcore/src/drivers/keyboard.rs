//! Порт `kernel/drivers/keyboard.c` + `keyboard.h` — PS/2 клавиатура.

use core::arch::asm;

pub const KEYBOARD_DATA_PORT: u16 = 0x60;
pub const KEYBOARD_STATUS_PORT: u16 = 0x64;
pub const KEYBOARD_BUFFER_SIZE: usize = 256;

// Скан-коды
pub const SC_ESCAPE: u8 = 0x01;
pub const SC_BACKSPACE: u8 = 0x0E;
pub const SC_TAB: u8 = 0x0F;
pub const SC_ENTER: u8 = 0x1C;
pub const SC_LCTRL: u8 = 0x1D;
pub const SC_LSHIFT: u8 = 0x2A;
pub const SC_RSHIFT: u8 = 0x36;
pub const SC_LALT: u8 = 0x38;
pub const SC_CAPSLOCK: u8 = 0x3A;
pub const SC_ARROW_UP: u8 = 0x48;
pub const SC_ARROW_DOWN: u8 = 0x50;
pub const SC_ARROW_LEFT: u8 = 0x4B;
pub const SC_ARROW_RIGHT: u8 = 0x4D;

// Специальные коды
pub const KEY_ARROW_UP: u8 = 128;
pub const KEY_ARROW_DOWN: u8 = 129;
pub const KEY_ARROW_LEFT: u8 = 130;
pub const KEY_ARROW_RIGHT: u8 = 131;

pub const RAW_KEY_BUF_SIZE: usize = 64;

/// Raw-событие клавиши.
#[derive(Copy, Clone)]
#[repr(C)]
pub struct RawKeyEvent {
    pub scancode: u8,
    pub pressed: u8,
}

/// ASCII-буфер для shell.
static mut KEY_BUFFER: [u8; KEYBOARD_BUFFER_SIZE] = [0; KEYBOARD_BUFFER_SIZE];
static mut BUFFER_HEAD: i32 = 0;
static mut BUFFER_TAIL: i32 = 0;

/// Raw-буфер.
static mut RAW_BUF: [RawKeyEvent; RAW_KEY_BUF_SIZE] = [RawKeyEvent { scancode: 0, pressed: 0 }; RAW_KEY_BUF_SIZE];
static mut RAW_HEAD: i32 = 0;
static mut RAW_TAIL: i32 = 0;

// Модификаторы
static mut SHIFT_PRESSED: i32 = 0;
static mut CTRL_PRESSED: i32 = 0;
static mut CAPS_LOCK: i32 = 0;

// Символы нижнего регистра
static SC_ASCII_LOWER: [u8; 128] = [
    0, 27, b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'0', b'-', b'=', 8,
    b'\t', b'q', b'w', b'e', b'r', b't', b'y', b'u', b'i', b'o', b'p', b'[', b']', b'\n',
    0, b'a', b's', b'd', b'f', b'g', b'h', b'j', b'k', b'l', b';', b'\'', b'`',
    0, b'\\', b'z', b'x', b'c', b'v', b'b', b'n', b'm', b',', b'.', b'/', 0,
    b'*', 0, b' ',
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

// Символы верхнего регистра
static SC_ASCII_UPPER: [u8; 128] = [
    0, 27, b'!', b'@', b'#', b'$', b'%', b'^', b'&', b'*', b'(', b')', b'_', b'+', 8,
    b'\t', b'Q', b'W', b'E', b'R', b'T', b'Y', b'U', b'I', b'O', b'P', b'{', b'}', b'\n',
    0, b'A', b'S', b'D', b'F', b'G', b'H', b'J', b'K', b'L', b':', b'"', b'~',
    0, b'|', b'Z', b'X', b'C', b'V', b'B', b'N', b'M', b'<', b'>', b'?', 0,
    b'*', 0, b' ',
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

fn buffer_put(c: u8) {
    unsafe {
        let next = (BUFFER_HEAD + 1) % KEYBOARD_BUFFER_SIZE as i32;
        if next != BUFFER_TAIL {
            KEY_BUFFER[BUFFER_HEAD as usize] = c;
            BUFFER_HEAD = next;
        }
    }
}

fn raw_put(scancode: u8, pressed: u8) {
    unsafe {
        let next = (RAW_HEAD + 1) % RAW_KEY_BUF_SIZE as i32;
        if next != RAW_TAIL {
            RAW_BUF[RAW_HEAD as usize].scancode = scancode;
            RAW_BUF[RAW_HEAD as usize].pressed = pressed;
            RAW_HEAD = next;
        }
    }
}

pub fn keyboard_init() {
    unsafe {
        BUFFER_HEAD = 0;
        BUFFER_TAIL = 0;
        RAW_HEAD = 0;
        RAW_TAIL = 0;
        SHIFT_PRESSED = 0;
        CTRL_PRESSED = 0;
        CAPS_LOCK = 0;
    }
}

/// Обработчик прерывания (из asm `keyboard_handler`).
#[no_mangle]
pub extern "C" fn keyboard_callback(scancode: u32) {
    let code = (scancode & 0xFF) as u8;
    let released = code & 0x80 != 0;
    let code = code & 0x7F;

    // Всегда в raw
    raw_put(code, if released { 0 } else { 1 });

    // Модификаторы
    if code == SC_LSHIFT || code == SC_RSHIFT {
        unsafe { SHIFT_PRESSED = if released { 0 } else { 1 } }
        return;
    }
    if code == SC_LCTRL {
        unsafe { CTRL_PRESSED = if released { 0 } else { 1 } }
        return;
    }
    if code == SC_CAPSLOCK {
        if !released {
            unsafe { CAPS_LOCK = if CAPS_LOCK == 0 { 1 } else { 0 } }
        }
        return;
    }

    // В ASCII-буфер — только press
    if released {
        return;
    }

    // Стрелки
    if code == SC_ARROW_UP {
        buffer_put(KEY_ARROW_UP);
        return;
    }
    if code == SC_ARROW_DOWN {
        buffer_put(KEY_ARROW_DOWN);
        return;
    }
    if code == SC_ARROW_LEFT {
        buffer_put(KEY_ARROW_LEFT);
        return;
    }
    if code == SC_ARROW_RIGHT {
        buffer_put(KEY_ARROW_RIGHT);
        return;
    }

    if code < 58 {
        let mut use_shift = unsafe { SHIFT_PRESSED } != 0;
        if unsafe { CAPS_LOCK } != 0 && code >= 0x10 && code <= 0x32 {
            use_shift = !use_shift;
        }

        let c = if use_shift { SC_ASCII_UPPER[code as usize] } else { SC_ASCII_LOWER[code as usize] };
        if c != 0 {
            buffer_put(c);
        }
    }
}

pub fn keyboard_haschar() -> i32 {
    unsafe { if BUFFER_HEAD != BUFFER_TAIL { 1 } else { 0 } }
}

pub fn keyboard_getchar() -> u8 {
    unsafe {
        if BUFFER_HEAD == BUFFER_TAIL {
            return 0;
        }
        let c = KEY_BUFFER[BUFFER_TAIL as usize];
        BUFFER_TAIL = (BUFFER_TAIL + 1) % KEYBOARD_BUFFER_SIZE as i32;
        c
    }
}

pub fn keyboard_wait_char() -> u8 {
    while keyboard_haschar() == 0 {
        unsafe { asm!("hlt", options(nostack)); }
    }
    keyboard_getchar()
}

pub fn keyboard_clear_buffer() {
    unsafe {
        BUFFER_HEAD = 0;
        BUFFER_TAIL = 0;
    }
}

pub fn keyboard_shift_pressed() -> i32 {
    unsafe { SHIFT_PRESSED }
}

pub fn keyboard_ctrl_pressed() -> i32 {
    unsafe { CTRL_PRESSED }
}

pub fn keyboard_raw_has() -> i32 {
    unsafe { if RAW_HEAD != RAW_TAIL { 1 } else { 0 } }
}

pub fn keyboard_raw_get(ev: *mut RawKeyEvent) -> i32 {
    unsafe {
        if RAW_HEAD == RAW_TAIL {
            return 0;
        }
        *ev = RAW_BUF[RAW_TAIL as usize];
        RAW_TAIL = (RAW_TAIL + 1) % RAW_KEY_BUF_SIZE as i32;
        1
    }
}