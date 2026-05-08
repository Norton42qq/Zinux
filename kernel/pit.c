#include "pit.h"
#include "io.h"
#include "system.h"

static volatile uint32_t _ticks = 0;

void pit_tick(void) {
    _ticks++;
    pic_send_eoi(0);
}

void pit_init(uint32_t hz) {
    uint32_t divisor = PIT_BASE_HZ / hz;

    // Канал 0, доступ lo/hi, режим 3 (square wave), двоичный счёт
    outb(PIT_COMMAND, 0x36);
    outb(PIT_CHANNEL0, (uint8_t)(divisor & 0xFF));
    outb(PIT_CHANNEL0, (uint8_t)((divisor >> 8) & 0xFF));
}

uint32_t pit_ticks(void) {
    return _ticks;
}

void pit_sleep(uint32_t ms) {
    uint32_t end = _ticks + ms;
    while (_ticks < end) {
        __asm__ volatile("hlt");
    }
}