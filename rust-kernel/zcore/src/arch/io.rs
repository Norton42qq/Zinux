//! Порт `kernel/io.h` — порты ввода/вывода x86.

use core::arch::asm;

#[inline]
pub unsafe fn outb(port: u16, val: u8) {
    asm!("out dx, al", in("dx") port, in("al") val, options(nomem, nostack, preserves_flags));
}

#[inline]
pub unsafe fn inb(port: u16) -> u8 {
    let mut val: u8;
    asm!("in al, dx", out("al") val, in("dx") port, options(nomem, nostack, preserves_flags));
    val
}

#[inline]
pub unsafe fn outw(port: u16, val: u16) {
    asm!("out dx, ax", in("dx") port, in("ax") val, options(nomem, nostack, preserves_flags));
}

#[inline]
pub unsafe fn inw(port: u16) -> u16 {
    let mut val: u16;
    asm!("in ax, dx", out("ax") val, in("dx") port, options(nomem, nostack, preserves_flags));
    val
}

#[inline]
pub unsafe fn outl(port: u16, val: u32) {
    asm!("out dx, eax", in("dx") port, in("eax") val, options(nomem, nostack, preserves_flags));
}

#[inline]
pub unsafe fn inl(port: u16) -> u32 {
    let mut val: u32;
    asm!("in eax, dx", out("eax") val, in("dx") port, options(nomem, nostack, preserves_flags));
    val
}

#[inline]
pub unsafe fn io_wait() {
    outb(0x80, 0);
}