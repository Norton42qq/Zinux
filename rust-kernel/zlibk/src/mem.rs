//! Порт `libc/string.c` (функции работы с памятью).
//!
//! `memcpy`, `memset`, `memmove`, `memcmp`, `memchr` экспортируются как
//! `extern "C" no_mangle` — на них ссылается `core` и быстрые вставки
//! `compiler_builtins` (без фичи `compiler-builtins-mem`).

use core::ptr;

/// `void* memset(void*, int, size_t)`
#[no_mangle]
pub unsafe extern "C" fn memset(dst: *mut u8, c: i32, n: usize) -> *mut u8 {
    let mut p = dst;
    let mut i = n;
    while i > 0 {
        ptr::write(p, c as u8);
        p = p.add(1);
        i -= 1;
    }
    dst
}

/// `void* memcpy(void*, const void*, size_t)`
#[no_mangle]
pub unsafe extern "C" fn memcpy(dst: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    let mut p = dst;
    let mut q = src;
    let mut i = n;
    while i > 0 {
        ptr::write(p, ptr::read(q));
        p = p.add(1);
        q = q.add(1);
        i -= 1;
    }
    dst
}

/// `void* memmove(void*, const void*, size_t)`
#[no_mangle]
pub unsafe extern "C" fn memmove(dst: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    let d = dst as usize;
    let s = src as usize;
    if d < s || d >= s + n {
        ptr::copy_nonoverlapping(src, dst, n);
    } else {
        let mut p = dst.add(n);
        let mut q = src.add(n);
        let mut i = n;
        while i > 0 {
            p = p.sub(1);
            q = q.sub(1);
            ptr::write(p, ptr::read(q));
            i -= 1;
        }
    }
    dst
}

/// `int memcmp(const void*, const void*, size_t)`
#[no_mangle]
pub unsafe extern "C" fn memcmp(a: *const u8, b: *const u8, n: usize) -> i32 {
    let mut p = a;
    let mut q = b;
    let mut i = n;
    while i > 0 {
        let x = ptr::read(p) as i32;
        let y = ptr::read(q) as i32;
        if x != y {
            return x - y;
        }
        p = p.add(1);
        q = q.add(1);
        i -= 1;
    }
    0
}

/// `void* memchr(const void*, int, size_t)`
#[no_mangle]
pub unsafe extern "C" fn memchr(s: *const u8, c: i32, n: usize) -> *mut u8 {
    let mut p = s;
    let mut i = n;
    while i > 0 {
        if ptr::read(p) == c as u8 {
            return p as *mut u8;
        }
        p = p.add(1);
        i -= 1;
    }
    core::ptr::null_mut()
}