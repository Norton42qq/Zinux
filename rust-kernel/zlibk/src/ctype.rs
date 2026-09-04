//! Порт `libc/ctype.h` — классификация и преобразование символов.
//! Реализуются через функции `unsafe fn`, безопасные обёртки не требуются
//! (работают с целочисленными кодами).

pub fn isdigit(c: i32) -> i32 {
    if c >= b'0' as i32 && c <= b'9' as i32 { 1 } else { 0 }
}
pub fn isalpha(c: i32) -> i32 {
    if (c >= b'A' as i32 && c <= b'Z' as i32) || (c >= b'a' as i32 && c <= b'z' as i32) {
        1
    } else {
        0
    }
}
pub fn isalnum(c: i32) -> i32 {
    isdigit(c) | isalpha(c)
}
pub fn isspace(c: i32) -> i32 {
    if c == b' ' as i32 || c == b'\t' as i32 || c == b'\n' as i32 || c == b'\r' as i32
        || c == 0x0B || c == 0x0C
    {
        1
    } else {
        0
    }
}
pub fn isupper(c: i32) -> i32 {
    if c >= b'A' as i32 && c <= b'Z' as i32 { 1 } else { 0 }
}
pub fn islower(c: i32) -> i32 {
    if c >= b'a' as i32 && c <= b'z' as i32 { 1 } else { 0 }
}
pub fn isprint(c: i32) -> i32 {
    if c >= 0x20 && c <= 0x7E { 1 } else { 0 }
}
pub fn iscntrl(c: i32) -> i32 {
    if (c >= 0x00 && c <= 0x1F) || c == 0x7F { 1 } else { 0 }
}
pub fn isxdigit(c: i32) -> i32 {
    isdigit(c) | is_xdigit_letter(c)
}
fn is_xdigit_letter(c: i32) -> i32 {
    if (c >= b'a' as i32 && c <= b'f' as i32) || (c >= b'A' as i32 && c <= b'F' as i32) {
        1
    } else {
        0
    }
}
pub fn ispunct(c: i32) -> i32 {
    if isprint(c) == 1 && isalnum(c) == 0 && c != b' ' as i32 { 1 } else { 0 }
}

pub fn toupper(c: i32) -> i32 {
    if islower(c) == 1 { c - 32 } else { c }
}
pub fn tolower(c: i32) -> i32 {
    if isupper(c) == 1 { c + 32 } else { c }
}