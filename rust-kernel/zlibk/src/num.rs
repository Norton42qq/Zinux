//! Порт `libc/stdlib.c` — числовые преобразования, rand/srand,
//! qsort/bsearch. Используется ядром (api/int_to_str и т.д.).

use core::ptr;
use crate::ctype;

/// `strtol` с автоопределением основания (base==0).
#[no_mangle]
pub unsafe extern "C" fn strtol(s: *const u8, endptr: *mut *const u8, base: i32) -> i32 {
    let mut p = s;
    while ctype::isspace(*p as i32) != 0 {
        p = p.add(1);
    }
    let neg = if *p == b'-' as u8 { p = p.add(1); true } else { if *p == b'+' as u8 { p = p.add(1) } false };

    let mut base = base;
    if base == 0 {
        if *p == b'0' as u8 && (*p.add(1) == b'x' as u8 || *p.add(1) == b'X' as u8) {
            base = 16;
            p = p.add(2);
        } else if *p == b'0' as u8 {
            base = 8;
            p = p.add(1);
        } else {
            base = 10;
        }
    } else if base == 16 && *p == b'0' as u8 && (*p.add(1) == b'x' as u8 || *p.add(1) == b'X' as u8) {
        p = p.add(2);
    }

    let start = p;
    let mut result: i32 = 0;
    while *p != 0 {
        let c = *p as i32;
        let digit = if ctype::isdigit(c) != 0 {
            c - b'0' as i32
        } else if c >= b'a' as i32 && c <= b'f' as i32 {
            c - b'a' as i32 + 10
        } else if c >= b'A' as i32 && c <= b'F' as i32 {
            c - b'A' as i32 + 10
        } else {
            break;
        };
        if digit >= base {
            break;
        }
        result = result.wrapping_mul(base).wrapping_add(digit);
        p = p.add(1);
    }

    let stop = if p == start { start } else { p };
    if !endptr.is_null() {
        *endptr = stop;
    }
    if neg { -result } else { result }
}

/// `strtoul`.
#[no_mangle]
pub unsafe extern "C" fn strtoul(s: *const u8, endptr: *mut *const u8, base: i32) -> u32 {
    let mut p = s;
    while ctype::isspace(*p as i32) != 0 {
        p = p.add(1);
    }
    if *p == b'+' as u8 {
        p = p.add(1);
    }

    let mut base = base;
    if base == 0 {
        if *p == b'0' as u8 && (*p.add(1) == b'x' as u8 || *p.add(1) == b'X' as u8) {
            base = 16;
            p = p.add(2);
        } else if *p == b'0' as u8 {
            base = 8;
            p = p.add(1);
        } else {
            base = 10;
        }
    } else if base == 16 && *p == b'0' as u8 && (*p.add(1) == b'x' as u8 || *p.add(1) == b'X' as u8) {
        p = p.add(2);
    }

    let start = p;
    let mut result: u32 = 0;
    while *p != 0 {
        let c = *p as i32;
        let digit = if ctype::isdigit(c) != 0 {
            c - b'0' as i32
        } else if c >= b'a' as i32 && c <= b'f' as i32 {
            c - b'a' as i32 + 10
        } else if c >= b'A' as i32 && c <= b'F' as i32 {
            c - b'A' as i32 + 10
        } else {
            break;
        };
        if digit >= base {
            break;
        }
        result = result.wrapping_mul(base as u32).wrapping_add(digit as u32);
        p = p.add(1);
    }

    let stop = if p == start { start } else { p };
    if !endptr.is_null() {
        *endptr = stop;
    }
    result
}

/// `atoi`.
#[no_mangle]
pub unsafe extern "C" fn atoi(s: *const u8) -> i32 {
    strtol(s, ptr::null_mut(), 10)
}

/// `itoa`: число в строку по основанию (0..=16), как в C.
pub unsafe fn itoa(mut val: i32, buf: *mut u8, base: i32) {
    const DIGITS: &[u8] = b"0123456789abcdef";
    let mut p = buf;
    if val < 0 && base == 10 {
        *p = b'-' as u8;
        p = p.add(1);
        val = -val;
    }
    let start = p;
    if val == 0 {
        *p = b'0' as u8;
        p = p.add(1);
    } else {
        while val != 0 {
            *p = DIGITS[(val % base) as usize];
            p = p.add(1);
            val /= base;
        }
    }
    // завершающий ноль, затем разворот от start (на позиции после знака)
    *p = 0;
    let mut hi = start;
    let mut lo = p.sub(1);
    while (hi as usize) < (lo as usize) {
        let t = *hi;
        *hi = *lo;
        *lo = t;
        hi = hi.add(1);
        lo = lo.sub(1);
    }
}

