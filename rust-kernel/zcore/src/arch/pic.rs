//! Порт `kernel/system.c` (PIC-часть): 8259 Programmable Interrupt Controller.

use super::io::{outb, io_wait};

/// Инициализация PIC (порт `pic_init`).
pub fn pic_init() {
    unsafe {
        // ICW1: инициализация
        outb(0x20, 0x11);
        io_wait();
        outb(0xA0, 0x11);
        io_wait();

        // ICW2: векторные смещения
        outb(0x21, 0x20); // Master: IRQ 0-7 -> INT 32-39
        io_wait();
        outb(0xA1, 0x28); // Slave: IRQ 8-15 -> INT 40-47
        io_wait();

        // ICW3: каскадирование
        outb(0x21, 0x04);
        io_wait();
        outb(0xA1, 0x02);
        io_wait();

        // ICW4: режим 8086
        outb(0x21, 0x01);
        io_wait();
        outb(0xA1, 0x01);
        io_wait();

        // Маскировка всего, кроме таймера (IRQ0) и клавиатуры (IRQ1)
        outb(0x21, 0xFC);
        outb(0xA1, 0xFF);
    }
}

/// Отправка EOI (порт `pic_send_eoi`).
pub fn pic_send_eoi(irq: u8) {
    unsafe {
        if irq >= 8 {
            outb(0xA0, 0x20);
        }
        outb(0x20, 0x20);
    }
}