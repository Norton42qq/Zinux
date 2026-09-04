//! zcore — ядро Zinux (порт kernel/ из C в Rust).
//!
//! Вход: `_start` в `arch/entry.rs` (полный аналог `kernel_entry.asm`),
//! затем вызывается `kernel_main` из `system.rs` (порт `kernel/kernel.c`).

#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

mod lang;
mod arch;
mod drivers;
mod fat16;
mod panic;
mod api;
mod zxe;
mod system;
mod shell;

/// Принудительная линковка zlibk (runtime-функции mem* должны попасть в ядро).
#[used]
#[no_mangle]
pub static ZLIBK_LINK: unsafe extern "C" fn(*mut u8, i32, usize) -> *mut u8 = zlibk::mem::memset;