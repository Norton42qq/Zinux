//! Zinux SDK — минимальная библиотека для написания пользовательских программ
//! под 32-битную ОС Zinux (Ring 2, формат .ZXE) на чистом `no_std` Rust.
//!
//! # Состав
//! - [`syscall`] — низкоуровневые обёртки `int 0x80` (putc / exit / api);
//! - [`api`] — безопасный API: `clear`, `print`, `color`, `waitkey` и др.;
//! - глобальный `#[panic_handler]`, завершающий программу через `syscall_exit(1)`.
//!
//! # Точка входа
//! Программа должна объявить `#[unsafe(no_mangle)] pub extern "C" fn _start() -> !`
//! (аналог `main`). Ядро прыгает в неё по адресу 0x200030 с esp = 0x2F0000.
//! `argc`/`argv` в стек не кладутся — читать их запрещено. В конце `_start`
//! обязан вызвать [`syscall::syscall_exit`], возврат (`ret`) приведёт к #GP.
//!
//! # Сборка
//! Требуется nightly + `-Z build-std=core`: см. `.cargo/config.toml`,
//! `i686-zinux.json` (без SSE/SSE2 и таблиц переходов) и `link.ld`
//! (линковка текста с 0x200030). Готовый ELF упаковывается в .zxe утилитой
//! `tools/zxe-pack`.

#![no_std]
#![allow(clippy::missing_safety_doc)]

pub mod api;
pub mod syscall;

use core::panic::PanicInfo;

/// Глобальный обработчик паники: печатает сообщение (если есть) и завершает
/// программу с кодом 1 через системный вызов exit.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(msg) = info.message().as_str() {
        for &b in msg.as_bytes() {
            syscall::syscall_putc(b);
        }
    }
    syscall::syscall_exit(1)
}

/// Обнуление секции .bss. Вызывается из `_start` каждой программы до любых
/// обращений к статическим данным.
///
/// # Safety
/// Только при старте программы; символы заданы линкер-скриптом.
pub unsafe fn zero_bss() {
    extern "C" {
        static _bss_start: u8;
        static _bss_end: u8;
    }
    let mut p = core::ptr::addr_of!(_bss_start) as *mut u8;
    let end = core::ptr::addr_of!(_bss_end) as *mut u8;
    while p < end {
        core::ptr::write_volatile(p, 0);
        p = p.add(1);
    }
}