//! Порт `libc/string.c` — строковые функции C (NUL-terminated строки).
//! Все функции небезопасны для сырых указателей, но логика 1-в-1 с C.

use core::ptr;
use crate::ctype;
use crate::alloc::{__malloc, __free};

/// Длина строки.
#[no_mangle]
pub unsafe extern "C" fn strlen(s: *const u8) -> usize {
    let mut n = 0usize;
    while *s.add(n) != 0 {
        n += 1;
    }
    n
}

/// `strcmp`: разница первых отличающихся байт.
#[no_mangle]
pub unsafe extern "C" fn strcmp(a: *const u8, b: *const u8) -> i32 {
    let mut p = a;
    let mut q = b;
    while *p != 0 && *p == *q {
        p = p.add(1);
        q = q.add(1);
    }
    (*p as i32) - (*q as i32)
}

/// `strncmp` с ограничением n.
#[no_mangle]
pub unsafe extern "C" fn strncmp(a: *const u8, b: *const u8, n: usize) -> i32 {
    let mut i = 0usize;
    while i < n && *a.add(i) != 0 && *a.add(i) == *b.add(i) {
        i += 1;
    }
    if i == n {
        return 0;
    }
    (*a.add(i) as i32) - (*b.add(i) as i32)
}

/// `strcasecmp`: сравнение без учёта регистра.
pub unsafe fn strcasecmp(a: *const u8, b: *const u8) -> i32 {
    let mut p = a;
    let mut q = b;
    while *p != 0 && lower(*p) == lower(*q) {
        p = p.add(1);
        q = q.add(1);
    }
    (lower(*p) as i32) - (lower(*q) as i32)
}

/// `strncasecmp`.
pub unsafe fn strncasecmp(a: *const u8, b: *const u8, n: usize) -> i32 {
    let mut i = 0usize;
    while i < n && *a.add(i) != 0 && lower(*a.add(i)) == lower(*b.add(i)) {
        i += 1;
    }
    if i == n {
        return 0;
    }
    (lower(*a.add(i)) as i32) - (lower(*b.add(i)) as i32)
}

fn lower(c: u8) -> u8 {
    if c >= b'A' && c <= b'Z' {
        c + 32
    } else {
        c
    }
}

/// `strcpy`: копирует строку вместе с завершающим нулём.
#[no_mangle]
pub unsafe extern "C" fn strcpy(dst: *mut u8, src: *const u8) -> *mut u8 {
    let mut d = dst;
    let mut s = src;
    while {
        *d = *s;
        *s != 0
    } {
        d = d.add(1);
        s = s.add(1);
    }
    dst
}

/// `strncpy`: копирует до n байт, добивает нулями.
#[no_mangle]
pub unsafe extern "C" fn strncpy(dst: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    let mut i = 0usize;
    while i < n && *src.add(i) != 0 {
        *dst.add(i) = *src.add(i);
        i += 1;
    }
    while i < n {
        *dst.add(i) = 0;
        i += 1;
    }
    dst
}

/// `strcat`.
#[no_mangle]
pub unsafe extern "C" fn strcat(dst: *mut u8, src: *const u8) -> *mut u8 {
    let mut d = dst;
    while *d != 0 {
        d = d.add(1);
    }
    let mut s = src;
    while {
        *d = *s;
        *s != 0
    } {
        d = d.add(1);
        s = s.add(1);
    }
    dst
}

/// `strncat`.
#[no_mangle]
pub unsafe extern "C" fn strncat(dst: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    let mut d = dst;
    while *d != 0 {
        d = d.add(1);
    }
    let mut i = 0usize;
    while i < n && *src.add(i) != 0 {
        *d = *src.add(i);
        d = d.add(1);
        i += 1;
    }
    *d = 0;
    dst
}

/// `strchr`: поиск символа. c = 0 — возвращает указатель на конец строки.
#[no_mangle]
pub unsafe extern "C" fn strchr(s: *const u8, c: i32) -> *mut u8 {
    let mut p = s;
    while *p != 0 {
        if *p == c as u8 {
            return p as *mut u8;
        }
        p = p.add(1);
    }
    if c == 0 {
        p as *mut u8
    } else {
        core::ptr::null_mut()
    }
}

/// `strrchr`: поиск последнего вхождения.
#[no_mangle]
pub unsafe extern "C" fn strrchr(s: *const u8, c: i32) -> *mut u8 {
    let mut last: *const u8 = core::ptr::null();
    let mut p = s;
    while *p != 0 {
        if *p == c as u8 {
            last = p;
        }
        p = p.add(1);
    }
    if c == 0 {
        p as *mut u8
    } else {
        last as *mut u8
    }
}

