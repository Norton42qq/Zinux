#ifndef PIT_H
#define PIT_H

#include <stdint.h>

// IRQ0 перемаплен PIC-ом на INT 32
#define PIT_IRQ_VECTOR 32

// Порты PIT (Intel 8253/8254)
#define PIT_CHANNEL0   0x40
#define PIT_COMMAND    0x43

// Базовая частота PIT в Гц
#define PIT_BASE_HZ    1193182

// Целевая частота тиков
#define PIT_TARGET_HZ  1000

void     pit_init(uint32_t hz);   // настроить и запустить
uint32_t pit_ticks(void);         // миллисекунды с запуска
void     pit_sleep(uint32_t ms);  // блокирующая задержка

// Вызывается из обработчика прерывания
void     pit_tick(void);

#endif