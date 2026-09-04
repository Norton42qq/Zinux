//! Порт `kernel/pit.c` + `kernel/pit.h` — таймер (8253/8254).

use core::arch::asm;

use super::io::outb;
use super::pic;

pub const PIT_IRQ_VECTOR: u32 = 32;
pub const PIT_CHANNEL0: u16 = 0x40;
pub const PIT_COMMAND: u16 = 0x43;
pub const PIT_BASE_HZ: u32 = 1193182;
pub const PIT_TARGET_HZ: u32 = 1000;

static mut TICKS: u32 = 0;

/// Обработчик IRQ0 (вызывается из asm `timer_handler`).
#[no_mangle]
pub extern "C" fn pit_tick() {
    unsafe {
        TICKS += 1;
    }
    pic::pic_send_eoi(0);
}

/// Настроить и запустить таймер (порт `pit_init`).
pub fn pit_init(hz: u32) {
    let divisor = PIT_BASE_HZ / hz;

    // Канал 0, доступ lo/hi, режим 3 (square wave), двоичный счёт
    unsafe {
        outb(PIT_COMMAND, 0x36);
        outb(PIT_CHANNEL0, (divisor & 0xFF) as u8);
        outb(PIT_CHANNEL0, ((divisor >> 8) & 0xFF) as u8);
    }
}

/// Миллисекунды с запуска (порт `pit_ticks`).
pub fn pit_ticks() -> u32 {
    unsafe { TICKS }
}

/// Блокирующая задержка (порт `pit_sleep`).
pub fn pit_sleep(ms: u32) {
    let end = unsafe { TICKS } + ms;
    while unsafe { TICKS } < end {
        unsafe { asm!("hlt", options(nostack)); }
    }
}