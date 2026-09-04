//! Порт `kernel/drivers/mouse.c` + `mouse.h` — PS/2 мышь (IRQ12).

use crate::arch::io::{inb, io_wait, outb};

pub const MOUSE_BTN_LEFT: u8 = 1 << 0;
pub const MOUSE_BTN_RIGHT: u8 = 1 << 1;
pub const MOUSE_BTN_MIDDLE: u8 = 1 << 2;

// Порты
const PS2_DATA: u16 = 0x60;
const PS2_STATUS: u16 = 0x64;
const PS2_CMD: u16 = 0x64;

// Биты статусного регистра
const ST_OBF: u8 = 1 << 0;
const ST_IBF: u8 = 1 << 1;
const ST_AUX: u8 = 1 << 5;

// Команды контроллера i8042
const CTL_READ_CFG: u8 = 0x20;
const CTL_WRITE_CFG: u8 = 0x60;
const CTL_ENABLE_AUX: u8 = 0xA8;
const CTL_TEST_AUX: u8 = 0xA9;
const CTL_WRITE_AUX: u8 = 0xD4;

// Команды мыши
const M_RESET: u8 = 0xFF;
const M_ENABLE: u8 = 0xF4;
const M_DISABLE: u8 = 0xF5;
const M_SET_RATE: u8 = 0xF3;
const M_GET_ID: u8 = 0xF2;
const M_SET_RES: u8 = 0xE8;
const M_ACK: u8 = 0xFA;
const M_RESEND: u8 = 0xFE;

// Биты первого байта пакета
const P0_LEFT: u8 = 1 << 0;
const P0_RIGHT: u8 = 1 << 1;
const P0_MIDDLE: u8 = 1 << 2;
const P0_ALWAYS1: u8 = 1 << 3;
const P0_XOVER: u8 = 1 << 6;
const P0_YOVER: u8 = 1 << 7;

/// Тип события мыши (`MouseEventType`).
#[derive(Copy, Clone, PartialEq)]
#[repr(u32)]
pub enum MouseEventType {
    MouseEvMove = 0,
    MouseEvPress = 1,
    MouseEvRelease = 2,
    MouseEvScroll = 3,
}

/// Состояние мыши (`MouseState`).
#[derive(Copy, Clone)]
#[repr(C)]
pub struct MouseState {
    pub x: i32,
    pub y: i32,
    pub buttons: u8,
    pub wheel: i32,
}

/// Событие мыши (`MouseEvent`).
#[derive(Copy, Clone)]
#[repr(C)]
pub struct MouseEvent {
    pub ev_type: MouseEventType,
    pub x: i32,
    pub y: i32,
    pub dx: i32,
    pub dy: i32,
    pub wheel: i32,
    pub buttons: u8,
}

pub type MouseEventFn = unsafe extern "C" fn(*const MouseEvent);

static mut G_MOUSE: MouseState = MouseState { x: 0, y: 0, buttons: 0, wheel: 0 };
static mut G_CB: Option<MouseEventFn> = None;

static mut PKT: [u8; 4] = [0; 4];
static mut PKT_POS: i32 = 0;
static mut PKT_LEN: i32 = 3;
static mut READY: i32 = 0;

static mut MAX_X: i32 = 0;
static mut MAX_Y: i32 = 0;
static mut SENS_X: i32 = 1;
static mut SENS_Y: i32 = 1;

fn wait_write() -> i32 {
    let mut n = 100000i32;
    unsafe {
        while { n -= 1; n > 0 && (inb(PS2_STATUS) & ST_IBF) != 0 } {
            io_wait();
        }
    }
    if n > 0 { 1 } else { 0 }
}

fn wait_read() -> i32 {
    let mut n = 100000i32;
    unsafe {
        while { n -= 1; n > 0 && (inb(PS2_STATUS) & ST_OBF) == 0 } {
            io_wait();
        }
    }
    if n > 0 { 1 } else { 0 }
}

fn flush() {
    let mut n = 32i32;
    unsafe {
        while {
            n -= 1;
            n >= 0 && (inb(PS2_STATUS) & ST_OBF) != 0
        } {
            io_wait();
            let _ = inb(PS2_DATA);
            io_wait();
        }
    }
}

fn ctl_cmd(c: u8) -> i32 {
    if wait_write() == 0 {
        return 0;
    }
    unsafe {
        outb(PS2_CMD, c);
        io_wait();
    }
    1
}

fn ctl_data(d: u8) -> i32 {
    if wait_write() == 0 {
        return 0;
    }
    unsafe {
        outb(PS2_DATA, d);
        io_wait();
    }
    1
}

fn read_byte(out: *mut u8) -> i32 {
    if wait_read() == 0 {
        return 0;
    }
    unsafe {
        *out = inb(PS2_DATA);
    }
    1
}