/// `strstr`: поиск подстроки.
#[no_mangle]
pub unsafe extern "C" fn strstr(hay: *const u8, needle: *const u8) -> *mut u8 {
    let nlen = strlen(needle);
    if nlen == 0 {
        return hay as *mut u8;
    }
    let mut p = hay;
    while *p != 0 {
        if strncmp(p, needle, nlen) == 0 {
            return p as *mut u8;
        }
        p = p.add(1);
    }
    core::ptr::null_mut()
}

static mut STRTOK_PTR: *const u8 = core::ptr::null();

/// `strtok`: нереентерабельный токенизатор (тот же статический saveptr).
#[no_mangle]
pub unsafe extern "C" fn strtok(s: *mut u8, delim: *const u8) -> *mut u8 {
    strtok_r(s, delim, &mut STRTOK_PTR)
}

/// `strtok_r`.
pub unsafe fn strtok_r(s: *mut u8, delim: *const u8, saveptr: &mut *const u8) -> *mut u8 {
    if !s.is_null() {
        *saveptr = s;
    }
    if (*saveptr).is_null() {
        return core::ptr::null_mut();
    }
    // пропускаем ведущие разделители
    while **saveptr != 0 && !strchr(delim, **saveptr as i32).is_null() {
        *saveptr = (*saveptr).add(1);
    }
    if **saveptr == 0 {
        *saveptr = core::ptr::null();
        return core::ptr::null_mut();
    }
    let start = *saveptr as *mut u8;
    while **saveptr != 0 && strchr(delim, **saveptr as i32).is_null() {
        *saveptr = (*saveptr).add(1);
    }
    if **saveptr != 0 {
        *((*saveptr) as *mut u8) = 0;
        *saveptr = (*saveptr).add(1);
    } else {
        *saveptr = core::ptr::null();
    }
    start
}

/// `strspn`.
#[no_mangle]
pub unsafe extern "C" fn strspn(s: *const u8, accept: *const u8) -> usize {
    let mut n = 0usize;
    let mut p = s;
    while *p != 0 && !strchr(accept, *p as i32).is_null() {
        p = p.add(1);
        n += 1;
    }
    n
}

/// `strcspn`.
#[no_mangle]
pub unsafe extern "C" fn strcspn(s: *const u8, reject: *const u8) -> usize {
    let mut n = 0usize;
    let mut p = s;
    while *p != 0 && strchr(reject, *p as i32).is_null() {
        p = p.add(1);
        n += 1;
    }
    n
}

/// `strpbrk`.
#[no_mangle]
pub unsafe extern "C" fn strpbrk(s: *const u8, accept: *const u8) -> *mut u8 {
    let mut p = s;
    while *p != 0 {
        if !strchr(accept, *p as i32).is_null() {
            return p as *mut u8;
        }
        p = p.add(1);
    }
    core::ptr::null_mut()
}

/// `strdup`: копия строки через malloc.
pub unsafe fn strdup(s: *const u8) -> *mut u8 {
    let len = strlen(s) + 1;
    let copy = __malloc(len) as *mut u8;
    if !copy.is_null() {
        ptr::copy_nonoverlapping(s, copy, len);
    }
    copy
}

/// `strndup`: копия первых n байт (с нулём).
pub unsafe fn strndup(s: *const u8, n: usize) -> *mut u8 {
    let mut len = strlen(s);
    if len > n {
        len = n;
    }
    let copy = __malloc(len + 1) as *mut u8;
    if !copy.is_null() {
        ptr::copy_nonoverlapping(s, copy, len);
        *copy.add(len) = 0;
    }
    copy
}

/// `strtrim`: обрезает ведущие/хвостовые пробелы на месте.
pub unsafe fn strtrim(s: *mut u8) -> *mut u8 {
    // хвост
    let mut end = strlen(s) as i32 - 1;
    while end >= 0 && ctype::isspace(*s.add(end as usize) as i32) != 0 {
        end -= 1;
    }
    *s.add((end + 1) as usize) = 0;
    // начало
    let mut p = s;
    while *p != 0 && ctype::isspace(*p as i32) != 0 {
        p = p.add(1);
    }
    if p != s {
        ptr::copy(p, s, strlen(p) + 1);
    }
    s
}

/// `strrev`: разворот строки на месте.
pub unsafe fn strrev(s: *mut u8) {
    let mut a = s;
    let mut b = s.add(strlen(s) - 1);
    while (a as usize) < (b as usize) {
        let t = *a;
        *a = *b;
        *b = t;
        a = a.add(1);
        b = b.sub(1);
    }
}