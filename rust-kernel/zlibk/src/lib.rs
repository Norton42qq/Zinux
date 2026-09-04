//! zlibk — ограниченная freestanding-библиотека ядра Zinux (порт прежней `libc/`).
//!
//! Не использует выделение памяти поверх `core` (кроме ручных аллокаторов),
//! не зависит от библиотек хоста. Предоставляет:
//! - `mem`   — memcpy/memset/memmove/memcmp/… (`no_mangle`-символы для `core`)
//! - `str`   — строковые функции (строки C, NUL-terminated)
//! - `ctype` — классификация/преобразование символов
//! - `num`   — atoi/strtol/strtoul/itoa/… (+ qsort/bsearch)
//! - `alloc` — free-list аллокатор кучи ядра (порт `libc/memory.c`)

#![no_std]

pub mod mem;
pub mod str;
pub mod ctype;
pub mod num;
pub mod alloc;

/// Размер, как `size_t` в C (u32 на i386).
pub type SizeT = u32;
/// Указатель для C-ABI (обычно *mut u8).
pub type Char = u8;