fn mouse_cmd(cmd: u8) -> i32 {
    let mut r = 3;
    let mut a: u8 = 0;
    while {
        r -= 1;
        r >= 0
    } {
        if ctl_cmd(CTL_WRITE_AUX) == 0 || ctl_data(cmd) == 0 {
            return 0;
        }
        if read_byte(&mut a) == 0 {
            return 0;
        }
        if a == M_ACK {
            return 1;
        }
        if a != M_RESEND {
            return 0;
        }
    }
    0
}

fn mouse_cmd2(c: u8, a: u8) -> i32 {
    if mouse_cmd(c) == 0 {
        return 0;
    }
    mouse_cmd(a)
}

fn irq12_mask() {
    unsafe { outb(0xA1, inb(0xA1) | (1 << 4)); }
}

fn irq12_unmask() {
    unsafe { outb(0xA1, inb(0xA1) & !(1 << 4)); }
}

fn irq2_unmask() {
    // каскад slave PIC
    unsafe { outb(0x21, inb(0x21) & !(1 << 2)); }
}

/// Инициализация мыши (порт `mouse_init`).
pub fn mouse_init(screen_w: i32, screen_h: i32) {
    let mut cfg: u8 = 0;

    unsafe {
        MAX_X = screen_w - 1;
        MAX_Y = screen_h - 1;
        G_MOUSE.x = screen_w / 2;
        G_MOUSE.y = screen_h / 2;
        G_MOUSE.buttons = 0;
        G_MOUSE.wheel = 0;
        PKT_POS = 0;
        PKT_LEN = 3;
        READY = 0;
        G_CB = None;
    }

    irq12_mask();
    flush();

    // Шаг 1: включить AUX порт
    if ctl_cmd(CTL_ENABLE_AUX) == 0 {
        goto_fail_with(cfg);
        return;
    }
    flush();

    // Шаг 2: считать конфиг, выключить AUX IRQ
    if ctl_cmd(CTL_READ_CFG) == 0 || read_byte(&mut cfg) == 0 {
        goto_fail_with(cfg);
        return;
    }
    cfg &= !(1u8 << 1); // AUX IRQ off
    cfg &= !(1u8 << 5); // AUX clock on (бит5=1 = disable)
    if ctl_cmd(CTL_WRITE_CFG) == 0 || ctl_data(cfg) == 0 {
        goto_fail_with(cfg);
        return;
    }

    // Шаг 3: тест AUX - 0x00 = OK
    let mut tr: u8 = 0xFF;
    if ctl_cmd(CTL_TEST_AUX) == 0 || read_byte(&mut tr) == 0 {
        goto_fail_with(cfg);
        return;
    }
    if tr != 0x00 {
        goto_fail_with(cfg);
        return;
    }

    // Шаг 4: сброс мыши
    if mouse_cmd(M_RESET) == 0 {
        goto_fail_with(cfg);
        return;
    }
let mut b: u8 = 0;
        if read_byte(&mut b) == 0 || b != 0xAA {
            goto_fail_with(cfg);
            return;
        }
        if read_byte(&mut b) == 0 {
            goto_fail_with(cfg);
            return;
        } // обычно 0x00
        let _ = b;
    flush();

    // Шаг 5: детект IntelliMouse
    mouse_cmd2(M_SET_RATE, 200);
    mouse_cmd2(M_SET_RATE, 100);
    mouse_cmd2(M_SET_RATE, 80);
    if mouse_cmd(M_GET_ID) == 0 {
        goto_fail_with(cfg);
        return;
    }
    let mut id: u8 = 0;
    if read_byte(&mut id) == 0 {
        goto_fail_with(cfg);
        return;
    }
    let _ = id;
    unsafe { PKT_LEN = 3; }
    flush();

    // Шаг 6: чувствительность
    mouse_cmd2(M_SET_RES, 3);
    mouse_cmd2(M_SET_RATE, 100);

    // Шаг 7: включить AUX IRQ ДО ENABLE
    cfg |= 1u8 << 1; // AUX IRQ on
    cfg &= !(1u8 << 5);
    if ctl_cmd(CTL_WRITE_CFG) == 0 || ctl_data(cfg) == 0 {
        goto_fail_with(cfg);
        return;
    }

    // Шаг 8: разрешить мыши слать пакеты
    if mouse_cmd(M_ENABLE) == 0 {
        goto_fail_with(cfg);
        return;
    }
    flush();

    unsafe { READY = 1; }
    irq2_unmask();
    irq12_unmask();
}

/// Общий выход `fail` из `mouse_init`.
fn goto_fail_with(cfg: u8) {
    unsafe {
        READY = 0;
        PKT_POS = 0;
    }
    mouse_cmd(M_DISABLE);
    flush();
    let mut cfg2 = cfg;
    cfg2 &= !(1u8 << 1);
    if ctl_cmd(CTL_WRITE_CFG) != 0 {
        let _ = ctl_data(cfg2);
    }
    irq12_mask();
}