/// `utoa`.
pub unsafe fn utoa(mut val: u32, buf: *mut u8, base: i32) {
    const DIGITS: &[u8] = b"0123456789abcdef";
    let mut p = buf;
    let start = p;
    if val == 0 {
        *p = b'0' as u8;
        p = p.add(1);
    } else {
        while val != 0 {
            *p = DIGITS[(val % base as u32) as usize];
            p = p.add(1);
            val /= base as u32;
        }
    }
    *p = 0;
    let mut hi = start;
    let mut lo = p.sub(1);
    while (hi as usize) < (lo as usize) {
        let t = *hi;
        *hi = *lo;
        *lo = t;
        hi = hi.add(1);
        lo = lo.sub(1);
    }
}

pub fn abs(x: i32) -> i32 {
    if x < 0 { -x } else { x }
}

/// `rand` (LCG, тот же seed по умолчанию 12345).
static mut RAND_SEED: u32 = 12345;
pub const RAND_MAX: u32 = 0x7FFF;

pub fn rand() -> i32 {
    unsafe {
        RAND_SEED = RAND_SEED.wrapping_mul(1664525).wrapping_add(1013904223);
        (RAND_SEED & RAND_MAX) as i32
    }
}

pub fn srand(seed: u32) {
    unsafe { RAND_SEED = seed; }
}

/// `qsort` (introsort = quicksort + insertion sort, порт C-кода).
type CmpFn = unsafe extern "C" fn(*const core::ffi::c_void, *const core::ffi::c_void) -> i32;

unsafe fn swap_bytes(mut a: *mut u8, mut b: *mut u8, size: usize) {
    let mut k = size;
    while k > 0 {
        let t = *a;
        *a = *b;
        *b = t;
        a = a.add(1);
        b = b.add(1);
        k -= 1;
    }
}

unsafe fn insertion_sort(base: *mut u8, n: usize, size: usize, cmp: CmpFn) {
    for i in 1..n {
        let mut j = i;
        while j > 0 {
            let a = base.add((j - 1) * size);
            let b = base.add(j * size);
            if cmp(a as *const _, b as *const _) <= 0 {
                break;
            }
            swap_bytes(a, b, size);
            j -= 1;
        }
    }
}

unsafe fn quicksort(base: *mut u8, n: usize, size: usize, cmp: CmpFn) {
    if n < 8 {
        insertion_sort(base, n, size, cmp);
        return;
    }
    let mut pivot = base.add((n / 2) * size);
    let mut left = base;
    let mut right = base.add((n - 1) * size);

    while (left as usize) <= (right as usize) {
        while cmp(left as *const _, pivot as *const _) < 0 {
            left = left.add(size);
        }
        while cmp(right as *const _, pivot as *const _) > 0 {
            right = right.sub(size);
        }
        if (left as usize) <= (right as usize) {
            swap_bytes(left, right, size);
            if pivot == left {
                pivot = right;
            } else if pivot == right {
                pivot = left;
            }
            left = left.add(size);
            right = right.sub(size);
        }
    }

    let left_n = (right as usize - base as usize) / size + 1;
    let right_n = n - (left as usize - base as usize) / size;
    if left_n > 1 {
        quicksort(base, left_n, size, cmp);
    }
    if right_n > 1 {
        quicksort(left, right_n, size, cmp);
    }
}

/// `qsort` — сортировка с компаратором cdecl, как в C (`qsort`).
#[no_mangle]
pub unsafe extern "C" fn qsort(base: *mut core::ffi::c_void, n: usize, size: usize, cmp: CmpFn) {
    if n < 2 || size == 0 {
        return;
    }
    quicksort(base as *mut u8, n, size, cmp);
}

/// `bsearch` — бинарный поиск, порт C-кода.
#[no_mangle]
pub unsafe extern "C" fn bsearch(
    key: *const core::ffi::c_void,
    base: *const core::ffi::c_void,
    n: usize,
    size: usize,
    cmp: CmpFn,
) -> *mut core::ffi::c_void {
    let mut lo: *const u8 = base as *const u8;
    if n == 0 {
        return ptr::null_mut();
    }
    let mut hi: *const u8 = lo.add((n - 1) * size);
    while (lo as usize) <= (hi as usize) {
        let mid = lo.add(((hi as usize - lo as usize) / size / 2) * size);
        let r = cmp(key, mid as *const _);
        if r < 0 {
            hi = mid.sub(size);
        } else if r > 0 {
            lo = mid.add(size);
        } else {
            return mid as *mut core::ffi::c_void;
        }
    }
    ptr::null_mut()
}