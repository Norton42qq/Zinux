//! Порт `kernel/system.c` (IDT-часть) + `kernel/system.h` (типы IDT).

use core::ptr;

use super::pic;

/// Запись IDT (8 байт). Совпадает с `idt_entry_t` (packed).
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct IdtEntry {
    offset_low: u16,
    selector: u16,
    zero: u8,
    type_attr: u8,
    offset_high: u16,
}

/// Указатель на IDT (6 байт, для `lidt`).
#[repr(C, packed)]
pub struct IdtPtr {
    pub limit: u16,
    pub base: u32,
}

/// Глобальная IDT (256 записей).
static mut IDT: [IdtEntry; 256] = [IdtEntry { offset_low: 0, selector: 0, zero: 0, type_attr: 0, offset_high: 0 }; 256];

/// Хендлеры из `arch/entry.S`.
extern "C" {
    fn isr0(); fn isr1(); fn isr2(); fn isr3(); fn isr4();
    fn isr5(); fn isr6(); fn isr7(); fn isr8(); fn isr9();
    fn isr10(); fn isr11(); fn isr12(); fn isr13(); fn isr14();
    fn isr15(); fn isr16(); fn isr17(); fn isr18(); fn isr19();
    fn timer_handler();
    fn keyboard_handler();
    fn mouse_irq_handler();
    fn syscall_handler();
    pub fn load_idt(ptr: *const IdtPtr);
}

/// Установка записи в IDT (порт `idt_set_gate`).
pub fn idt_set_gate(num: usize, handler: u32, selector: u16, flags: u8) {
    unsafe {
        let e = &mut *ptr::addr_of_mut!(IDT[num]);
        e.offset_low = handler as u16;
        e.offset_high = (handler >> 16) as u16;
        e.selector = selector;
        e.zero = 0;
        e.type_attr = flags;
    }
}

/// Инициализация IDT и PIC (порт `idt_init`).
pub fn idt_init() {
    unsafe {
        (ptr::addr_of_mut!(IDT) as *mut u8).write_bytes(0u8, core::mem::size_of::<IdtEntry>() * 256);
    }

    // Исключения CPU 0-19
    const ISRS: [unsafe extern "C" fn(); 20] = [
        isr0, isr1, isr2, isr3, isr4, isr5, isr6, isr7, isr8, isr9,
        isr10, isr11, isr12, isr13, isr14, isr15, isr16, isr17, isr18, isr19,
    ];
    for (n, h) in ISRS.iter().enumerate() {
        idt_set_gate(n, *h as u32, 0x08, 0x8E);
    }

    // IRQ0 = таймер PIT (INT 32), IRQ1 = клавиатура (INT 33), IRQ12 = мышь (INT 44)
    idt_set_gate(32, timer_handler as u32, 0x08, 0x8E);
    idt_set_gate(33, keyboard_handler as u32, 0x08, 0x8E);
    idt_set_gate(44, mouse_irq_handler as u32, 0x08, 0x8E);
    // INT 0x80 — ring2 -> kernel trap gate (keep IF)
    idt_set_gate(0x80, syscall_handler as u32, 0x08, 0xEF);

    let mut idt_ptr = IdtPtr { limit: 0, base: 0 };
    unsafe {
        let p = core::ptr::addr_of_mut!(idt_ptr);
        (*p).limit = (core::mem::size_of::<IdtEntry>() * 256 - 1) as u16;
        (*p).base = core::ptr::addr_of!(IDT) as u32;
        load_idt(&idt_ptr);
    }

    pic::pic_init();
}