pub fn mouse_get_state() -> MouseState {
    unsafe { G_MOUSE }
}

pub fn mouse_set_callback(fn_: Option<MouseEventFn>) {
    unsafe { G_CB = fn_; }
}

fn clamp() {
    unsafe {
        if G_MOUSE.x < 0 {
            G_MOUSE.x = 0;
        }
        if G_MOUSE.y < 0 {
            G_MOUSE.y = 0;
        }
        if G_MOUSE.x > MAX_X {
            G_MOUSE.x = MAX_X;
        }
        if G_MOUSE.y > MAX_Y {
            G_MOUSE.y = MAX_Y;
        }
    }
}

/// Обработчик IRQ12 (из asm `mouse_irq_handler`; аргумент — мусор из `push 0`).
#[no_mangle]
pub extern "C" fn mouse_handler(_r: *mut u8) {
    unsafe {
        let st = inb(PS2_STATUS);
        if st & ST_OBF == 0 {
            return;
        } // буфер пуст

        let byte = inb(PS2_DATA); // читаем ВСЕГДА

        if READY == 0 {
            PKT_POS = 0;
            return;
        }

        // Синхронизация пакета
        if PKT_POS == 0 && byte & P0_ALWAYS1 == 0 {
            return;
        }
        if PKT_POS > 0 && byte & P0_ALWAYS1 != 0 {
            PKT[0] = byte;
            PKT_POS = 1;
            return;
        }

        let pos = PKT_POS as usize;
        PKT[pos] = byte;
        PKT_POS += 1;
        if PKT_POS < PKT_LEN {
            return;
        } // ожидание остатка пакета

        PKT_POS = 0; // сброс позиции

        let flags = PKT[0];
        if flags & (P0_XOVER | P0_YOVER) != 0 {
            return;
        } // overflow - мусор

        // Кнопки
        let old_btn = G_MOUSE.buttons;
        G_MOUSE.buttons = 0;
        if flags & P0_LEFT != 0 {
            G_MOUSE.buttons |= MOUSE_BTN_LEFT;
        }
        if flags & P0_RIGHT != 0 {
            G_MOUSE.buttons |= MOUSE_BTN_RIGHT;
        }
        if flags & P0_MIDDLE != 0 {
            G_MOUSE.buttons |= MOUSE_BTN_MIDDLE;
        }

        // Движение
        let mut dx = PKT[1] as i8 as i32;
        let mut dy = -(PKT[2] as i8 as i32);
        if SENS_X > 1 {
            dx /= SENS_X;
        }
        if SENS_Y > 1 {
            dy /= SENS_Y;
        }
        G_MOUSE.x += dx;
        G_MOUSE.y += dy;
        clamp();

        // Колесо
        G_MOUSE.wheel = 0;
        if PKT_LEN == 4 {
            let mut w = (PKT[3] & 0x0F) as i8;
            if w & 0x08 != 0 {
                w |= 0xF0u8 as i8;
            }
            G_MOUSE.wheel = w as i32;
        }

        let cb = G_CB;
        if cb.is_none() {
            return;
        }

        let pressed = (!old_btn) & G_MOUSE.buttons;
        let released = old_btn & (!G_MOUSE.buttons);
        if pressed == 0 && released == 0 && dx == 0 && dy == 0 && G_MOUSE.wheel == 0 {
            return;
        }

        let mut ev = MouseEvent {
            ev_type: MouseEventType::MouseEvMove,
            x: G_MOUSE.x,
            y: G_MOUSE.y,
            dx,
            dy,
            wheel: G_MOUSE.wheel,
            buttons: G_MOUSE.buttons,
        };

        if pressed & MOUSE_BTN_LEFT != 0 {
            ev.ev_type = MouseEventType::MouseEvPress;
        } else if released & MOUSE_BTN_LEFT != 0 {
            ev.ev_type = MouseEventType::MouseEvRelease;
        } else if G_MOUSE.wheel != 0 {
            ev.ev_type = MouseEventType::MouseEvScroll;
        } else {
            ev.ev_type = MouseEventType::MouseEvMove;
        }

        cb.unwrap()(&ev);
    }
}

pub fn mouse_set_sensitivity(sx: i32, sy: i32) {
    unsafe {
        SENS_X = if sx < 1 { 1 } else if sx > 8 { 8 } else { sx };
        SENS_Y = if sy < 1 { 1 } else if sy > 8 { 8 } else { sy };
    }
}

pub fn mouse_set_pos(x: i32, y: i32) {
    unsafe {
        G_MOUSE.x = x;
        G_MOUSE.y = y;
    }
    clamp();
}

pub fn mouse_set_bounds(w: i32, h: i32) {
    unsafe {
        MAX_X = w - 1;
        MAX_Y = h - 1;
    }
    clamp();
}

pub fn mouse_is_ready() -> i32 {
    unsafe { READY }
}