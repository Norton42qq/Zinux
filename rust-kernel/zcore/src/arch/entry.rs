//! Ассемблерная часть ядра — порт `kernel/kernel_entry.asm`.
//!
//! Синтаксис GAS Intel (`entry.S`), подключён через `global_asm!`.
//! Символы `_start`, `keyboard_handler`, `timer_handler`, `syscall_handler`,
//! `mouse_irq_handler`, `load_idt`, `enable_interrupts`, `disable_interrupts`,
//! `run_user_program`, `abort_user_program`, `pm_to_rm_write`, `isr0..isr19`
//! определены там же (см. декларации в `arch::cpu`).

use core::arch::global_asm;

global_asm!(include_str!("entry.S"));

/// Готовый трамплин (258 байт, собран nasm) — копируется в 0x5000 в `_start`.
/// См. `arch/entry.S` (TRAMP_BASE/TRAMP_LEN) и генерацию `arch/tramp.bin`.
#[no_mangle]
#[used]
pub static tramp_blob: [u8; 258] = *include_bytes!("tramp.bin");