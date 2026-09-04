//! Компиляторные (lang) элементы: panic-хендлер, oom-хендлер.

use core::panic::PanicInfo;

/// Фолбэк вывода в нижний текст (0xB8000), затем halt-цикл.
fn panic_fallback() -> ! {
    let fb = 0xB8000usize as *mut u8;
    const MSG: &[u8] = b"KERNEL PANIC";
    let mut i = 0usize;
    while i < MSG.len() {
        unsafe {
            fb.add(i * 2).write_volatile(MSG[i]);
            fb.add(i * 2 + 1).write_volatile(0x1F);
        }
        i += 1;
    }
    loop {
        unsafe { core::arch::asm!("cli; hlt") }
    }
}

#[panic_handler]
fn rust_panic(info: &PanicInfo) -> ! {
    // Сообщение паники идёт через video-слой только если он инициализирован;
    // в любом случае — прямой вывод в VGA-текст и останов.
    let _ = info;
    panic_fallback()
}

#[alloc_error_handler]
fn oom(_: core::alloc::Layout) -> ! {
    panic_fallback()
}