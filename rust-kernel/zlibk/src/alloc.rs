//! Порт `libc/memory.c` — free-list аллокатор кучи ядра.
//!
//! Верхняя граница кучи `HEAP_END = 0x1000000` (16MB). Начало —
//! символ компоновщика `_heap_start`. Разметка блока должна совпадать
//! с C-версией (`size_t size; uint8_t free; struct block* next;` — 12 байт,
//! `next` со смещением 8), чтобы выравнивания/разбиения вели себя так же.

use core::fmt;
use core::ptr;

pub const HEAP_END: usize = 0x1000000;
const BLOCK_HDR: usize = core::mem::size_of::<Block>();
const MIN_SPLIT: usize = 16;

#[repr(C)]
struct Block {
    size: u32,   // размер полезной нагрузки (без заголовка)
    free: u8,
    _pad: [u8; 3],
    next: *mut Block,
}

impl Block {
    unsafe fn payload<'a>(&self) -> &'a mut [u8] {
        core::slice::from_raw_parts_mut(
            (self as *const Block as usize + BLOCK_HDR) as *mut u8,
            self.size as usize,
        )
    }
}

extern "C" {
    /// Символ из linker-скрипта: начало кучи.
    static _heap_start: u8;
}

static mut HEAP_HEAD: *mut Block = ptr::null_mut();

/// Инициализация кучи (аналог `memory_init`).
pub fn memory_init() {
    unsafe {
        let base = &_heap_start as *const u8 as *mut u8;
        HEAP_HEAD = base as *mut Block;
        let head = HEAP_HEAD;
        (*head).size = (HEAP_END - base as usize - BLOCK_HDR) as u32;
        (*head).free = 1;
        (*head).next = ptr::null_mut();
    }
}

/// Слияние соседних свободных блоков.
unsafe fn coalesce() {
    let mut cur = HEAP_HEAD;
    while !cur.is_null() && !(*cur).next.is_null() {
        if (*cur).free != 0 && (*(*cur).next).free != 0 {
            let nxt = (*cur).next;
            (*cur).size += (BLOCK_HDR + (*nxt).size as usize) as u32;
            (*cur).next = (*nxt).next;
        } else {
            cur = (*cur).next;
        }
    }
}

/// `malloc` (аналог C): поиск свободного блока, разбиение, метка занятости.
pub unsafe fn __malloc(size: usize) -> *mut u8 {
    if size == 0 {
        return ptr::null_mut();
    }
    // выравнивание до 4 байт
    let mut size = (size + 3) & !3usize;

    let mut cur = HEAP_HEAD;
    while !cur.is_null() {
        if (*cur).free != 0 && (*cur).size as usize >= size {
            if (*cur).size as usize >= size + BLOCK_HDR + MIN_SPLIT {
                let split = ((cur as usize) + BLOCK_HDR + size) as *mut Block;
                (*split).size = ((*cur).size as usize - size - BLOCK_HDR) as u32;
                (*split).free = 1;
                (*split).next = (*cur).next;
                (*cur).size = size as u32;
                (*cur).next = split;
            }
            (*cur).free = 0;
            return (cur as usize + BLOCK_HDR) as *mut u8;
        }
        cur = (*cur).next;
    }
    ptr::null_mut() // OOM
}

/// `free`.
pub unsafe fn __free(ptr: *mut u8) {
    if ptr.is_null() {
        return;
    }
    let blk = (ptr as usize - BLOCK_HDR) as *mut Block;
    (*blk).free = 1;
    coalesce();
}

/// `calloc`.
pub unsafe fn __calloc(count: usize, size: usize) -> *mut u8 {
    let total = count * size;
    let p = __malloc(total);
    if !p.is_null() {
        ptr::write_bytes(p, 0, total);
    }
    p
}

/// `realloc`.
pub unsafe fn __realloc(ptr: *mut u8, size: usize) -> *mut u8 {
    if ptr.is_null() {
        return __malloc(size);
    }
    if size == 0 {
        __free(ptr);
        return ptr::null_mut();
    }
    let blk = (ptr as usize - BLOCK_HDR) as *mut Block;
    if (*blk).size as usize >= size {
        return ptr; // текущий блок достаточно велик
    }
    let new = __malloc(size);
    if new.is_null() {
        return ptr::null_mut();
    }
    ptr::copy_nonoverlapping(ptr, new, (*blk).size as usize);
    __free(ptr);
    new
}

/// Занято в куче.
pub fn heap_used() -> usize {
    let mut used = 0usize;
    unsafe {
        let mut cur = HEAP_HEAD;
        while !cur.is_null() {
            if (*cur).free == 0 {
                used += (*cur).size as usize;
            }
            cur = (*cur).next;
        }
    }
    used
}

/// Свободно в куче.
pub fn heap_free() -> usize {
    let mut free = 0usize;
    unsafe {
        let mut cur = HEAP_HEAD;
        while !cur.is_null() {
            if (*cur).free != 0 {
                free += (*cur).size as usize;
            }
            cur = (*cur).next;
        }
    }
    free
}

/// Диагностический вывод карты блоков (аналог `heap_dump`), в writer.
pub fn heap_dump(w: &mut impl fmt::Write) {
    let mut i = 0usize;
    let _ = w.write_str("heap dump:\n");
    unsafe {
        let mut cur = HEAP_HEAD;
        while !cur.is_null() {
            let _ = writeln!(
                w,
                "  [{:>3}] addr=0x{:08x} size={} {}",
                i,
                (cur as usize + BLOCK_HDR) as u32,
                (*cur).size,
                if (*cur).free != 0 { "free" } else { "used" }
            );
            cur = (*cur).next;
            i += 1;
        }
    }
}